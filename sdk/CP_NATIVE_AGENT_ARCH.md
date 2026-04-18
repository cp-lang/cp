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

为了让 CP 语言生成的 Zig 代码能被 Bun-BEAM 底座加载，CP 编译器必须遵循以下规范。

### 3.1 引入纯净的 `beam.zig` SDK

底座将提供一个 `beam.zig`，屏蔽了繁琐的 C 指针和 `erl_nif.h`，提供了面向对象的 Zig API。**CP 编译器生成的代码必须通过 `const beam = @import("beam.zig");` 来调用底层能力。**

**`beam.zig` 核心 API 概览：**
```zig
pub const Beam = struct {
    // 数据透传与路由
    pub fn getBinary(env: *Env, term: Term) ![]const u8 { ... }
    pub fn makeFastMessage(env: *Env, target_id: u32, action_id: u32, payload: []const u8) !Term { ... }

    // 抢占式调度：协作式让权
    pub fn consumeTimeslice(env: *Env, percent: c_int) bool { ... }
    
    // NIF 安全内存分配 (严禁使用系统原生 malloc/free 以防干扰 BEAM 监控)
    pub fn alloc(size: usize) ?*anyopaque { ... }
    pub fn free(ptr: *anyopaque) void { ... }
};

// CP 语言的运行时上下文 (被 CPS 切断的状态机)
pub const Context = extern struct {
    state_id: u32,
    memory_len: usize,
};

// 预定义生命周期系统 Action
pub const SystemAction = enum(u32) {
    NULL_ACTION = 0,
    WAKE = 1,
    SLEEP = 2,
    STOP = 3,
};
```

### 3.2 内存布局与 Term (ERL_NIF_TERM) 规范
CP 编译器不能去猜测或修改 `Term` 的内存布局。`Term` 是一个 Opaque Word（不透明的系统字），所有的装箱（Boxing）和拆箱（Unboxing）必须使用 SDK 提供的 `Beam.getInt()`, `Beam.makeTuple()` 等方法，绝不可直接通过指针偏移强转。

### 3.3 导出 NIF 生命周期钩子
在编译生成的入口文件中，必须导出底层的 C 初始化结构体。`beam.zig` 提供了一个编译期的宏 `defineNifEntry` 来辅助生成它：

```zig
export const _nif_entry = beam.defineNifEntry(
    "cp_agents_lib", 
    &[_]beam.NifFunc{
        .{ .name = "native_dispatch", .arity = 5, .fptr = native_dispatch_nif, .flags = 0 }
    }, 
    on_load, null, on_unload
);

export fn nif_init() *beam.NifEntry {
    return &_nif_entry;
}
```

### 3.4 Action 路由与生命周期映射规范

Native Agent 依靠 8 字节的二进制头 (`[TargetID:32, ActionID:32]`) 进行极速通信。CP 编译器在生成 Zig 代码时，必须遵循以下路由规范：

1. **系统预留号段 (System Reserve Range)**：
   Action ID `0` 到 `99` 严格保留给 Bun-BEAM 底座的系统生命周期与内部指令（例如 `WAKE=1`, `SLEEP=2`, `STOP=3`）。**业务层自定义的 Enum 必须从 `100` 起步**。

2. **跨层传输的盲区 (Enum Transmission)**：
   BEAM 虚拟机（`actor.erl`）和底座**绝不关心**业务 Action 的具体含义。对底座而言，`101` 仅仅是一个 `u32` 的整形负载。CP 编译器负责将 TS/CP 中的业务枚举编译为 Zig 的 `enum(u32)`，在调用 SDK 的 `makeFastMessage` 时，只需强制转换 `@intFromEnum(MyAction.MARKET_UPDATE)` 即可。

3. **O(1) 静态分发 (Native Dispatch)**：
   与 JS Agent 在运行时利用反射查找 `onMarketUpdate` 字符串不同，Native Agent 拥有极致的性能。CP 编译器必须在 `native_dispatch_nif` 函数内部，根据编译期已知的类和方法，生成一个静态的 `switch (action_id)` 块。
   * 当收到 `101`，直接编译跳转至 `CP_TraderAgent_onMarketUpdate(ctx, payload)`。

