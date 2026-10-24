# CP Native Agent 架构集成规范 (Bun-BEAM 混合底座)

## 1. 架构愿景与定位

Bun-BEAM 底座已经实现了 **JS Agent** 的百万级高并发与极速二进制路由（单机广播吞吐量达 ~21.5万 msg/sec，内存占用极低）。
目前需要将使用 **CP 语言** (基于 Actor 模型与 CPS 变换的编译型语言) 编写的 **Native Agent** 无缝接入此底座。

**核心目标：**
* **控制面统一 (Control Plane)**：不论是 JS Agent 还是 Native Agent，均在 JS/TS 端通过 `globalThis.Beam.spawn(ClassName, Id)` 统一启动，通过 `Beam.sendFast` 极速收发 8 字节二进制头消息。
* **数据面异构执行 (Data Plane)**：
  * **JS Agent**：消息通过 MPSC 无锁队列进入 Bun (JSC) Worker 线程执行。
  * **Native Agent (CP)**：消息**不出 BEAM 虚拟机**，直接在宿主 Erlang Actor 进程内通过调用底层 NIF 原地极速执行！
* **抢占式调度共存**：CP 编译器注入的让权代码，必须通过调用 BEAM API `enif_consume_timeslice` 汇报 CPU 消耗，并在时间片耗尽或遇到 `await` 时保存状态机上下文 (Context) 并返回给 Erlang Actor 挂起，实现完美融合。

---

## 2. 交互拓扑与生命周期流图

当 JS 层调用 `Beam.spawn("CPTraderAgent", "N-001")` 时：

```mermaid
graph TD
    A[JS Main Thread: Beam.spawn] -->|JSON/Binary| B(Erlang: actor.erl)
    B --> C{Is Native Class?}
    C -->|Yes| D[Erlang: spawn native_agent_loop]
    C -->|No| E[Erlang: spawn agent_loop for JS]
    
    D --> F[Wait in receive for Msg]
    F -->|fast_action, Action, Payload| G[Call CP NIF: native_dispatch_nif]
    
    subgraph "CP Compiled Native Code (libcp_agents.so)"
        G --> H[Deserialize Context]
        H --> I[Execute Static Switch via Action ID]
        I --> J[Execute CP State Machine (Business Logic)]
        I --> K{Hit await, sleep, or timeslice?}
        K -->|Yes| L[Serialize new Context]
    end
    
    L -->|Return {yield/sleep/reschedule, Context}| F
    F --> M[Erlang: Save Context to ETS or Wait]
```

---

## 3. CP 编译器的优化与适配要求 (To CP Compiler Devs)

为了让 CP 语言生成的 Zig 代码能被 Bun-BEAM 底座加载，CP 编译器必须遵循以下规范。

### 3.1 引入纯净的 `beam.zig` SDK

底座提供一个标准化的 SDK 文件 `/packages/bun-beam/sdk/beam.zig`。**CP 编译器生成的代码必须通过 `const beam = @import("beam.zig");` 来调用底层能力。**

**`beam.zig` 核心能力：**
* **RocksDB 原生访问**：支持 `beam.RocksDB.put/get/open`，实现 0 NIF 损耗的磁盘存取。
* **信号感知**：`beam.Beam.shouldTerminate(env)` 用于在长循环中感知进程被 Supervisor 杀死的信号。
* **内存归属**：必须使用 `beam.Beam.alloc(size)` 申请堆内存，确保内存生命周期挂载在宿主 Actor 身上。
* **错误隔离**：使用 `beam.Beam.raiseException(env, reason)` 抛出可被底座捕获的异常，防止段错误引发全局崩溃。

### 3.2 行为对齐规范 (Behavioral Parity)

1. **信号检查点 (Signal Checkpoints)**：
   在 CPS 变换产生的每一个状态机跳转点（即 `await` 恢复之后），必须插入以下逻辑：
   ```zig
   if (beam.Beam.shouldTerminate(env)) {
       return {stop, <<>>}; // 接收到信号，优雅退出状态机
   }
   ```
2. **内存亲和性 (Memory Affinity)**：
   严禁使用系统原生 `malloc`。所有业务对象空间必须通过 `beam.Beam.alloc` 申请。这保证了在百万级并发下，BEAM 的内存监控能够精准闭环。

---

## 4. 系统指令集规范 (0-99)

| Action ID | 动作名 | 目标网关 | 说明 |
| :--- | :--- | :--- | :--- |
| 1 | WAKE | - | 状态恢复 / 强制唤醒 (防御性编程预留 0 为 NULL) |
| 2 | SLEEP | - | 保存状态至 ETS 并休眠 (JS 内存释放) |
| 3 | STOP | - | 彻底销毁 Agent |
| 10-19 | NET I/O | 99990001 | 网络连接与读写 |
| 20-29 | FS I/O | 99990001 | 文件系统读写 |
| 30-39 | HTTP I/O | 99990001 | 高层 fetch 请求 |
| 40-49 | DB REQ | 99990002 | SQLite 数据库操作 (路由至 DB Worker) |
| 99 | IO_ERROR | - | 统一的异步结果 / 错误回调 (4xx/5xx 状态码) |

**注：业务自定义 Action ID 必须从 100 开始。**

---

## 5. 路由与分发规范

### 5.1 O(1) 静态分发 (Native Dispatch)
CP 编译器必须在 `native_dispatch_nif` 内部根据 ActionID 生成一个静态的 `switch` 块。
* 当收到 `1`，执行 WAKE 逻辑还原 Context。
* 当收到 `100+`，直接编译跳转至对应的业务函数（如 `onMarketUpdate`）。

### 5.2 兜底回退机制 (Fallback)
当 `action_id` 无法匹配任何预定义函数时：
* **首选**：将其放入 Native Agent 的内部 `mailbox` 队列，供 `await receive()` 循环主动拉取。
* **次选**：如果用户定义了 `onMessage(id, payload)`，则在 `switch` 的 `default:` 分支中调用它。

---

## 6. 底座改造清单 (Bun-BEAM Core Actions)

1. **`actor.erl` 路由升级**：已实现对 `native_classes` 的识别和原地 NIF 分发。
2. **物理链接**：Bun 核心已物理链接 RocksDB、zstd、lz4。
3. **分层存储**：已实现 ETS -> DETS/RocksDB 的后台透明换页。
4. **统一注册 API**：
   - 使用 `Beam.Agent.register("ClassName", "./path/lib")` 统一注册，底座自动剥离平台后缀。

## 7. 联调测试标准

* **测试用例**：`test/tiered_storage_test.js`。
* **成功标准**：Native Agent 发起 I/O 或休眠请求后，内存能通过分层存储及时释放，并在数据返回后由 8 字节极速头精准唤醒复活。
