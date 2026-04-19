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
        I --> J[Execute CP State Machine]
        J --> K{Hit await, sleep, or timeslice?}
        K -->|Yes| L[Serialize new Context]
    end
    
    L -->|Return {yield/sleep/reschedule, Context}| F
    F --> M[Erlang: Save Context to ETS or Wait]
```

---

## 3. CP 编译器的优化与适配要求 (To CP Compiler Devs)

### 3.1 引入纯净的 `beam.zig` SDK

底座提供一个标准化的 SDK 文件 `/packages/bun-beam/sdk/beam.zig`。**CP 编译器生成的代码必须通过 `const beam = @import("beam.zig");` 来调用底层能力。**

**`beam.zig` 核心 API 概览：**
```zig
// 系统标准行为动作枚举 (System Action)
// 必须严格遵守以下 ID 映射，业务自定义 ID 请从 100 开始。
pub const SystemAction = enum(u32) {
    NULL_ACTION = 0,    // 防御性预留
    WAKE = 1,           // 状态恢复/冷启动
    SLEEP = 2,          // 强制休眠，保存状态至 ETS
    STOP = 3,           // 销毁 Agent

    // --- 委托 I/O 指令 (10-89 自动路由至 I/O 网关 99990001) ---
    
    // 网络 I/O (10-19)
    NET_CONNECT_REQ = 10,
    NET_CONNECT_RES = 11,
    NET_WRITE_REQ   = 12,
    NET_READ_REQ    = 13,
    NET_READ_RES    = 14,

    // 文件 I/O (20-29)
    FS_READ_REQ     = 20,
    FS_READ_RES     = 21,
    FS_WRITE_REQ    = 22,
    FS_WRITE_RES    = 23,

    // 高层 HTTP (30-39)
    HTTP_FETCH_REQ  = 30,
    HTTP_FETCH_RES  = 31,

    // --- 统一错误回传 (99) ---
    // Payload 约定：前 4 字节为 HTTP 风格错误码 (如 404, 500)
    IO_ERROR        = 99, 
};
```

### 3.2 异步 I/O 委托模型 (Scalable I/O)

Native Agent 严禁直接调用阻塞式 C API 执行 I/O。必须采用 **“外包模式”**：
1. **发起请求**：Native Agent 构建 8 字节极速头消息，目标 ID 设为 **`99990001`** (系统 I/O 网关)。
2. **挂起等待**：Native Agent 立即执行 `yield`。
3. **执行与唤醒**：底座利用 Bun 的 `io_uring` 异步完成 I/O，并由 Erlang 路由带回 `RES` 指令（或 `99` 错误指令）唤醒 Native Agent。

### 3.3 内存布局与 Term 规范
CP 编译器不能窥探 `Term` 的内存布局。必须使用 SDK 提供的 `Beam.getInt()`, `Beam.makeTuple()` 等方法进行装拆箱。

### 3.4 O(1) 静态分发 (Native Dispatch)
CP 编译器必须在 `native_dispatch_nif` 内部生成一个静态的 `switch (action_id)` 块。

---

## 4. 底座改造清单 (Bun-BEAM Core Actions)

1. **`actor.erl` 路由升级**：
   - 监听 `10-89` 号段指令，自动排队并转发至 ID `99990001`。
   - 维护并发令牌桶（默认 Max=500）。
2. **JS I/O 守护进程**：
   - 运行于 `99990001` 的 `IOGatewayWorker`。
   - 封装 Bun 的 `fetch` 和 `Bun.file` 能力，支持类 HTTP 错误码返回。
3. **统一注册 API**：
   - 使用 `Beam.Agent.register("ClassName", "./path/lib")` 统一注册。

## 5. 联调测试标准

* **测试用例**：`test/unified_io_gateway.js`。
* **成功标准**：Native Agent 发起文件读取请求后能瞬间进入 `SLEEP`，并在数据准备好后由 8 字节头消息精准唤醒，整个过程无任何 JSON 参与。