4. **兜底回退机制 (Fallback Handling)**：
   当收到的 `action_id` 无法匹配任何预定义的 `switch` case 时：
   * **首选**：将其打包放入该 Agent 的内部 Mailbox（一个 Zig 的 ArrayList），供用户使用 `await receive()` 的循环主动拉取。
   * **兜底**：如果用户定义了一个通用的 `onMessage(action_id: u32, payload: []const u8)`，则编译器应在 `switch` 的 `default:` 分支中调用此方法。

### 3.5 实现抢占式让权 (Cooperative Yielding)

**强制规范：** BEAM 调度器是软实时的，任何单个 NIF 调用如果占用 CPU 超过 1 毫秒，都会引发严重的系统抖动。
* CP 编译器在生成的循环或繁重计算逻辑中，**必须定期调用** `Beam.consumeTimeslice(env, 1)`（例如每消耗相当于 1% 时间片的算力调用一次）。
* 如果该函数返回 `true`（非零），表示当前时间片已耗尽。CP 生成的代码必须**立即中断当前执行，打包当前的所有上下文到 `CPContext`，并向 Erlang 抛出 `{reschedule, CPContext}` 信号。** (底座会负责将其重新排队)。

### 3.6 暴露统一的 NIF 路由分发函数

CP 编译器会将一个项目中的所有 `actor` 类编译为一个统一的动态链接库（例如 `libcp_agents.so`）。该库必须导出一个统一的 C NIF 入口函数，供 Erlang 宿主调用。

**签名要求：**
```zig
// 接收参数: ClassName(String), AgentId(u32), CPContext(Binary), ActionId(u32), Payload(Binary)
export fn native_dispatch_nif(env: ?*erl.ErlNifEnv, argc: c_int, argv: [*c]const erl.ERL_NIF_TERM) callconv(.C) erl.ERL_NIF_TERM {
    // 1. 解析参数
    // 2. 根据 ClassName 和 ActionId (Static Switch) 路由到对应的 CP 类恢复函数
    // 3. 构造返回值：
    //    - 遇到 await 或执行结束，挂起等待新消息: return {yield, NewContextBinary}
    //    - 时间片耗尽，需要重新放入调度队列: return {reschedule, NewContextBinary}
    //    - 业务要求休眠以释放内存 (如写入 ETS): return {sleep, StateBinary}
}
```

---

## 4. 底座改造清单 (Bun-BEAM Core Actions)

为了对接 CP 编译器，Bun-BEAM 底座团队（当前会话）将执行以下改造：

1. **`actor.erl` 升级**：
   - 增加 `Beam.Agent.registerNative` 的处理指令，动态加载 `libcp_agents.so`。
   - 新增 `native_agent_loop/3`，针对 Native Agent 旁路分发，实现 `yield`, `reschedule`, `sleep` 三种返回状态的接管。
2. **`beam.zig` 库提供**：
   - 纯净的 SDK 文件已发布在 `/data/buns/bun/packages/bun-beam/sdk/beam.zig`，供 CP 编译器在生成产物时直接 `import`。
3. **JS/TS 控制面映射**：
   - 扩展 `BeamEngine` API，允许开发者在主脚本中优雅地注册和编排 Native 实体：
     ```typescript
     engine.registerNative("CPTraderAgent", "./build/libcp_agents.so");
     engine.spawn("CPTraderAgent", 999);
     ```

## 5. 联调测试标准

对接成功后，应能运行以下混合测试：
1. **启动 50万个 JS Agent 和 50万个 CP Native Agent**。
2. JS 主控发送一条 **8 字节极速广播** (Action: `MARKET_UPDATE`)。
3. Erlang 调度器瞬间将 50万条消息推入 Bun Worker MPSC 队列，将另 50万条消息推入 `native_agent_loop` 进程。
4. **JS Agent 内存隔离，Native Agent 原地拉起**，混合大军同时处理数据并相互通信。无死锁、无超时、延迟极低。