# CP 语言设计规范与技术白皮书 (The CP Language Specification)

**版本**：v0.7.0  
**定级**：工业级官方参考规范手册与底层架构指南 (The Definitive Reference Manual)  
**定位**：TypeScript 的直觉表达力 + Rust 的内存安全 + Zig 的极致底层掌控力与无 GC 暴力并发，专为高安全、高并发、高性能、小体积、低内存与AI、百万级AI Agent并发而设计的系统级编程语言。
**特点**：上手像TypeScript，安全模型对标Rust，底层能力继承Zig。既能写优雅的业务代码，又能下到裸机、系统、高性能服务，不用在“好写”和“安全高性能”之间二选一。同时在AI与AI智能体编程方面有独特优势。

---

## CP 语言规范 · 开篇

```text
  ██████╗ ██████╗
 ██╔════╝ ██╔══██╗   CP : Concurrent Plus
 ██║      ██████╔╝   The AI Agent & Native Systems Language
 ██║      ██╔═══╝    [ TS-Syntax • Rust-Safety • Zig-Performance • Nest-Actor ]
 ╚██████╗ ██║
  ╚═════╝ ╚═╝
```

> CP (Concurrent Plus)：一门基于Zig、拥有Rust级内存安全、语法贴近TypeScript/ES6的全平台编译型系统语言，兼顾易用、安全与底层高性能, 在AI、AI Agent、高安全、高并发、高性能、小体积、低内存场景独具优势。

CP 为AI以及海量（百万级以上）AI Agent 并发运行提供了专门支持，提供编译选项（-nest）编译为智能体容器动态加载运行，抢占式协程调度，避免单智能体死循环影响其他智能体运行。

CP 是 C/C++ 在并发与智能体时代的终极演进：TS 直觉 + Rust 安全 + Zig 底层 + Nest 并发。

## 一、CP 语言哲学、核心愿景与全景场景矩阵

### 1.1 核心设计定位

CP = AI（智能体） + C/C++（资源） + Zig（性能） + Rust（安全） + Typescript/ES6（易用） + Java（功能）。

CP 融汇了当今三大现代语言流派的顶峰设计理念：

- **TypeScript 的高生产力与直觉语法**：摒弃冗余的样板代码，结构清晰，贴合现代工程直觉；
- **Rust 的无畏并发与内存安全**：在编译期通过所有权（Ownership）与非词法生命周期（NLL）借用检查，彻底杜绝悬垂指针、Double-Free 与数据竞态，避免内存泄露与程序崩溃；
- **Zig 的无隐藏控制流与底层控制力**：无障碍C/C++互操作，彻底废除垃圾回收器（No GC），裸机能力，将控制权完全还给硬件，全平台编译机器码直接运行于各大操作系统，无虚拟机，具备百微秒级的启动与纳秒级出让性能。

CP 底层基于 Zig 构建，继承 Zig 强大的编译期计算、全平台交叉编译、无障碍C 互操作与无运行时能力；借鉴 Rust 的所有权与借用检查保障默认内存安全；同时采用贴近 TypeScript/ES6 的简洁语法，容易上手，对JavaScript程序员友好，并提供解构、模式匹配、管道、注解与装饰器等现代语法特性。

CP 支持渐进式开发：默认类似Rust编译器安全检查，通过 `unsafe` 块与内嵌 `zig{}` 代码块提供底层逃逸出口。没有虚拟机、没有垃圾回收，编译产出静态单二进制文件直接运行于操作系统。适合高安全、高性能、高并发后端，CLI 工具、嵌入式固件、Wasm、智能体程序与系统组件开发。目标是让开发者不用在「好写」、「内存安全」、「底层高性能」三者之间做取舍。

---

### 1.2 CP/Rust/Zig/Go/TypeScript 精简对比表

#### 1.2.1 对比竞品，CP最独特的差异化标签

| 语言    | 核心短板                                     | CP对应的差异化                        |
| ------- | -------------------------------------------- | ------------------------------------- |
| Rust    | 陡峭学习曲线、生命周期难写、语法笨重         | TS风格简洁语法，同样所有权安全        |
| Go      | GC，无法极致低延迟，缺少强内存安全           | 无GC，借用安全，单静态二进制          |
| Zig     | 默认没有借用检查，容易内存越界，原生语法简陋 | 上层自带借用安全，保留全部Zig底层能力 |
| C++     | 坑多、未定义行为复杂                         | 安全默认，unsafe显式隔离              |
| C#/Java | 虚拟机、GC、部署笨重                         | 原生编译，轻量无runtime               |

#### 1.2.2 对比竞品，CP优势

| 项目         | CP                                            | Rust                           | Zig                              | Go                               | TypeScript                 |
| ------------ | --------------------------------------------- | ------------------------------ | -------------------------------- | -------------------------------- | -------------------------- |
| 编译模型     | 编译到 Zig，原生 AOT                          | AOT 原生编译                   | AOT 原生编译                     | AOT，带轻量 runtime              | 转译到 JS，解释执行        |
| 内存管理     | 所有权+借用检查，无GC；显式unsafe             | 所有权+借用检查，无GC          | 手动管理，无默认借用检查         | GC + 逃逸分析                    | JS GC                      |
| 语法风格     | TS/ES6 风，解构、模式匹配、管道               | 自成体系，生命周期、trait 较重 | 极简底层风格，错误类型`!T`       | 简约，语法克制                   | JS 超集，类型标注          |
| 编译期能力   | 继承 Zig comptime，编译期函数、元编程         | 宏 + const 编译期能力          | 原生 comptime，强项              | 编译期能力弱                     | 只有类型层面编译检查       |
| C 互操作     | 直接复用 Zig C 互操作                         | 支持，需要封装                 | 原生一流支持                     | cgo，有开销                      | 通过 JS FFI                |
| 二进制产物   | 静态单文件，体积小、无依赖                    | 静态单文件，体积偏大           | 静态单文件，体积很小             | 静态单文件，runtime 嵌入         | JS 文本                    |
| 安全边界     | 默认安全；`unsafe` / `zig{}` 作为逃逸口       | 默认安全；`unsafe` 逃逸口      | 默认不安全，靠开发者自律         | 内存安全（GC），数据竞争依然存在 | 内存安全，类型仅编译期检查 |
| 核心优势     | 上手简单 + 内存安全 + 底层全能                | 生态成熟，安全体系完善         | 编译速度快、交叉编译一流、无包袱 | 并发模型简单，工程化成熟         | 前端生态，学习门槛低       |
| 主要短板     | 新生语言，生态尚在建设                        | 学习曲线陡峭，生命周期难       | 缺少内置借用检查，语法现代性弱   | GC抖动，缺少强内存安全           | 不适合高性能系统、裸机     |
| 典型主打场景 | Agent智能体、系统工具、后端服务、嵌入式、Wasm | 操作系统、库、高性能服务       | 底层工具、固件、C 替换           | 云原生后端、微服务               | Web 前端、Node.js 业务     |

---

### 1.3 为AI而生：CP在`AI/AI智能体`领域的独到优势与设计哲学

#### 1.3.1 五大主流语言在 AI 智能体领域的“物理死穴”对比

当今主流的 AI 智能体开发几乎被 **Python**（LangChain/AutoGen/CrewAI/PyTorch）和 **TypeScript**（ElizaOS/LangChain.js）统治，底层系统与基础设施由 **Rust/C++** 扛鼎，网络高并发常用 **Go**，而 **Zig** 提供了极致的底层掌控。

然而，**当 AI 智能体从“单体玩具（Single Agent / ChatBot）”演进到“群体智能（Multi-Agent Swarm / 百万级并发智能体仿真）”时，现有全部编程语言都会撞上一堵厚重的物理高墙**：

| 语言           | 当前优势                                   | 在海量 AI Agent 并发场景下的致命痛点                                                                                                                                                                                                     |
| :------------- | :----------------------------------------- | :--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Python**     | AI 生态第一，库极其丰富，胶水能力极强      | **单机并发极低，内存极度膨胀**。GIL 锁死多核；每个线程消耗 ~8MB 栈；`asyncio` 是单线程协作，任何 Agent 稍微执行一点向量距离计算或复杂 JSON 解析就会**卡死整个事件循环**；一个 Agent 动辄消耗数十 MB 内存，单机根本无法支撑上万个 Agent。 |
| **TypeScript** | 事件驱动，异步 I/O 直觉，JSON 零阻抗       | **单线程事件循环阻塞，无原生多核并行**。海量 Agent 在进行 Prompt 模板拼接、Tool Schema 校验、正则清洗时会持续占用主线程，导致高并发网络 I/O 延迟剧增；对象内存占用高且受限于 V8 堆内存上限。                                             |
| **Go**         | Goroutine 极轻（~2KB 栈），网络高并发极强  | **Stop-The-World (STW) 垃圾回收噩梦**。当 100 万个 Agent 各自持有庞大的对话上下文、历史记忆切片时，堆内存中散落着数亿个对象指针。Go 的三色标记 GC 每隔数秒必须遍历扫描全堆，产生严重的 **GC 停顿与 CPU 算力白白空转**。                  |
| **Rust**       | 内存极其安全，零成本抽象，无 GC，性能霸权  | **编写业务级 Agent 极其痛苦，生命周期与并发心智地狱**。跨越异步 `await` 的状态管理需要满屏的 `Arc<Mutex<T>>`、`Box<dyn Future>` 与 `Pin`；Actor 框架并非一等公民；且异步任务默认**无法被抢占**，长思考任务仍会饿死并发队列。             |
| **Zig**        | 无隐藏控制流，极致纯粹，直接掌控内存与硬件 | **缺乏高级抽象与并发信箱原语**。纯手工内存管理（Allocator 手动申请与释放）在编写极其多变、动态交互的 Agent 工具链（Tool Calling）和反思链路时，极易诱发 UAF 内存泄漏，开发心智负担过重。                                                 |

---

#### 1.3.2 CP 语言在 AI 与智能体领域的六大独到优势与核心架构解析

CP-Nest 确立了**“脑（调度）、神经（感知）、肌肉（执行）、血液（数据）”**四位一体的智能体专有架构：

```mermaid
flowchart LR
    subgraph CP_Nest_Architecture ["CP-Nest 百万级 AI 智能体终极运行时"]
        Brain["🧠 脑：Nest 抢占式调度内核<br/>(Reductions 计数 / 绝对杜绝长思考任务饿死)"]
        Nerve["⚡ 神经：高性能异步 I/O 事件循环<br/>(流式 SSE / WebSocket / Tool Calling 零阻塞)"]
        Muscle["💪 肌肉：Zig Native CPS 状态机<br/>(彻底无 GC / 百微秒级启动 / 上下文纳秒级出让)"]
        Blood["🩸 血液：零拷贝 SharedTensor 全局共享堆<br/>(向量、模型权重、大文本切片原子引用跨信箱穿透)"]
    end
    Brain --> Nerve
    Nerve --> Muscle
    Muscle --> Blood
```

##### 1. 独创“双轨派生模型”：`spawn`（协程池）vs `spawn.cpu`（Worker 物理线程池）

- **传统困境**：Python/TS 遇到本地 Embedding 余弦计算或重度正则匹配就会卡死 I/O；Go/Rust 则需要开发者手动设计复杂的 Worker Pool 和 Channel 路由。
- **CP 独到设计**：在语言级提供双轨调度：
  - **`spawn(async () => ...)`**：派生到**抢占式协程池**，仅消耗百字节级内存，支撑数百万 Agent 的 I/O 监听、工具流式调用、网络信箱收发；
  - **`spawn.cpu(() => ...)`**：一键将繁重的小模型本地推理、向量检索、矩阵转置直接派发到底层的 **OS 物理线程池**，榨干 CPU 多核多线程算力，**完全不占用、不阻塞协程调度器**！

##### 2. 彻底无 GC（No GC）+ CPS 状态机切分：终结百万 Agent 内存爆炸

- **传统困境**：Go/TS/Python 的垃圾回收器是百万级 Agent 的天敌。100 万个 Agent 挂起等待 LLM 回复时，堆内存扫描开销就会击垮系统。
- **CP 独到设计**：
  - CP 语言在编译器前端彻底抛弃传统运行时调用栈，每个带 `async` 的函数在编译期被切分为精简的 **CPS 状态机结构体**；
  - 当 Agent 执行 `await llm.chat()` 挂起时，它直接执行物理 `return` 归还操作系统线程，其挂起状态仅占用一个极小的 Context 结构体（百字节级）；
  - **全生命周期无 GC 扫描开销**，通过作用域边界自动织入 `defer` / `errdefer` 保证零内存泄漏，单台服务器轻松驻留数百万个活跃 Agent！

##### 3. 血液流淌：`SharedTensor` 全局共享堆与零拷贝跨 Agent 通信

- **传统困境**：在 Python/Node 多进程架构中，Agent A 把一段 100k Token 的长上下文或一个 4096 维的向量 Embedding 发送给 Agent B，必须经过低效的 IPC 序列化与内存深拷贝，系统总线迅速被撑爆。
- **CP 独到设计**：
  - **小数据（指令、状态标记）**：走轻量信箱物理值拷贝；
  - **大数据（Prompt 上下文、Embedding 向量、模型权重）**：分配于全局共享堆 `SharedTensor` / `Refc Binaries`；
  - 智能体间通过信箱传递消息时，**仅传递指针与 `std.atomic` 原子引用计数**，数据零拷贝、零延迟瞬间到达目标 Agent！配合 CP 的 `move` 移动语义，直接保证了单写者所有权安全。

##### 4. 抢占式调度（Reductions 时间片）：杜绝复杂 Prompt / CoT 思考饿死集群

- **传统困境**：在 Node.js 或 Python 中，如果某个 Agent 正在进行长达几秒的重度思维链（Chain of Thought）文本处理或死循环，同一个事件循环线程上的其余所有 Agent 全部陷入僵死。
- **CP 独到设计**：
  - CP 继承了 BEAM/Nest 虚拟机的抢占哲学，以虚拟机指令减少计数（Reductions）作为时间片基准；
  - 任何 Agent 的计算任务一旦达到设定阈值（如 2000 个 Reductions），无论其逻辑是否结束，调度器强制将其暂停并压入队列尾部，将执行权出让给等待流式响应的其他 Agent，**杜绝单点饥饿**。

##### 5. 双轨对象模型 + 原生 3-State 错误体系：天生为 LLM Tool Calling 而生

- **传统困境**：AI 智能体有 70% 的代码都在跟 LLM 吐出的动态 JSON、Function Call 参数、网络断连容错打交道。强类型语言（Rust/C++）写反序列化极其繁琐，动态语言（Python/JS）又缺乏类型检查极易运行时崩溃。
- **CP 独到设计**：
  - **双轨对象模型**：内部计算走**静态匿名结构体 `{ x, y }`**，零额外开销对齐硬件；对接 LLM 动态返回走**动态字典对象 `#{ key: val }`**，零摩擦任意增删改查；
  - **对象解构 `...rest` 强类型提取**：如 `let { tool_name, ...args } = llm_response;`，直接解耦核心参数与动态扩展参数；
  - **原生 3-State 错误体系（`! E`, `T?`, `?`, `? default`）**：彻底消灭嵌套的 `try/catch` 和繁重的 `Result` 胶水装箱。Tool 超时、LLM 幻觉产生格式错误时，通过 `?` 自动冒泡或 `? default` 优雅降级，让 Agent 工具链代码极致优雅且绝对坚固。

##### 6. Crash-Proof 故障隔离（“任其崩溃”哲学）：工业级多 Agent 容灾

- **传统困境**：在一个由 100 个 Agent 协作的任务中，某一个 Agent 因为网络波动、未预期输入或显存越界崩溃抛出未捕获异常，很容易引发整个服务进程崩溃。
- **CP 独到设计**：
  - 每个 Agent 拥有独立的隔离边界。某个 Agent 崩溃死亡时，仅其自身的 Actor 实例被销毁，其持有的所有底层句柄与显存张量由底层的 **NIF Resource Objects** + `release()` 自动释放；
  - 其余并发 Agent 完全不受干扰，宿主监控树（Supervisor Tree）会自动感知死亡事件并按策略执行毫秒级热拉起重启。

---

#### 1.3.3 典型场景落地优势实测直观对比

| 真实业务场景                                           | 传统语言方案 (Python / Node / Go / Rust)                                                                    | CP-Nest 解决方案优势                                                                                                 |
| :----------------------------------------------------- | :---------------------------------------------------------------------------------------------------------- | :------------------------------------------------------------------------------------------------------------------- |
| **场景一：万人同域/NPC 智能体小镇仿真 (Virtual Town)** | **Python/Node 彻底趴窝**（内存数天崩溃，单机难撑 500 个）；**Go 频遭 GC 卡顿**；**Rust 开发周期长达数月**。 | **单机并发支撑 100 万 Agent**。每个 Agent 挂起态仅百字节，零 GC 运行，抢占调度杜绝卡顿，单机即可模拟庞大的数字社会。 |
| **场景二：多智能体实时金融高频量化决策群**             | **Python 延迟在几十毫秒级**，受限于 GIL 和 IPC 拷贝，无法应对毫秒级盘口。                                   | **微秒级信箱吞吐**。CPS 状态机配合 `SharedTensor` 零拷贝穿透，Agent 决策与风控判定在纳秒级完成。                     |
| **场景三：边缘端嵌入式 / 本地小型化 Agent 设备**       | **Python 无法运行在无环境边缘设备**；Node 运行时体积高达 80MB+。                                            | **`cap build -s` 吐出独立静态二进制**（仅数 MB），零外部依赖，极速在边缘工控机/手机上跑满 Agent 群。                 |

> **Python 赋予了 AI 思考的模型，TypeScript 带来了连接前端的敏捷，Rust 构筑了底层算力的基石。**  
> **而 CP 语言，则是专为让这百万级智能体在同一台物理机器上“群智涌现、永不休眠、安全共生”而诞生的原生智能体母语。**

### 1.4 高性能服务后端（替代部分 Go/C++/C/Rust/Zig）

痛点现状：

- Go：GC停顿、缺少严格所有权，高并发下大量堆分配、内存抖动；没有强编译期检查；
- Rust：学习曲线陡峭，语法重、生命周期标注折磨新手，很多人喜欢它安全但是写着累；
- C++：内存坑多、各种未定义行为、编译慢；
- Java/C#：JVM虚拟机运行，依赖重、GC，很难做低延迟、资源紧的服务。
- Typescript/ES6：V8虚拟机解释执行，运行效率低，GC 内存占用大，容易内存泄露，没有安全检查，程序容易崩溃。
- Zig：没有内存安全检查，容易内存泄露，著名的Bun.js（Claude Cli底层） 就是因为内存泄露问题，百万级源代码从Zig迁移到Rust。

CP优势：

- 所有权借用安全，**没有GC**，和Rust一样内存安全，但是语法是TS风格，好读好写；
- 编译到Zig，天然跨平台、小体积单二进制，不需要虚拟机、不需要runtime，相当于Zig的升级版，可完全替代Zig；
- 直接复用Zig的C interop，调用C/C++库零成本，可替代C/C++做嵌入式、系统级编程；
- 自带管道`|>`、ES6风格解构、清晰的模式匹配，业务代码写起来比Rust清爽很多。熟悉JavaScript、Typescript程序员直接上手。

适合：网关、消息中间件、时序数据服务、高频RPC服务、代理服务。
吸引人群：想写无GC低延迟服务，但不想啃Rust生命周期、讨厌C++坑的后端。

### 1.5 CLI工具、开发工具链（替代 Go / Zig）

现在大量CLI是Go写的（比如各种cli、脚手架、代理工具），Go的优势是单文件打包，缺点GC；Zig本身CLI能力很强，但Zig原生语法比较硬核、模板和类型写法不友好。

CP优势：

- 编译产物和Zig一样：静态单文件、无依赖、体积小；
- comptime编译期函数、元编程能力继承Zig；
- 装饰器、注解、宏体系更现代，TS风格，写命令行参数解析、代码生成器、linter、脚手架比原生Zig舒服；
- 所有权安全，处理流、大文件、管道数据不容易内存泄漏。

适合：代码生成器、静态分析工具、CLI运维工具、文件处理工具、自定义构建工具。
吸引人群：Zig爱好者、工具作者，厌倦Go GC、又觉得Zig原生语法不够顺手的人。

### 1.6 嵌入式、裸机、RTOS（Zig原生主场，CP是加分版）

Zig这一块本身就很强：无runtime、无GC、可以写裸机、MCU、RTOS，不需要操作系统依赖。
Rust嵌入式：生态不错，但`#[no_std]`、堆、alloc、生命周期在嵌入式里非常折磨；C：内存灾难。

CP优势：

- 底层直接Zig，天然支持`no_std`、裸机、MCU；
- 上层带Rust级别的借用检查，**在裸机环境下也能拿到内存安全**，这是C、原生Zig都没有的；
- 语法简单，嵌入式工程师很多熟悉JS/TS，上手门槛远低于Rust；
- 可以随时切入`zig{ ... }`内嵌块，碰到必须绕开安全检查的底层寄存器操作直接写Zig。

适合：单片机固件、物联网设备、RTOS任务、硬件驱动。
吸引人群：嵌入式开发者，想要内存安全，但不想承受Rust嵌入式的沉重负担。

### 1.7 游戏引擎底层、高性能引擎脚本/绑定层（替代 C++、部分Rust）

游戏领域现状：C++为主，坑极多；Rust尝试很多，但引擎集成成本高、和传统C++引擎互操作麻烦；Lua/TS是脚本，性能不够。

CP优势：

- 无GC，可控内存；所有权模型避免常见野指针、资源释放漏；
- 无缝C互操作，可以接入现有C/C++游戏引擎；
- 语法TS风，策划、客户端前端背景的人更容易读懂；
- comptime适合做引擎层的编译期配置、资源序列化、静态类型反射；
- unsafe + 内嵌zig块给引擎底层留后门，必要时可以直接操作内存。

适合：引擎工具、资源管线、游戏逻辑高性能模块、引擎插件系统。
吸引人群：独立游戏开发者，厌倦C++内存bug，又觉得Rust太重的团队。

### 1.8 跨平台系统软件、Daemon、系统组件（替代C / C++）

比如跨平台后台驻留程序、磁盘工具、网络栈组件、监控采集器。
C的问题：全靠人小心，内存错误高危；Rust写没问题，但招人、阅读成本高。

CP卖点：

- 内存安全是默认；
- 编译出来和Zig一样，原生支持Linux/macOS/Windows，交叉编译一键搞定；
- 二进制小，部署简单；
- 可以渐进式：先用CP写业务逻辑，底层必要组件直接内嵌Zig。

### 1.9 高性能WebAssembly（Wasm）

Zig编译Wasm成熟，Rust Wasm生态强但包偏大、编译慢；TS转Wasm（AssemblyScript）类型安全弱。

CP优势：

- 编译目标Wasm复用Zig后端；
- 所有权模型，Wasm内部内存安全；
- 语法接近TS，前端开发者迁移成本极低；
- 可以写高性能前端计算模块：图像处理、加密、解析器，比AssemblyScript更严谨。

适合：浏览器内重计算插件、Wasm服务组件。
吸引人群：前端团队，想写高性能Wasm，不想学Rust。

### 1.10 教育、入门系统级编程（这个是很特别的差异化赛道）

这是一个很容易被忽略、但是潜力很大的场景：
现在教系统编程两条路：

- C/C++：一上来就是内存错误、野指针，新手很容易劝退；
- Rust：理念很好，但语法门槛太高，不友好、不优雅，学生卡在生命周期。
- JavaScript/Typescript：解释执行脚本，语言能力弱，浏览器页面或V8虚拟机运行，无法做系统级编程。
- Python：解释执行脚本，语言能力弱，与标准编程语言差异大，不适合教学。

CP：
✅ 专为AI而生，AI时代必须掌握AI及AI智能体编程。
✅ 语法像TS（绝大多数程序员都懂TS），易学易用；
✅ 同时**正经教所有权、借用、内存安全这些核心概念**；
✅ 不是玩具语言，编译到Zig，通过Zig编译为全平台二进制机器码，能做系统级编程；
✅ 与操作系统底层以及庞大的C/C++库无障碍交互，继承人类宝贵资源；
✅ 可以循序渐进：先写简单业务，再接触所有权借用，最后再接触unsafe/zig裸指针内嵌块。

> 一句话：它可以作为「系统编程入门语言」—— 既能理解现代内存安全模型，又不像Rust那样一上来就把人挡住。
> 学会了CP，等于学会了 Typescript、Rust、Zig、C、C++。
> 这个场景非常吸引高校、培训、自学人群。

### 1.11 现有Zig项目的上层迁移层（Zig生态内部的升级选择）

很多Zig开发者喜欢Zig的编译模型、comptime、C interop、交叉编译，但不满意Zig语法：

- Zig的错误处理`!T`、if err、payload写法比较啰嗦；
- 缺少优雅解构、模式匹配、管道、装饰器注解等现代语言能力；
- 没有内置的所有权借用检查（Zig是裸指针，安全靠自己），容易内存泄露、程序崩溃。

CP对他们来说 = **保留Zig全部能力，叠加Rust安全检查 + TS风格舒服语法**。
他们可以增量迁移：CP本身就是编译为Zig代码的，因此可以一部分模块用CP写，CP中也能通过zig{}编写zig代码，底层核心zig块不动，混合共存。

---

### 1.12 边界：哪些场景CP不适合主推

- 重型企业业务（Spring、.NET那套CRUD）：生态成熟度暂时打不过Java/C#，不是首发主攻；
- 纯前端页面开发：没必要，就是TS主场；
- 重度已有Rust生态绑定（比如很多成熟Rust库）：需要时间积累绑定层。

---

## 二、`cap` 统一工具链与模块生态体系 (The `cap` All-in-One Toolchain & Ecosystem)

### 2.1 架构愿景：CAP (Concurrent Application Plus) 核心定位与品牌标识

```text
  ██████╗      ██╗     ██████╗
 ██╔════╝     ████╗    ██╔══██╗   CAP : Concurrent Application Plus
 ██║         ██╔═██╗   ██████╔╝   The All-in-One AI Agent & Native Systems Toolchain
 ██║        ██╔╝ ╚██╗  ██╔═══╝    [ Package Manager • Compiler • Runner • Test • Bench ]
 ╚██████╗  ██╔╝   ╚██╗ ██║        ^ (Upward Arrow) Linking C & P as the Letter 'A' (Application)
  ╚═════╝  ╚═╝     ╚═╝ ╚═╝
```

#### 1. 深度对标 Bun.js 的全合一哲学 (All-in-One Philosophy)

**Bun.js 之于 TypeScript/JavaScript，犹如 CAP 之于 CP。**

在传统系统编程与 AI 框架中，开发者饱受工具链碎片化之苦：

- 在 **JavaScript/TypeScript** 生态中，开发者必须痛苦组合 `npm`/`pnpm` + `vite`/`esbuild` + `jest`/`vitest` + `tsc` + `tsx`，并在数以万计的小文件与深度嵌套依赖中挣扎；
- 在 **Rust** 生态中，开发者需要在 `cargo`, `rustc`, `rustup`, `clippy`, `cargo-watch` 之间频繁切换，每次重新编译耗时漫长；
- 在 **Python** 生态中，`pip`, `poetry`, `conda`, `venv` 互不兼容，环境污染与依赖版本冲突屡见不鲜。

**CAP 彻底终结了这一工程痛点**：

- `cap` 是一个采用 Zig 原生编写的高性能单独立二进制程序（Single Static Executable），体积小、零系统外部依赖、冷启动仅数毫秒；
- **全生态归一**：开发者使用 CP 语言的主要入口就是 `cap` 命令，无论是新建项目、安装依赖、日常秒级调试运行、单元测试、矩阵交叉编译发布，均由 `cap` 一站式全盘接管；
- **生态生命线**：CP 智能体与原生库的繁荣分发完全依托 `cap`，它是开发者与底层硬件、网络、智能体集群交互的核心指挥塔。

#### 2. 品牌寓意：CP 与 CAP 的血脉同构 (Concurrent Plus & Concurrent Application Plus)

- **语言本体：CP (Concurrent Plus)**：
  - **C (Concurrent)**：专为百万级 AI Agent 与高并发系统设计的无 GC、抢占式调度并发内核；
  - **P (Plus)**：对 C++ 历史演进的正统继承与全维度增益（TypeScript 直觉语法 + Rust 借用所有权内存安全 + Zig 零成本裸机控制力）。
- **工具链体系：CAP (Concurrent Application Plus)**：
  - **在 `C` 与 `P` 之间注入字母 `A` (`Application`)**：视觉上由向上腾飞的箭头符号 **`^`** 精准呈现，直观牢固地链接语言内核（C）与生态增益（P），完整拼写出 **`CAP`**；
  - **代表 CAP 绝非“C 的 P 次方”**：它象征着从“语言规范（CP）”通过“应用工程赋能（Application / `^` 腾飞之矢）”迈向全场景生产级交付（CAP）；
  - **C (Concurrent)**：并发调度底座与多核算力引擎；
  - **A / ^ (Application & Ascend)**：全场景应用支撑（涵盖 AI Agent 集群、高性能服务器、系统 CLI、嵌入式固件等全部应用形态），寓意算力与开发效率的向上跃升；
  - **P (Plus)**：跨平台构建交付矩阵、统一模块依赖拓扑与自动化测试评测生态。

---

### 2.2 `cap` CLI 全指令全景矩阵

`cap` 提供了比肩 `bun`、超越传统 `cargo`/`npm` 的极速统一指令行体系：

| 用户命令                        | 底层优化模式 | 功能定位与行为说明                                                                                                  | 实装状态       |
| :------------------------------ | :----------- | :------------------------------------------------------------------------------------------------------------------ | :------------- |
| `cap <file/dir>`                | Debug        | **日常极速运行**：直接运行 `.cp` 脚本或目录，百毫秒级 JIT 式极速构建并在内存/本地直接运行。                         | **已全面实装** |
| `cap run <file/dir>`            | Debug        | **显式运行模式**：与 `cap <file>` 等价，支持透传参数给用户程序。                                                    | **已全面实装** |
| `cap build [dir]`               | ReleaseSafe  | **当前平台安全发布**：支持增量构建检查，仅编译变动文件，输出独立 Native 可执行文件至 `bin/`。                       | **已全面实装** |
| `cap build -B` (或 `--force`)   | ReleaseSafe  | **强制全量编译**：忽略 mtime 与缓存状态，强制重新生成中间 Zig 代码与 Native 独立二进制。                            | **已全面实装** |
| `cap build -r` (或 `--release`) | ReleaseSafe  | **终极矩阵跨平台构建**：单机一口气编译输出 Windows、macOS-ARM、macOS-Intel、Linux-x64、Linux-ARM 全套生产级二进制！ | **已全面实装** |
| `cap build -f`                  | ReleaseFast  | **极限计算性能优化**：榨干硬件算力，消除一切额外运行时检查（可结合 `-r`：`cap build -r -f`）。                      | **已全面实装** |
| `cap build -s`                  | ReleaseSmall | **极小体积构建**：极限剔除调试信息与体积优化，专用于边缘计算、嵌入式设备与轻量容器（可结合 `-r`）。                 | **已全面实装** |
| `cap build --exe`               | -            | （默认行为）输出独立的系统原生可执行程序。                                                                          | **已全面实装** |
| `cap build --obj`               | -            | 输出底层的 `.o` 机器码对象文件，供外部链接。                                                                        | **已全面实装** |
| `cap build --lib`               | -            | 输出系统级静态库 `.a`，供 C/C++ 或宿主语言无缝桥接。                                                                | **已全面实装** |
| `cap build -b` (或 `--beam`)    | ReleaseFast  | **Nest 智能体容器动态库**：编译为导出标准 C-ABI 的纯状态机动态库（`.so`/`.dll`），供 Nest 容器原生动态热加载。      | **已全面实装** |
| `cap clean`                     | -            | **构建缓存清理**：一键安全清理 `.cap/zig/`、`.cap/zig-cache/`、`.cap/obj/` 与输出目录 `bin/`。                      | **已全面实装** |
| `cap add <pkg>`                 | -            | **项目级依赖安装**：极速拉取并软链接到 `cap_modules/`，自动将依赖与 Semver 写入 `cap.json`。                        | **已全面实装** |
| `cap add -g <pkg>`              | -            | **全局依赖注入**：将公共高频依赖安装到 `~/.cap/global/cap_modules/`，作为当前系统所有项目的公共兜底池。             | **已全面实装** |
| `cap test [dir]`                | Debug        | **单元与集成测试**：递归扫描并执行测试用例，结果输出至 `bin/test/`，与业务产物物理隔离。                            | **已全面实装** |
| `cap init [template]`           | -            | **开箱即用工程脚手架**：快速初始化模板工程（支持 `app` 独立应用、`agent` 智能体服务、`lib` 库模块）。               | **已全面实装** |
| `cap remove <pkg>`              | -            | **依赖卸载**：清理指定依赖项及其孤立软链接，同步更新 `cap.json` 清单。                                              | **已全面实装** |
| `cap update [pkg]`              | -            | **语义化版本升级**：按照 `^`/`~` 规则安全升级依赖包至符合 Semver 规范的最新版本。                                   | **已全面实装** |
| `cap bench <file.cp>`           | ReleaseFast  | **微秒级基准测试**：纳秒级高精度计时度量状态机、闭包与数据吞吐耗时。                                                | **已全面实装** |
| `cap fmt`                       | -            | **代码格式化**：原生百毫秒级格式化全部 `.cp` 源代码，保持工业级统一风格。                                           | **规划实现**   |
| `cap lint`                      | -            | **静态分析与安全审计**：扫描生命周期逃逸、借用冲突与性能反模式。                                                    | **规划实现**   |
| `cap publish`                   | -            | **去中心化生态发布**：将编译好的原生包或源码包签名并发布至 CP 官方 Registry。                                       | **规划实现**   |
| `cap cache clean`               | -            | **全局缓存治理**：检查并清理 `~/.cap/cache/` 下的历史过期缓存。                                                     | **规划实现**   |

---

### 2.3 全局缓存与软链接零拷贝注入机制 (Symlink & Global Cache)

为了彻底终结 Node.js `node_modules` 深度嵌套与硬盘吞噬（磁盘黑洞），`cap` 借鉴了 `bun` 与 `pnpm` 的先进哲学，自研了**全局单实例物理存储 + 局部原子软链接注入机制**：

```mermaid
flowchart TD
    Registry["📦 CP Registry / Git 远程仓库"] -->|极速并发拉取 / 构建| GlobalCache["🗄️ 全局中心物理存储<br/>(~/.cap/cache/<pkg>@<ver>/)"]
    GlobalCache -->|原子符号软链接 (Symlink) 零拷贝注入| ProjectA["📁 项目 A: ./cap_modules/<pkg>"]
    GlobalCache -->|原子符号软链接 (Symlink) 零拷贝注入| ProjectB["📁 项目 B: ./cap_modules/<pkg>"]
    GlobalCache -->|cap add -g 全局注入| GlobalModules["🌐 全局公共依赖池<br/>(~/.cap/global/cap_modules/<pkg>)"]
```

#### 核心优势：

1. **零重复磁盘占用**：同版本依赖在整台操作系统中物理磁盘上永远只保留一份，无论开启 100 个项目，都不会产生冗余存储；
2. **毫秒级依赖还原**：`cap install` 在解析依赖树后，仅需在 `cap_modules/` 中创建轻量级符号链接，即使面对数百个依赖，整个安装过程通常在 200 毫秒内瞬间完成；
3. **全局依赖与无感知免装运行**：使用 `cap add -g <pkg>` 安装的高频库，任意目录下的独立 `.cp` 脚本均可直接 `import` 引用（由编译器自动攀升向上寻找兜底），打造类似 Python 脚本随写随跑、却享有 Rust 级别性能与安全的开发体验。

---

### 2.4 `cap.json` 工程清单权威规范

#### 1. `cap.json` 示例

每个 CP 工程均以根目录下的 `cap.json` 作为唯一工程元数据清单与构建指挥塔：

```json
{
  "name": "nexus-agent",
  "version": "1.0.0",
  "description": "High-concurrency distributed AI Agent built with CP & Nest",
  "author": "Antigravity Team",
  "license": "Apache-2.0",
  "main": "src/main.cp",
  "mode": "parallel", // 可选: "parallel" (多核工作窃取) | "single" (单线程事件循环)
  "scripts": {
    "start": "cap src/main.cp",
    "build:prod": "cap build -r -f",
    "test": "cap test",
    "bench": "cap bench"
  },
  "dependencies": {
    "tensor-math": "^0.2.1",
    "http-router": "~1.0.0",
    "agent-actor": "1.2.0"
  },
  "dev_dependencies": {
    "test-kit": "^0.1.0"
  },
  "workspaces": ["packages/*"],
  "build": {
    "opt": "fast",
    "out_dir": "bin",
    "c_flags": ["-O3"],
    "link_libs": ["c", "m"]
  },
  "nest": {
    "mailbox_capacity": 65536,
    "reduction_slice": 2000
  },
  "runtime": {
    "workers": "auto", // "auto" 自动检测核心数，或固定数字如 16
    "max_blocking": 512 // 弹性阻塞计算线程池上限
  }
}
```

#### 2. 关键字段说明：

- **`main`**：工程默认入口源文件；
- **`mode`**：并发引擎模式，支持 `"parallel"`（开启底层 Worker 多物理线程池）或 `"single"`（严格单线程确定性隔离运行）；
- **`dependencies`**：生产环境依赖，严格遵循 Semver 语义化版本规则（`^` 次版本兼容、`~` 修订版本兼容、精确固定版本）；
- **`dev_dependencies`**：开发与测试依赖，使用 `cap add -d <pkg>` 自动注入；
- **`workspaces`**：Monorepo 多包工作区支持，各子包之间自动创建内部软链接引用；
- **`build`**：构建控制块，定义输出目录、优化等级与目标平台矩阵：
  - **`opt`**：编译优化模式，直接对接底层 Zig 4 级优化：
    - `"debug"`：**Debug 模式**，编译速度最快（毫秒级），保留全部调试符号与运行期安全检查。日常 `cap <file>` 与 `cap test` 默认模式；
    - `"safe"`：**ReleaseSafe 模式**（`cap build` 默认模式），平衡编译速度与执行性能，保留溢出、数组越界安全检查与崩溃调用栈；
    - `"fast"`：**ReleaseFast 模式**（`cap build -f`），追求极限单核/多核算力，消除一切额外运行时断言，榨干硬件性能；
    - `"small"`：**ReleaseSmall 模式**（`cap build -s`），极限剔除调试表并压制机器码体积，专用于边缘计算、嵌入式 MCU 与轻量微容器；
  - **`out_dir`**：产物输出目录，默认 `"bin"`（亦可通过 CLI `-o <dir>` 临时覆盖）；
  - **`target`**：单目标交叉编译平台（例如 `"x86_64-linux"`）；
  - **`targets`**：多平台矩阵发布列表（例如 `["x86_64-linux", "aarch64-macos", "x86_64-windows"]`），执行 `cap build -r` 时自动读取；
  - **`c_flags`**：透传给底层 C 编译器的专属参数（如 `["-O3", "-Wall"]`）；
  - **`link_libs`**：外部系统链接库数组（如 `["c", "m", "pthread"]`，自动转化为 `-lc -lm -lpthread` 链接）。
- **`nest`**：Nest 智能体容器专有参数，定义智能体默认信箱容量上限与抢占式 Reductions 时间片阈值。
- **`runtime`**：mode 为parallel时启动Worker线程与并发协程。

#### 3. 构建参数覆盖优先级裁定 (Precedence Hierarchy)

构建参数严格遵循以下三层优先级决策链：

1. **最高优先级**：命令行显式参数（如 `-f`、`-s`、`--debug`、`--safe`、`--opt=fast`、`-o <dir>`、`-t <triple>`、`-r`）；
2. **次高优先级**：当前项目 `cap.json` 中的 `"build"` 配置块；
3. **内置默认值**：`cap build` 默认 `ReleaseSafe`，`cap run`/`test` 默认 `Debug`，默认产物目录为 `bin/`。

---

### 2.5 ES6 风格模块系统与依赖解析引擎

CP 彻底摒弃了老旧沉重的 C/C++ `#include` 头文件包含与 CommonJS 的 `require` 动态阻塞机制，全面原生采纳现代 ES6 模块语义标准。

#### 1. 标准导入与导出语法

```typescript
// 1. 命名导出与默认导出
export const PI = 3.1415926;
export fn add(a: i32, b: i32): i32 { return a + b; }
// 说明：default export 适合主类型、主类；一个模块不允许多个 `export default`。
export default class AgentCore {}

fn foo() {}
fn bar() {}
export { foo, bar };
export { foo as default, bar };

// 2. 命名导入与命名空间导入
import { AgentCore, add } from "./core";
import * as math from "./utils/math";

// 3. 别名更名导入与导出 (使用 as 关键字)
import { query_user as getUser } from "./db";
export { add as sum, PI as TAU };

// 4. 内置标准库协议（使用专有的 cap:* 统一协议通道）
import { print, print_err } from "cap:io";
import { SharedTensor } from "cap:mem";
import { spawn, spawn_cpu } from "cap:task";
import { @spawn, @receive, @reply } from "cap:actor";

// 5. 第三方包导入 (自动触发向上攀升解析引擎)
import { Matrix } from "tensor-math";

// 导入默认导出
import App from "./app.cp";

// 默认 + 具名混合导入
import App, { start } from "./app.cp";
import {default as App, start } from "./app.cp";

// 6. 模块重导出 (Re-export / Facade 统一出口)
export { Reader, Writer, Closer } from "./traits";
export { BufferReader, BufferWriter } from "./buffer_stream";
export * from "./io/index";
```

#### 1.1 模块重导出规范（Re-export 与门面模式）

CP 语言原生支持标准的重导出语法，广泛应用于复合标准库及包门面（Facade）：
- **具名重导出**：`export { foo, bar } from "./sub_module";` —— 将子模块声明的符号直接公开导出，无需显式写 `import` 再 `export`；
- **全量重导出**：`export * from "./sub_module";` —— 门面文件将子目录/子模块入口中的所有公开符号一次性透明对外暴露；
- **标准库目录化规范（单一真理来源，杜绝冗余门面）**：
  - **单文件简单模块**：仅包含少量声明的标准库直接以单文件形态存在于 `lib/` 根目录（如 `lib/actor.cp`、`lib/beam.cp`、`lib/mem.cp`、`lib/worker.cp`）；
  - **复合多文件模块**：包含多个子领域文件的标准库统一采用目录化布局 `lib/<name>/`，由 `lib/<name>/index.cp` 作为唯一规范的对外出口（如 `lib/io/index.cp`、`lib/sync/index.cp`、`lib/http/index.cp`、`lib/router/index.cp` 等）；
  - **杜绝空壳门面堆积**：根目录下严禁创建多余的同名门面文件（如禁止在已有 `lib/io/` 的情况下创建 `lib/io.cp`），编译器自动识别目录并直通命中 `index.cp`，确保标准库结构绝对清爽与单一真理来源。

#### 1.2 内置标准库 `cap:*` 协议映射规则

- 所有的内置官方标准库统一通过 `cap:<module>` 协议通道导入（如 `cap:io`、`cap:fs`、`cap:http`、`cap:sync`）；
- 编译器解析器（`resolver.rs`）在解析时自动剥离 `cap:` 前缀，直接在纯正唯一的官方标准库根目录 `lib/` 下精准检索：
  - **单文件标准模块**（如 `cap:fs`、`cap:mem`）：直接命中 `lib/fs.cp`、`lib/mem.cp`；
  - **复合目录标准模块**（如 `lib/io/`、`lib/sync/`、`lib/http/`、`lib/router/` 等）：自动识别目录并优先检索 `lib/<module>/index.cp`，零冗余直通加载。

#### 2. 禁止行为

1. 没有 `use`，禁止写 `use xxx;`。
2. 不能在函数内部、代码块内部写 `import / export`，导入导出只能在模块顶层。
3. 不能 export 局部变量、分支内临时绑定。
4. 裸指针 `*T!`、`zig{}` 内嵌块内部定义不能跨模块导出（unsafe 内部实现隔离）。

#### 3. 智能目录入口解析优先级

当开发者书写 `import { something } from "./my_dir"` 或在命令行执行 `cap ./my_dir` 时，`cpc` 编译器与 `cap` 运行时将按照以下优先级自动侦测缺省入口：

1. 🥇 `my_dir/index.cp`（首选：沿用现代前端/模块化规范习惯）
2. 🥈 `my_dir/main.cp`（次选：沿用系统级后端/Rust/Go 主入口习惯）
3. 🥉 解析 `my_dir/cap.json`，读取其中的 `"main"` 字段定位入口文件

#### 4. 三层智能向上攀升依赖查找机制（Upward Resolution）

对于所有的第三方库引用（例如 `import { Matrix } from "tensor-math"`，而非以 `./`、`../` 或 `cap:` 开头），CP 编译器具备强大的向上攀升查找能力：

1. **第一层（项目私有）**：首先在当前源码文件所在目录的父级中检索 `cap_modules/tensor-math/`；
2. **第二层（工作区/Monorepo 共享）**：若未找到，逐层向父级目录递归向上攀升寻找 `cap_modules/`，直至硬盘盘符根目录。这使得在 Monorepo 架构下，所有子工程可以零拷贝共享根目录的统一依赖池；
3. **第三层（系统全局兜底）**：若整个目录树均未命中，系统自动启动终极兜底策略，直通操作系统的全局缓存区（`~/.cap/global/cap_modules/tensor-math/`）。

---

### 2.6 CAP 特性演进与对标路线图 (Implementation Status & Roadmap)

| 核心特性领域              | 对标 Bun / Cargo 对应机制     | CAP 现行架构设计与能力                                                                           | 演进状态       |
| :------------------------ | :---------------------------- | :----------------------------------------------------------------------------------------------- | :------------- |
| **单二进制极速工具链**    | `bun` (C++ & Zig 编写)        | `cap` 采用 Zig 纯原生编写，零系统动态依赖，单静态可执行文件直通系统底层。                        | **已全面实装** |
| **极速依赖管理**          | `bun install` / `pnpm`        | 全局中心物理存储（`~/.cap/cache/`）+ 毫秒级符号软链接注入（`cap_modules/`），无磁盘冗余。        | **已全面实装** |
| **全局依赖无感运行**      | `bun install -g`              | `cap add -g` 全局安装，结合编译器三层向上攀升查找引擎，免配置随写随跑。                          | **已全面实装** |
| **跨平台矩阵构建**        | Zig `b.standardTargetOptions` | `cap build -r` 单机一键并发交叉编译 macOS (ARM/Intel)、Linux (x64/ARM)、Windows 全矩阵二进制。   | **已全面实装** |
| **智能体容器桥接**        | NIF / Node-API                | `cap build -b` 打包为纯 CPS 状态机动态库（`.so`/`.dll`），无死循环主线程，供 Nest 容器原生接管。 | **已全面实装** |
| **单元与集成测试**        | `bun test` / `cargo test`     | `cap test` 自动递归扫描测试文件，自动编译执行，测试产物隔离至 `bin/test/`。                      | **已全面实装** |
| **开箱脚手架初始化**      | `bun create` / `cargo init`   | `cap init` 提供 `agent`、`app`、`lib`、`wasm` 官方标准项目模板脚手架。                           | **正在演进**   |
| **高精度基准测评**        | `bun:bench` / `cargo bench`   | `cap bench` 原生集成纳秒级计时器与 CPS 状态机调度吞吐压测分析器。                                | **正在演进**   |
| **语义依赖增删与更新**    | `bun add/remove/update`       | `cap remove` 与 `cap update` 针对 `cap.json` 的安全 Semver 版本自动修补与孤立软链接清理。        | **正在演进**   |
| **代码格式化与 Linter**   | `bun fmt` / `cargo fmt`       | `cap fmt` 与 `cap lint` 针对 CP 语法规范与借用安全生命周期约束进行静态格式化与合规审计。         | **规划实现**   |
| **去中心化生态 Registry** | `npm publish` / `crates.io`   | `cap publish` 与官方包注册中心联动，支持语义化包分发与数字签名验证。                             | **规划实现**   |
| **单文件静态全打包**      | `bun build --compile`         | `cap bundle` 将源码、静态资源与 Nest 状态机完全打包进单一微型 Native 可执行程序。                | **规划实现**   |

---

## 三、类型系统、不可变哲学与双轨对象模型

### 3.1 标量基础类型系统

CP 是一门严格无 GC 的静态类型编译型语言，禁止任何隐式类型提升：

| 标量类型  | 内存宽度                   | 说明                                                               |
| :-------- | :------------------------- | :----------------------------------------------------------------- |
| `i8`      | 8-bit 有符号整型           | 默认整数字面量类型                                                 |
| `u8`      | 8-bit 无符号整型           | 无符号索引、尺寸                                                   |
| `i16`     | 16-bit 有符号整型          | 默认整数字面量类型                                                 |
| `u16`     | 16-bit 无符号整型          | 无符号索引、尺寸                                                   |
| `i32`     | 32-bit 有符号整型          | 默认整数字面量类型                                                 |
| `u32`     | 32-bit 无符号整型          | 无符号索引、尺寸                                                   |
| `i64`     | 64-bit 有符号整型          | 高精度时间戳、大计数器                                             |
| `u64`     | 64-bit 无符号整型          | 高精度时间戳、大计数器                                             |
| `i128`    | 128-bit 有符号整型         | 超大整型计算、高精度中间量                                         |
| `u128`    | 128-bit 无符号整型         | 超大整型计算、位运算混淆计算                                       |
| `u1`~`u128` / `i1`~`i128` | 任意位宽原生整型 | 精准位宽标量，零开销映射硬件寄存器与 `packed struct` 位字段 |
| `usize`   | 无符号系统字长 (32/64-bit) | 内存偏移量、容器长度                                               |
| `isize`   | 与符号系统字长 (32/64-bit) | 内存偏移量、容器长度                                               |
| `f32`     | 32-bit 单精度浮点          | 图形计算、Tensor 运算基准                                          |
| `f64`     | 64-bit 双精度浮点          | 默认浮点字面量类型                                                 |
| `bool`    | 1-bit / 1-byte             | 布尔真假值 (`true` / `false`)                                      |
| `string`  | 连续只读 UTF-8 内存视图    | 附带长度信息的不可变切片视图                                       |
| `cstring` | 以 null 结尾的连续字节切片 | 对接 C/Bun 原生库，映射为 Zig `[:0]const u8` (可变借用为 `[:0]u8`) |
| `void`    | 0-bit                      | 空单元类型，表示无返回值                                           |

---

### 3.1.1 编译期自动推导占位符 auto 与历史兼容 anytype 的本质界限

很多初学者容易将 CP 语言中的泛型推导关键字与动态弱类型 `any` 混为一谈。在此必须明确：

1. **标准语法确立为 `auto`**：CP 官方首选且唯一的标准语法关键字是 **`auto`**，语义为 **“编译期自动推导类型 (Automatic Comptime Type Deduction)”**，心智模型清晰直观，完全杜绝与 `any` 的任何联想混淆；
2. **`anytype` 仅作历史与 Zig 互操作兼容**：CP 词法与语法分析器保留了 `anytype` 关键字，但其**仅用于向下兼容早期历史代码以及无缝对接 Zig 原生源码直译**，新代码一律强制推荐使用 `auto`；
3. **绝对禁止运行期动态类型 `any`**：CP 语言严厉废除运行期弱类型 `any`（报 `E0105`），杜绝隐式堆装箱、类型擦除和非确定性 GC。

| 核心维度       | 运行期动态弱类型 `any`（❌ 彻底废除）          | 编译期自动推导占位符 `auto`（✅ 官方标准一等公民）                        |
| :------------- | :--------------------------------------------- | :------------------------------------------------------------------------ |
| **生命周期**   | **运行期（Runtime）**                          | **编译期（`@comptime`）**                                                 |
| **类型本质**   | 类型擦除（Type Erasure），动态类型标签         | **不是类型**，是**静态单态化泛型占位符**（Monomorphization Placeholder）  |
| **开销代价**   | 隐式装箱（Box）、堆分配、GC 跟踪、动态虚表派发 | **零装箱、零堆分配、零运行时开销（Zero Cost）**，完全单态化生成独立机器码 |
| **类型安全性** | 彻底丧失静态类型检查，运行期崩溃隐患           | **100% 强类型保留**，编译器针对实参类型进行全量类型与借用安全检查         |
| **标准语法**   | ❌ 语法彻底禁用（报 `E0105`）                  | ✅ **官方推荐标准为 `auto`**（兼容别名 `anytype` 仅供历史互操作）         |
| **对标特性**   | JS/TS 的 `any`，Python 的动态类型              | C++ 的 `auto` 概念泛化，Rust 的 `impl Trait` / `<T>`                      |

#### 1. 语法规范与四大铁律约束

1. **铁律一：仅限作为函数形参的泛型占位符**：
   - 允许在通用泛型函数或编译期高阶函数中使用标准 `auto`：
     ```typescript
     // 泛型打印函数：标准推荐 auto，编译期根据实参自动单态化展开
     pub fn print(v: auto): void { ... }

     // 编译期高阶跳板函数：以编译期符号接收任意具体函数
     pub fn wrapper(@comptime callback: auto) { ... }

     // 历史代码兼容写法（等价于 auto，内部统一映射为 Type::Auto）：
     // pub fn wrapper(@comptime callback: anytype) { ... }
     ```
2. **铁律二：严禁用于普通局部变量类型声明**：
   - ❌ 错误：`let x: auto = 10;` 或 `let x: anytype = 10;`（编译器静态拦截）；
   - ✅ 正确：局部变量直接使用自动类型推断 `let x = 10;` 或显式强类型 `let x: i32 = 10;`。
3. **铁律三：严禁用于类/结构体字段类型声明**：
   - ❌ 错误：`struct Container { data: auto; }`（结构体内存布局必须确定，禁止持有未单态化的占位符）；
   - ✅ 正确：结构体泛型必须使用类级泛型参数：`struct Container<T> { data: T; }`。
4. **铁律四：强调编译期求值时与 `@comptime` 强绑定**：
   - 当参数涉及类型计算、编译期常量或静态函数内联时，必须显式标注 `@comptime callback: auto`，强制调用方在编译期提供已知静态符号，杜绝运行期动态传递。

---

### 3.2 复合容器类型

```typescript
// 1. 动态连续数组与切片：[T]
let nums: [i32] = [10, 20, 30];
let slice: [i32] = nums[1..3];

// 2. 静态定长数组与重复初始化：[T; N] / [N]T 与 [val; count]
let fixed_arr: [u8; 4] = [1, 2, 3, 4];   // 亦支持 Zig 风格 [4]u8
let zero_buf: [u8; 256] = [0; 256];      // 编译期生成 ([_]u8{ 0 } ** 256)
let lookup_table: [u8; 256] = [0xFF; 256];

// 3. 静态定长元组：[T1, T2, ...]
let pair: [string, i32] = ["alpha", 100];

// 4. 字典哈希表：Dict<K, V> 或 Map<K, V>
let scoreTable: Dict<string, i32> = Dict();
scoreTable.set("Alice", 98);

// 5. 哈希集合：Set<T>
let visited: Set<i32> = Set();
visited.add(42);
```

---

### 3.3 变量可变性与不可变哲学

CP 彻底废除 `let mut` 与 `var`，全面拥抱 TypeScript 直觉与编译期所有权安全：

- **`let`**：变量声明，**默认可变**。局部可变变量写 `let x = ...;`，模块顶层可变变量写 `pub let g_state = ...;`。
- **`const`**：常量与深度不可变声明。编译期阻止一切重分配与内部修改。
- 🚨 **严禁使用 `var`**：CP 语法中不存在 `var` 关键字（`var` 是 Zig 遗留语法）。在 CP 源代码中，可变变量必须 100% 写作 `let`。

```typescript
let count = 1;
count = count + 1; // 合法

pub let global_counter: usize = 0; // 顶层可变变量声明

const MAX_LIMIT = 1024;
// MAX_LIMIT = 2048; // 编译报错：Cannot assign to constant 'MAX_LIMIT'
```

#### 3.3.0 C-ABI 调用约定与修饰器规范（首选 `@c`）

在声明导出函数、静态回调或 C-ABI 兼容函数指针时，**强烈推荐使用 `@c` 代替冗长的 `@callconv("c")`**：

```typescript
// 类内静态回调方法
class TimerController {
    @c pub static fn onTimeout(timer: *uws.Timer): void {
        // C-ABI 导出，无 implicit this receiver
    }
}

// 顶层 C 兼容函数
@c pub fn native_callback(status: i32, ctx: ?*anyopaque): void {
    // ...
}
```

#### 3.3.0.1 嵌套结构体定义与命名空间规范

在类或结构体内部定义内嵌结构体时，必须采用赋值表达式形式并以分号结尾：

```typescript
pub const LibInfo = struct {
    pub const HandleReply = struct {
        pub fn get(): ?*anyopaque { return null; }
    };
};
```

#### 3.3.0.2 错误联合类型（Result）声明顺序

CP 语法遵从 TypeScript / Rust 成功优先直觉，Result 类型统一采用 `OkType!ErrType`：
- `void!bun.JSError`（成功返回 void，失败携带 JSError）；
- `jsc.JSValue!bun.JSTerminated`（成功返回 JSValue，失败携带 JSTerminated）。
转译器（`cpc`）在生成 Zig 代码时会自动转为 Zig 的 `ErrType!OkType`。

---

#### 3.3.1 类型转换体系（四层黄金法则）

- 可空、错误联合不能自动脱壳；脱壳必须通过 `switch`、`?`、`??`

结合 TS 的直觉表达 + Rust 的安全严谨 + Zig 的底层透传，CP 的类型转换建议确立为以下 四层正交体系：

```text
┌──────────────────────────────────────────────────────────────┐
│ 第四层：高阶语义受检转换 (禁止 as！走 Trait / 方法: parse()?)      │
├──────────────────────────────────────────────────────────────┤
│ 第三层：底层裸指针与物理地址转换 (强制在 unsafe { ... } 内 + as)    │
├──────────────────────────────────────────────────────────────┤
│ 第二层：有损截断 / 符号重解释 / 类型断言 (强制显式 as)              │
├──────────────────────────────────────────────────────────────┤
│ 第一层：无损自然流转 (Zero-Cost Coercion，禁止写 as，自动适配)     │
└──────────────────────────────────────────────────────────────┘
```

##### 第一层：无损自然流转（Implicit Coercion - 零噪音，无需 as）

凡是数学上 100% 绝对无损、内存语义单向安全的场景，编译器自动向上提升，严禁强加 as 视觉噪音：

1.  字面量自适应推导 (Literal Fitting)：
    let a: u64 = 100; // 自动推断为 u64，绝不需要 100 as u64
    let b: f32 = 3.14; // 自动推断为 f32
2.  数值无损拓宽 (Safe Widening)：
    • 小无符号 → 大无符号：u8 → u16 → u32 → u64 → usize（无损拓宽，值绝对不变）；
    • 小有符号 → 大有符号：i8 → i16 → i32 → i64（无损符号扩展）；
    • 整数 → 大浮点数：i32 → f64，f32 → f64。

let small: u8 = 10;
let big: usize = small; // 100% 静态安全无损，直接流转

3.  安全结构向上包装：
    • 具体值自动包装进可选类型：val → val: T?；
    • 具体值自动包装进 3-State 错误联合：return 42;（函数签名为 i32? 时自动作为成功值返回）；
4.  借用权限收紧：
    • *T（独占可变借用）可隐式传给接收 &T（只读借用）的参数；
5.  面向对象与 Trait 向上多态：
    • 子类实例可直接赋给基类变量；
    • 实现了 Trait 的结构体借用可直接赋给 &dyn Trait 契约指针。
6.  字节切片向不可变字符串自然流转 (Byte Slice to String Coercion)：
    • 原生动态字节切片 [u8]、定长字节数组 [u8; N] 以及 cstring 在只读上下文中直接自然流转为 string（底层不可变 []const u8 视图），享受零开销自然适配，无需显式转换或内存深拷贝。
    ──────

##### 第二层：有损/重新解释转换（Explicit Cast - 强制必须写 as）

凡是存在 截断、精度丢失、符号位翻转或类型重新解释 的场景，编译器必须强制开发者显式写 as，表明“我知晓潜在风险”：

1.  高位截断 (Narrowing / Truncation)：
    let big: u64 = 1000;
    let small = big as u8; // 强制 as：开发者明确知晓高位可能被截断

2.  符号位重解释 (Sign Reinterpretation)：
    let signed: i32 = -1;
    let unsigned = signed as u32; // 强制 as：负数转无符号存在位解释变更

3.  浮点数与整型互转 (Precision Loss)：
    let pi: f64 = 3.14159;
    let int_pi = pi as i32; // 强制 as：小数位丢失

4.  枚举与数值互转：
    let code = Status.Ok as u32;
    let status = raw_int as Status;

5.  动态消息投影 (Type Assertion - TS 兼容)：
    从通用动态联合体 Message / any 中断言取出具体强类型：
    let str = raw_msg as string;

---

##### 第三层：底层裸指针与物理地址转换（强制在 unsafe { } 内 + as）

裸指针的重解释是引发内存段错误（Segfault）的核心根源，CP 规定：裸指针之间的强转不仅必须显式 as，还必须置于 unsafe 特权作用域中：

```ts
unsafe {
  // 裸指针类型重新按位解释
  let raw_u8: *u8 = ptr_i32 as *u8;

  // 裸指针与物理地址数值互转
  let addr: usize = ptr_i32 as usize;
  let new_ptr: *i32 = addr as *i32;
}
```

若在普通安全代码块中试图使用 as 强转裸指针，cpc 编译器前端静态报错：CP Compile Error: Raw pointer casting is only allowed inside unsafe block。

---

##### 第四层：高阶语义转换（禁止使用 as，走语义化方法与 Trait）

在 JavaScript / TypeScript 中，很多人习惯用强转去变异数据（例如通过断言把包含数字的字符串当数字用），这在系统级语言中是灾难。

规则：凡是可能失败的解析、复杂的对象构造，禁止使用 as！必须走标准库方法并配合 CP 优雅的 3-State 错误传播符 ?：

```ts
// ❌ 严禁：字符串不是标量，不能通过 as 强转数字
// let n = "123" as i32;

// ✅ 正确：走标准受检解析函数，优雅冒泡错误
let port: i32 = i32.parse(env_str)?;

// ✅ 标量转字符串：走 to_string() 方法
let text: string = port.to_string();

// ✅ 复杂结构体之间的转换：实现 Into<T> / From<T> 契约
let vec = queue.into();
```

---

#### 3.3.2 类型转换演化建议

| 转换类别        | 是否需要 as      | 规则与判定条件                                   | 示例                      |
| --------------- | ---------------- | ------------------------------------------------ | ------------------------- |
| 字面量适配      | 不需要           | 字面量数值未超出目标类型取值范围                 | `let x: u64 = 100;`       |
| 无损安全拓宽    | 不需要           | 小范围到大范围（u8 → u32, i16 → i64, i32 → f64） | `let b: u64 = small_u8;`  |
| 容器/借用提升   | 不需要           | 值转 T?，*T 传给 &T，子类转父类                  | `let opt: string? = str;` |
| 有损位截断      | 必须 as          | 大整数转小整数（高位截断）                       | `big_u64 as u32`          |
| 符号重新解释    | 必须 as          | 有符号与无符号互转                               | `i32_val as u32`          |
| 浮点截断        | 必须 as          | 浮点数转整型（丢失小数），f64 → f32              | `pi as i32`               |
| 枚举互转        | 必须 as          | 枚举变体与底层数值互转                           | `Color.Red as u32`        |
| 裸指针再解释    | 必须 as + unsafe | 裸指针类型强转、裸指针与 usize 地址互转          | `unsafe { p as *u8 }`     |
| 字符串/高级解析 | 禁止 as          | 必须使用标准库方法，走 ? 错误传播                | `i32.parse(s)?`           |

---

#### 3.3.3 `undefined` 关键字与未初始化内存语义（对齐 Zig 原生与 C-FFI）

在 CP 语言中，`undefined` 是一个**底层的系统级关键字**，其语义与 JavaScript / TypeScript 的 `undefined` 有着本质的区别：

- **TypeScript 的 `undefined`**：代表“变量尚未赋值或属性不存在”的高级动态值（属于 JavaScript 运行时的一种类型）；
- **CP 语言的 `undefined`**：直接继承并映射为 Zig 的 `undefined`，代表**“该内存区域保留未初始化状态（物理上包含随机脏数据），禁止编译器生成多余的零填充指令（Zero-initialization）”**。

##### 1. 核心设计目的与工程价值

在系统编程与 C-FFI 互操作场景中，许多高性能 C 库（如 `picohttpparser`、`epoll`、`read/recv` 等）要求调用方在栈上预分配一块数组或结构体作为**输出缓冲区（Out Parameter）**。
若编译器强制对这块即刻被覆写的内存执行 `memset(0)`，会造成昂贵的 CPU 周期浪费与内存带宽抖动。
通过 `undefined`，开发者可以明确指示编译器：“**请为我开辟栈空间，但跳过清零指令，稍后底层 C 函数会直接填充数据**”。

```typescript
// 1. 在栈上分配未初始化的定长数组缓冲区（零 memset 开销）
let header_buf: [16]phr_header = undefined;

// 2. 作为输出参数传入底层 C 解析函数直接填充
phr_parse_req(req, req_len, ..., header_buf as [*]phr_header!, ...);
```

##### 2. 使用边界与安全约束

1. **未初始化变量检查器（Definite Assignment Checker）**：
   在 Safe 作用域中，严禁在未向 `undefined` 缓冲区写入有效数据前直接读取其内容。若在安全代码中直接读取未写入的 `undefined` 变量，将触发未定义行为（UB）。在 Debug 模式下，底层 Zig 会自动填充 `0xaa` 调试字节以侦测非法读取。
2. **适用场景**：
   - 底层 C-FFI 输出数组与结构体缓冲区：`let buf: [1024]u8 = undefined;`
   - 与 `as [*]T!` 联合使用作为底层接口实参。
3. **禁止场景**：
   - 严禁将业务状态对象初始化为 `undefined`；
   - 业务逻辑中的“空”必须使用安全的可选类型 `T?` 与 `null`，严禁滥用 `undefined`。

---

#### 3.3.4 类型系统禁止行为

1. 禁止隐式把 `T` 提升为 `T?` 或者 `T!E`；必须显式标记。
2. 禁止 `??` 捕获错误（`??` 只管 null）。
3. 禁止 `?` 冒泡 null（`?` 只管错误联合）。
4. 安全域中禁止裸指针类型 `*T!` / `**T!`。
5. 借用类型不允许自动降级为裸指针，必须 unsafe + as。
6. 不能把 `&T` / `*T` 存入同一个可变容器引发数据竞争，借用检查器拦截。

### 3.4 双轨对象模型（静态匿名对象 vs 动态字典对象）

CP 创新性地提供“双轨对象模型”，完美兼顾零成本强类型计算与灵活动态配置：

```typescript
// 1. 静态匿名对象 (Anonymous Struct)
// 语法：{ key: value }
// 特性：编译期推导确切结构，零堆开销栈分配，直接对齐 Zig 匿名结构体
let point = { x: 10, y: 20 };
let sum = point.x + point.y;

// 2. 动态匿名对象 / 字典对象 (Dynamic Dict)
// 语法：#{ key: value }
// 特性：堆分配动态哈希字典，支持运行时自由插入、修改键值对，原生对齐 JSON 动态交互
let config = #{ host: "127.0.0.1", port: 8080 };
config.set("timeout", 5000);
```

---

### 3.5 枚举与代数数据类型（Enum & Tagged Union）

CP 支持全特性的代数数据类型（ADT，底层对齐 Zig 的带标签联合体 `union(enum)`）。定义语法全面融入 TypeScript / ES6 现代风格，彻底告别 Rust 晦涩的元组嵌套与冗长样板代码：

#### 3.5.1 枚举与变体定义

- **纯标签变体**：`Idle,`
- **单类型载荷**：`Text: string,`
- **元组载荷**：`Move: [i32, i32],`
- **结构体载荷**：`Compute: { batch: usize, priority: i32 }`

```typescript
enum Task {
    Idle,
    Text: string,
    Move: [i32, i32],
    Compute: { batch: usize, priority: i32 }
}
```

#### 3.5.2 上下文前缀推导与初始化语法

在上下文类型已知（变量类型标注、函数参数、返回值或 switch 比较项）时，**支持直接省略枚举名前缀，使用 `.Variant` 简写构造**：

```typescript
// 1. 无载荷变体构造
let t1: Task = .Idle;

// 2. 单类型载荷变体构造
let t2: Task = .Text("hello");

// 3. 元组载荷变体构造（直接使用方括号，无需冗余外层小括号）
let t3: Task = .Move [15, 25];

// 4. 结构体载荷变体构造
let t4: Task = .Compute { batch: 200, priority: 7 };
```

#### 3.5.3 特殊符号变体、底层标签与嵌套逃逸舱 (Escaped Identifiers & Nested Escapes)

针对网络协议、系统 API 与 Web 标准等场景中出现的含连字符名称（如 `same-origin`）或语言保留关键字（如 `error`）：

- CP 原生支持 `@"identifier"` 逃逸标识符字面量；
- 支持为无载荷枚举指定底层定长整型标签类型（如 `: u2`、`: u3`、`: u8`）；
- 支持在枚举体内直接使用局部 `zig { ... }` 逃逸舱注入编译期元数据（如 `ComptimeStringMap`），实现与宿主系统的零开销互操作：

```typescript
export enum FetchRequestMode: u2 {
    @"same-origin",
    @"no-cors",
    cors,
    navigate,

    zig {
        pub const Map = bun.ComptimeStringMap(FetchRequestMode, .{
            .{ "same-origin", .@"same-origin" },
            .{ "no-cors", .@"no-cors" },
            .{ "cors", .cors },
            .{ "navigate", .navigate },
        });
        pub const toJS = @import("../http_jsc/fetch_enums_jsc.zig").fetchRequestModeToJS;
    }
}
```

---

## 四、解构赋值与 `switch` 模式匹配体系

### 4.1 数组与元组解构体系

CP 拥有现代语言中最精炼的通配符与解构语法：

- **`*`（单元素跳过通配符）**：跳过并丢弃单个元素；
- **`...`（切片提取与区间忽略通配符）**：丢弃或提取 0 到任意多个元素（彻底摒弃 `..`，`..` 符号专属作为范围运算符）。

```typescript
let arr = [10, 20, 30, 40, 50];

// 1. 单元素通配跳过 (*)
let [first, *, third, ...] = arr;
// first = 10, third = 30

// 2. 剩余切片捕获 (...rest)
let [head, ...rest] = arr;
// head = 10, rest = [20, 30, 40, 50]

// 3. 首尾两头提取，忽略中间任意长度 ([head, ..., tail])
let [startVal, ..., endVal] = arr;
// startVal = 10, endVal = 50
```

---

### 4.2 对象解构体系

```typescript
let user = { id: 1001, name: "Antigravity", role: "Admin", score: 99 };

// 1. 基础解构与别名映射
let { id, name: userName } = user;

// 2. 字段忽略通配符 (...)
// 仅提取指定属性，明确忽略对象中声明的其他属性
let { id, ... } = user;

// 3. 剩余属性提取绑定 (...rest)
// 提取未显式指定的剩余属性并自动按强类型打包为包含剩余字段的对象结构体
let { id, ...extraProps } = user;
// extraProps 为包含 { name: "Antigravity", role: "Admin", score: 99 } 的强类型对象，零堆分配开销
```

---

### 4.3 `switch` 原生模式匹配体系

CP 官方正式定稿采用 **`switch`** 作为唯一标准匹配关键字（`match` 仅保留作为编译器的透明向下兼容别名）。匹配分支的缺省通配统一使用 `*`。

```typescript
let status_code = 404;

let msg = switch (status_code) {
    200 => "OK",
    // 联合多模式匹配（支持 | 或 , 逗号合并）
    400 | 401 | 403 => "Client Error",
    // 闭区间匹配
    500..=599 => "Server Error",
    // 缺省通配分支（统一使用 * 号，不占用 _）
    * => "Unknown Status"
};
```

#### 4.3.1 原生三态解包匹配

```typescript
let result = load_data();

switch (result) {
    // 1. 成功正常值解包
    data => print("Loaded payload:", data),
    // 2. 显式错误匹配与提取
    err(e) => print("Load failed with error:", e),
    // 3. 空值匹配
    null => print("No data present")
}
```

#### 4.3.2 编译期内建表达式匹配 (`@field` 动态字段与元编程模式)

在系统级编程与元编程场景中（如枚举反射、网络协议状态机），分支目标可能是由编译期计算决定的。CP 支持在 `switch` 分支中直接书写 `@field(Container, "name")` 等编译期求值表达式：

```typescript
pub const Tag = enum {
    foo,
    bar,
    baz,
};

pub fn checkTag(tag: Tag): i32 {
    switch (tag) {
        @field(Tag, "foo") => 100,
        @field(Tag, "bar") => 200,
        * => 300,
    }
}
```

---


### 4.4 守卫条件表达式与穷尽性检查

分支支持 `if` 守卫条件；编译器会对匹配项进行全域穷尽性检查：

```typescript
let info = switch (person) {
    { age, role } if age >= 18 && role == "Admin" => "Adult Superuser",
    { age, ... } if age < 18 => "Minor",
    else => "Standard User" // 支持 `else =>` 或 `* =>` 作为默认后备分支
};
```

---

### 4.5 枚举载荷解构、Walrus 绑定与动态类型匹配

CP 的 `switch` 匹配体系彻底废除早期 Zig 风格的 `|fields|` 管道捕获语法，全面拥抱原生解构、海象操作符（`:=`）绑定与动态类型匹配：

#### 4.5.1 枚举载荷解构匹配

- **结构体载荷精确解构**：`.Compute { batch, priority }`
- **结构体剩余字段整包捕获**：`.Compute { ...fields }`（将所有未显式列出的字段整包绑定为只读结构体变量 `fields`，零堆开销）
- **元组载荷解构**：`.Move [x, y]`（直接使用方括号解构，无需冗余外层小括号）
- **单载荷变量捕获**：`.Text text` 或 `.Text(text)`

```typescript
// 枚举载荷
enum Message {
       Idle,
       Compute: { batch: usize },
       Move: [i32, i32]
   }

fn process_task(task: Task): i32 {
    return switch (task) {
        .Idle => 0,
        // 单载荷变量捕获
        .Text text => text.len as i32,
        // 原生元组方括号解构
        .Move [x, y] => x + y,
        // 结构体属性精确解构
        .Compute { batch, priority } => (batch as i32) + priority,
        // 结构体剩余/整包解构捕获
        .Compute { ...fields } => fields.batch as i32,
        * => -1
    };
}
```

#### 美学的完美对称：花括号对花括号，方括号对方括号

当我们把定义、构造与解构匹配三者放在一起看时，.Move [x, y] 展现出了绝妙的自洽性与对称美感：

| 枚举类型                 | 定义（Definition）          | 构造（Initialization）    | 模式匹配解构（Pattern Destructuring）  |
| ------------------------ | --------------------------- | ------------------------- | -------------------------------------- |
| **无载荷（Unit）**       | `Idle`                      | `.Idle`                   | `.Idle => ...`                         |
| **结构体载荷（Object）** | `Compute: { batch: usize }` | `.Compute { batch: 100 }` | `.Compute { batch } => ...` （花括号） |
| **元组载荷（Tuple）**    | `Move: [i32, i32]`          | `.Move [10, 20]`          | `.Move [x, y] => ...` （方括号）       |

#### 4.5.2 Walrus（海象操作符 `:=`）子模式绑定

在复杂子模式（如区间匹配、复合对象解构）中，CP 使用现代化的 `ident := pattern` 语法（取代 Rust 晦涩的 `@` 符号），在成功匹配模式的同时将原值完整绑定到变量：

```typescript
let grade = switch (score) {
    s := 90..=100 => s * 2,
    s := 60..=89 => s + 10,
    s := 0..=59 => s,
    * => 0
};

let user_role = switch (session) {
    // 匹配包含 role: "admin" 的对象，同时将整个 session 对象绑定到 ctx
    ctx := { role: "admin", ... } => ctx.id,
    * => 0
};
```

#### 4.5.3 动态类型分支匹配（`ident: Type`）

在处理动态多态、Trait 对象或动态类型（`dyn` / `any`）时，CP 允许直接使用 `ident: Type` 进行类型窄化与向下转型绑定：

```typescript
switch (worker) {
    w: AIWorker => w.work(100),
    s: string => print(`字符串长度: ${s.len}`),
    n: i32 => print(`整型数值: ${n}`),
    * => print("其他未知类型")
}
```

> [!NOTE] 语法精简与废弃说明
> CP 已彻底移除旧版 Zig 风格的 `|fields|` 管道捕获语法。在模式匹配中，变量捕获统一使用标准解构（`{ batch }`, `[x, y]`）、单载荷直捕（`.Text text`）、海象绑定（`ident := subpat`）与类型匹配（`ident: Type`）。

---

### 4.6 流式类型智能收窄与 Fail-Fast 解包体系 (Flow-Sensitive Type Narrowing)

在现代高并发 Agent 与系统编程中，空值与指针校验是极其高频的代码路径。传统低级语言往往要求开发者在完成判空后依然繁琐地手工解包（如 Rust 的 `val.unwrap()` 或 Zig 的 `val.?`），带来冗余的心智负担。

CP 编译器内建了工业级**流式类型智能收窄机制（Flow-Sensitive Type Narrowing）**，使得类型系统具备感知程序控制流的能力：

#### 4.6.1 正向条件收窄（Positive Narrowing）

在 `if (v != null)`、`while (v != null)` 或 `if (null != v)` 的条件真值作用域内，任何可空类型（如 `T?`、`*T!?`、`User?`）会被编译器自动收窄为确定非空类型（`T`、`*T!`、`User`）：

```typescript
struct User {
    name: string,
    age: i32,
}

fn render(u: User?): void {
    if (u != null) {
        // 智能收窄生效：在当前作用域内，u 直接被视为 User 类型
        // 彻底免除 u.?.name 或 u!?.age 冗余写法
        print(`Name: ${u.name}, Age: ${u.age}`);
    }
}
```

#### 4.6.2 复合条件短路收窄（Short-Circuit Narrowing）

在逻辑与 `&&` 表达式中，左侧的判空条件会立刻收窄右侧表达式的类型上下文：

```typescript
fn check_user(u: User?): bool {
    // && 右侧的 u.age 直接享受左侧 u != null 的智能类型收窄
    if (u != null && u.age >= 18) {
        return true;
    }
    return false;
}
```

#### 4.6.3 负向分支与 Fail-Fast 早期返回收窄（Early-Return Guard Narrowing）

CP 完美支持现代工程中推崇的卫语句（Guard Clauses）模式：

```typescript
fn process_handle(loop: *us_loop_t!?): void {
    // 1. 负向分支匹配：else 分支内自动收窄为非空
    if (loop == null) {
        @panic("Loop is null");
    } else {
        // 此处 loop 自动收窄为 *us_loop_t!
        us_loop_run(loop);
    }
}

fn process_user_guard(u: User?): void {
    // 2. 卫语句早期返回：return 后的作用域自动推导为非空
    if (u == null) return;

    // 此处继续执行的代码中，u 已被证明绝对非空，类型直接收窄为 User
    print(`Processing valid user: ${u.name}`);
}
```

#### 4.6.4 后缀可选裸指针收窄（`*T!?` -> `*T!`）

当对接底层的 C-ABI 或 FFI 时，外部函数返回的裸指针可能为 null（表示为 `*T!?`）。在完成 `ptr != null` 校验后，该指针在分支内自动收窄为可安全解引用的非空裸指针 `*T!`，保证了低级系统编程中与现代高级语言一致的优雅流畅体验。

---

## 五、函数系统与参数方法契约

> 核心设计思路：统一普通运行期函数、`comptime`编译期函数；支持闭包、管道传参；参数区分所有权/借用；返回类型支持 `void`、可空 `T?`、错误联合 `T!E`；错误冒泡 `?` 作用于调用表达式。

### 5.1 函数基础语法

关键字 `fn` 定义函数；冒号 `:` 之后标注返回类型；`void` 返回类型**可以省略**。
标准格式：

```ts
fn 函数名(参数列表): 返回类型 {
    语句
}
```

示例：

```ts
fn add(a:i32, b:i32): i32 {
    return a + b;
}

// 无返回
fn print_msg(msg:string): void {
    print(msg);
}

// 无返回、可空
fn print_msg(msg:string): void? {
    print(msg);
}

// 无返回、可错
fn print_msg(msg:string): void! {
    print(msg);
}

// 动态链接库与 C FFI 符号导出：export fn
export fn calculate(val: i32): i32 {
    return val * 2;
}

```

---

### 5.2 参数分类：值、所有权、只读借用、可变借用

参数的传递语义写在参数类型前，**四种参数修饰**，严格区分：

1. **值/只读借用参数**：`arg: T`
   - 基础Copy类型（bool、整数、浮点数）：传入副本。
   - 复合类型：只读借用，调用方传入只读借用变量或所有权变量，所有权变量编译时，自动转为只读借用变量。
   - 只读借用只读访问，不获取所有权；不能修改内部数据。
2. **所有权转移参数**：`move arg: T`
   - 函数取得这个值的所有权；函数退出时资源释放
   - 调用方只可传入所有权变量。
3. **可变借用参数**：`arg: *T`
   - 独占可变访问，可以修改；遵守借用互斥规则

示例汇总：

```ts
// 值参数，复合类型自动转为只读借用
fn accept_val(u: i32) {}
// data 默认为只读借用，传参零拷贝，禁止内部任何原地修改
fn inspect(data: [i32]) {}
// move 参数，函数接管所有权
fn take_own(move u: User) {}
fn take_ownership(move data: Tensor) {
    // data 所有权完全移交进函数体，调用方原变量变为未初始化失效态
}
// 只读借用
fn read_u(u: User) {}
// 可变借用
fn modify_u(u: *User) {}
fn append_seed(*data: [i32], seed: i32) {
    data.push(seed); // 允许对调用方传入的实参进行原地修改
}

let u = User{id:1};
accept_val(u.id);
take_own(u);
read_u(u);
modify_u(*u);

```

> 重要：借用参数不会触发所有权转移；借用生命周期不能超过调用方原始所有者。

---

### 5.3 返回值体系（void / 普通 / 可空 / 错误联合）

四种返回类型形式，语义严格隔离：

1. `void`：无返回，可省略
2. `T`：正常返回 T，非空、无错误
3. `T?`：可空返回，结果为 `T` 或者 `null`，使用 `??`、`?.` 处理 null
4. `T!E`：错误联合返回，正常是 `T`，错误是 `E`，使用 `?` 冒泡错误，`err(...)` 构造错误

```ts
// 可空返回
fn find_user(id:u64): User? {
    return null;
}

// 错误联合返回
fn load_config(path:string): Config ! LoadErr {
    return err(LoadErr.NotFound);
}
```

return 规则：

- 返回所有权：`return val` 自动移交所有权给调用方，**不需要在 return 写 move**
- 返回借用：必须保证借用指向的值存活；编译器生命周期检查，禁止悬垂借用返回

```ts
fn ok(): &i32 {
    let x = 10;
    return &x; // ❌ 编译报错，悬垂借用
}
```

---

### 5.4 函数调用、错误冒泡 `?`

普通调用：`add(1,2)`
如果返回是 `T!E`，调用表达式末尾加 `?`：**错误冒泡**

- 成功：取出 T，继续执行
- 失败：向上抛出错误，当前函数必须同是 `T!E` 返回类型

```ts
let cfg = load_config("cfg.toml")?;
```

> 重点区分：
>
> - `expr?`：只冒泡错误（`T!E`），不处理 null
> - `expr ??：只兜底 null（`T?`），不处理错误

switch 是另一种处理错误/可空的方式，适合分支处理。

---

### 5.5 匿名函数 & 闭包

箭头 `=>` 定义匿名函数（闭包）；可以捕获外层变量，捕获遵守所有权/借用规则。

```ts
let sum = (a: i32, b: i32) => a + b;

// 多行闭包，加大括号
let print_fn = (msg: string) => {
  print(msg);
};
```

闭包捕获规则：

- 默认捕获借用；
- 如果闭包需要拿走所有权，捕获时显式标记 `move`；
- 闭包同样受借用检查约束，不能捕获已经失效、move 走的值。

闭包属于一等公民，可以作为参数、返回值、存入变量。

函数类型写法：`(A,B): R`

```ts
let op: (i32,i32): i32 = (a,b) => a * b;
```

---

### 5.6 管道 `|>` 和函数调用交互规则

管道 `|>`：把左侧结果，作为右侧函数的第一个入参；管道内部隐式值用 `.` 指代。

```ts
// 等价于 add(10, 20)
10 |> add(*, 20)
```

- 管道可以链式连续调用
- 管道传递的值同样遵守所有权规则：如果是复合类型，需要 move 则在左侧写 move

```ts
move u |> modify_u(*)
```

- 管道表达式可以是可空、错误联合；`?`、`??` 可以加在整个管道表达式尾部

```ts
"config.txt" |> load_config(*)?
```

---

### 5.7 comptime 编译期函数

> 区分：普通函数运行时执行；`comptime` 函数在编译阶段求值，属于编译期计算。
> 语法标记：函数定义前加 `comptime`；调用使用 `@func(...)`。

```ts
fn square(n: u32): u32 {
    return n * n;
}

let size = @square(4); // 编译期直接算出 16
```

> 和宏区分：comptime 函数是普通函数语义，只是执行时机在编译期；宏是代码展开，二者不是一回事。

---

### 5.8 泛型函数

尖括号 `<T>` 声明泛型参数；支持类型参数和 comptime 常量参数。

```ts
fn max<T>(a:T, b:T): T {
    if (a > b) { a } else { b }
}

let m = max<i32>(10, 20);
```

泛型约束（后续 trait 章节完善）：

```ts
fn sort<T: Ord>(arr: []T) {}
```

编译期单态化（monomorphization），每个实例生成独立代码，无运行时开销。

---

### 5.9 方法（struct / class 上的函数）

- 在 `struct` / `class` 内部定义 `fn` 即为方法；
- 方法内使用 `this` 指代当前实例；`_` 是 `this` 的简写。
- 只读借用可省略。

```ts
class User {
    id: u64,
    fn set_id(*_, new_id:u64) {
        _.id = new_id;
    }
    fn get_id(): u64 {
        return _.id;
    }
}
```

> 方法第一个参数约定：
>
> - `&_`：只读方法，只读借用实例, 可省略。
> - `*_`：可变方法，可变借用实例。
> - `move _`：消耗实例，方法拿到所有权，调用后实例失效

调用：

```ts
let u = User{id:1};
u.set_id(99);
print(u.get_id());
```

---

### 5.10 重载、递归、函数类型

1. **函数重载：CP 不支持同名重载**
   同一个命名空间，函数名唯一；区分功能使用泛型或者不同名字。
2. **递归：支持递归调用**
   递归函数需要提前声明原型（如果前向引用）。

```ts
   fn fib(n:u32): u32 {
       if (n <= 1) { return n; }
       return fib(n-1) + fib(n-2);
   }

```

3. 函数指针 / 函数类型：
   函数类型 `(A,B)->R` 是一等类型，可以存入结构体、作为返回值。

---

### 5.11 约束与禁止行为汇总

1. move 参数，调用点必须显式 `move`；禁止隐式转移所有权
2. 借用参数遵守借用互斥规则：不能同时存在活跃可变借用
3. `?` 只能附加在 `T!E` 调用表达式后；不能用来传播 null
4. `??` 只能用于 `?T`；不能捕获错误
5. comptime 函数 `@name()` 不能在运行时表达式位置调用
6. 不能返回悬垂借用；生命周期检查拦截
7. 闭包捕获同样遵守所有权、move、借用规则
8. 不支持函数重载；函数名在模块内唯一

---

### 5.12 函数修饰符与前缀矩阵

函数修饰符保持正交、严格正序：

- `pub`：公开函数；
- `priv`：私有函数；
- `static`：类中静态方法，不绑定实例；
- `async`：异步函数，切分为 CPS 状态机；
- `unsafe`：不安全函数，调用方必须位于 `unsafe { }` 作用域内。

在 CP 语言中，函数前缀 pub 与 priv 是用于控制**符号可见性（Visibility）与接口封装性（Encapsulation）**的访问控制修饰符。
它们在**模块级（顶层函数）与面向对象级（类/结构体方法）**中发挥着关键作用：
──────

#### 5.12.1 核心作用与分层语义

                            ┌── pub   => 公开可见（对外导出，跨模块/类外部可调用）
    函数可见性修饰符 (Visibility) ┤
                            └── priv  => 私有封装（仅当前文件内 / 当前类内部可见）

#### 5.12.1 模块层级（Top-level Functions）

| 修饰符                | 示例                                         | 作用与行为                                                                                    | 底层行为与映射                            |
| --------------------- | -------------------------------------------- | --------------------------------------------------------------------------------------------- | ----------------------------------------- |
| **pub**               | `pub fn serve() {}` (等价于 `export fn`)     | 公开导出（Exported）：允许其他 `.cp` 文件通过 `import { serve } from "./server"` 导入并调用。 | 映射为 Zig 的 `pub fn`，生成导出符号。    |
| **priv** (或默认缺省) | `priv fn helper() {}` (或直接 `fn helper()`) | 模块私有（Module-private）：仅供当前 `.cp` 文件内部的逻辑使用，禁止被外部模块 import。        | 映射为 Zig 内部局部函数，不对外暴露符号。 |

#### 5.12.2 类与结构体层级（Class / Struct Methods & Fields）

在 CP 语言中，依据“数据与行为解耦、纯数据透明与面向对象强封装”的设计哲学，`struct` 与 `class` 的成员默认可见性进行了明确且符合工程直觉的分流设计：

1. **`struct`（纯数据容器 / DTO）**：
   - **默认全员 `pub` 公开**：作为纯值类型与系统内存布局容器，结构体字段与方法默认全为 `pub`，外部可直接透明访问，无需手写繁琐的 `pub` 样板代码；
   - 若个别内部辅助方法需隐藏，可显式标注 `priv`。
2. **`class`（面向对象 / 业务状态机）**：
   - **默认全员 `priv` 私有封装**：作为持有状态与生命周期的业务类，内部字段与方法默认均为 `priv`，外部访问将被编译器静态拦截；
   - 只有显式标注 `pub`（或 `export`）的字段与方法才对外公开暴露。
3. **`static class` / `static struct`（纯静态命名空间）**：
   - **全员自动静态**：内部成员全部自动为静态成员，无需每个成员手写 `static`，且严禁实例化。

| 容器类型                  | 成员默认可见性          | 显式修饰符支持 | 设计目的与典型场景                                           |
| :------------------------ | :---------------------- | :------------- | :----------------------------------------------------------- |
| **`struct`**              | **`pub` 公开（默认）**  | `priv` / `pub` | 纯数据模型、C ABI 互操作、高效网络 DTO，透明访问零摩擦       |
| **`class`**               | **`priv` 私有（默认）** | `pub` / `priv` | 面向对象封装、防卫性状态管理、内部不变量保护                 |
| **`static class/struct`** | **全员自动静态**        | `pub` / `priv` | 纯工具函数库、全局数学常量、硬件命名空间，彻底杜绝实例与开销 |

> **生命周期方法豁免**：
> `init` 构造器、`release` 析构器以及编译器底层内部方法（`_create`、`__*`）属于对象的生命周期管理特权函数，不受外部私有属性/方法拦截限制。

```typescript
// 1. struct 默认所有成员 pub
struct Point {
    x: f64; // 默认 pub
    y: f64; // 默认 pub
}
let p = Point { x: 10.0, y: 20.0 };
print(p.x); // ✅ 直接访问合法

// 2. class 默认所有成员 priv
class BankAccount {
    pub owner: string;       // 显式 pub：外部可读
    priv balance: i64;       // 显式 priv：私有安全
    account_no: string;      // 未标注：默认等价于 priv

    pub fn deposit(*_, amount: i64) {
        if (validate_amount(amount)) { // 内部调用私有方法
            balance = balance + amount;
        }
    }

    pub fn get_balance(): i64 {
        return balance;
    }

    fn validate_amount(amt: i64): bool { // 默认 priv 私有方法
        return amt > 0;
    }
}

let acc = new BankAccount("Alice", 100);
print(acc.owner);          // ✅ 正常：pub 属性
// print(acc.balance);     // ❌ 编译拦截：Cannot access private field 'balance'
// acc.validate_amount(50);// ❌ 编译拦截：Cannot call private method 'validate_amount'
acc.deposit(50);           // ✅ 正常：pub 方法
```

### 5.13 方法接收者（Receiver）终极哲学

在 CP 语言中，**`_` 被确立为方法接收者（Receiver）的唯一专属符号**（等同于 TS 中的 `this`）：

```typescript
class Account {
    balance: f64;

    // 1. 只读调用：参数中的 `_` 可完全缺省省略（默认注入只读借用 *const Self）
    pub fn get_balance(): f64 {
        // 方法体内部可直接裸写 balance 访问属性，无需 _. 或 this.
        return balance;
    }

    // 2. 状态变更调用：必须显式声明 *_ 或 mut _（独占可变借用）
    pub fn deposit(*_, amount: f64) {
        balance = balance + amount;
    }

    // 3. 消费自身调用：必须显式声明 move _（转移所有权，调用后原实例被销毁）
    pub fn close(move _): f64 {
        let final_cash = balance;
        balance = 0.0;
        return final_cash;
    }
}
```

> **方法前缀与静态函数规则**：
>
> 1. **只读接收者缺省**：非 `static` 函数在类/结构体中默认均为实例方法，若仅为只读借用，形参列表里的 `_` **完全可以省略**；
> 2. **内部成员直接访问**：方法体内访问属性或兄弟方法时，直接裸写 `field` 或 `method(...)`。只有当形参或局部变量与类字段同名发生**作用域遮蔽（Shadowing）**时，才需使用 `_.field` 消除二义性；
> 3. **静态函数明确隔离**：静态方法必须显式冠以 `static` 关键字（如 `static fn create(): Account`），与实例方法正交隔离，绝不产生识别混淆。

---

### 5.14 返回值类型与省略原则

- **`void` 省略哲学**：当函数无返回值时，直接省略 `: void` 声明；
- **错误联合类型与正交省略规范**：
  - **极简推导错误联合（`T!`）**：正常类型 `T` 永远置前，后缀感叹号 `!` 代表函数成功时返回 `T`，失败时抛出通用推导错误 `err`（在底层 Zig 中映射为编译期自动收集的开放错误集 `!T`）：
    ```typescript
    fn calc(source: string): usize! {
        return source.len * 4 / 3;
    }
    ```
  - **`void!` 正交省略语法（`fn sum()!` 或 `fn sum(): !`）**：
    当函数无成功返回值（成功返回 `void`），但可能产生错误时，**允许省略 `void` 并保留后缀修饰符 `!`**：
    ```typescript
    // 推荐最简形式：省略 void，保留 ! 显式预警
    fn flush_cache()! {
        step()?;
        if (failed) return err.IoError;
    }

    // 等价兼容写法：
    fn flush_cache2(): ! { ... }
    fn flush_cache3(): void! { ... }
    fn print_msg(msg:string)? {
      print(msg);
    }

    fn print_msg(msg:string)?! {
      print(msg);
    }
    ```
    > [!IMPORTANT]
    > **契约强防御设计戒律**：严禁无任何标记的隐式推导（即 `fn sum() { ... }` 绝不可静默隐式推导为 `void!`）。函数的失败能力属于对外接口强契约（Contract），必须在签名处显式保留 `!`，以彻底避免内部改动导致调用方全局报错的错误蔓延灾难，并保持编译器 O(1) 极速解析。
  - **强类型显式错误联合（`T ! ErrorSet`）**：指定具体错误枚举时：
    ```typescript
    fn save(): void ! IOError  // 或简写为 fn save() ! IOError
    fn find(): User ?! DBError
    ```

---

### 5.15 静态类与静态结构体（static class 与 static struct）

#### 5.15.1 设计定位与工程价值

在系统与应用工程中，大量业务需要组织**纯工具函数库、全局数学常量、硬件命名空间或纯静态状态管理器**。在这些场景下，我们只需要一个纯净的代码与数据命名空间，而不需要堆分配、虚表寻址，更不需要对象实例化。

在传统语言中：

- **TypeScript / JavaScript**：虽然支持 `class`，但必须在每一个字段和方法前重复手写 `static` 关键字；若不小心写了普通方法，实例化后会产生原型链与内存开销；
- **C#**：引入了专用的 `static class` 关键字，被编译器严格限制为纯静态容器，无法实例化，非常严谨优雅；
- **Rust**：没有 `class` 关键字，通常使用 `struct` 加 `impl`（不带 `&self` 接收者的方法作为关联函数），或使用 `mod utils { pub fn ... }` 模块；
- **Go**：没有 `class` 也不支持类静态成员，只能通过包级导出函数（如 `utils.Add()`）；
- **Python**：通过模块级函数或在类中使用 `@staticmethod` / `@classmethod` 装饰器。

CP 语言汲取了现代语言设计的精髓，引入一等公民的 **`static class`** 与 **`static struct`**：

1. **全员自动静态**：类/结构体内部的所有字段（`let` / `const`）和方法（`fn`）**默认全为静态成员**，开发者无需且不应在每个成员前冗余手写 `static` 关键字。
2. **免除生命周期与实例负担**：纯静态容器在概念上没有实例概念，因此**彻底无需也不允许声明 `init` 构造函数与 `release` 析构函数**。
3. **彻底杜绝误实例化**：编译期强力拦截 `new StaticClass()` 以及结构体实例化表达式，违规调用直接报编译硬错误。
4. **禁止类继承**：`static class` 严禁使用 `extends` 继承或被继承，语义绝对纯粹。
5. **严禁实例接收者**：所有方法内部没有实例上下文，严禁声明带 `_`、`*_`、`move _` 或 `this` 的方法接收者。

#### 5.15.2 语法规范与使用示例

```typescript
// 1. 静态类：纯工具方法与全局只读常量
static class MathUtils {
    // 字段自动成为静态成员，支持 const 常量与 let 可变状态
    pub const PI: f64 = 3.141592653589793;
    pub const E: f64 = 2.718281828459045;
    pub let call_count: i32 = 0;

    // 方法自动成为静态方法，无需接收者 _ / this
    pub fn add(a: i32, b: i32): i32 {
        call_count += 1;
        return a + b;
    }

    pub fn max(a: f64, b: f64): f64 {
        if (a > b) {
            return a;
        }
        return b;
    }
}

// 2. 静态结构体：用于底层命名空间与无实例辅助容器
static struct StringHelper {
    pub const TAG: string = "StringHelper";

    pub fn is_empty(s: string): bool {
        return s.len == 0;
    }
}

// 调用方式：通过 类型名.成员名 直接访问
let sum = MathUtils.add(10, 20);
let pi = MathUtils.PI;
let empty = StringHelper.is_empty("");
```

#### 5.15.3 编译期五重硬拦截法则

编译器前端（Parser、Sema 与 Codegen）对 `static class` / `static struct` 构建了五重防御校验：

1. **拦截实例化**：
   - 表达式 `new MathUtils(...)` -> `CP Compile Error: Cannot instantiate static class or struct 'MathUtils'`
   - 表达式 `StringHelper { ... }` -> `CP Compile Error: Cannot instantiate static class or struct 'StringHelper'`
2. **拦截实例接收者**：
   - 声明 `pub fn foo(this, a: i32)` 或 `pub fn foo(*_, a: i32)` -> `CP Parser Error: Static class or struct 'MathUtils' cannot declare instance method 'foo' with receiver 'this' or '_'`
3. **拦截实例生命周期钩子**：
   - 声明 `pub fn init(...)` 或 `pub fn release(...)` -> `CP Parser Error: Static class or struct 'MathUtils' cannot declare instance lifecycle method 'init'`
4. **拦截类继承**：
   - 声明 `static class Foo extends Bar` -> `CP Parser Error: Static class or struct 'Foo' cannot extend another class`
5. **豁免实例内存分配**：
   - 代码生成阶段，编译器自动豁免生成底层的 `_create` 堆分配器函数与实例字段内存布局，直接转译为 Zig 原生容器级的纯净命名空间，零运行时内存开销。

---

### 5.16 枚举与富方法扩展 (Enums with Methods)

CP 语言将 `enum` 提升为一等行为实体，不仅支持命名变体与数值绑定，还允许直接为枚举声明成员方法与静态方法：

```typescript
enum LogLevel {
    Debug = 0,
    Info = 1,
    Error = 2,

    pub fn is_critical(this): bool {
        return this == LogLevel.Error;
    }

    pub fn tag(this): string {
        if (this == LogLevel.Error) return "[ERR]";
        if (this == LogLevel.Info) return "[INF]";
        return "[DBG]";
    }
}

let level = LogLevel.Error;
let critical = level.is_critical(); // 返回 true
let label = level.tag();            // 返回 "[ERR]"
```

- **语义分析推导**：`sema.rs` 自动在 `Expr::Member` 与 `Expr::Call` 中索引枚举方法签名，精准推导返回值类型，彻底杜绝类型退化为 `any`；
- **底层零成本映射**：枚举方法直接映射为 Zig 编译期内联函数，无需虚表分配，享受零运行时开销。

---

### 5.17 C-FFI 底层内存布局与联合体 (packed/extern struct & union)

针对操作系统内核对接、网络协议栈与 POSIX 原生绑定场景，CP 原生支持 C 兼容与位级紧凑内存布局：

```typescript
// 1. 位级紧凑对齐（用于硬件寄存器与网络二进制协议帧）
packed struct Header {
    version: u4,
    ihl: u4,
    tos: u8,
    total_len: u16,
}

// 2. 严格遵循 C ABI 内存对齐
extern struct PosixStat {
    dev: u64,
    ino: u64,
    mode: u32,
}

// 3. 联合体：多成员共享同一物理内存
packed union SocketAddr {
    raw: [16]u8,
    v4: u32,
}

// 4. 不透明指针结构体（仅持有句柄指针，外部不展开成员）
opaque struct us_loop_t {}
```

- **词法与解析安全**：解析器内建检查点（Checkpoint）机制，自动区分带字段名结构体字面量 `.{ .key = val }` 与顺序列表 `.{ val }`，并在遇到 `packed`/`extern`/`opaque` 时精准提取原始布局，零解析歧义。

---

## 六、所有权、借用检查与内存安全模型（Zero-Friction Memory Safety）

> 底层模型借鉴Rust所有权系统，但是语法采用CP自定义借用标记：`&T` 只读借用、`*T` 可变借用；显式 `move` 关键字转移所有权。
> 核心纲领：**每一个值有且仅有一个所有者；所有者离开作用域，资源自动释放；借用不接管所有权，遵守借用互斥规则。**

CP 语言通过编译器底层的“所有权、隐式借用、读写互斥与零摩擦生命周期”四大支柱，实现了对标 Rust 的 100% 编译期内存安全，同时彻底摒弃了 Rust 繁琐的 `<'a>` 生命周期泛型传染与冗长的 `&mut` 噪音，完美保留了 TypeScript 般丝滑的开发心智。

### 6.1 借用、所有权与指针符号统一标准 (Global Symbol Matrix)

- **`&`（Ampersand）**：代表 **只读借用（Read-Only Borrow）**（在默认只读的位置可省略 `&`）。
- **`*`（Star，无尾置 `!`）**：代表 **安全可变借用（Mutable Borrow）**（因指针解引用采用后缀 `p.*`，前置 `*` 全局专职代表受编译器生命周期与读写互斥保护的可变借用）。
- **`*... !`（前置 `*` + 尾置 `!`）**：代表 **Unsafe 裸指针（Raw Pointer）**（前置 `*` 个数代表指针层级，尾置 `!` 代表非托管裸指针，仅限 `unsafe` 块内使用）。
- **`move`（及直接赋值 `let a = b`）**：代表 **所有权转移（Ownership Transfer）**。

### 6.2 语法规范

| 语法场景                    | 只读借用 (Read-Only)                                                | 可变借用 (Mutable)                                     | 所有权转移 (Move)                              | Unsafe 裸指针 (Raw Ptr)          |
| :-------------------------- | :------------------------------------------------------------------ | :----------------------------------------------------- | :--------------------------------------------- | :------------------------------- |
| **1. 函数参数**             | `fn f(x: T)` _(默认省略)_<br>_(兼容显式 `fn f(&x: T)`)_             | **`fn f(*x: T)`**<br>_(或 `fn f(x: *T)`)_              | **`fn f(move x: T)`**                          | `unsafe fn f(p: *T!)`            |
| **2. 变量赋值**             | **`let a = &b`**                                                    | **`let a = *b`**                                       | **`let a = b`**<br>_(或 `let a = move b`)_     | `let a = *b!`<br>`let pp = **a!` |
| **3. 类型声明 / 返回值**    | **`: &T`**                                                          | **`: *T`**                                             | **`: T`**                                      | **`: *T!`** / **`: **T!`**       |
| **4. 对象/结构 `this`/`_`** | `fn get()` _(默认省略 `_`)_<br>*(兼容 `fn get(_)` / `fn get(&_)`) * | **`fn set(*_, v: i32)`**<br>_(兼容 `*this` / `mut _`)_ | **`fn close(move _)`**<br>_(兼容 `move this`)_ | —                                |
| **5. `for` 循环迭代**       | `for (x of list)` _(默认省略)_<br>_(兼容 `for (&x of list)`)_       | **`for (*x of list)`**                                 | **`for (move x of list)`**                     | —                                |

### 6.3 所有权与自动释放 (Ownership & Auto-Drop)

- **局部变量去冗余**：废除局部变量的 `let mut`，统一使用 `let`（可变绑定）与 `const`（深度不可变）。
- **严禁同作用域重复声明变量（No Same-Scope Redeclaration & `*` Wildcard）**：对标 TypeScript 与 Zig 的工程安全直觉，在同一个 `{}` 块作用域或同一个解构/模式匹配中严禁重复绑定同名变量（如连续写 `let v = 10; let v = [10, 20];`、`let [a, a] = [1, 2];` 或 `let [_, _] = [1, 2];`），编译器将直接报错拦截：`Compile Error: Cannot redeclare block-scoped variable 'v'`。**注意：CP 的模式匹配与解构通配符是 `*`（而非 `_`），`_` 是类/结构体方法接收者 `this` 的别名或普通变量标识符，重复使用 `_` 同样触发同名编译报错，丢弃元素请使用 `[*, second, *]` 或 `let * = expr;`。**
- **创建即拥有**：`let tensor = new Tensor();` 创建对象时，`tensor` 拥有该堆内存的唯一所有权。
- **作用域释放 (RAII)**：当所有者变量离开作用域且未被转移所有权时，编译器自动注入 `defer tensor.release()`（无需 GC，零停顿）。
- 函数返回自动移交所有权，不需要在return处额外写move：

```cp
fn new_user(): User {
return User{id:3, name:"abc"}; // 返回值所有权交给调用方
}
```

- **赋值即 Move**：`let a = b;`（或显式 `let a = move b;`）会将 `b` 的所有权转移给 `a`；而 `let a = &b;` 创建只读借用，`let a = *b;` 创建可变借用。所有权转移后继续访问 `b` 将触发编译期错误：`CP Compile Error: Use of moved value`。

### 6.4 隐式借用与读写推导 (Implicit Borrow Deduction)

函数传参默认**不转移所有权**，而是由编译器自动生成**隐式借用**。借用权限由函数签名中的**参数修饰符**决定：

- **只读借用 (默认或 `&`)**：`fn view(data: Tensor)`（或 `fn view(&data: Tensor)`）-> 调用 `view(a)` 时，自动传入只读引用 `&T`，函数内禁止修改。
- **可变借用 (`*`)**：`fn modify(*data: Tensor)`（或 `fn modify(data: *Tensor)`，兼容 `mut data: Tensor`）-> 调用 `modify(a)` 时，自动传入可变引用 `*T`，允许原地修改。
- **转移所有权 (`move`)**：`fn consume(move data: Tensor)` -> 调用 `consume(a)` 时，转移所有权，调用后外层 `a` 立即失效。
- **借用本身也可以被借用**：可以对 `&T`、`*T` 再次取只读借用。

### 6.5 借用检查器规则与可变借用操作规范 (Borrow Checker & Mutable Borrow Operations)

- **读写互斥**：在同一有效生命周期内，允许存在多个只读借用（`&T`），或仅允许存在**唯一一个**可变借用（`*T`）。可变借用与任何其他借用严格互斥。
- **借用期间锁定所有者**：当变量 `a` 存在尚未结束的活跃借用 `r`（即 `r` 在后续代码中还会被读取）时，严禁将 `a` 的所有权 Move 转移或对其进行破坏性修改。不能对已经move走的值创建借用。
- **可变借用变量的三类核心操作（自动解引用与显式 `.*`）**：
  1. **基础值类型（`i32`, `f32`, `bool` 等）**：通过后缀解引用 `p.* = val` 原地修改所借用变量的值；
  2. **结构体 / 对象字面量（`{a: 10, b: 20}` / `struct` / `class`）**：支持自动解引用，直接通过 `p.a = val` 原地修改字段；
  3. **容器类型（`Array` / `Vec` / `T[]`, `Map`, `Set`, `Dict`）**：支持自动解引用，直接通过 `p.push(...)`、`p.pop()`、`p.set(...)`、`p.add(...)`、`p.delete(...)`、`p.clear()`、`p[idx] = val` 原地修改容器内容。
  ```cp
  // 1. 基础类型可变借用：后缀 .* 解引用赋值
  {
      let v = 10;
      let p = *v;
      p.* = 20; // v 同步变为 20
  }

  // 2. 对象/结构体可变借用：自动解引用修改字段
  {
      let v = {a: 10, b: 20};
      let p = *v;
      p.a = 100; // v.a 同步变为 100
  }

  // 3. 数组/容器可变借用：自动解引用调用可变方法与索引赋值
  {
      let v = [10, 20];
      let p = *v;
      p.push(30); // v 同步扩容为 [10, 20, 30]
      p[1] = 25;
  }
  ```
- **容器改变操作全面纳入可变借用检查 (`Array` / `Vec`, `VecDeque`、`BinaryHeap`、`BTreeMap`、`BTreeSet`、`Map`/`HashMap`、`Set`/`HashSet`、 `Dict`)**：
  - **受检容器改变操作清单**：
    - `Array<T>` / `Vec<T>` / `T[]`：`push(v)`、`pop()`、`clear()`、索引写入 `arr[i] = v`、`for (*x of arr)` 原地可变遍历；
    - `Map<K, V>`：`set(k, v)`、`delete(k)`、`clear()`、索引写入 `map[k] = v`；
    - `Set<T>`：`add(v)`、`delete(v)`、`clear()`；
    - `Dict<K, V>` (`#{}`)：`set(k, v)`、`put_all(other)`、`delete(k)`、`clear()`、索引写入 `dict[k] = v`。
  - **四重编译期拦截保障**：
    1. **禁止通过只读借用 (`&v`) 或只读参数改变容器**：如 `let r = &v; r.push(30);` 直接报错 `Cannot perform mutable operation (container method 'push') through read-only borrow 'r'`；
    2. **读写互斥与活跃借用冲突拦截**：当容器存在活跃的只读借用 `let r = &m;` 或另一个可变借用时，对原容器或冲突借用执行 `set`/`push`/`delete`/`clear`/`[] =` 立即报错 `Borrow conflict on 'm': cannot borrow as mutable while 'r' is still active`；
    3. **防御迭代器失效（Loop-Borrow Lock）**：在 `for (x of container)` 或 `for (*x of container)` 遍历期间，严禁在循环体内对该容器（或其借用别名）调用 `push`、`pop`、`set`、`add`、`push_back`、`push_front`、`insert`、`remove`、`delete`、`clear` 或重赋值（禁止 `v[0] = 6`）；
    4. **Use-After-Move 拦截**：容器所有权 `move` 转移后，任何读写或方法调用立即报错拦截。

```typescript
let v = [1, 2, 3];
let p = &v; // 建立只读借用

// v[0] = 6;
// 编译拦截并抛出硬错误：
// CP Compile Error: Borrow conflict on 'v': cannot borrow as mutable while 'p' is still active
print(p[0]); // p 在此为最后一次使用
v[0] = 6;    // 合法：p 生命周期已由 NLL 释放
```

### 6.6 借用、可空、错误联合交互规则

1. `T?`（可空类型）：只有当switch确认不为null之后，才允许对内部T创建借用；对null分支取借用编译报错。
2. `T!E`（错误联合）：只有switch匹配到正常T分支，才能对T创建借用；err分支不能取T的借用。
3. `?` 错误冒泡、`??` null兜底之后得到的有效值，才支持创建借用。
4. 借用本身也可以包装成可空、错误联合：`(&T)?`、`&(T!E)` 合法，同样遵守借用检查。

```cp
let maybe: User? = User{id:5};
switch(maybe) {
    v => {
        let rv = &v; // ✅ 分支内v有效，可以借用
    }
    null => {}
    * => {}
}
```

### 6.7 借用与裸指针边界（unsafe逃逸与指针转换体系）

> 裸指针 `*T!` / `**T!` / `*const T!` / `[*]T!` / `*any!` 不属于借用体系，**不受借用检查器（Sema）生命周期保护**，仅能在 `unsafe {}` 块内使用。

#### 6.7.1 核心区分：安全借用 (`*T`) vs 底层裸指针 (`*T!`)

- **`*T`（无尾置感叹号）**：**安全可变借用**，属于编译器安全体系，严格受借用检查器（Sema）、读写互斥与非词法生命周期（NLL）约束；
- **`*T!`（尾置感叹号）**：**底层可变裸指针**，映射至 C/Zig 单项指针 `*T`，用于底层系统级编程与 C-ABI 交互；
- **`*T?!` 或 `*T!?`**：**底层可选裸指针**，映射至 Zig `?*T`；
- **`**T!`**：**二级底层裸指针**，映射至 C/Zig `**T`，用于外部 C 库二级出参句柄初始化；
- **`*const T!`**：**底层只读裸指针**，映射至 Zig `*const T`，禁止写入操作；可选版为 `*const T?!`，映射至 Zig `?*const T`；
- **`[*]T!`**：**未定长多项连续裸指针**，映射至 Zig `[*]T`，支持下标 `ptr[i]` 与切片 `ptr[a..b]` 操作；可选版为 `[*]T?!`，映射至 Zig `?[*]T`；
- **`[*c]T!`**：**C 兼容裸指针**，映射至 Zig `[*c]T`，用于与传统 C-ABI 头文件直接无损对接；可选版为 `[*c]T?!`，映射至 Zig `?[*c]T`；
- **`[*:0]T!`**：**哨兵终止裸指针**，映射至 Zig `[*:0]T`，支持内部元素按字节就地修改与 C 风格字符串缓冲区操作（如 `[*:0]u8!`）；可选版为 `[*:0]T?!`，映射至 Zig `?[*:0]T`；
- **`[*:null]T!`**：**null 哨兵终止裸指针**，映射至 Zig `[*:null]T`，用于以 null 结尾的指针数组；
- **`*any!`**：**通用非类型化裸指针**，映射至 Zig `*anyopaque`（即 C 语言的 `void*`）；可选版为 `*anyopaque?!`，映射至 Zig `?*anyopaque`；
- **括号优先级消歧 `(Type)`**：当裸指针修饰符组合嵌套时，支持小括号显式指明内层指针与外层修饰符结合性：
  - `[*]([*:0]u8!)?!` 映射为 Zig `?[*][*:0]u8`（以 0 结尾的字符串指针的多项连续可选数组）；
  - `[*:null]([*:0]u8?!)?!` 映射为 Zig `?[*:null]?[*:0]u8`（以 null 结尾的可选 C 字符串指针数组）；
  - `*(*T?!)` 映射为 Zig `*?*T`（指向可选指针的指针，常用于 C 库二级出参返回）。

#### 6.7.1.1 `cstr` / `cstring` 一等公民规范与内存越界防护机制

在 CP 语言中，与 C-ABI 原生兼容的零结尾（Null-Terminated）字符串拥有双层抽象：

- **`cstring`（带 0 哨兵切片 `[:0]const u8` / `[:0]u8`）**：附带长度信息的**胖指针（Fat Pointer = 指针 `ptr` + 长度 `len` + 结尾 `\0` 哨兵）**，是 CP 中最高频、最安全的一等公民；
- **`cstr`（零结尾连续指针 `[*:0]const u8` / `[*:0]u8`）**：C-ABI 原生薄指针（Thin Pointer），无内联长度字段，专注于高频系统 FFI 传参与底层操作。

##### 1. 一等公民地位：彻底脱离 `unsafe {}` 限制

在许多系统语言中，操作 C 风格字符串或裸指针必须全程被丑陋的 `unsafe { ... }` 块包裹。而在 CP 语言中，**`cstr` 与 `cstring` 均是一等公民（First-Class Citizens）**，在安全代码中拥有完整的语法和生命周期支持：

- **无需 `unsafe` 即可声明、传参和读取**：
  ```typescript
  // 1. 在普通安全函数中直接声明与传递
  fn print_c_info(title: cstr, version: cstring): void {
      print(`Title: ${string.from(title)}, Version len: ${version.len}`);
  }

  // 2. 局部变量直接初始化
  let app_name: cstring = "Antigravity-Engine";
  let c_title: cstr = "CP Native Runtime";
  print_c_info(c_title, app_name);
  ```
- **借用检查与所有权严密保护**：
  编译器对 `cstr` / `cstring` 的参数和变量同样运行完整的借用检查（Borrow Checker）。只读参数不可被可变借用，生命周期超出宿主作用域的逃逸会被编译期硬性拦截。

##### 2. `cstr` / `cstring` 支持的方法与操作全景

| 方法 / 语法特性           | `cstring` (带长度切片 `[:0]u8`)      | `cstr` (C原生指针 `[*:0]u8`)             | 功能说明与性能特性                                                |
| :------------------------ | :----------------------------------- | :--------------------------------------- | :---------------------------------------------------------------- |
| **`.len` 属性**           | ✅ **O(1) 瞬时获取**                 | ❌ 无内联字段（需 `string.from(s).len`） | `cstring` 自带切片长度信息，**无需调用昂贵的 `strlen` O(N) 扫描** |
| **`.ptr` 属性**           | ✅ 获取底层的 `[*:0]const u8`        | ✅ 自身即为原生指针                      | 用于无缝直通传给第三方底层 C 动态库函数                           |
| **下标读取 `s[i]`**       | ✅ 支持，返回第 `i` 个字节 `u8`      | ✅ 支持，返回第 `i` 个字节 `u8`          | 随机读取指定偏移处的字节值                                        |
| **下标写入 `s[i] = 'A'`** | ✅ 仅当 `let` 可变或 `*s` 可变借用时 | ✅ 仅当 `let` 可变或 `*s` 可变借用时     | 允许对缓冲区就地可变覆写（受只读借用保护）                        |
| **普通切片 `s[a..b]`**    | ✅ 支持，返回普通切片 `string`       | ✅ 支持，截取为普通只读切片              | 零拷贝内存视图重构                                                |
| **哨兵切片 `s[a..b :0]`** | ✅ 支持，保留零结尾哨兵特性          | ❌ 不直接支持                            | 截取带零结尾的子切片                                              |
| **`string.from(s)`**      | ✅ 零拷贝转换为 CP `string`          | ✅ 零拷贝转换为 CP `string`              | 自动计算/透传长度，零堆分配、零内存复制                           |
| **`s.to_string()`**       | ✅ 堆分配克隆为独立字符串            | ✅ 堆分配克隆为独立字符串                | 产生独立的堆内存副本                                              |

##### 3. 可变性规范：`let` vs `const` 与参数传递契约

1. **`let s: cstr = "hello";`（可变 C 字符串）**：
   - 映射为 Zig `var s: [*:0]u8`；若字面量初始化则由编译器前端自动注入安全 `@constCast`；
   - **允许按字节就地修改**：支持 `s[idx] = val`；
2. **`const s: cstr = "hello";`（不可变只读 C 字符串）**：
   - 映射为 Zig `const s: [*:0]const u8`；
   - **借用检查器硬性拦截**：禁止任何按字节索引修改（尝试 `s[idx] = val` 在语义分析期抛出 `SemaError::ConstAssignment`）；
3. **函数参数传递契约**：
   - 只读借用参数 `fn foo(s: cstr)`：映射为 Zig `s: [*:0]const u8`，函数体内尝试 `s[idx] = val` 编译拦截并抛错：`cannot perform mutable operation on read-only parameter 's'`；
   - 独占可变借用参数 `fn foo(*s: cstr)`：映射为 Zig `s: [*:0]u8`，允许函数体内部对字符串就地修改。
4. **显式借用类型**：
   - `&cstr` 映射为 `[*:0]const u8`（只读借用）；
   - `*cstr` 映射为 `[*:0]u8`（独占可变借用）；
   - `*cstring` 映射为 `[:0]u8`（独占可变带哨兵切片）。

##### 4. 越界写入与三重安全防御网（The Triple Safety Guardrails）

> **疑问解答**：如果声明了可变字符串或缓冲区，执行 `s[9999] = 'A'` 越界写入，程序会崩溃吗？CP 语言中如何确保安全？运行时触发 Fail-Fast Panic 的作用是什么，能通过捕获异常防止崩溃吗？

在传统的 C/C++ 语言中，裸指针 `char*` 没有长度信息，向未验证的下标写入（如 `s[9999] = 'A'`）会直接引发恶名昭彰的 **堆栈缓冲区溢出（Buffer Overflow）**，静默覆写相邻变量、破坏调用栈帧或触发未定义行为（UB）。

CP 语言通过以下**三重安全防御网**，彻底杜绝越界隐患：

###### 第一重防御：编译期已知边界检查（Comptime Bounds Check）

对于字面量、编译期已知物理大小的定长缓冲区（如 `host_buf: [256]u8`）或带静态长度的切片，当访问的索引是常量表达式时，CP 编译器的常量折叠与语义分析期（Sema）会**在编译期静态发现越界并直接拒绝编译**：

```typescript
let host_buf: [256]u8 = undefined;
let s: cstr = &host_buf[0];

// ❌ cpc 编译期静态检测出索引 300 超过 256 物理边界，直接报错拦截！
s[300] = 'A';
```

编译器输出 AI 友好的三段式诊断：

```text
error[E0217]: index out of bounds: index 300 is out of bounds for buffer of length 256
  --> src/main.cp:12:3
   |
12 |   s[300] = 'A';
   |   ^
   |
   = cause: index 300 exceeds compile-time known length 256 of the buffer.
   = help: accessing a fixed-size array or known-length buffer beyond its bound is a compile-time fatal error.
   = suggestion: ensure index is within 0..256, or use safe slicing: 'slice = s[0..max_len]'
```

在编译期直接扼杀隐患，零运行时损耗！

###### 第二重防御：CP 官方倡导的“切片优先”安全惯用法（Slice-First）

在安全代码（Safe Code）中，修改 C 字符串或底层内存的最佳实践是**先切片再索引修改**，或直接使用自带长度元数据的安全切片 `cstring`（`[:0]u8`）：

```typescript
let s: cstring = "hello"; // 具备确定长度 len 为 5 (有效范围 0..4，5 为哨兵 \0)

// 转换或提取带明确长度的安全切片
let slice = s[0..max_len]; // 转换为 []u8
slice[1] = 'A'; // 若动态索引超出 max_len，触发带行号与完整调用栈的安全 Panic！
```

切片自带动态长度，在 `Debug` 与 `ReleaseSafe` 模式下，底层机器码在每一次内存寻址写入前，会自动注入硬件级边界校验（`index >= slice.len`）。一旦越界，在任何破坏性物理写入发生之前，立即触发确定性 Fail-Fast Panic，绝不发生静默脏数据污染！

###### 第三重防御：无界裸指针修改强制隔离进 `unsafe {}`

对于来源未知、无编译期长度元数据的裸内存下标操作（如 C FFI 传入的未定长指针 `[*]u8!` 或裸 `*cstr!`），CP 划出绝对安全红线：**安全域（Safe Context）中严禁直接对无界裸指针进行下标写入！**

```typescript
// ❌ 普通 Safe 域中盲目下标写入，cpc 直接编译拦截：
raw_ptr[100] = 'A';
// error[E0209]: mutation via unbounded raw pointer or cstr 'raw_ptr' requires 'unsafe' block
//   = cause: modifying memory via unbounded pointer/cstr 'raw_ptr' without statically known length is unsafe.
//   = help: raw pointers and unbounded cstr buffers do not carry runtime length metadata. Modifying memory through them can lead to buffer overflows.
//   = suggestion: wrap the mutation in 'unsafe { raw_ptr[...] = ...; }' or slice it first: 'let slice = raw_ptr[0..len]; slice[...] = ...;'

// ✔️ 合法：显式纳入 unsafe 块，审计面极度收窄
unsafe {
    raw_ptr[100] = 'A';
}
```

---

##### 5. Fail-Fast Panic 的本质作用与高可用自愈架构（Fail-Fast & Crash-Proof Architecture）

> **架构释疑**：底层注入边界检查触发 `panic: index out of bounds: index 10, len 5` 的作用是什么？能通过类似 `try/catch` 捕获异常来防止程序崩溃吗？

###### 5.1 Panic（致命逻辑 Bug / 不变式破坏）vs Exception/Error（业务可预期分支）

在现代顶级系统级语言（Rust、Zig、CP）中，Panic 与业务错误有着本质分野：

1. **业务错误（Business Errors：`T ! Err` / `?` 传播）**：
   - 网络断开、文件未找到、请求超时、输入校验不通过等，属于**系统在设计上已知且必然会发生的正常业务分支**；
   - CP 废除传统低效且容易遗漏的 `try/catch`，统一使用强类型错误联合 `T ! E`、错误传播符 `?` 以及模式匹配 `switch`，进行确定性、显式的流式处理。
2. **安全 Panic（Invariants Violation / Fatal Logic Bug）**：
   - 数组/切片越界访问、除以零、空指针解引用断言失败等，代表**程序员的代码存在严重的代码 Bug，或者底层数据结构、内存的不变式（Invariants）已被物理破坏**！

###### 5.2 为什么绝不能通过 Try/Catch 捕获越界 Panic 并“带病运行”？

很多动态脚本语言（如 Python、JS）允许用 `try { slice[999] = 'A' } catch { ... }` 盲目吞掉越界异常，但在高并发系统与金融/AI 级工程中，这是**极其危险的反模式**：

- **静默脏数据灾难**：如果越界发生在核心数据结构写入的中间状态，一旦被捕获强行继续往下跑，程序将携带已被损坏的半成品内存状态继续执行：
  - 示例：在转账事务中，扣减账户 A 成功，但在写入收款方 B 时越界；若捕获异常并吞掉，事务后续提交，将导致永久性的资金凭空蒸发与不可逆的历史错账！
  - 示例：在 AI Agent 推理中，张量索引越界被吞掉，模型在错位权重下持续输出具有严重误导性的幻觉结果，造成静默业务事故。
- **杜绝安全漏洞跳板**：C 语言之所以漏洞频发，正是因为越界后“没有立即崩溃”，而是继续拿着被踩烂的内存继续执行，最终被黑客利用篡改指令指针（RIP）。

**CP 的安全哲学：在任何破坏性物理写入发生之前，以确定性断言（Fail-Fast）立即熔断！终止受污染的执行流，把破坏范围精确锁死在 0 扩散！**

###### 5.3 不捕获 Panic，程序如何做到 99.999% 高可用？—— Actor 物理隔离与监控树自愈

既然越界不能捕获，程序整体会全部崩溃死机吗？
**答案是：绝对不会！CP 通过 Nest-Actor 容器架构提供了高可用解法：**

```text
                  ┌───────────────────────────────┐
                  │    Supervisor (集群监控树)    │
                  └───────────────┬───────────────┘
                                  │ (监控与故障自愈)
            ┌─────────────────────┼─────────────────────┐
            ▼                     ▼                     ▼
   ┌─────────────────┐   ┌─────────────────┐   ┌─────────────────┐
   │ Agent #1 (Actor)│   │ Agent #2 (Actor)│   │ Agent #N (Actor)│
   │ 独立堆栈与信箱  │   │ [💥 越界 Panic] │   │ 独立堆栈与信箱  │
   │ [ 正常运行中 ]  │   │ 立即熔断退出    │   │ [ 正常运行中 ]  │
   └─────────────────┘   └────────┬────────┘   └─────────────────┘
                                  │
                                  ▼ (微秒级响应)
                         Supervisor 自动原地重启
                         生成状态干净的全新 Agent #2
```

1. **Actor 微进程沙箱物理隔离（Process Isolation）**：
   - 在 CP-Nest 中，海量 AI Agent 与高并发任务是以轻量级 Actor（微进程）为单位独立运行的；
   - 每个 Actor 拥有完全隔离的独立堆栈、私有内存与 Reductions 抢占调度凭据；
   - 当某个 Agent 因代码 Bug 触发越界 Panic 时，熔断的作用域**严格受限于该单个 Agent 实例**；
   - 包含文件行号与调用栈的 Panic 诊断信息被确定性输出，该 Agent 立即释放自身占用的外部资源；
   - **系统中的其余 999,999 个 Agent 完全不受任何干扰，全速并发运行！**
2. **监控树自愈机制（Let It Crash & Supervisor Recovery）**：
   - 继承自 Erlang/OTP 顶级电信级高可用哲学：上层的 Supervisor（监控树）实时监控所有子 Agent；
   - 当收到子 Agent 的崩溃终止信号后，监控树根据策略（如 `OneForOne`）在几微秒内自动拉起一个状态纯净的新 Agent 实例；
   - **既杜绝了脏数据在内存中的传染扩散，又保障了百万并发集群整体 7x24 小时永不停机！**

---

#### 6.7.2 变量常量性与指针只读性的正交解耦

在 CP 中，“变量本身的重新赋值约束”与“指针指向的内存数据只读性”是两个完全解耦的维度：

1. **`const p: *T! = ...;`（指针自身不可变，指向的内存可写）**：
   - `const` 约束的是变量 `p` 自身，不能让 `p` 重新指向别的地址（禁止 `p = other;`）；
   - 但指针类型为 `*T!`，开发者依然可以通过 `p.* = val` 修改目标内存中的值。
2. **`let p: *const T! = ...;`（指针自身可变，指向的内存只读）**：
   - 类型为 `*const T!`，编译器在静态语义期**硬性拦截任何修改操作**（写入 `p.* = val` 抛出编译错误：`Cannot mutate through read-only pointer`）；
   - 但因为声明为 `let`，变量 `p` 可以被重新赋值指向另一个目标对象（允许 `p = other;`）。
3. **`const p: *const T! = ...;`（双重不可变）**：
   - 指针不可换，指向的内存数据亦绝对只读。

---

#### 6.7.3 获取变量裸指针的标准方式

在 CP 中，获取变量的底层裸指针支持以下三种标准方式（均须在 `unsafe` 上下文中进行）：

##### 方式一：直接裸指针字面语法（`*v!` 与 `**v!`）

CP 提供了前置 `*` 与尾置 `!` 的对称取指针语法，前置星号数量表示指针层级：

```typescript
let num: i32 = 100;

unsafe {
    // 1. 获取一级裸指针 (*num!)
    let p: *i32! = *num!;
    p.* = 200; // num 变为 200

    // 2. 获取二级裸指针 (**p!)
    let p_copy: *i32! = p;
    let pp: **i32! = **p_copy!;
    pp.*.* = 300; // num 变为 300
}
```

##### 方式二：通过安全借用显式转换（`&v as *T!` 与 `*v as *T!`）

通过安全域的只读借用（`&v`）或可变借用（`*v`），在 `unsafe` 块内使用 `as` 显式转换为目标裸指针：

```typescript
let v: i32 = 42;

unsafe {
    // 1. 从只读借用 &v 转换为只读裸指针 *const i32!
    let p_ro: *const i32! = &v as *const i32!;

    // 2. 从只读借用 &v 转换为可变裸指针 *i32! (底层强制升级，需由开发者负责)
    let p_rw: *i32! = &v as *i32!;

    // 3. 从可变借用 *v 转换为可变裸指针 *i32!
    let p_mut: *i32! = *v as *i32!;
    p_mut.* = 88; // v 变为 88
}
```

---

#### 6.7.4 完整裸指针类型转换矩阵 (Type Casting Matrix in Unsafe)

所有涉及裸指针的类型转换均须在 `unsafe` 块内通过 `as` 完成：

```typescript
unsafe {
    let num: i32 = 1000;
    let p: *i32! = *num!;

    // 1. 裸指针按位重解释（指针类型互转）
    let p_u8: *u8! = p as *u8!;

    // 2. 裸指针转换为物理内存整型地址 (usize)
    let addr: usize = p as usize;

    // 3. 物理内存整型地址还原为强类型裸指针
    let p_restored: *i32! = addr as *i32!;
    print(p_restored.*); // 1000

    // 4. 裸指针转换为通用非类型化指针 (*any!，等价 C void*)
    let generic: *any! = p as *any!;
    let back: *i32! = generic as *i32!;

    // 5. 数组切转为未定长多项裸指针 ([*]T!)
    let buf: [4]u8 = [1, 2, 3, 4];
    let many: [*]u8! = buf as [*]u8!;
    let val = many[0]; // 1
}
```

`zig{}` 内嵌块：内嵌Zig代码完全脱离CP借用检查，等价于最大程度unsafe，由Zig自身规则管控。

### 6.8 零摩擦生命周期 (Zero-Friction Lifetimes)

#### 6.8.1 设计哲学

CP 彻底废除了 Rust 晦涩且具有传染性的 `<'a, 'b>` 生命周期泛型符号，采用 **“局部 NLL 全自动推导 + 函数返回三级梯队 + 结构体免传染”** 架构，实现零摩擦生命周期 (Zero-Friction Lifetimes)。

CP 保留生命周期检查保证借用不悬垂，但是**不暴露显式生命周期标注**（这是一个重要差异化，对比Rust）。

- 编译器自动推导借用生命周期；
- 禁止借用逃逸出所有者的作用域；
- 暂时不支持手写生命周期参数，降低复杂度。
- 借用不能比所有者活得更久（生命周期校验）；所有者先销毁，则所有借用全部失效，编译拦截。

❌ 悬垂借用（编译拦截）

```cp
fn bad() -> &i32 {
    let x = 10;
    return &x; // 报错：x函数结束销毁，返回的借用会悬垂
}
```

> 想要返回“可修改的引用”场景，优先返回 `move` 值，或者使用class堆对象。

#### 6.8.2 生命周期架构

##### （1）局部作用域：非词法生命周期 (NLL) 与防悬垂推导（100% 零标注）

- **作用域深度防悬垂 (Scope Depth Check)**：外层变量深度小（活得长），内层块 `{}` 变量深度大（死得早）。**严禁将内层短命变量的借用赋值给外层长命变量**，否则编译报错：`Borrowed value does not live long enough`。
- **非词法生命周期 (Non-Lexical Lifetimes - NLL)**：借用变量 `let r = &a;`（只读）或 `let m = *a;`（可变）的生命周期**并非持续到花括号结束，而是精准终止于该借用变量的最后一次使用（Last Use）处**。在最后一次使用之后，原变量 `a` 立即解除锁定，可安全进行 `move` 或可变操作。

##### （2）函数返回借用：三级推导梯队（99% 零标注，1% 参数名直观绑定）

首先，**严禁函数返回内部创建的局部变量的借用**（悬垂返回直接编译报错）。当函数返回一个借用（只读 `: &Tensor` 或可变 `: *Tensor`）时，按以下三级梯队解析来源：

1. **第 1 级（自动省略 Elision）**：若只有一个引用类参数，或为类方法（含 `this` / `_`），返回值的生命周期自动绑定到该唯一参数或 `this` / `_`。
2. **第 2 级（默认交集 Default Union —— 无需标注）**：若有多个对象参数且未写标注（如 `fn pick(a: Tensor, b: Tensor): &Tensor`），编译器**自动将返回值生命周期绑定为所有对象参数生命周期的交集 `min(a, b)`**。只要调用处使用返回值时 `a` 和 `b` 均存活，即完全合法！
3. **第 3 级（极简参数名绑定 `&param` / `*param` —— 告别 `<'a>`）**：仅当存在多个对象参数，但返回值只借用自特定参数（且希望其他参数提前释放）时，直接在 `&` 或 `*` 后指认参数名：
   ```cp
   // 明确声明：返回的只读借用 Tensor 仅借用自参数 x，与 y 的死活无关！
   fn selectFirst(x: Tensor, y: Tensor): &x Tensor {
       return &x;
   }
   ```
   若可能借用自多个指定参数，支持 TS 风格的联合指认：`: &(x | y) Tensor` 或 `: *(x | y) Tensor`。

##### （3）结构体与并发逃逸法则：Shared<T> 与 栈绑定视图 (Stack-Bound View / ref struct)

为了从根源上消灭 Rust 结构体 `struct S<'a>` 带来的全局 `<'a>` 病毒式传染，同时不损失哪怕 1% 的底层极致性能，CP 将类/结构体严格划分为两类：

1. **普通结构体 / 类（可自由逃逸、跨协程传递）**：
   - 字段只允许存储**拥有所有权的数据（Move 传入）**或**共享所有权句柄 `Shared<T>`**。
   - **`Shared<T>`（对标 Rust `Arc<T>` / Nest `Refc Binary`）**：基于原子引用计数与全局共享堆，支持零拷贝跨函数逃逸、跨 Actor 传递。为杜绝数据竞争与循环引用，**`Shared<T>` 默认只读不可变**。
   - **极简函数式构造语法**：废除繁琐怪异的 `Shared.new(...)`，直接采用 TS 风格一等公民构造写法：
     ```typescript
     let config = Shared(ModelConfig {
         hidden_size: 4096,
         num_heads: 32,
     });
     ```
   - **多线程可变共享与内部可变性（Interior Mutability）**：若需跨物理线程共享可变状态，CP 推荐采用与操作系统锁协作的内部可变性模式 `Shared(Mutex(data))` 或 `Shared(RwLock(data))`，在多线程同抢碰撞时通过硬件与操作系统级自适应避让保证原子排他安全（详见第 10.5 节）。
2. **栈绑定借用视图（借鉴 C# `ref struct`，零开销局部借用，干掉最后 1% 缺陷）**：
   - 支持显式声明 `ref class` / `ref struct`，或者当一个类/结构体的字段中包含裸借用（如只读借用 `data: &Tensor` 或可变借用 `data: *Tensor`）时，编译器自动将其推导为 **栈绑定视图（Stack-Bound View）**。
   - **零开销优势**：无需原子引用计数，直接持有裸指针/切片，支持原地可变借用视图（`*T`），性能与手写 C/Rust 100% 一致，且**依然无需编写 `<'a>` 泛型**！
   - **逃逸清零铁律**：编译器对栈绑定视图执行严格的逃逸拦截：
     - ❌ 严禁将其 `return` 出所借用变量的存活作用域（或返回出当前函数，除非绑定到输入参数生命周期）；
     - ❌ 严禁将其赋值给外层更长生命周期的变量；
     - ❌ 严禁将其装入堆容器或通过 `spawn` / 消息发送给其他 Actor 协程。

### 6.9 `trait`、单继承 `class` 与自身实例（`_` / `this`）的所有权、借用与生命周期

CP 语言将普通函数的“默认/`&` 只读借用、`*` 可变借用、`move` 所有权转移”法则通过首参 `_`（等同于 `this`）完美统一到了面向对象与 Trait 契约体系中：

#### 6.9.1 `_` 等同于 `this` 与成员互调省略

- **`_` 作为 `this` 的一等极简别名**：在 `class`、`struct`、`trait` 与 `impl` 成员函数中，`_` 与 `this` 完全等价（推荐写 `_`，代码更精简）。
- **成员互调与字段访问省略前缀**：在同一类或 Trait 内部，成员函数互相调用可直接省略 `_.` / `this.`（如直接写 `helper(x)`），不与局部变量冲突时也可直接访问字段。
- **模式匹配通配符消歧**：为避免与代表自身实例的 `_` 冲突，`switch` 模式匹配与解构中的通配符统一使用 `*`（如 `* => handleDefault()`）。

#### 6.9.2 首参 `_` 的三维借用与所有权标记

| 实例权限模式                  | 方法声明语法（推荐 `_`，兼容 `this`）                                                      | 编译期安全检查规则                                                                                                                                  |
| :---------------------------- | :----------------------------------------------------------------------------------------- | :-------------------------------------------------------------------------------------------------------------------------------------------------- |
| **1. 只读借用 (默认或 `&_`)** | `fn getVal(): i32`<br>_(缺省省略 `_`，等价于 `fn getVal(_): i32` 或 `fn getVal(&_): i32`)_ | • 内部禁止修改 `_.field`<br>• 内部禁止调用 `*_` 或 `move _` 方法<br>• 调用 `obj.getVal()` 产生只读借用 `&obj`                                       |
| **2. 可变借用 (`*_`)**        | **`fn setVal(*_, v: i32)`**<br>_(兼容 `*this` / `mut _`)_                                  | • 允许修改自身字段及调用其他 `*_` 方法<br>• 调用 `obj.setVal(1)` 时，若 `obj` 存在未结束的只读/可变借用，立即报错拦截                               |
| **3. 所有权消费 (`move _`)**  | `fn close(move _)`<br>`fn intoVal(move _): i32`                                            | • 调用 `obj.close()` 后，外层 `obj` **所有权立即失效**（后续访问报 `Use of moved value`）<br>• 自动取消外层 `defer obj.release()`，杜绝 Double Free |
| **4. 静态方法 (`static`)**    | `static fn create(): Tensor`                                                               | • 无实例接收者 `_`，通过 `ClassName.create()` 调用                                                                                                  |

#### 6.9.3 实例方法的生命周期联动推导

- **第 1 级自动绑定 `_`**：类/Trait 方法返回借用（如 `fn getSlice(): &Tensor` 或 `fn getMutSlice(*_): *Tensor`）时，无需任何标注，返回值的生命周期自动绑定到当前实例 `_` (`this`)。在外层借用变量存活期间（NLL），原实例对象自动锁定，禁止调用冲突的 `*_` 或 `move _` 方法。
- **第 3 级显式参数绑定 `&_` / `*_`**：当方法含多个引用参数且需明确绑定来源时，支持 `: &_ Tensor` 或 `: *_ Tensor`（以及 `: &otherParam Tensor`）。

#### 6.9.4 单继承 `class`（`extends`）与 Rust 级 `trait`

- **单继承 `class`**：支持 `class Child extends Parent`，子类自动继承父类的字段与方法，支持方法重写与 `super.method()` 调用；重写方法时，首参 `_` / `*_` / `move _` 的权限修饰必须与父类契约严格一致。
- **强悍 `trait` 系统**：
  - 同时支持 `struct` 与单继承 `class`；
  - 支持携带 `_` / `*_` / `move _` 权限契约的方法签名及**默认方法实现（Default Methods）**（默认方法内可直接省略 `_.` 互调同 Trait 的抽象方法）；
  - 支持 TS 风格内联 `class A implements MyTrait` 与 Rust 风格后向扩展 `impl MyTrait for A`；
  - 当函数参数类型为 `Trait`（如 `fn run(item: MyTrait)` vs `fn runMut(*item: MyTrait)`）时，编译器静态校验实参类型是否实现了该 Trait，并严格拦截在只读 Trait 引用上调用 `*_` / `move _` 方法的行为。

---

### 6.10 借用在 switch 模式匹配里的行为

switch 分支捕获的值：

- 如果是值捕获：拿到副本/所有权；
- 如果需要指针/引用借用，支持显式前缀 `*` 捕获：`variant => |*ptr| { ... }` 获得载荷的独占可变借用；
- 亦可在分支内部显式写 `&v` / `*v`；
- switch通配符为 `*`，不影响借用规则。

```cp
switch (some_user) {
    .active => |*user| {
        user.score += 10; // 原地可变借用与修改
    }
    u => {
        let r = *u; // 创建u的可变借用
    }
    err(e) => {}
    null => {}
    * => {}
}
```

### 6.11 禁止行为汇总

1. 不能隐式move复合类型，必须写 `move`；
2. 不能在存在活跃借用的时候，move所有者；
3. 不能让借用生命周期超过所有者；
4. 不能同时存在可变借用和只读借用；
5. 安全域不能声明、使用裸指针 `*T!`；
6. 不能对err、null变体创建T类型借用；
7. 借用检查器不跟踪unsafe块内裸指针访问，责任交给开发者。

---

### 6.12 其他高级生命周期与自动化析构

#### 6.12.1 结构体生命周期、非词法作用域（NLL）与自动化析构

- 编译期基于 AST 进行精准的 **最后使用位置（Last-Use Map）分析**，变量一旦离开生命周期窗口，其持有的借用即刻失效解禁。
- 变量离开自身词法作用域时，编译器**自动在其后织入 Zig 级别的 `defer` 与 `errdefer` 资源清理指令**，开发者无需手写脆弱的析构胶水。

---

#### 6.12.2 跨越 `await` 的资源生命周期安全

跨越协程挂起点（`await`）的底层句柄、套接字与显存张量，底层全数封装为 **NIF Resource Objects**。当 Agent 或协程由于崩溃意外中断退出时，Nest 宿主容器的析构追踪器会兜底激活 Zig 终结器，确保显存和物理资源绝对零泄露。

---

#### 6.12.3 闭包逃逸与 Arena 作用域内存生命周期

当闭包发生逃逸时，捕获的局部上下文由编译器自动克隆并提升至作用域堆或当前任务所属的专属 `Arena` 分配器中，杜绝栈帧销毁后的野指针访问。

---

## 七、面向对象与契约解耦体系

### 7.1 数据与行为彻底解耦哲学

CP 彻底废除传统的 `interface` 混合模型，实现彻底解耦：

- **`struct` / `class`**：仅用于定义纯内存布局容器（Memory Layout Container），无任何虚表指针额外开销；
- **`@packed` / `@c` / `@root` / `@opaque` / `@value` / `@align`**：底层系统级布局与类型注解：
  - `@packed struct Foo { ... }`：紧凑位内存布局，禁止填充对齐，不自动生成默认 `init` 构造函数；
  - `@c struct Bar { ... }`：严格遵循 C ABI 内存物理对齐布局（向下兼容 `@extern struct`），用于无缝对接 C 头文件结构体；
  - `@root struct Name { ... }`：映射为文件顶层根作用域 `@This()`，支持字段声明与方法，带 Prelude 运行时命名隔离；
  - `@opaque class Baz { ... }`：不透明类型声明（转译映射为 Zig 原生 `pub const Baz = opaque { ... };`），专用于 C/C++ 动态库返回的抽象指针或无确定静态尺寸句柄类型（如套接字底层 `us_socket_t`、`udp.Socket`）。编译器自动豁免堆分配器 `_create` 实例构造，支持挂载只读借用（`_`）与独占可变借用（`*_`）方法；
  - `@value struct Qux { ... }`：值语义紧凑结构体，作为连续内存直接嵌入栈上或父结构体内部，按值传递，豁免堆指针封装与间接寻址开销；
  - `@align(N)`：结构体与字段级物理内存字节对齐注解。
- **`trait`**：声明纯行为契约与默认实现；
- **`impl Trait for Target`**：独立实现契约，支持零侵入地为已有第三方结构体实现新行为。

```typescript
class Point {
    x: f64;
    y: f64;
}

trait Measurable {
    fn distance_to_origin(_): f64;
}

impl Measurable for Point {
    fn distance_to_origin(_): f64 {
        return Math.sqrt(_.x * _.x + _.y * _.y);
    }
}
```

---

### 7.2 类单继承与完整生命周期钩子

```typescript
class Device {
    id: u32;

    pub fn init(*_, id: u32) {
        _.id = id;
    }

    // 显式析构钩子：离开作用域或被 release 时自动执行
    pub fn release(*_) {
        print("Releasing hardware device context:", _.id);
    }
}

class GpuDevice extends Device {
    cuda_stream: usize;

    pub fn init(*_, id: u32, stream: usize) {
        super.init(id);
        _.cuda_stream = stream;
    }
}
```

---

### 7.3 Trait 契约体系

Trait 支持方法签名契约及默认实现：

```typescript
trait Logger {
    fn log_raw(_, msg: string);

    // 默认方法实现
    fn log_info(_, msg: string) {
        _.log_raw("[INFO] " + msg);
    }
}
```

---

### 7.4 泛型与 Trait 约束 (Generic Bounds)

CP 语言在前端 AST 与语义分析（Sema）层原生支持完整的泛型与 Trait 约束契约校验，实现编译期 0 开销的单态化原生计算（Monomorphization）与 100% 友好的 AI 自愈诊断。

#### 7.4.1 核心语法与约束形式

1. **单 Trait 约束函数与方法**：
   ```typescript
   fn show<T: Printable>(item: T): void {
       item.display();
   }
   ```
2. **多 Trait 复合约束 (`+`)**：
   ```typescript
   fn sync_data<T: Reader + Writer>(stream: *T): void {
       // 要求类型 T 同时满足 Reader 与 Writer 行为契约
   }
   ```
3. **泛型类容器**：
   ```typescript
   class Box<T: Printable> {
       item: T;
       fn init(item: T) {
           this.item = item;
       }
       fn show(): void {
           this.item.display();
       }
   }
   ```
4. **编译期标量与常量泛型**：
   ```typescript
   class StaticBuffer<const N: usize> {
       data: [u8; N];
   }
   ```

#### 7.4.2 内置 Trait 体系 (Built-in Traits)

CP 编译器内置以下基础 Trait 契约，由标量基础类型天然满足或通过方法约定满足：
- **`Comparable`**：基础标量数值类型（`u8`~`u128`, `i8`~`i128`, `usize`, `isize`, `f32`, `f64`, `bool`, `string`）天生满足比较契约；类定义中实现 `compare` 或 `cmp` 方法（或显式声明 `implements Comparable`）亦满足；
- **`ToString`**：基础标量类型天生满足；类定义中提供 `toString` / `to_string` 方法即满足；
- **`Clone`**：基础标量类型天生满足；类提供 `clone` 方法即满足；
- **`Hash`**：基础标量类型天生满足；类提供 `hash` 方法即满足；
- **`Debug`**：基础标量类型天生满足；类提供 `debug` 或 `format` 方法即满足。

示例：泛型比较器函数天然支持强类型数值与浮点：
```typescript
fn max_val<T: Comparable>(a: T, b: T): T {
    if (a > b) {
        return a;
    }
    return b;
}

let m1 = max_val(10, 20);       // T 推导为 i32，返回 20
let m2 = max_val(3.14, 2.71);   // T 推导为 f32，返回 3.14
```

#### 7.4.3 前端拦截与 AI-First 自愈诊断 (`E0301`)

彻底消除底层 Zig 编译器在单态展开时泄露的数十行难懂 VTable / missing field 报错。所有未满足 Trait 契约的调用均在 `sema` 前端精准拦截：

1. **类声明未实现 Trait 必需方法拦截**：
   若类声明 `class Dog implements Printable`，但遗漏了 Trait 的无默认实现方法 `display`，前端直接发出自愈建议：
   ```text
   error[E0301]: class 'Dog' does not satisfy trait 'Printable'
     --> src/animal.cp:5:7
      |
      5 | class Dog implements Printable {
      |       ^^^
      |
      = cause: class 'Dog' declares 'implements Printable' but is missing required method 'display'.
      = help: all methods declared in trait 'Printable' without default implementations must be implemented.
      = suggestion: implement method 'display' in class 'Dog':
     fn display(...): void { ... }
   ```

2. **泛型函数实参不满足约束拦截**：
   若向 `show<T: Printable>(item: T)` 传入未实现 `Printable` 的 `Rock` 实例：
   ```text
   error[E0301]: type 'Rock' does not satisfy trait bound 'Printable'
     --> src/main.cp:15:10
      |
     15 |     show(r);
      |          ^
      |
      = cause: type 'Rock' is missing required method 'display' from trait 'Printable'.
      = help: trait 'Printable' requires method 'display'. Ensure 'Rock' implements it.
      = suggestion: implement method 'display' on 'Rock':
     fn display(...): void { ... }
   ```

#### 7.4.4 底层 Native 单态化转译机制

在转译为 Zig 时，泛型参数映射为 Zig 原生高性能 `anytype` 编译期内联参数，泛型返回类型自动使用 Zig 原生 `@TypeOf(param)` 精确推导返回类型，实现 100% 原生零成本抽象，不产生任何运行时虚表寻址开销：
```zig
pub fn max_val(a: anytype, b: anytype) @TypeOf(a) {
    if ((a > b)) {
        return a;
    }
    return b;
}
```

---

### 7.5 动态分发胖指针与运行时向下转型

当需要异构多态容器时，使用 `Box<dyn Trait>` 胖指针（包含数据真实指针与 vtable 虚表指针）：

```typescript
let handlers: [Box<dyn EventHandler>] = [];
handlers.push(Box.new(NetworkHandler.init()));

// 运行时安全向下转型 (Downcast)
for (handler of handlers) {
    if (net := NetworkHandler>) {
        net.flush_socket();
    }
}
```

---

### 7.6 Trait 静态契约与关联构造函数（Trait Static Contracts）

在面向契约和泛型编程中，大量经典设计模式需要**无实例接收者的静态工厂函数**（例如类似 Rust 的 `Default::default()`，或类似泛型反序列化 `FromStr::from_str`）。

CP 的 `trait` 不仅支持实例方法（首参含 `_` / `*_` / `move _`），也一流支持**静态方法契约（Static Associated Functions）**：

```typescript
// 声明含有静态构造契约的 Trait
trait Factory<T> {
    // 静态契约：无 _ 或 this 接收者
    static fn create(name: string): T;

    // 亦支持省略 static 前缀的无接收者抽象方法声明（由编译器根据无 _ 接收者推导）
    fn default_instance(): T;
}

// 为具体类型实现 Trait 静态契约
class User {
    pub name: string;
    pub fn init(*_, name: string) {
        _.name = name;
    }
}

impl Factory<User> for User {
    static fn create(name: string): User {
        return new User(name);
    }

    fn default_instance(): User {
        return new User("Guest");
    }
}

// 泛型调用静态工厂
fn create_entity<T: Factory<T>>(name: string): T {
    return T.create(name);
}
```

---

### 7.7 现代系统语言的方法容器与数据解耦演进（CP vs Rust vs Zig vs TS）

#### 7.7.1 为什么 Zig 与 Rust 彻底废除了 class 和继承？

在系统级编程和高并发架构演进中，面向对象的“单根继承（Inheritance）”与“数据行为强耦合（Class）”被现代工业界证实是产生内存膨胀、虚表间接寻址开销以及复杂死锁的重灾区：

- **Rust 的选择**：彻底废除 `class` 与继承。只保留纯数据结构体 `struct`（定义纯内存布局，零虚表开销），通过 `impl Struct` 挂载方法，通过 `trait` 声明行为契约，通过 `impl Trait for Struct` 声明后向实现。这实现了“数据容器与行为契约的彻底解耦”。
- **Zig 的选择**：Zig 更加纯粹，不仅没有 `class`，甚至没有 `trait` 和接口语法。Zig 中的 `struct` 本质上就是一个**强类型的命名空间（Namespace）**。任何以 `self: *Self` 或 `self: Self` 作为首参的函数都可以通过 `instance.method()` 语法糖调用；若首参没有 `self`，则自动成为静态函数，通过 `StructName.method()` 调用。
- **CP 的统一融合之道**：
  1. **纯数据容器（`struct`）**：作为纯值类型容器，零堆分配、零 GC，内存布局直通 C ABI / Zig 原生。`struct` **默认成员全部为 `pub`**（公开透明数据契约），符合数据传输对象（DTO）和系统级编程直觉；
  2. **面向对象与封装边界（`class`）**：保留单继承 `class` 专用于高层业务建模与状态机封装。`class` **默认成员为 `priv`**（私有封装，必须显式标注 `pub` 导出），提供严谨的封装安全；
  3. **行为契约（`trait`）**：全面采用强大灵活的 `trait` 代替 TS 的 `interface`，支持默认方法、泛型约束与动态分发胖指针 `Box<dyn Trait>`；
  4. **纯静态容器（`static class` / `static struct`）**：提供一等公民的静态容器，全员自动静态，兼顾 TS 的优雅书写与 Zig 原生命名空间的零开销。

---

## 八、原生三态错误处理体系

### 8.1 终结传统 Result/Option 胶水装箱

CP 原生引入**三态原生值类型（3-State Value Type）**，彻底终结了 Rust/TS 中沉重的 `Result<T, E>` / `Option<T>` / `Some` / `None` / `Ok` 胶水包装：

- **正常值**：直接返回值 `val`（零额外栈开销）；
- **错误值**：`! E`（由联合错误集限定）；
- **空值**：`T?`（允许赋值为 `null`）。
- **空值或错误**：`T?!`（允许赋值为 `null`、错误）。

---

### 8.2 错误类型体系与正交规范 (Unified Error Architecture)

CP 语言构建了一套将 **Zig 原生全局错误集（AnyErr / anyerror）的零开销暴力性能**、**Rust 灵活具名与枚举错误集**、以及 **TypeScript/现代业务动态错误（Err）** 融为一体的统一错误架构。

#### 8.2.1 错误返回类型分类与 `T!` 省略规则

在 CP 中，所有可产生错误的函数，其返回值类型在底层均对应明确的错误集或错误结构体：

1. **全局错误推导与 `T!`（等价于 `T ! AnyErr`）**：
   - **`T!` 是 `T ! AnyErr` 的官方语法糖省略**（成功返回 `T`，失败返回全局错误集合 `AnyErr`，在底层 Zig 中映射为 `anyerror!T`）；
   - **闭环与快译**：在 `T!` 或 `T ! AnyErr` 函数体内，**只能返回 `err.TagName` 即席错误字面量**，或通过 `?` 冒泡其他返回 `T ! AnyErr` 的函数；
   - **编译期严禁动态错误装箱**：在 `T!` / `T ! AnyErr` 函数内若试图 `return err(...)`，**编译器在编译期直接报错拦截**，严禁深层隐式推导，保障编译器百毫秒级极速构建。

2. **业务级动态错误（`T ! Err` / `T ! err`）**：
   - 内置轻量动态错误结构体 `Err`（原 `Error` 保留为完全兼容别名）；
   - 包含属性：`code: i32`、`msg: string`、`cause: ?*anyopaque`，并具备 `downcast` 胖指针还原能力；
   - 函数体内若通过 `err("msg")` 或 `err(code, "msg")` 抛出错误，函数签名**必须显式声明返回类型为 `T ! Err`（或 `T ! err`）**，不可省略为 `T!`。

3. **具名错误集（Dedicated Error Set：`err SetName { ... }`）**：
   - 使用 `err SetName { ... }` 声明原生错误集合；
   - 函数签名标注为 `T ! SetName`；
   - **函数内直接返回变体，无需 `err()` 包裹**：`return SetName.NotFound;` 或简写 `return .NotFound;`。

4. **多错误联合与即席合并（`T ! E1 || E2`）**：
   - 生产级代码 / 跨模块公共 API / 大型项目构建（严谨工程流）：
     - `T !` 等同 `T ! AnyErr`（全局即席错误集，映射为 Zig 原生 `anyerror!T`）
     - `T ! FileErr`（单一具名错误集，映射为 Zig 原生 `FileErr!T`）
     - `T ! FileErr || NetErr`（多错误集合并，映射为 Zig 原生 `(FileErr || NetErr)!T`，零额外开销）
     - `T ! FileErr || err{Timeout, Canceled}`（具名 + 匿名即席联合，映射为 Zig 原生 `(FileErr || error{Timeout, Canceled})!T`）
     - `T ! FileErr || Err`（具名错误集 + 动态业务错误，底层映射为 `CpResult(T, Err)`，具名错误自动升格）
   - 享受秒级极速并行编译，以及最严格的架构契约保护。

5. **普通枚举作为错误类型（`enum EnumName { ... }`）**：
   - 普通枚举可直接作为强类型业务错误码集合（如 `enum HttpError { BadRequest = 400, ServerError = 500 }`）；
   - 函数签名标注为 `T ! HttpError`；
   - **函数内必须使用 `err()` 包裹以标明错误态（类似 Rust 的 `Err()`）**：`return err(HttpError.BadRequest);` 或点号推导 `return err(.BadRequest);`。

```typescript
// 1. 具名错误集：直接返回变体，无需 err() 包裹
err FileErr {
    NotFound,
    AccessDenied
}

err NetErr {
    Timeout,
    ConnectionReset
}

// 模块级导出错误集（直接对应 Zig 的 pub const Error = error{ ... };）
pub err Error {
    OutOfMemory,
    SyntaxError,
    UnexpectedToken,
}

// 2. 具名 + 匿名错误联合声明
fn test_anon_union(flag: i32): string ! FileErr || err{Timeout, Canceled} {
    if (flag == 1) {
        return FileErr.NotFound; // 具名变体
    }
    if (flag == 2) {
        return err.Timeout;      // 即席匿名标签
    }
    return "ok_anon";
}

// 3. 具名 + 动态 Err 联合声明
fn test_err_union(flag: i32): string ! FileErr || Err {
    if (flag == 1) {
        return FileErr.NotFound; // 自动转换为 Err
    }
    if (flag == 2) {
        return err(500, "internal server failure"); // 动态业务错误
    }
    return "ok_err";
}

fn open_file(path: string): string ! FileErr {
    if (path == "missing.txt") {
        return FileErr.NotFound; // 或直接 return .NotFound;
    }
    return "file content";
}

// 2. 普通枚举错误：必须使用 err() 包裹转换为错误态
enum HttpError {
    BadRequest = 400,
    ServerError = 500
}

fn handle_req(id: i32): string ! HttpError {
    if (id < 0) {
        return err(HttpError.BadRequest); // 或 return err(.BadRequest);
    }
    return "ok";
}

// 3. 业务动态错误 Err：签名必须显式标注 T ! Err，不能写成 T!
fn query_db(sql: string): string ! Err {
    if (sql == "") {
        return err(404, "record not found"); // 动态 Err 实例
    }
    return "result";
}

// 4. 全局错误 AnyErr / T!：函数内只能返回 err.TagName 即席错误
fn calc(val: i32): i32! { // 展开等价于 i32 ! AnyErr
    if (val < 0) {
        return err.NegativeNotAllowed; // 静态即席标签
    }
    // return err("bad"); // ❌ 编译拦截：Cannot return dynamic 'err(...)' in 'T ! AnyErr'
    return val * 2;
}
```

---

#### 8.2.2 错误类型自动转换（Auto Conversion）与声明式 `from` 规则

在系统分层调用中，CP 提供了安全的**编译期自动错误类型提升与转换机制**：

1. **安全拓扑自动转换**：
   - 具名错误集 `FileErr`、枚举错误 `HttpError`、全局错误 `AnyErr` 在通过 `?` 冒泡到返回 `T ! Err` 的函数时，编译器自动织入 `cp_convert_err`，将其自动转换为统一的动态错误实例 `Err`，并将原始错误无损记录至 `cause`；
2. **声明式 `from` / `__from` 自动桥接**：
   - 若目标错误类型定义了 `pub fn from(e: SourceErr): Self`，在通过 `?` 传播时，编译器优先调用用户声明的 `from` 进行确定性语义转换。

```typescript
class ServiceErr {
    pub message: string;
    pub fn from(fe: FileErr): ServiceErr {
        return ServiceErr { message: "Transformed from FileErr" };
    }
}

fn run_pipeline(): string ! ServiceErr {
    let content = open_file("missing.txt")?; // 自动调用 ServiceErr.from(FileErr.NotFound)
    return content;
}
```

---

#### 8.2.3 全景 `switch` 模式匹配错误解构

CP 的 `switch` 具备一流的**错误联合值解构与分支匹配**能力，无需手工解包即可在单个 `switch` 表达式中同时处理成功值与各类错误形式：

| 匹配模式                | 示例语法                                                       | 语义说明                                     |
| ----------------------- | -------------------------------------------------------------- | -------------------------------------------- |
| **具名错误变体**        | `FileErr.NotFound => ...` / `.NotFound => ...`                 | 匹配具名错误集中的指定标签                   |
| **枚举错误包裹**        | `err(HttpError.BadRequest) => ...` / `err(.BadRequest) => ...` | 匹配经 `err()` 包裹的枚举错误变体            |
| **动态 Err 代码与信息** | `err(404, "record not found") => ...` / `err(404, msg) => ...` | 匹配动态 Err 的特定 code 与 msg              |
| **动态 Err 变量解构**   | `err(code, msg) => ...`                                        | 提取动态 Err 的错误码与错误描述字符串        |
| **动态 Err 单参数匹配** | `err("timeout") => ...` / `err(500) => ...`                    | 按消息字符串或错误码单维度匹配               |
| **即席错误标签**        | `err.InvalidPadding => ...`                                    | 匹配 AnyErr / anyerror 中的具体静态错误标签  |
| **通用捕获通配符**      | `err(e) => ...` / `err => ...`                                 | 兜底捕获未匹配的任意错误对象或标签           |
| **正常成功值**          | `val => ...`                                                   | 当表达式未发生错误时，提取解包后的成功载荷值 |

```typescript
fn inspect_outcome(res: string ! FileErr) {
    switch (res) {
        .NotFound => print("文件未找到"),
        .AccessDenied => print("无权访问"),
        content => print("成功读取内容:", content)
    }
}

fn inspect_dynamic(res: string ! Err) {
    switch (res) {
        err(404, msg) => print("404 资源未找到:", msg),
        err(code, msg) => print("通用业务错误:", code, msg),
        err(e) => print("其他错误:", e),
        data => print("业务执行成功:", data)
    }
}
```

---

#### 8.2.4 胖指针动态错误与 `downcast` 运行时类型还原（双轨解构语法）

针对极其复杂的微服务错误栈或插件体系，CP 提供了与 Rust 类似但更轻量安全的 **`Box<dyn Trait>` 胖指针与 `downcast` 机制**：

- 动态错误 `Err` 可通过 `Err.with_cause(move box_err)` 挂载堆上动态分配的胖指针；
- 在模式匹配与函数调用中，**双轨支持 `:` 与 `:=`（Walrus 海象解构符）**，均能精确向下转型并绑定解构后的具体错误实例：
  - `err(e: ConcreteError) => ...`（类型窄化匹配与绑定）
  - `err(e := ConcreteError) => ...`（显式解构赋值与绑定）
- 两种写法在语义与底层发射上完全等价，在成功 downcast 后，变量 `e` 将安全获得 `*const ConcreteError` 具体类型指针，支持直接访问其属性与方法：

```typescript
switch (res) {
    // 语法 1：类型窄化风格
    err(e: TimeoutError) => {
        print("精确匹配到超时错误实例，重试时长:", e.retry_after);
    },
    // 语法 2：Walrus 海象解构绑定风格（完全等价）
    err(e := SqlError) => {
        print("精确匹配到 SQL 错误实例，SQLState:", e.sql_state);
    },
    err(500, msg) => print("通用 500 业务错误:", msg),
    err(e) => print("其他泛型错误:", e),
    ok_val => print("成功结果:", ok_val)
}
```

---

#### 8.2.5 统一错误类型体系与自动化提升机制（完全实装架构）

在 CP 语言中，错误类型不仅兼具 Zig 的轻量零开销错误集，更融合了 Rust 般强大的类型转换与现代业务动态错误体系：

1. **已实装的核心错误形态与极简字面量**：
   - **标量即席错误免手动构建**：
     - 在函数内可直接返回字符串字面量或状态码：`return err("not found");` 或 `return err(404);`；
     - **底层自动化包装**：编译器在代码生成时自动将其降解并填充为标准 `Err.init(0, "not found")` 或 `Err.init(404, "")`，开发者完全无需手动调用构造器或 new 实例，心智负担降至最低；
   - **Zig 原生轻量错误集**：`err FileErr { ... }`、全局 `AnyErr`（`anyerror`）、匿名即席集合 `err{A, B}`；
   - **多错误联合类型**：`FileErr || NetErr`（零成本原生联合）、`FileErr || err{...}`、`FileErr || Err`；
   - **动态业务错误**：内置轻量结构体 `Err`（`code`, `msg`, `cause`），原 `Error` 保持完全兼容；
   - **普通枚举转错误**：`enum HttpError`，通过 `err(HttpError.BadRequest)` 包裹升格为强类型错误；
   - **多态胖指针错误**：`Box<dyn Trait>`，通过 `Err.with_cause(move b)` 挂载，并在 `switch` 中支持 `err(e: T)` 与 `err(e := T)` 双轨安全 downcast 还原具体出错实例。

2. **已实装的 `From` 声明式与自动类型提升机制**：
   - **枚举声明式 `from` 桥接**：在定义错误枚举时，可声明 `enum AppErr { Io(FileErr) from FileErr, Net(NetErr) from NetErr }`。当子函数返回 `! FileErr` 时，使用 `?` 冒泡会自动根据 `from FileErr` 规则转换为 `AppErr.Io(fe)`；
   - **结构体 `from` 自定义转换**：目标错误类型只需提供静态方法 `pub fn from(e: SourceErr): Self`，在 `?` 错误传播时，底层 `cp_convert_err` 会自动侦测并调用该转换逻辑；
   - **向统一 `Err` 的自动无损提升**：任意具名错误变体或普通枚举变体，通过 `?` 冒泡到返回 `T ! Err` 的函数时，`cp_convert_err` 会自动将其提升为带有确切状态码与标签名的 `Err` 动态对象。

---

#### 8.2.6 自定义结构体错误（Struct Errors）与类错误（Class Errors）深度对比与实战

在 CP 语言中，`struct` 与 `class` 均可作为强类型错误直接返回（通过 `return err(...)`）：

| 特性维度                | 结构体错误（`struct`）                        | 类错误（`class`）                                                   |
| ----------------------- | --------------------------------------------- | ------------------------------------------------------------------- |
| **内存语义**            | **值类型（Value Type）**                      | **引用类型（Reference Type）**                                      |
| **分配位置**            | 纯栈上分配 / 内联值复制，**零堆分配、零 GC**  | 通过 `new` 在堆上分配，传递对象指针（`*ClassName`）                 |
| **适用场景**            | 高频、高性能、高吞吐的轻量纯数据错误          | 富上下文业务错误诊断、错误链回溯、支持继承与多态                    |
| **函数返回签名**        | `fn foo(): T ! MyStructError`                 | `fn foo(): T ! MyClassError`                                        |
| **自动转统一 `Err`**    | 自动读取 `code`、`msg`/`message` 拷贝至 `Err` | 自动提取 `code`、`msg`，**并将指针自动挂载到 `cause`**              |
| **downcast 运行时还原** | 值类型直接在 `switch` 中进行模式解构匹配      | 可通过 `err(e: MyClassError)` 或 `err(er := MyClassError)` 完美还原 |

```typescript
// 1. 结构体错误：纯值类型，零堆内存分配
struct MyStructError {
    code: i32;
    msg: string;
}

fn fail_struct(): i32 ! MyStructError {
    return err(MyStructError { code: 404, msg: "struct not found" });
}

// 2. 类错误：引用类型，堆分配，支持全链路溯源挂载
class MyClassError {
    pub code: i32;
    pub msg: string;
    pub fn init(c: i32, m: string) {
        this.code = c;
        this.msg = m;
    }
}

fn fail_class(): i32 ! MyClassError {
    let ce = new MyClassError(500, "class server crash");
    return err(ce);
}

// 自动向统一 Err 提升并保留 cause 胖指针
fn fail_class_to_err(): i32 ! Err {
    return err(new MyClassError(502, "bad gateway"));
}

// 消费端匹配
fn handle_errors() {
    // 结构体直接解构模式匹配
    let r1 = fail_struct();
    switch (r1) {
        err(MyStructError { code, msg }) => print(`Struct matched: ${code} ${msg}`),
        val => print(val)
    }

    // 类错误直接匹配与 downcast 还原
    let r2 = fail_class_to_err();
    switch (r2) {
        // 支持类型窄化匹配风格 (: 语法)
        err(e: MyClassError) => print(`Downcasted (:): ${e.code} ${e.msg}`),
        // 同时支持 Walrus 海象解构风格 (:= 语法)
        err(er := MyClassError) => print(`Downcasted (:=): ${er.code} ${er.msg}`),
        err(code, msg) => print(`Err fallback: ${code} ${msg}`),
        val => print(val)
    }
}
```

---

#### 8.2.7 复杂枚举载荷错误（Enum Payload Errors）：定义、返回与多维解构匹配

对于多变体且携带异构载荷的错误场景，CP 提供完全自洽的定义、返回与解构模式匹配体系：

```typescript
// 1. 枚举错误定义（含单值标量、元组、结构体等多样载荷）
enum TaskErr {
    Idle,                                      // 无载荷变体
    Text(string),                              // 标量载荷变体
    Move([i32, i32]),                          // 元组载荷变体
    Compute { batch: i32, priority: i32 }      // 结构体属性载荷变体
}

// 2. 函数签名与错误返回（支持显式前缀与点推断语法）
fn fail_scalar(): i32 ! TaskErr {
    return err(TaskErr.Text("scalar_failed"));
}

fn fail_tuple(): i32 ! TaskErr {
    return err(TaskErr.Move([10, 20]));        // 或 TaskErr.Move(10, 20)
}

fn fail_struct_payload(): i32 ! TaskErr {
    return err(TaskErr.Compute { batch: 64, priority: 1 });
}

fn fail_dot_inference(): i32 ! TaskErr {
    // 点推断（Dot Inference）：自动根据函数返回签名推导 TaskErr 枚举类型
    return err(.Compute { batch: 128, priority: 2 });
}

// 3. 全维度 switch 模式匹配解构
fn process_err(res: i32 ! TaskErr) {
    switch (res) {
        // 单载荷变量捕获
        err(.Text text) => print(`Text payload: ${text}`),
        // 原生元组方括号解构
        err(.Move [x, y]) => print(`Move tuple: ${x}, ${y}`),
        // 结构体精确解构
        err(.Compute { batch, priority }) => print(`Compute exact: ${batch}, ${priority}`),
        // 结构体剩余/整包解构捕获 (...fields)
        err(.Compute { ...fields }) => print(`Compute rest: ${fields.batch}`),
        // 兜底错误或成功
        err(e) => print(`Other err: ${e}`),
        val => print(`Success: ${val}`)
    }
}
```

---

### 8.3 错误传播链与解包原语

```typescript
// 1. 错误传播操作符 (?)
// 遇到 Err 立即向上冒泡返回，遇到正常值解包出原始值
let response = ping("api.cp.org")?;

// 2. 默认降级容错 (? default)
// 发生任何 Err 时，优雅降级为缺省值
let safe_res = ping("bad.host") ? "FALLBACK_HOST";

// 3. 空值合并操作符 (?? default)
// 遇到 null 时回退
let opt_val: string? = null;
let text = opt_val ?? "default_text";

// 4. 可选类型强制显式解包操作符 (.?)
// 断言目标必不为空并将 T? 解包为 T；若遇到 null 则立即触发 Fail-Fast 断言中断 (Panic)
let hard_val: string = opt_val.?;
```

#### 8.3.1 解包原语 (`.?`)、可选链 (`?.`) 与空值合并 (`??`) 对比规范

| 操作符         | 语法形态       | 类型变换        | 遇 null 行为         | 核心哲学与适用场景                                               |
| :------------- | :------------- | :-------------- | :------------------- | :--------------------------------------------------------------- |
| **`u.?`**      | 后置点号问号   | `T?` → **`T`**  | **立即中断 (Panic)** | **断言非空**：用于测试断言、启动硬依赖、程序员铁律保证非空的场景 |
| **`u?.`**      | 属性前置问号点 | `T?` → **`U?`** | **短路返回 `null`**  | **安全导航**：多层级可能为空的安全属性读取与调用                 |
| **`u ?? def`** | 双问号中缀     | `T?` → **`T`**  | **降级返回 `def`**   | **优雅降级**：业务层存在合理默认值的场景                         |

---

### 8.4 错误捕获表达式与结构化处理 (Catch Expression & Fallback)

CP 语言彻底废除传统的运行时异常体系与低效的 `try/catch` 动态调用栈展开（若书写传统 `try/catch` 编译器将触发 `E0103` 智能自愈提示）。错误处理统一采用零开销的表达式级 `catch`、3-State 传播符 `?` 以及 `switch` 模式匹配：

```typescript
// 1. 行内表达式 catch：遇错就地捕获并降级为缺省值
let cfg = load_config("config.json") catch e => #{ port: 8080 };

// 2. 链式业务与错误传播 (?)：各步骤通过 ? 自动冒泡错误，零样板代码
fn process_transaction(): void! {
    let raw = read_file("agent.json")?;
    let obj = parse_json(raw)?;
    save_to_db(obj)?;
}

// 3. 业务层统一捕获：在调用端使用 catch 执行优雅降级、日志记录或重试
process_transaction() catch err => {
    print("Failed transaction:", err);
};

// 4. 前缀 try 表达式兼容语法（Zig/Rust 迁移兼容与极速适配）
// 支持直接书写前缀 try expr，前端自动映射为标准错误传播（Expr::Question）：
fn send_data(*socket: Socket, data: []const u8): void! {
    try socket.write(data);
    const amount = try socket.flush();
}
```

---

### 8.5 业务层统一 Error 类

内置标准业务级 `Error` 结构，附带代码与堆栈上下文追踪：

```typescript
let e = Error.init(500, "Internal Server Failure");
print(e.code, e.message);
```

---

### 8.6 胖指针动态错误与错误集自动转换

当不同模块的子错误集向上传播时，编译器自动织入 `cp_convert_err` 进行错误集合并与自动升级，亦支持通用动态错误胖指针 `Box<dyn Error>`。

---

### 8.7 守卫控制流三剑客（The Guard Trinity）：`||`、`??` 与 `?` 的发散控制与解包

在工程级系统开发与海量智能体业务中，卫语句（Guard Clauses）与错误/空值早退是出现频率最高的模式。CP 原生设计了覆盖三大维度的**守卫控制流三剑客**，全面支持发散控制流（`continue`、`break`、`return [expr]`）：

| 守卫操作符 | 适用类型       | 触发发散条件   | 正常执行行为       | 底层 Zig 转译映射   | 典型业务场景                 |
| :--------- | :------------- | :------------- | :----------------- | :------------------ | :--------------------------- |
| **`        |                | `**            | `bool`             | 条件为 `false`      | 表达式求值为 `true`          | `if (!cond) action;` | 状态校验、循环边界、权限判定 |
| **`??`**   | 可选类型 `T?`  | 值为 `null`    | **自动解包**为 `T` | `opt orelse action` | 空值过滤、早退拦截、默认降级 |
| **`?`**    | 错误联合 `T!E` | 结果为 `error` | **自动解包**为 `T` | `res catch action`  | 故障自愈、失败重试、错误发散 |

#### 1. 语法形态与代码示例

```typescript
// 1. || 布尔条件守卫：条件不满足时早退/跳出
while (i < max_len) {
    let cur = items[i];
    i += 1;
    is_valid(cur) || continue;     // 若无效直接跳过本次迭代
    (cur < 100) || break;          // 若超出范围直接跳出循环
    check_auth(cur) || return 403; // 若无权限直接早期返回
}

// 2. ?? 可选值守卫：遇 null 发散，非空直接解包为 T
while (has_more) {
    let entry = fetch_next_opt() ?? continue; // 为 null 时跳过，非空则 entry: Entry
    let config = load_config_opt() ?? break;  // 缺失时跳出循环
    let user = find_user(id) ?? return null;  // 缺失时直接返回 null
}

// 3. ? 错误联合守卫：遇 error 发散，成功直接解包为 T
while (task_running) {
    let packet = read_socket() ? continue;       // 出错时丢弃并继续
    let token = parse_header(buf) ? break;       // 解析失败跳出循环
    let result = execute_step() ? return Err(500); // 故障时携带错误早期返回
}
```

#### 2. 设计与性能优势

- **零成本抽象（Zero Cost）**：`||` 直接编译为底层单条分支跳转指令 `if (!cond)`；`??` 与 `?` 直接编译为 Zig 原生内建 `orelse` 与 `catch`，不产生任何闭包或中间包装开销。
- **对称优雅**：消除了传统语言中满屏的冗长 `if (...) { return; }`，让核心业务主干代码一目了然。

---

### 8.8 控制流条件与循环迭代圆括号强制铁律（The Parentheses Iron Rule）与 AI-First 智能自愈提示

为了彻底根除传统语法中因无括号条件/迭代与结构体实例化（Struct Literal）产生的严重 AST 二义性，并保持与 TypeScript/C-Family 高度一致的直觉美感，CP 制定了清晰严格的控制流圆括号铁律：

#### 1. 语法规范：`if`、`while` 条件与 `for` 迭代头必须强制外包 `()`

所有 `if`、`while` 条件表达式以及 `for` 循环迭代表达式，必须强制使用圆括号 `()` 包裹：

```typescript
// ✅ 正确规范写法
if (addr_str.len >= INET6_ADDRSTRLEN) { return null; }
while (true) { break; }
if (x > 0 && y < 10) { ... }
for (item of numbers) { print(item); }
for (*item of numbers) { item.* += 1; }
for (move item of numbers) { sink(item); }
for (i of 0..10) { print(i); }

// ❌ 严禁无括号写法（编译器拒绝通过，直接触发 E0101）
if addr_str.len >= INET6_ADDRSTRLEN { return null; } // 编译报错！
while true { break; }                                // 编译报错！
for item of numbers { print(item); }                 // 编译报错！
for i of 0..10 { print(i); }                         // 编译报错！
```

#### 2. 二义性根除原理

在 CP 语言中，大写标识符后紧跟 `{` 是结构体字面量的语法（如 `Point { x: 1, y: 2 }`），而解构与模式匹配中也普遍包含 `{ ... }`。若允许 `if CONST { ... }` 或 `for item of coll { ... }`，词法与语法解析器会将 `{ ... }` 误判为结构体字段列表或模式，从而报出莫名其妙的 `Expected token: Colon, found ...`。强制括号 `if (...) { ... }`、`while (...) { ... }`、`for (...) { ... }` 彻底消除二义性，极大加快编译解析速度。

#### 3. AI-First 智能自愈诊断信息

当开发者或 AI Agent 偶然遗漏括号时，CP 编译器绝不输出冷冰冰的语法错误，而是附带具备修正建议的三段式自愈诊断：

```text
error[E0101]: CP Syntax Error: 'for' loop statement requires parentheses.
  --> src/main.cp:15:5
   |
15 |     for i of 0..times {
   |         ^
   = cause: loop specification lacks required outer parentheses.
   = help: in CP, for loop expressions must be enclosed in parentheses: for (item of iterable) { ... }
   = suggestion: wrap the loop target in parentheses:
   |
15 |     for (i of 0..times) {
   |         +             +
```

智能体识别到 `= suggestion:` 后，单轮对话耗时 0 秒即可自动采纳并修复，大幅提高无人值守自主迭代效率。

---

### 8.9 控制流可选值/错误联合解包与就地指针修改（Value & Pointer Unwrapping）

在现代系统开发中，从可选容器（`T?`）或错误联合（`T ! E`）中提取有效值，或直接对其底层物理内存进行就地覆写（In-Place Mutation）是极为高频的需求。CP 彻底废除 Zig 风格的管道解包语法（`|val|` 与 `|*val|`），统一采用优雅现代的**海象解包语法**（Walrus Unwrapping）：

#### 1. 普通值只读解包：海象语法 (`val :=`)

当目标非空（非 `null` 或非 `Err`）时，自动解包出内部成功载荷值，并将捕获变量声明为当前代码块内的局部常量（`const`）：

```typescript
let opt_user: User? = find_user(1);

// 海象表达式解包绑定
if (user := opt_user) {
    print(user.name);
}
```

#### 2. 可变指针就地解包：`*val :=`

当需要对被包含的值进行**就地可变更新**（例如链表节点替换、状态机翻转、复杂结构体字段修改）时，在捕获变量前加上解包指针修饰符 `*`：

```typescript
let state: i32? = 100;

// 海象指针解包：获得底层可变指针借用并就地更新底层内存
if (*val := state) {
    val = 200; // 自动就地覆写底层 state 的物理内存，无需重新包装！
}

if (*val := state) {
    val = 300;
}

print(state.?); // 输出 300
```

> [!IMPORTANT]
> **语法纯洁性保证**：CP 严禁使用 Zig 风格的管道解包语法（如 `if (opt) |*val|` 或 `while (opt) |val|`）。若在代码中误用管道语法，编译器前端将在解析阶段直接触发 `E0101` 智能自愈诊断，并给出单步修正建议。

#### 3. 循环结构中的流式指针迭代

在 `while` 循环遍历带有内部缓冲区的对象或自定义流式迭代器时，结合指针解包可以直接对每一次返回的条目进行无拷贝修改：

```typescript
while (*item := buffer.next()) {
    item.processed = true;
}
```

#### 4. 忽略绑定 (`_ = expr`)

当调用产生返回值的表达式但明确无需保留其结果时，支持使用单下划线 `_` 显式丢弃：

```typescript
_ = flush_socket(); // 显式丢弃结果，抑制未捕获警告
```

---

### 8.10 具名循环标签与跨层跳出 (`label: for` / `label: while` 与 `break :label` / `continue :label`)

在深层嵌套遍历（如二维矩阵搜索、协议分帧解析、嵌套事件循环）中，直接 `break` 或 `continue` 只能作用于最内层循环。CP 语言原生支持具名循环标签与跨层跳转：

1. **语法定义**：
   - 可以在 `while` 或 `for` 语句前直接指定标签：`label: while (...) { ... }` 或 `label: for (...) { ... }`；
   - 跨层跳出外层循环：`break :label;`；
   - 跨层跳转到外层循环的下一次迭代：`continue :label;`；
   - 具名代码块带值提前跳出：`label: { ... break :label value; }`。
2. **底层映射**：
   - 编译器将其零开销映射为 Zig 原生带标号循环 `:label while (...)` 与 `:label for (...)` 以及 `break :label` / `continue :label`，完全没有额外布尔标志位与多次判断开销。

```typescript
outer: for (i of 0..10) {
    inner: for (j of 0..10) {
        if (matrix[i][j] == target) {
            found = true;
            break :outer; // 👈 瞬时跳出最外层循环
        }
        if (matrix[i][j] < 0) {
            continue :outer; // 👈 跨层进入外层循环的下一轮迭代
        }
    }
}
```

---

### 8.11 行末分号可选（Optional Semicolons）与 ASI 换行断句规范

CP 采用符合现代前端（TypeScript/ES6）与系统语言（Go/Swift）直觉的分号机制：

1. **100% 可选分号**：在语句末尾（如 `let` 变量声明、表达式语句、`return` 语句、`import` 语句等），分号 `;` 为完全可选项。写分号或不写分号均完全合规，不影响编译产物。
2. **物理词元换行判定（ASI）**：编译器通过底层物理词元边界（Token Range）的换行符 `\n` 进行精准自动断句。
3. **换行安全防御规则**：
   - 换行后的首字符若为左圆括号 `(`（如独立表达式 `(a < b) || break`），**严禁**被贪婪解析为上一行末尾表达式的后缀函数调用（`CallExpr`）；
   - 无参数的 `return` 语句若遇到换行，自动断句，不会将下一行的独立语句误吞为返回值。

---

### 8.12 循环步进表达式 (`while (cond) : (step)`) 与函数类型别名 (`const F = fn(...) Ret`)

为大幅降低从 Zig / Bun.js 底层系统库直译到 CP 的摩擦力，CP 编译器在语法与类型系统层面原生打通了以下能力：

1. **`while` 循环步进表达式 (`: (step_expr)`)**：
   - 支持在条件括号闭合后直接书写 `: (step_expr)`，定义在每次迭代回边（Continue）时必定执行的步进表达式：
     ```typescript
     let i: usize = 0;
     while (i < 10) : (i += 1) {
         if (i == 5) break;
     }
     ```
   - 零额外控制开销，直接对齐发射为 Zig 寄存器级 `while (...) : (...)` 原生循环。

2. **顶层函数签名类型别名 (`const F = fn (...) Ret;`)**：
   - 除标准的 `type F = fn (x: u8): bool;` 之外，完全允许使用 Zig 风格的 `const` 定义裸函数签名类型（支持省略返回类型前的冒号 `:`）：
     ```typescript
     const MyCallback = fn (x: u8) bool;
     let cb: MyCallback = my_handler;
     let ok = cb(42);
     ```
   - 编译器在前端自动将其单态化提升为强类型函数指针类型，打通完整的参数类型检查与调用推导。

---

## 九、现代语法糖与函数式特性

### 9.1 管道操作符与通用通配占位符

CP 的管道操作符 `|>` 与全能通配符 `*` 深度解耦，彻底避免与方法接收者 `_` 混淆：

```typescript
fn add(a: i32, b: i32): i32 { return a + b; }
fn mul(a: i32, b: i32): i32 { return a * b; }

let x = 10;

// 1. 默认作为唯一/首参传入
let r1 = x |> mul(2); // 20

// 2. 使用通配占位符 (*) 精确指定插入位置
let r2 = r1 |> add(5, *); // add(5, 20) -> 25

// 3. 点号无参方法链式调用简写
let trimmed = "  cp-lang  " |> .trim(); // "cp-lang"
```

---

### 9.2 多模式现代集合遍历

```typescript
let numbers = [1, 2, 3, 4];

// 1. 只读借用遍历 (只读零拷贝)
for (item of numbers) {
    print(item);
}

// 2. 可变借用遍历 (原地修改元素)
for (*item of numbers) {
    item.* = item.* * 10;
}

// 3. 所有权转移遍历 (消费源集合)
for (move item of numbers) {
    sink(item);
}
```

---

### 9.3 范围表达式与切片语法

```typescript
// 范围循环迭代
for (i of 0..10) {    // 半开区间：0 到 9
    print(i);
}

for (i of 0..=10) {   // 闭区间：0 到 10
    print(i);
}

// 数组高速切片视图
let head_part = numbers[0..2];
let tail_part = numbers[2..];
```

---

#### 9.3.1 循环控制与跳转语句 (`while`, `break`, `continue`)

```typescript
let idx: usize = 0;
const len = buf.len;

while (idx < len) {
  // continue: 跳过当前循环，进入下一轮迭代
  if (buf[idx] == 0x20) {
    idx = idx + 1;
    continue;
  }

  // break: 立即跳出当前循环体
  if (buf[idx] == 0x00) {
    break;
  }

  process_byte(buf[idx]);
  idx = idx + 1;
}
```

---

#### 9.3.2 Walrus 海象操作符条件绑定与解包循环 (`while (binding := expr)`)

针对流式解析、字符串切分器等迭代器与生成器模式，CP 全面采用海象操作符（`:=`）实现条件赋值与可选类型非空解包，彻底废除旧版 Zig 风格的 `while (expr) |capture|` 语法。

循环在每次迭代前对表达式 `expr` 求值：若返回非 `null` 值，则自动解包并将非空值绑定到循环局部变量 `binding`，进入循环体执行；当表达式求值为 `null`（或布尔为假）时，循环自动优雅终止。

```typescript
let iter = std.mem.splitScalar(u8, if_none_match, ','); // 按逗号切分
while (tag_str := iter.next()) {
    let parsed = parse(tag_str);
    if (weakMatch(our_parsed.tag, our_parsed.is_weak, parsed.tag, parsed.is_weak)) {
        return true;
    }
}
```

- **类型收窄（Type Narrowing）**：若 `iter.next()` 的返回类型为可选类型 `T?`，在循环体内部，`tag_str` 的类型自动收窄为不可空的 `T`；
- **作用域隔离**：`binding` 变量的作用域严格限制在 `while` 块内部，杜绝作用域污染；
- **零成本转译**：底层 1:1 发射 Zig 原生的高效 `while (iter.next()) |tag_str|` 循环指令，零额外堆栈开销。

---

#### 9.3.3 Walrus 海象操作符条件解包语句 (`if (binding := expr)`)

同理，海象操作符（`:=`）在 `if` 条件语句中提供对称的条件非空解包能力，彻底取代了 Zig 风格的 `if (expr) |capture|`：

```typescript
if (user := find_user_by_id(user_id)) {
    // 成功分支：user 自动解包并收窄为不可空的 User 类型
    print(`Found user: ${user.name}`);
} else {
    // 空值分支：find_user_by_id 返回 null 时执行
    print("User not found");
}
```

> [!NOTE] 语法废弃规范说明
> CP 已彻底废除 Zig 风格的管道捕获 `while (...) |capture|` 与 `if (...) |capture|`。在 CP 中，所有条件解包赋值必须统一使用海象操作符 `while (binding := expr)` 与 `if (binding := expr)`。若使用旧版管道捕获语法，编译器将在解析期直接报错拦截。

---

#### 9.3.3.1 控制流类型智能收窄 (Flow-Sensitive Type Narrowing: `if (v != null)`)

除了海象操作符声明新绑定的解包方式外，CP 编译器全面支持**控制流敏感的变量类型智能收窄（Flow-Sensitive Type Narrowing）**。当对已有可选变量（如 `v: T?`）进行非空判断时，在对应的分支块内，该变量自动由 `T?` 收窄为不可空类型 `T`，无需显式解包（无需书写 `v.?`）：

```typescript
let u: User? = find_user(10);

if (u != null) {
    // 1. 直接访问字段与方法，无需 u.?
    print(u.name);
    u.have_birthday();

    // 2. 直接参与算术或传参给要求非可选类型的函数
    save_user(u); // 参数类型为 User，自动匹配

    // 3. 兼容显式解包写法（杜绝二次重复解包）
    let unwrapped = u.?;
} else {
    // 此分支内 u 保持 null
}
```

- **全面支持的分支与判断形式**：
  - 正向非空收窄：`if (v != null)`、`if (null != v)`、`if (v != null && cond)`
  - 反向分支收窄：`if (v == null) { ... } else { /* 此处 v 自动收窄为 T */ }`
  - 循环守卫：`while (v != null) { ... }`
- **变量赋值安全与状态维护**：
  - 在收窄作用域内对被收窄变量重新赋值时（如 `count = 100`），左值保持变量原始内存槽，不会错误注入解包；
  - 若重新赋予 `null`（如 `count = null`），编译器自动解除该变量的收窄状态。

---

#### 9.3.4 原生类型标识符作为调用实参 (Native Type Tokens as Arguments)

CP 允许原生类型标识符（如 `u8`、`i32`、`usize` 等）直接作为泛型函数或标准库函数的实参传递，无缝契合系统级与标准库接口（如 `std.mem.trim(u8, ...)`、`std.mem.eql(u8, ...)`）：

```typescript
let str = std.mem.trim(u8, raw_str, " \t");
let is_same = std.mem.eql(u8, slice_a, slice_b);
```

---

### 9.4 字符串模板插值与原生正则表达式

```typescript
let agentName = "Nexus-9";
let taskCount = 42;

// 模板插值
let summary = "Agent ${agentName} executed ${taskCount} reductions.";

// 原生正则字面量
let emailRegex = /^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$/;
if (emailRegex.test("admin@cp-nest.org")) {
  print("Valid email format");
}
```

---

## 十、异步编程、双轨并行计算与 Nest 智能体容器

### 10.1 CPS 状态机异步内核

每个标记为 `async` 的函数在编译期被解构切片为一个紧凑的 Zig 结构体状态机。当遇到 `await` 时，函数保存程序计数器（`pc`）与局部上下文，执行物理 `return` 出让线程，彻底消灭运行时调用栈膨胀。

---

### 10.2 双轨派生模型：协程池与 Worker 物理线程池

CP 在语言级首创了**“双轨派生模型（Dual-Track Spawn）”**，以满足高吞吐信箱与重度科学计算的不同极端诉求：

```typescript
// 1. spawn(fn) —— 轻量级协程池 (Coroutine Pool)
// 纳秒级派发，依托 CPS 状态机与 Reductions 时间片切分，适合数百万高并发 Agent、网络 I/O、事件传递
let t1: Task = spawn(async () => {
  return fetch_agent_state();
});

// 2. spawn.cpu(fn) —— Worker 物理线程池 (CPU Thread Pool)
// 派生真实 OS 物理线程，脱离协程调度器，专门压榨多核算力，用于重度矩阵运算、Tensor 变换、图像渲染
let t2: Task = spawn.cpu(() => {
  return matrix_multiply(big_tensor_a, big_tensor_b);
});

// 同步协调与等待
let res = join(t2);
```

---

### 10.3 并发运行时配置体系（注解与 `cap.json`）

#### 10.3.1 项目级全局配置（`cap.json`）

```json
{
  "mode": "parallel"
}
```

#### 10.3.2 代码级入口注解声明

```typescript
// 显式声明开启多核并行引擎，并指定 8 个 Worker 物理线程
@parallel(8)
fn main() {
    print("Parallel runtime initialized with 8 OS workers.");
}

// 亦可声明强制单线程模式，便于精准隔离调试与竞态分析
@single_thread
fn debug_main() {
    // ...
}
```

---

### 10.4 跨并发边界内存安全防御（Send/Sync 铁律）

当向 `spawn` 或 `spawn.cpu` 派发闭包时，编译器启动静态逃逸守卫检查：

- **严禁捕获栈上引用结构体（`ref struct`）**；
- **严禁捕获局部临时借用（`&T`）**；
- 跨任务必须使用 `move` 显式转移所有权，或使用 `Shared<T>` / `SharedTensor` 走原子引用计数（`std.atomic`）。

---

### 10.5 多线程并发同步与原子原语：Mutex、RwLock 与 Atomic (内置标准库 cap:sync)

在多核物理线程并发（`spawn.cpu`）场景下，各线程对共享状态的同抢碰撞时机具有动态不确定性。仅靠编译期纯语法检查无法替代运行期的互斥与原子避让。CP 语言借鉴 Rust、Go、Zig 与 TypeScript 的精髓，在标准库 `cap:sync` 中提供了原生多线程同步原语 **`Mutex`（互斥锁）**、**`RwLock`（读写锁）** 与 **`Atomic`（原子操作）**。

#### 10.5.1 设计哲学与命名规范（TS-Style camelCase）

1. **全面采用 TS/JS 风格的小驼峰命名（camelCase）**：彻底告别下划线蛇形命名，使用 `tryLock()`、`isLocked()`、`withLock()`、`withRead()`、`withWrite()`、`tryRead()`、`tryWrite()`、`testAndSet()`，保持语言整体风格的统一与优美简洁。
2. **容器型（包装数据）与裸锁对称支持**：
   - `Mutex(data)` / `RwLock(data)`：数据与锁强绑定，外部无法绕过锁直接访问被保护数据；
   - `Mutex()` / `RwLock()`：裸排他锁，用于纯流程临界区同步与保护外部异构资源。
3. **双模式访问能力**：
   - **闭包模式（推荐，最优美）**：`withLock((s) => { ... })`、`withRead`、`withWrite`，编译期直接内联为零运行时开销的底层 RAII 临界区代码块，自动获取锁并在退出时通过 `defer` 绝对安全释放锁。
   - **守卫模式（RAII Guard）**：`lock()` 返回 `LockGuard<T>`，`read()` 返回 `ReadGuard<T>`，`write()` 返回 `WriteGuard<T>`。守卫对象支持透明字段解包，生命周期结束时由编译器自动织入 `defer guard.release()` 兜底解锁，亦可显式调用 `guard.release()` / `guard.unlock()` 提前释放。
4. **非阻塞与健康探测**：提供 `tryLock()` / `tryRead()` / `tryWrite()`（返回可选类型 `Guard?`，拿不到锁立即返回 `null`，绝不卡死线程）以及 `isLocked()` / `isReadLocked()` / `isWriteLocked()`（状态探测）。

```typescript
import { Mutex, RwLock, Atomic } from "cap:sync";
```

---

#### 10.5.2 互斥锁 Mutex：数据包装锁 `Mutex<T>` 与 裸锁 `Mutex()`

##### 1. API 接口规范

| 类别               | 方法签名                                   | 说明                                                     |
| :----------------- | :----------------------------------------- | :------------------------------------------------------- |
| **推荐闭包**       | `withLock<R>(fn: (*T) -> R): R`            | 执行闭包，进入前自动加锁，退出（含早退、异常）即刻释放锁 |
| **RAII 阻塞加锁**  | `lock(): LockGuard<T>`                     | 阻塞直到获取锁，返回守卫对象，作用域结束自动释放锁       |
| **非阻塞尝试拿锁** | `tryLock(): LockGuard<T>?`                 | 非阻塞尝试获取锁；成功返回守卫，失败立即返回 `null`      |
| **状态探测**       | `isLocked(): bool`                         | 探测锁当前是否已被任何线程持有（探针与调试监控用）       |
| **裸锁手动解锁**   | `unlock(): void` _(仅裸锁 `Mutex()` 提供)_ | 手动释放当前持有的锁                                     |

##### 2. 代码示例

```typescript
import { Mutex } from "cap:sync";

struct CounterState {
    count: i32;
    tag: string;
}

// 1. 包装数据的容器互斥锁
let state = Mutex(CounterState { count: 0, tag: "worker" });

// 方式 A：withLock 闭包（推荐：最优雅，自动 RAII）
state.withLock((s) => {
    s.count += 1;
});

// 方式 B：lock() 阻塞获取守卫
{
    let guard = state.lock();
    guard.count += 10; // 支持透明字段访问
    print(`Current count: ${guard.count}`);
    // 离开大括号作用域时，编译器自动 defer guard.release() 解锁
}

// 方式 C：tryLock() 非阻塞尝试（配合海象解包）
if (guard := state.tryLock()) {
    guard.count += 1;
    // 亦可显式提前解锁
    guard.release();
} else {
    print("Lock contention detected, skipping current frame.");
}

// 2. 裸互斥锁 Mutex()
let rawMu = Mutex();
rawMu.lock();
if (rawMu.isLocked()) {
    print("Locked!");
}
rawMu.unlock();
```

---

#### 10.5.3 读写锁 RwLock：数据包装读写锁 `RwLock<T>` 与 裸读写锁 `RwLock()`

##### 1. API 接口规范

| 类别             | 方法签名                         | 说明                                                      |
| :--------------- | :------------------------------- | :-------------------------------------------------------- |
| **推荐只读闭包** | `withRead<R>(fn: (&T) -> R): R`  | 多读并发：获取共享读锁并执行闭包，退出时自动释放读锁      |
| **推荐独占闭包** | `withWrite<R>(fn: (*T) -> R): R` | 单写排他：获取独占写锁并执行闭包，退出时自动释放写锁      |
| **共享读加锁**   | `read(): ReadGuard<T>`           | 阻塞获取共享读锁，返回只读守卫，离开作用域自动释放        |
| **独占写加锁**   | `write(): WriteGuard<T>`         | 阻塞获取独占写锁，返回可变写守卫，离开作用域自动释放      |
| **非阻塞尝试读** | `tryRead(): ReadGuard<T>?`       | 非阻塞尝试获取读锁；成功返回只读守卫，冲突立即返回 `null` |
| **非阻塞尝试写** | `tryWrite(): WriteGuard<T>?`     | 非阻塞尝试获取写锁；成功返回写守卫，冲突立即返回 `null`   |
| **读锁状态探测** | `isReadLocked(): bool`           | 探测当前是否有活跃的读者线程                              |
| **写锁状态探测** | `isWriteLocked(): bool`          | 探测当前是否被写者独占持有                                |

##### 2. 代码示例

```typescript
import { RwLock } from "cap:sync";

struct CacheData {
    version: i32;
    content: string;
}

let cache = RwLock(CacheData { version: 1, content: "Initial" });

// 多读并发：withRead 闭包
cache.withRead((c) => {
    print(`Version: ${c.version}, Content: ${c.content}`);
});

// 单写独占：withWrite 闭包
cache.withWrite((c) => {
    c.version += 1;
    c.content = "Updated";
});

// 守卫式读写与非阻塞尝试
if (r := cache.tryRead()) {
    print(`Read version: ${r.version}`);
}
```

---

#### 10.5.4 原子操作原语 Atomic<T>

`Atomic<T>` 直通 CPU 底层硬件级原子指令（如 x86 `LOCK CMPXCHG`、ARM64 `LDADD` / `CAS`），支持 `i32`、`u32`、`i64`、`u64`、`usize`、`bool` 等基础类型，具备纳秒级绝对并发安全：

##### 1. API 接口规范

| 方法签名                               | 说明                                                                             |
| :------------------------------------- | :------------------------------------------------------------------------------- |
| `load(): T`                            | 以顺序一致性（SeqCst）原子读取当前值                                             |
| `store(val: T): void`                  | 以顺序一致性原子写入新值                                                         |
| `swap(val: T): T`                      | 原子写入新值并返回旧值（Exchange）                                               |
| `cas(expected: T, desired: T): bool`   | 极简 CAS：若当前等于 `expected` 则写入 `desired` 并返回 `true`，否则返回 `false` |
| `compareAndSwap(exp: T, des: T): bool` | 全称 CAS 别名，与 `cas` 完全等价                                                 |
| `add(delta: T): T`                     | 原子加法，返回操作前的旧值（FetchAdd）                                           |
| `sub(delta: T): T`                     | 原子减法，返回操作前的旧值（FetchSub）                                           |
| `inc(): T`                             | 原子自增 1，返回操作前的旧值                                                     |
| `dec(): T`                             | 原子自减 1，返回操作前的旧值                                                     |
| `and(mask: T): T`                      | 按位与（FetchAnd），返回旧值                                                     |
| `or(mask: T): T`                       | 按位或（FetchOr），返回旧值                                                      |
| `xor(mask: T): T`                      | 按位异或（FetchXor），返回旧值                                                   |
| `testAndSet(): bool`                   | 布尔原子原语：原子设为 `true` 并返回之前的值（TAS）                              |
| `clear(): void`                        | 布尔原子原语：原子复位为 `false`（或数值 0）                                     |

##### 2. 代码示例

```typescript
import { Atomic } from "cap:sync";

let counter = Atomic(0); // 自动单态化为 Atomic<i32>
let flag = Atomic(false); // 自动单态化为 Atomic<bool>

// 原子自增与算术
counter.inc(); // 0 -> 1
counter.add(10); // 1 -> 11

// 原子 CAS（Compare-And-Swap）
let ok = counter.cas(11, 20); // 若当前为 11 则替换为 20，返回 true
let ok2 = counter.compareAndSwap(20, 30);

// 布尔原子操作
let wasActive = flag.testAndSet(); // 设为 true 并返回先前状态
flag.clear(); // 设为 false
```

---

#### 10.5.5 多线程共享模式：`Shared` + `Mutex` / `RwLock` 协同实战

在多核并行 Worker（`spawn.cpu`）之间共享并可变修改复杂对象时，采用 `Shared(Mutex(data))` 实现安全的内部可变性（Interior Mutability）：

```typescript
import { Mutex } from "cap:sync";

struct SharedMetrics {
    totalRequests: i32;
    activeConnections: i32;
}

let metrics = Shared(Mutex(SharedMetrics {
    totalRequests: 0,
    activeConnections: 0,
}));

let m1 = metrics.clone();
let t1 = spawn.cpu(() => {
    for (i of 0..1000) {
        m1.withLock((m) => {
            m.totalRequests += 1;
        });
    }
});

let m2 = metrics.clone();
let t2 = spawn.cpu(() => {
    for (i of 0..1000) {
        m2.withLock((m) => {
            m.totalRequests += 1;
        });
    }
});

join(t1);
join(t2);

metrics.withLock((m) => {
    print(`Final Total Requests: ${m.totalRequests}`); // 严格等于 2000
});
```

---

#### 10.5.6 内存边界对比：为什么跨线程/跨协程必须用 `Shared<T>` 而不能用 `&T`？

很多初学者容易混淆“只读借用 `&T`”与“共享所有权句柄 `Shared<T>`”，误以为两者都是只读，应该可以通用。在 CP 语言的设计哲学中，两者的底层物理布局与生命周期安全边界有着**质的差别**：

##### 1. 核心差异全景对比表

| 维度             | 栈绑定只读借用 (`&T`)                                   | 共享所有权句柄 (`Shared<T>`)                               |
| :--------------- | :------------------------------------------------------ | :--------------------------------------------------------- |
| **本质属性**     | 栈绑定的临时引用视图（Stack-Bound View）                | 全局共享堆上的原子引用计数句柄（Shared Ownership）         |
| **所有权归属**   | **零所有权**。完全依赖原宿主变量在外部栈帧中活着        | **共同所有权**。每个持有句柄的线程/协程都是合法所有者      |
| **生命周期**     | 严格绑定在当前局部作用域内，**绝对禁止逃逸**            | 动态生命周期（Dynamic Escape），最后一个持有者退出时才释放 |
| **内存开销**     | **绝对零开销**（底层就是裸指针，无堆分配，无计数）      | **极轻量**（堆分配一次 + 每次拷贝句柄原子引用计数 `+1`）   |
| **跨线程/Actor** | ❌ **编译期绝对拦截**（防止宿主函数栈销毁产生悬垂指针） | ✅ **无畏并发传递**（各线程安全零拷贝访问）                |
| **跨越 `await`** | ❌ **禁止跨越状态机**（切分后当前函数栈帧不复存在）     | ✅ **安全穿透**（句柄安全保存在异步 Context 中）           |
| **语法泛型负担** | 零 `<'a>` 生命周期标注，由编译器做 NLL 局部推导         | 零 `<'a>` 生命周期标注，享受 TS 引用般的自如               |

##### 2. 代码对比：借用逃逸拦截 vs Shared 零拷贝跨线程

```typescript
// ❌ 错误示范：试图把栈绑定的局部借用 &data 传递给跨线程任务（编译期精准拦截！）
fn launch_worker_invalid() {
    let local_cfg = ModelConfig { hidden_size: 4096 };
    let cfg_borrow: &ModelConfig = &local_cfg; // 栈绑定只读借用

    // ❌ CP 编译期报错拦截：Cannot move or pass stack-bound borrow '&local_cfg' across thread/spawn boundary!
    // 原因：launch_worker_invalid 随时可能 return 并销毁栈帧，若允许传给异步线程，将产生 Use-After-Free 严重崩溃！
    spawn.cpu(() => {
        print(`Hidden size: ${cfg_borrow.hidden_size}`);
    });
}

// ✅ 正确示范：使用 Shared(T) 建立零拷贝共享所有权（安全跨线程/协程并发）
fn launch_worker_valid() {
    let shared_cfg = Shared(ModelConfig { hidden_size: 4096 }); // 堆分配 + 原子引用计数

    let cfg_handle = shared_cfg.clone(); // 仅增加原子引用计数，零内存拷贝
    spawn.cpu(() => {
        // ✅ 即使 launch_worker_valid 函数退出销毁了外层栈，shared_cfg 的引用计数仍 >= 1
        // 底层内存依然安全存活，直到该后台线程运行完毕解构 cfg_handle 时才安全释放！
        print(`Worker read hidden size: ${cfg_handle.hidden_size}`);
    });
}
```

---

#### 10.5.7 多线程数据保护机制全景对比（CP vs Rust vs Go vs Zig）

| 语言     | 共享并发数据保护方案                    | 优势与亮点                                                                                                                                                                                                           | 痛点与开发心智负担                                                                                                                                                                                  |
| :------- | :-------------------------------------- | :------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | :-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **CP**   | **`Shared<T>` + `Mutex<T>` / `RwLock`** | • **TS 极简语法**：`Shared(Mutex(data))` 一键构造<br>• **数据与锁强绑定**：必须通过 `withLock` 或守卫访问数据<br>• **自动 RAII 解锁**：守卫属性透明访问，离开块即释放<br>• **零 GC 停顿**：直接编译为 Zig 原生状态机 | • 目前自适应锁基于高性能原子自旋与调度让步，重度长临界区仍需配合 OS 级 futex 调度优化                                                                                                               |
| **Rust** | `Arc<Mutex<T>>` / `Arc<RwLock<T>>`      | • 编译期 Borrow Checker 证明无数据竞态<br>• 静态类型系统完备                                                                                                                                                         | • **语法繁重冗长**：`Arc::new(Mutex::new(...))` 嵌套深<br>• **到处 unwrap**：每次锁操作必须 `.lock().unwrap()`<br>• **生命周期心智地狱**：结构体字段嵌套借用引发 `<'a>` 病毒式传染                  |
| **Go**   | `sync.Mutex` + `sync/atomic`            | • 语言内置关键字轻量，上手极快                                                                                                                                                                                       | • **数据与锁脱节（裸锁）**：锁是独立变量，程序员极易忘记加锁直接访问数据，引发未定义竞态<br>• **手动 defer 容易泄露**：循环体内 `defer` 不释放<br>• **GC STW 噩梦**：高并发海量对象引发显著 GC 停顿 |
| **Zig**  | `std.Thread.Mutex` + `std.atomic.Value` | • 绝对底层、零隐藏开销、透明可控                                                                                                                                                                                     | • **无语言级并发安全屏障**：没有所有权和借用检查，全靠程序员自律<br>• **缺少容器锁封装**：无泛型 `Mutex(T)` 数据绑定容器，必须人肉手写包装与内存释放                                                |

---

#### 10.5.8 工业级并发实战指南：5 大经典并发场景代码示例

##### 场景一：只读全局配置/大型张量在多线程之间零拷贝共享

```typescript
import { Shared } from "cap:sync";

struct AppConfig {
    pub maxClients: i32;
    pub apiEndpoint: string;
}

// 1. 创建只读共享全局配置
let globalConfig = Shared(AppConfig {
    maxClients: 10000,
    apiEndpoint: "https://api.cp-lang.org/v1",
});

// 2. 派发给多个并行 Worker（零物理拷贝，仅复制原子引用句柄）
let w1 = spawn.cpu(() => {
    let cfg = globalConfig.clone();
    print(`Worker 1 using endpoint: ${cfg.apiEndpoint}`);
});

let w2 = spawn.cpu(() => {
    let cfg = globalConfig.clone();
    print(`Worker 2 max clients: ${cfg.maxClients}`);
});

join(w1);
join(w2);
```

##### 场景二：多线程高频写入与内部可变性（`Shared(Mutex(T))`）

```typescript
import { Mutex, Shared } from "cap:sync";

struct WorkerTaskPool {
    pub activeWorkers: i32;
    pub completedTasks: i32;
}

// 内部可变性模式：外层 Shared 提供线程间安全共享句柄，内层 Mutex 提供可变修改互斥保护
let pool = Shared(Mutex(WorkerTaskPool {
    activeWorkers: 0,
    completedTasks: 0,
}));

let p1 = pool.clone();
let worker1 = spawn.cpu(() => {
    // 推荐模式：withLock 自动加锁与即刻解锁
    p1.withLock((p) => {
        p.activeWorkers += 1;
    });

    // 模拟业务处理...

    p1.withLock((p) => {
        p.completedTasks += 1;
        p.activeWorkers -= 1;
    });
});

let p2 = pool.clone();
let worker2 = spawn.cpu(() => {
    // 守卫模式：lock() 阻塞拿锁，离开当前大括号作用域自动释放锁
    {
        let guard = p2.lock();
        guard.completedTasks += 5; // 透明字段访问，无需额外解引用
    }
});

join(worker1);
join(worker2);

pool.withLock((p) => {
    print(`Final Completed Tasks: ${p.completedTasks}`);
});
```

##### 场景三：读多写少高并发缓存系统（`Shared(RwLock(T))`）

```typescript
import { RwLock, Shared } from "cap:sync";

struct UserCache {
    pub sessionToken: string;
    pub refreshCount: i32;
}

let cache = Shared(RwLock(UserCache {
    sessionToken: "initial-token-12345",
    refreshCount: 0,
}));

// 并发读：派发前 clone 独立句柄给后台线程，通过 withRead 获取共享读锁
let readerCache = cache.clone();
let reader = spawn.cpu(() => {
    // withRead 内部直接借用受保护数据，零 clone、零引用计数变化
    readerCache.withRead((c) => {
        print(`Session Token: ${c.sessionToken}`);
    });
});

// 排他写：派发前 clone 独立句柄给后台线程，通过 withWrite 获取独占写锁
let writerCache = cache.clone();
let writer = spawn.cpu(() => {
    // withWrite 内部直接借用受保护数据进行可变写入，零 clone、零引用计数变化
    writerCache.withWrite((c) => {
        c.sessionToken = "new-token-99999";
        c.refreshCount += 1;
    });
});

join(reader);
join(writer);
```

##### 场景四：无锁高性能计数器与探针（`Atomic<T>`）

```typescript
import { Atomic } from "cap:sync";

// 纳秒级无锁原子变量：适合指标统计、限流计数器与状态位标记
let qpsCounter = Atomic(0);
let serverRunning = Atomic(true);

// 1. 原子自增与算术
qpsCounter.inc(); // 0 -> 1
qpsCounter.add(10); // 1 -> 11

// 2. 原子比较交换 CAS（Compare-And-Swap）
let exchanged = qpsCounter.cas(11, 20); // 若当前为 11 则原子置为 20，返回 true
if (exchanged) {
  print("CAS Successfully updated counter to 20");
}

// 3. 布尔状态探测与清除（Test-And-Set）
if (serverRunning.load()) {
  let previousState = serverRunning.testAndSet(); // 原子设为 true 并返回先前状态
  serverRunning.clear(); // 优雅下线：原子复位为 false
}
```

##### 场景五：多资源协同与无载荷裸锁（`Mutex()` / `RwLock()`）

```typescript
import { Mutex } from "cap:sync";

// 无载荷裸互斥锁：适用于跨多个独立数据结构或硬件外设的原子编排
let busLock = Mutex();

busLock.lock();
// 探测锁状态
if (busLock.isLocked()) {
  print("Hardware bus locked, transferring frame...");
}
// 执行硬件总线或跨结构操作...
busLock.unlock(); // 手动释放裸锁
```

---

#### 10.5.9 并发与多线程内存模型权威问答（Concurrency & Memory Model FAQ）

针对程序员在多线程并发与系统级编程中普遍关注的核心问题，本节系统阐述 CP 语言的底层内存模型、引用计数机制与线程安全边界。

##### 1. `Shared<T>` 的 Clone 触发时机：何时必须 Clone，何时无需 Clone？

很多初学者容易将“跨线程派发句柄”与“锁临界区读写数据”混为一谈，甚至误以为 `withLock` / `withRead` 会自动 clone 数据。CP 的物理模型界限极其明确：

- **`withLock` / `withRead` / `withWrite` 会自动 Clone 吗？会增加引用计数吗？**
  - **绝对不会！零 Clone、零深拷贝、零引用计数变化！**
  - 它的底层实现是**零成本数据借用（Borrowing）**：
    1. 获取锁（阻塞或自旋）；
    2. 注册 `defer unlock()` 确保退出（含异常）即刻释放锁；
    3. **直接将锁内部真实受保护数据的裸指针借用（`*T` 或 `&T`）传递给闭包形参**；
    4. 闭包直接就地读写该内存，全程不调用任何 clone，不修改任何引用计数器。
- **那为什么多线程代码中会看到 `let p1 = pool.clone()`？**
  - **关键辨析**：`clone` 是因为 **`spawn.cpu`（跨物理线程）**，而**绝不是因为 `withLock`**！
  - 当你把 `pool` 派发给一个新的后台 OS 线程时，为了让后台线程在父线程提前退出时仍能安全存活（防止 Use-After-Free），必须在进入线程前调用 `let p1 = pool.clone()` 给新线程赋予一份**独立的原子句柄所有权**（此时句柄原子计数 +1，但包裹的数据依然零拷贝）。
  - 后台线程启动后，在线程内部调用 `p1.withLock((p) => ...)` 时，**完全不发生任何 clone，`p` 是受保护数据的直接引用**！
- **无需 Clone 的场景（同线程 / 同协程内零拷贝共享）**：
  在同一个物理线程或协程内部调用函数时，直接传递只读借用参数（如 `fn inspect(cfg: &AppConfig)`），编译器直接传递底层指针，**绝对无需调用 `.clone()`**，开销为纯粹的 0。在 `withLock`、`withRead`、`withWrite` 闭包内部访问受保护的数据时，闭包参数直接绑定到受保护数据，同样无需执行任何 clone。
- **必须 Clone 的场景（跨线程 / 跨协程派发所有权句柄）**：
  当需要将一个共享变量派发给后台物理线程（`spawn.cpu`）或独立协程（`spawn`）时，**必须在派发前调用 `let handle = shared.clone();` 并将 `handle` 传入闭包**。因为子线程拥有独立的执行生命周期，如果不传递独立持有的原子所有权句柄，当派发方外层函数提前返回并退出时，子线程将面临 Use-After-Free 悬垂指针风险。此外，当需要将共享对象持久化存入堆容器（如全局 Map、线程池任务队列）时，也必须克隆句柄。

##### 2. `Shared<T>` 原子引用计数的增减机制与运行期行为

- **原子引用计数增加（+1）的时机**：
  - 显式调用 `shared.clone()` 产生新句柄时；
  - 闭包捕获共享句柄并逃逸派发到 `spawn` / `spawn.cpu` 任务时；
  - 跨信箱将句柄作为消息发送给另一个 Actor 协程时；
  - 将句柄插入到 `Array`、`Map`、`Set` 等堆容器中时。
- **原子引用计数减少（-1）的时机**：
  - 持有句柄的局部变量离开自身作用域时（编译器自动织入 `defer shared.release()`）；
  - 变量被重新赋值时（旧句柄引用计数即刻 -1）；
  - 容器执行 `pop()`、`delete()` 或容器自身被销毁清空时。
- **引用计数归零与即刻确定性释放**：
  **当原子引用计数降为 0 的瞬间，变量所占用的内存会即刻释放！** 底层会立即执行用户析构钩子并调用分配器回收内存（`allocator.destroy`），完全由当前最后释放句柄的线程就地同步完成，**彻底杜绝垃圾回收器（No GC），绝对零 STW 停顿**。

##### 3. 引用克隆 vs 深拷贝：`clone()` 为什么只是 O(1) 原子引用 +1 而非 O(N) 深拷贝？

- **`shared.clone()` 是 O(1) 浅拷贝**：它仅仅在当前堆对象头部的原子计数器字段上执行一次 CPU 硬件级的 `std.atomic.fetch_add`（纳秒级开销），内部包裹的数据（哪怕是数千兆字节的大模型权重或大型张量）**物理内存绝对不发生任何字节搬移**。
- **深拷贝 `mut()` / `deepClone()` 的高昂代价与谨慎使用**：
  真正的深拷贝会沿着指针层层递归，把所有嵌套字段、动态数组和子对象完全在堆上新分配一份，这是极度消耗 CPU 算力与堆内存的 O(N) 繁重操作。在 CP 的高并发哲学中，**深拷贝只在极少数“必须彻底物理隔离、且需要对全量树形结构进行无锁独立可变修改”的场景下才谨慎使用**。日常并发与跨线程共享一律使用 `Shared(T)` 的 O(1) 引用克隆。

##### 4. 裸互斥锁 `let busLock = Mutex();` 的生命周期与遗漏 unlock 的死锁防范

- **裸锁的手动释放隐患**：裸锁 `Mutex()` 不包裹具体数据，调用 `busLock.lock()` 时不会生成守卫对象，如果程序员在后续代码分支、早退（`return`）、条件 `break` 或错误冒泡（`?`）时遗漏了 `busLock.unlock()`，锁将**永久处于持有态**，后续任何试图获取该锁的物理线程都会**无限卡死（死锁 Deadlock）**。
- **最佳工程实践**：
  1. 只要存在需要保护的数据，**永远优先使用容器互斥锁 `Mutex(data)`**，利用 `withLock` 闭包或 RAII 守卫；
  2. 若因硬件总线或流程编排必须使用裸锁 `Mutex()`，**务必在加锁后紧跟 `defer busLock.unlock()`**，确保无论函数是正常返回还是异常中断，锁都会由编译期确定的 `defer` 绝对安全释放。

##### 5. 编译期并发逃逸分析（Escape Analysis）：拦截只读/可变借用跨越线程边界

- **拦截跨线程借用的物理原因**：
  局部只读借用（`&T`）和可变借用（`*T`）在底层是指向**当前线程调用栈**的裸指针。一旦后台物理线程启动，父线程函数随时可能返回并销毁栈帧，若允许栈指针逃逸到后台线程，后台线程的并发解引用将直接沦为灾难性的 **Use-After-Free (UAF)**。
- **编译期静态屏障**：
  CP 编译器在语法分析和类型检查阶段内置了逃逸分析（Escape Analysis）。任何传入 `spawn` 或 `spawn.cpu` 的闭包，一旦捕获了局部借用 `&T`、`*T` 或栈绑定视图 `ref struct`，编译器**在编译期直接硬错误拦截并拒绝生成可执行文件**，从物理上杜绝悬垂指针与跨线程数据竞态。

##### 6. 所有权转移（Move）在多线程下的零竞态保证（Single Ownership）

- **单所有者铁律（Single Ownership）**：
  当使用 `spawn.cpu(() => process(move data));` 将所有权转移给新线程时，原线程中的 `data` 变量在编译期被即刻标记为**已失效（Moved / Dead）**。后续任何代码试图读取或写入 `data` 都会触发编译期硬错误：`CP Compile Error: Use of moved value`。
- **物理零竞态（Zero Race Conditions）**：
  由于在任何一个物理时间切片内，该数据块**有且仅有一个线程拥有独占访问权**，因此即便不使用任何 `Mutex` 互斥锁，也不会产生哪怕一个字节的数据同抢，实现了零同步开销的极致性能。

##### 7. Rust `Send` / `Sync` 机制在 CP 中的极简实现对比

- **Rust 的机制**：Rust 依赖两个内置的自动标记 Trait——`Send`（表示所有权可安全跨线程转移）和 `Sync`（表示不可变引用可安全在多线程间共享，即 `&T: Send`）。若类型包含裸指针 `*const T` 或非线程安全容器（如 `Rc<T>`、`RefCell<T>`），编译器自动移除其 `Send` / `Sync` 实现，禁止跨线程派发。
- **CP 的极简结构化方案**：
  CP 保留了 Rust 100% 的并发安全性，但**彻底摒弃了复杂的标记 Trait 与泛型病毒式传染**：
  1. **值类型与 Move 所有权一等合法**：基础标量、纯数据 `struct` 以及具有独立所有权的对象，天然支持跨线程 `move`（等价于自动满足 `Send`）；
  2. **`Shared<T>` 与内部可变性容器一等合法**：`Shared<T>`（T 为只读）与 `Shared(Mutex(T))` / `Shared(RwLock(T))` 天然支持跨线程安全派发（等价于自动满足 `Send + Sync`）；
  3. **借用与 Unsafe 指针绝对拦截**：栈绑定借用 `&T` / `*T`、栈视图 `ref struct`、以及 Unsafe 裸指针 `*T!` 受到编译期结构化逃逸分析的严格限制，**物理上严禁跨越 `spawn` / `spawn.cpu` 边界**。开发人员无需编写冗余的 Trait 约束，编译器根据物理内存结构直接完成秒级静态判定！

---

### 10.6 异步任务流与事件循环

```typescript
// 并发聚合协调器
let [user, metrics] = await Task.all([fetch_user(), fetch_metrics()]);

// 事件循环体系
let bus = EventEmitter();
bus.on("agent_death", (pid) => {
  print("Agent crashed:", pid);
});
bus.emit("agent_death", 1002);
```

---

### 10.7 Nest 智能体容器（Nest Agent Container / Nest Runtime）

CP 语言是 **Nest 智能体容器** 的原生执行体：

- **Actor 一等公民模型**：`actor`, `@spawn`, `@receive`, `@reply`；
- **抢占式调度器（Reductions 内核）**：以指令计数为依据实行强制抢占，长任务不可饿死其他智能体；
- **零拷贝通信**：小数据物理值拷贝；超大数据 / 显存 Tensor 采用全局共享堆 `SharedTensor` / `Refc Binaries`，仅跨信箱传递指针与原子引用计数；
- **动态链接与打包**：使用 `cap build --beam`（`-b`）编译导出为标准 C-ABI 动态库（`.so`/`.dll`），由 Nest 智能体容器极速加载接管。

---

## 十一、元编程、编译期计算与底层控制

### 11.1 编译全管线时序（Pipeline）

    源码输入
      │
      ▼
    [1. 词法与语法分析] ──生成原始 AST──> [收集元注解 MetaCollection]
      │
      ▼
    [2. Phase 1: 语法宏与编译期装饰器] ──> 在类型检查前执行，转换/包装原始 AST，允许新建类型
      │
      ▼
    [3. 第一轮基础类型检查 (Pass 1 Sema)] ──> 构建全局符号表，确立 Tier 1 TypeDeclInfo
      │
      ▼
    [4. Phase 2: 类型感知派生宏 (Type-Directed Macro)] ──> 运行在 CT Runtime 沙箱中
      │   ├── 读取完整的 Tier 1 / Tier 2 TypeInfo
      │   ├── 【铁律】仅允许新增 `impl` 块与辅助函数，严禁新增 class/struct/trait
      │   ├── 【铁律】生成的 AST 严禁附加任何宏注解
      │   └── 维护展开调用栈与深度熔断 (Max Depth: 128)
      │
      ▼
    [5. 对新增 impl 进行局部类型检查]
      │
      ▼
    [6. 所有权与借用检查 (Borrowck / NLL)]
      │
      ▼
    [7. Native Codegen] ──> 翻译为零开销 Zig 原生状态机 / 机器码

### 11.2 编译期元编程系统（Comptime Metaprogramming）

CP 语言深度借鉴并融合了 Zig 的编译期计算哲学，同时继承 TypeScript 简洁优美的语法表达。所有带 `@` 的编译期机制在编译期求值展开，运行时零开销，死代码自动修剪。

#### 11.2.1 编译期代码执行块（`@{ }` Comptime Block）

在 `@{ }` 内的代码完全在编译期由编译器执行求值，求值结果直接作为编译期常量或数据结构注入 AST（向下兼容 `!{ }`）：

```typescript
// 编译期执行计算，运行时直接作为常量 40 嵌入二进制
const COMPUTED_VAL = @{
    let a = 15;
    let b = 25;
    a + b
};

// 编译期生成预置哈希表 / 查找表
const HASH_TABLE = @{
    let table: [i32] = [];
    for (i of 0..256) {
        table.push(i * 31 % 255);
    }
    table
};
```

#### 11.2.2 编译期函数显式调用（`@fn(...)`）

函数调用前加 `@`，强制编译器在编译期对函数进行常量求值展开，若参数无法在编译期确定则触发编译报错：

```typescript
fn add(a: i32, b: i32): i32 {
    return a + b;
}

// 编译期直接折叠展开为 100
const SUM = @add(30, 70);
```

#### 11.2.3 编译期参数约束（`@comptime`）

函数形参标注 `@comptime`，强制要求调用方传入的值必须在编译期已知：

```typescript
fn create_buffer(@comptime size: usize): [u8] {
    return [0; size];
}

const buf = create_buffer(1024); // ✔️ 合法，1024 为编译期常量
```

#### 11.2.4 编译期自动推导参数（`val: auto`）

函数形参声明为 `auto`，代表该参数为编译期类型自动推导参数。编译器根据传入实参的具体类型自动单态化（Monomorphization）生成专门版本，无需手动书写泛型 `<T>`：

```typescript
fn get_type_size(val: auto): usize {
    return @sizeOf(@typeOf(val));
}

let int_size = get_type_size(42 as i32);   // 单态化为 i32 版本，返回 4
let float_size = get_type_size(3.14 as f64); // 单态化为 f64 版本，返回 8
```

#### 11.2.5 编译期控制流（`@if` / `@else` 与 `@for`）

- **`@if` / `@else`（静态分支修剪）**：条件必须在编译期可判定。编译器在编译期只保留命中分支的代码，死分支被彻底修剪，绝不残留到运行时，且死分支内的代码不会触发类型或借用错误。
- **`@for`（静态循环展开）**：循环边界必须在编译期可判定。编译器在编译期自动将循环体完全展开（Inline unrolling），消除循环跳转指令与分支开销。

```typescript
// 1. 编译期分支修剪
@if (true) {
    let mode = "fast_path";
} @else {
    let mode = "slow_path"; // 完全被编译器丢弃，零运行时开销
}

// 2. 编译期循环展开
let total = 0;
@for (i of 0..5) {
    total = total + (i as i32);
}
// 展开为: total = total + 0; total = total + 1; ... total = total + 4;
```

#### 11.2.6 编译期内建内省原语（Comptime Intrinsics）

CP 语言原生提供与底层无缝对接的编译期内省与诊断指令：

| 原语指令               | 说明                             | 示例                                        |
| :--------------------- | :------------------------------- | :------------------------------------------ |
| `@typeOf(val)`         | 获取表达式在编译期的类型         | `@typeOf(42)` -> `i32`                      |
| `@typeName(T)`         | 获取类型的字符串名称             | `@typeName(i32)` -> `"i32"`                 |
| `@sizeOf(T)`           | 获取类型所占用的字节大小         | `@sizeOf(i64)` -> `8`                       |
| `@alignOf(T)`          | 获取类型的内存对齐字节数         | `@alignOf(f64)` -> `8`                      |
| `@fieldsOf(T)`         | 获取结构体在编译期的字段数组     | `@fieldsOf(User)`                           |
| `@hasField(T, name)`   | 编译期探测类型是否拥有某字段     | `@hasField(User, "id")`                     |
| `@field(obj, name)`    | 编译期字段反射访问与赋值         | `@field(u, "name")`                         |
| `@compileError(msg)`   | 在编译期主动抛出编译失败错误     | `@compileError("Unsupported architecture")` |
| `@compileLog(args...)` | 在编译期向控制台打印编译诊断信息 | `@compileLog("Type size:", @sizeOf(T))`     |

#### 11.2.7 顶层与局部 Comptime 块 (`comptime { ... }` 与 `inline for`)

CP 支持在模块顶层与局部函数内使用 `comptime { ... }` 块（等价于 `@ { ... }`）：

1. **顶层 Comptime 块**：
   - 允许在模块顶层执行静态初始化、平台探测与编译期断言；
   - 编译器将其发射至 Zig 的顶层 `comptime { ... }`；
2. **`inline for` / `comptime for` 循环**：
   - 支持在循环前标注 `inline` 或 `comptime`，强制编译器在编译期对静态范围（如 `0..4` 或数组字段）进行完全内联展开。

```typescript
// 顶层编译期校验
comptime {
    if (@sizeOf(usize) < 8) {
        @compileError("CP 64-bit architecture is required");
    }
}

// 局部内联展开
fn unroll_example() {
    inline for (i of 0..4) {
        print(i);
    }
}
```

---

### 11.3 宏系统（Macro Metaprogramming）

> 宏的作用是在编译时自动插入模版代码，实现一些通用功能，不用每个结构、类都编写相同代码，实现这些功能，提高编码效率。

#### 11.3.1 底层到底是怎么实现的？（是像 C 语言那样的文本替换吗？）

绝对不是 C 语言那种原始、危险的字符串/Token 替换！
CP 的宏是强类型感知、结构化的 AST 过程宏（Type-Directed Procedural AST Synthesizer）。
整个底层展开流程发生在 Sema Phase 2（语义分析第二阶段），分为 4 步：

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│                      CP 宏底层展开流水线 (Sema Pass 2)                          │
├──────────────────────────────────────────────────────────────────────────────┤
│ 1. 提取类的字段元数据 (AST Inspection)                                          │
│    OrderItem 包含: [sku: string, price: f64, quantity: i32]                   │
│                                                                              │
│ 2. 动态拼装 AST 函数节点 (AST Code Synthesis)                                   │
│    • 遍历 cls.fields；                                                        │
│    • 在 Rust 内存中构造一个真实的 ast::Function 节点；                            │
│    • 函数体构造一个 ast::ReturnStmt，拼接字段表达式；                              │
│                                                                               │
│ 3. 注入类结构体 (AST Injection)                                                 │
│    • cls.methods.push(synthesized_fn)； // 物理塞入该类的方法列表中               │
│    • 更新全局符号表 scopes，让外部代码能正常调用 item.toJson()                      │
│                                                                               │
│ 4. 宏擦除与生成机器码 (Macro Erasure & Codegen)                                  │
│    • 宏本身在 AST 展开后被彻底销毁，不进入运行时；                                   │
│    • 合成的新方法与其他普通手写方法一样，无差别编译为原生 Zig 机器码状态机。             │
└───────────────────────────────────────────────────────────────────────────────┘
```

#### 11.3.2 AST 模板插值宏（Template Macros）

##### 1. 核心定位

- 心智模型：类似现代化、带强类型检查的语法模板（Quasi-Quoting）；
- 适用场景：参数驱动型代码生成。针对开发者传入的字段名、方法名前缀或特定类型，按模板快速派生出单个或一组辅助方法（如自动生成 Getter、Setter、Fluent 链式调用 Builder）。

##### 2. 示例代码

```ts
    import { print } from "cap:io";

    // ============================================================
    // 1. 声明 AST 模板宏：为指定字段自动生成 Getter 与 Setter
    //    ${name} 会在编译期将传入的标识符或字符串无缝拼接为 AST 方法名和属性访问
    // ============================================================
    macro Accessor(target, field, field_type) {
        // 自动生成 getter
        pub fn get_${field}(_): field_type {
            return _.${field};
        }

        // 自动生成 setter（支持 Fluent 链式调用，返回自身引用）
        pub fn set_${field}(*_, val: field_type): Self {
            _.${field} = val;
            return _;
        }
    }

    // ============================================================
    // 2. 声明 AST 模板宏：为类注入事件监听订阅方法
    // ============================================================
    macro EventHook(target, event_name) {
        pub fn on_${event_name}(_, handler: fn(string): void): void {
            @print("Registered hook for event: {s}", "${event_name}");
            handler("Event Payload from " + "${event_name}");
        }
    }

    // ============================================================
    // 3. 在业务类上应用模板宏
    // ============================================================
    class Account {
        balance: f64;
        owner: string;

        // 直接在类体内应用模板宏注入方法
        @Accessor("balance", f64)
        @Accessor("owner", string)
        @EventHook("transfer")

        fn init(owner: string, balance: f64) {
            _.owner = owner;
            _.balance = balance;
        }
    }

    // ============================================================
    // 4. 业务调用验证
    // ============================================================
    fn main(): void {
        let acc = new Account("Alice", 1000.0);

        // 调用由 @Accessor("balance") 自动展开生成的 getter
        @print("Initial Balance: {}", acc.get_balance()); // 1000.0

        // 调用由 @Accessor 自动生成的链式 setter
        acc.set_balance(2500.0).set_owner("Alice Cooper");

        @print("Updated Owner: {s}", acc.get_owner());     // Alice Cooper
        @print("Updated Balance: {}", acc.get_balance());  // 2500.0

        // 调用由 @EventHook("transfer") 自动展开生成的事件订阅方法
        acc.on_transfer((msg) => {
            @print("Received callback message: {s}", msg);
        });
    }
```

---

#### 11.3.3 Comptime 过程宏（Comptime Procedural Macros）

##### 11.3.3.1 核心定位

- 心智模型：结合 CP 的 Comptime 编译期图灵完备计算（@{ }） 与 静态内省原语（@fieldsOf、@typeOf）；
- 适用场景：全自动化整类派生型代码生成。无需开发者手工传入字段名，宏自动在编译期全量扫描目标类的所有字段，完成如通用克隆、重置初始值、自动化校验或自定义 YAML/JSON 格式化。

##### 11.3.3.2 示例代码

```ts
    import { print } from "cap:io";

    // ============================================================
    // 1. 声明 Comptime 过程宏：AutoReset
    //    自动遍历 target 类的全部字段，并将数值归 0、字符串归空、布尔归 false！
    // ============================================================
    macro AutoReset(target) {
        // 注入 reset 方法
        pub fn reset(*_): void {
            // ⬇️ 利用 Comptime 编译期循环展开：遍历 target 类的字段元数据
            @for (f of @fieldsOf(target)) {
                @if (f.type == i32 || f.type == f64) {
                    _.${f.name} = 0;
                } @else @if (f.type == string) {
                    _.${f.name} = "";
                } @else @if (f.type == bool) {
                    _.${f.name} = false;
                }
            }
        }
    }

    // ============================================================
    // 2. 声明 Comptime 过程宏：AutoClone（深拷贝派生）
    //    自动提取字段构造参数，为类动态注入深拷贝 clone() 方法！
    // ============================================================
    macro AutoClone(target) {
        pub fn clone(&_): target {
            // 编译期生成 new Target(_.f1, _.f2, ...) 的完整 AST 构造调用
            return new target(
                @{
                    // 编译期动态组装构造参数元组
                    ...
    		}
            );
        }
    }

    // ============================================================
    // 3. 声明 Comptime 过程宏：ToYaml（自定义 YAML 文本序列化）
    // ============================================================
    macro ToYaml(target) {
        pub fn toYaml(&_): string {
            let yaml_output = "---\n";
            // 编译期全静态展开，运行时零反射开销！
            @for (f of @fieldsOf(target)) {
                yaml_output = yaml_output + `  ${f.name}: ` + `${_.${f.name}}\n`;
            }
            return yaml_output;
        }
    }

    // ============================================================
    // 4. 在业务类上应用 Comptime 过程宏
    // ============================================================
    @[AutoReset, ToYaml]
    class GamePlayer {
        nickname: string;
        level: i32;
        score: f64;
        isVip: bool;

        fn init(nickname: string, level: i32, score: f64, isVip: bool) {
            _.nickname = nickname;
            _.level = level;
            _.score = score;
            _.isVip = isVip;
        }
    }

    // ============================================================
    // 5. 业务调用验证
    // ============================================================
    fn main(): void {
        let player = new GamePlayer("ShadowHunter", 99, 15800.5, true);

        // 1. 调用由 @ToYaml 自动展开生成的 YAML 序列化方法
        let yaml_str = player.toYaml();
        @print("--- Generated YAML Output ---\n{s}", yaml_str);
        /* 运行时输出：
           ---
             nickname: ShadowHunter
             level: 99
             score: 15800.5
             isVip: true
        */

        // 2. 调用由 @AutoReset 自动根据类型重置所有字段的方法
        @print("Resetting player data...");
        player.reset();

        @print("After Reset YAML Output ---\n{s}", player.toYaml());
        /* 运行时输出：
           ---
             nickname:
             level: 0
             score: 0
             isVip: false
        */
    }
```

---

#### 11.3.4 两种模式的横向对比与完美分工

| 维度             | 方案 A：AST 模板插值宏                           | 方案 B：Comptime 过程宏                                     |
| ---------------- | ------------------------------------------------ | ----------------------------------------------------------- |
| 驱动机制         | 形参驱动：开发者显式传入 `${field_name}`         | 内省驱动：通过 `@fieldsOf(target)` 自动扫描目标类型         |
| 核心语法         | AST/语法模板插值 `${name}`、`${field}`           | 编译期执行 `@{ }`、`@fieldsOf`、`@for`、`@if`               |
| 典型场景         | 字段级 Getter/Setter、Builder、事件钩子 `on_xxx` | 整类全自动派生：AutoClone、ToYaml、Reset、Validate          |
| 代码量与认知     | 代码量极少，几行模板即可；直观，近似高级文本宏   | 代码可复用度高；能力图灵完备，但内省/合成逻辑的认知成本更高 |
| 适用边界         | 单字段/单成员粒度的重复代码消除                  | 跨成员、需要读取类型结构才能生成的代码                      |
| 失败模式         | 模板变量名拼错 → 展开期报错，定位直观            | 内省逻辑错误 → 宏展开期报错，堆栈较深                       |
| 与另一方案的关系 | 可作为 B 的底层原语                              | 内部可用 A 做代码合成                                       |

#### 11.3.5 内置宏

```cp
@[Clone, Comparable, Hash, ToString, Json]
class OrderItem {
  sku: string;
  price: f64;
  quantity: i32;
}
```

这段代码是现代系统级语言中最经典、最富表现力的**“数据布局与能力契约自动装配”**范式。

CP 语言**彻底切除并废除了 Rust 遗留的 `@derive(...)` 冗余语法**，唯一标准统一为原生批量元注解语法 `@[Clone, Comparable, Hash, ToString, Json]`，实现装饰器、元数据与派生宏在语法形态上的绝对纯粹统一。

以下从代码内涵、批量派生的本质、5 大能力的实现现状，以及宏与 Trait 的深层协作机制进行全面透彻的拆解：

##### 1. 这段代码在表达什么

- 数据容器极度纯粹：开发者只关注核心业务数据的内存排布（sku, price, quantity 三个字段），不写任何冗长机械的方法；
- 能力一键静态织入：通过顶部的 `@[...]` 批量注解，命令编译器在**编译期（Compile-Time）**自动为该类合成并注入 5 种工业级能力（深拷贝、全序比较、哈希运算、调试打印、JSON序列化）；
- 零运行时开销：所有方法在编译期直接由 AST 代码生成器写好，生成高效的裸机机器码，零反射损耗、零 GC 开销。

---

##### 2. 原生批量注解（Batch Annotation Bundling）

`@[Clone, Comparable, Hash, ToString, Json]` 是 CP 的原生一等公民语法：

- 它的语法定位：
  统一整洁的聚合语法糖。写 `@[Clone, ToString, Json]` 等价于依次写独立的派生注解：

  ```cp
  @Clone
  @Comparable
  @Hash
  @ToString
  @Json
  class OrderItem { ... }
  ```

- 它的执行时期：
  在编译器前端的 Sema Phase 2（类型导向宏展开阶段）执行；
- 它的工作原理：
  编译器扫描到 `@[...]` 方括号内的宏标识符列表，将 `OrderItem` 的 AST 结构体（字段名、字段类型）作为输入，在内存中动态构造出一组 `Function` 抽象语法树节点，并自动推入 `cls.methods` 中，然后无缝更新编译器的全局符号表。

---

##### 3. 这 5 大能力的作用分别是什么？

| 能力 / 契约    | 核心作用与调用方式                                                                | 目前在 CP 编译器中的实装现状                                                        |
| :------------- | :-------------------------------------------------------------------------------- | :---------------------------------------------------------------------------------- |
| **Json**       | 自动生成 `toJson(): string` 方法，支持多层嵌套类的深层 JSON 序列化                | ✅ **100% 已实现并测试通过**，在 `sema.rs` 中动态生成 AST，已在测试集验证。         |
| **ToString**   | 自动生成 `toString(): string`，输出 `OrderItem(sku=..., price=..., quantity=...)` | ✅ **100% 已实现并测试通过**，在 `sema.rs` 中动态生成字符串拼接 AST。               |
| **Debug**      | 自动生成 `fmt_debug(&_): string` 与 `toString(_): string`，支持调试日志格式化输出 | ✅ **100% 已实现并测试通过**，一键派生格式化调试与打印字符串。                      |
| **Clone**      | 自动生成 `clone(_): Self` 深拷贝方法，创建并返回独立所有权的新对象                | ✅ **100% 已实现并测试通过**，自动合成 `return new Class(_.f1, _.f2, ...)`。        |
| **Comparable** | 自动生成 `cmp(_, other: &Self): i32` 字典序比对与 `eq(_, other: &Self): bool`     | ✅ **100% 已实现并测试通过**，底层全面打通 `cp_lt` / `cp_gt` / `cp_eq` 强类型比对。 |
| **Hash**       | 自动生成 `hash(_): usize`，自动遍历字段级联计算 FNV-1a 哈希                       | ✅ **100% 已实现并测试通过**，内置 `@hash` 原语对数值、字符串、布尔及引用统一计算。 |
| **Table**      | 自动生成 `getTableName(): string` 数据库映射方法                                  | ✅ **100% 已实现并测试通过**，支持显式表名 `@Table("t_users")` 或隐式推导。         |

此外，CP 编译器引入了**零样板成员构造函数自动合成**机制：当类中包含字段且开发者未手写 `fn init(...)` 时，编译器在 Sema Phase 2 自动生成全参数 `fn init(f1: T1, f2: T2, ...)`，彻底告别冗长的样板构造赋值代码！

---

##### 4. 这些宏（@Clone, @Hash, @Comparable）与 Trait 的深层关系

这是理解现代编译型语言最核心、最精妙的关键点：宏与 Trait 不是二选一，而是“契约”与“自动化劳工”的双簧关系！

```text
    ┌────────────────────────────────────────────────────────────────────────────────────┐
    │                           Trait 与 Macro 的职责划分与协同                             │
    ├───────────────────────────────────────────────────────┬────────────────────────────┤
    │                Trait（契约规范 - What）                 │  Macro 宏（代码生成 - How）   │
    ├───────────────────────────────────────────────────────┼─────────────────────────────┤
    │ • 定义“接口规范”（如：必须有个 cmp 方法）                  │ • 充当“打字员与自动化代码生成器” │
    │ • 用于编译期泛型约束：fn sort<T: Comparable>(arr: [T])   │ • 自动遍历字段，生成满足契约的代码│
    │ • 决定类型【能不能】传入泛型算法                           │ • 解决开发者【不想手写】大量样板 │
    └───────────────────────────────────────────────────────┴─────────────────────────────┘
```

##### 5. 场景 1：如果“只有 Trait，没有宏”会发生什么？

假设标准库有 `trait Clone { fn clone(&_): Self; }`。

为了让 OrderItem 支持克隆，开发者必须手动写出冗长的代码：

```typescript
    impl Clone for OrderItem {
        fn clone(&_): OrderItem {
            return new OrderItem(_.sku, _.price, _.quantity); // 繁琐，字段一多极易漏写
        }
    }
```

痛点：类型系统很严谨，但开发者的体验非常痛苦，充斥着无意义的手工样板代码。

##### 6. 场景 2：如果“只有宏，没有 Trait”会发生什么？

宏虽然自动帮 OrderItem 生成了 `clone()` 方法，你也可以写 `item.clone()`。
痛点：但是当你写一个通用函数时：

```typescript
    fn duplicate<T>(item: &T): T {
        return item.clone(); // ❌ 编译报错！编译器不知道泛型 T 到底有没有 clone 方法！
    }
```

因为没有 Trait 作为公认的契约签名，泛型系统就无法在编译期进行类型约束。

##### 7. 场景 3：Trait 与 宏 珠联璧合（CP 的终极设计）

- Trait 负责立宪立约：标准库定义 `trait Clone`、`trait Hash`、`trait Comparable`，供泛型算法（如 `sort<T: Comparable>`、`Map<K: Hash, V>`）作为类型边界；
- 宏负责自动干脏活：开发者打上 `@[Clone, Comparable, Hash]` 后，编译器在后台全自动合成了对应的方法实现！

---

##### 8. 编译后在底层实际替开发者写了什么代码？

当编译器看到 `@[Clone, Comparable, Hash, ToString, Json]` 时，在 Phase 2 内部等价于为 `OrderItem` 自动合成了以下完整实现：

```typescript
    // 1. @Clone 替你生成的：
    fn clone(_): OrderItem {
        return new OrderItem(_.sku, _.price, _.quantity);
    }

    // 2. @ToString 替你生成的：
    fn toString(_): string {
        return `OrderItem(sku=${_.sku}, price=${_.price}, quantity=${_.quantity})`;
    }

    // 3. @Comparable 替你生成的：
    fn eq(_, other: &OrderItem): bool {
        if (_.sku != other.sku) return false;
        if (_.price != other.price) return false;
        if (_.quantity != other.quantity) return false;
        return true;
    }

    fn cmp(_, other: &OrderItem): i32 {
        if (_.sku < other.sku) return -1;
        if (_.sku > other.sku) return 1;
        if (_.price < other.price) return -1;
        if (_.price > other.price) return 1;
        if (_.quantity < other.quantity) return -1;
        if (_.quantity > other.quantity) return 1;
        return 0;
    }

    // 4. @Hash 替你生成的：
    fn hash(_): usize {
        return @hash(_.sku, _.price, _.quantity);
    }

    // 5. @Json 替你生成的：
    fn toJson(_): string {
        return "{\"sku\":\"" + _.sku + "\",\"price\":" + _.price + ",\"quantity\":" + _.quantity + "}";
    }
```

##### 9. 迭代器契约与双轨 `for ... of` 迭代机制

CP 语言的 `for ... of` 迭代语法采用**双轨智能分流架构**，既保证了原生连续内存遍历的绝对裸机速度，又原生支持流式（Stream）与无限序列生成器：

1. **快轨（Fast Path，连续内存与切片/区间）**：
   - 目标为数组或切片时，底层直接下沉为 Zig 原生切片遍历 `cp_iter_slice(arr)`，零虚表、零调用开销；
   - 目标为 `0..10` 时，直接发射为寄存器级 `while (i < 10) : (i += 1)` 计数器循环；
   - 目标为 `@comptime` 时，直接发射为编译期 `inline for` 展开。
2. **流式轨（Iterator Stream Path，任意具备 `next()` 方法的类型）**：
   - 任何实现了 `next(*_): Item?` 契约的对象或流，编译器自动将其展开为零分配的 `while (stream.next()) |item|` 循环，当 `next()` 返回 `null` 时自动优雅终止循环！

```cp
// 示例：自定义流式迭代器
class NumberStream {
    current: i32;
    limit: i32;

    fn init(start: i32, limit: i32) {
        _.current = start;
        _.limit = limit;
    }

    fn next(*_): i32? {
        if (_.current >= _.limit) {
            return null; // 耗尽流，自动终止循环
        }
        let val = _.current;
        _.current = _.current + 1;
        return val;
    }
}

let stream = new NumberStream(1, 5);
for (n of stream) {
    // 自动按流式展开调用 stream.next()
    print(n);
}
```

##### 10. 泛型 Trait Bound 契约约束

在通用算法或工具函数中，开发者可以通过 Trait Bound 约束类型参数，例如约束类型必须拥有调试输出能力：

```cp
// 1. 定义或使用标准库 Trait
trait Debug {
    fn fmt_debug(&this): string;
}

// 2. 泛型函数约束：仅允许实现了 Debug 契约的类型传入
fn log(item: &Debug): void {
    print(item.fmt_debug());
}
```

配合 `@[Debug]`，类型声明只需一行注解，编译器即全自动补齐 `fmt_debug` 与 `toString`，完美满足 Trait 约束！

##### 11. 总结

• `OrderItem` 是干净的数据；
• Trait 是标准能力接口契约；
• `@[...]` 是全自动实现这些契约的强劲发动机。

---

### 11.4 自定义元注解与类/方法装饰器

#### 11.4.1 核心定位与极简设计原则

CP 语言将**元注解（`meta`）**与**装饰器（`dec`）**从语法与执行模型上彻底解耦：

- **`meta`**：纯静态数据标记，编译期零开销，运行时进静态反射表（不执行任何逻辑代码）；
- **`dec`**：可执行切面逻辑（AOP），在目标方法调用前后进行包装、拦截、重试或增强。

#### 11.4.2 四大极简铁律：

1. **极短关键字 `dec`**：统一采用 3 字母的 `dec` 声明装饰器，干脆利落；
2. **消灭闭包样板的洋葱切面模型**：无参切面入参直写 `next`，自动捕获并透明透传原函数参数与返回值；
3. **编译期静态合成 (Static Synthesis AOP)**：彻底消灭动态代理（Dynamic Proxy）、动态猴子补丁与运行时闭包堆分配，编译器就地展开静态包装函数（`_raw_xxx` 与 `_wrap_xxx`），直达裸机速度；
4. **调用端三免法则**：无参免括号（`@timer`）、单参免键名（`@retry(3)`）、多装饰器洋葱式嵌套堆叠。

---

#### 11.4.3 装饰器定义与切面语法

##### 11.4.3.1 无参直接切面（Around Aspect）

```cp
dec timer(next) {
    let t0 = clock.now();
    let res = next(); // 自动无损透传原参数，捕获返回值
    print(`[Timer] 耗时: ${clock.now() - t0}ms`);
    return res;
}

dec doubleResult(next) {
    let res = next();
    return res * 2;
}
```

##### 11.4.3.2 带参装饰器（双轨风格支持）

开发者可以自由选择以下两种风格之一：

- **风格 A：直接参数风格（Direct Style）**

  ```cp
  dec tag(next, prefix: string) {
      let res = next();
      return "[" + prefix + "] " + res;
  }
  ```

- **风格 B：高阶工厂风格（Higher-Order Closure Factory）**
  ```cp
  dec retry(times: i32 = 3) {
      return fn(next) {
          for (i of 0..times) {
              let res = next();
              if (res != null) { return res; }
          }
          return null;
      };
  }
  ```

---

#### 11.4.4 洋葱模型（Onion Execution Model）

##### 11.4.4.1 当一个方法堆叠了多个装饰器时，其执行顺序严格遵循工业级洋葱模型（从外层进入，内层返回）：

```cp
@outerWrap
@innerWrap
fn compute(x: i32): i32 {
    return x * 2;
}
```

- **调用链展开**：
  `compute(x)` 入口 -> 执行 `outerWrap` 前置 -> 调用 `next()` -> 执行 `innerWrap` 前置 -> 调用 `next()` -> 执行 `_raw_compute` -> 返回并执行 `innerWrap` 后置 -> 返回并执行 `outerWrap` 后置。

  在 CP 编译器底层，洋葱模型（Onion Execution Model）完全是在编译期通过 AST 静态重写（AST Rewriting）实现的，零堆分配、零闭包开销、零函数指针间接跳转。

──────

##### 11.4.4.2 next 到底是什么

next 是装饰器 dec 声明中的编译期逻辑占位符（Syntactic Placeholder），代表“被当前装饰器拦截的目标逻辑（管道下一环）”。

##### 11.4.4.3 洋葱模型的物理调用全景

整个调用过程没有任何动态中间件调度器（Middleware Runner），而是化解成了纯静态函数调用链（Direct Function Call Chain）：

```text
  调用入口：compute(x)
  │
  ▼
  ┌─────────────────────────────────────────────────────┐
  │ 🧅 [Outer Layer] outerWrap 作用域                    │
  │ 1. 执行 outerWrap 前置逻辑 (如计时起点、打印日志)         │
  │ 2. 调用 next() ══════════════════════════════════╗   │
  │                                                 ║   │
  │ ┌─────────────────────────────────────────────┐ ║   │
  │ │ 🧅 [Inner Layer] innerWrap 作用域            │ ║   │
  │ │ 1. 执行 innerWrap 前置逻辑                    │ ║   │
  │ │ 2. 调用 next() ══════════════════════╗       │ ║   │
  │ │                                     ║       │ ║   │
  │ │ ┌────────────────────────────────┐  ║       │ ║   │
  │ │ │ 🎯 [Core] 核心逻辑 _raw_compute │  ║       │ ║   │
  │ │ │ 计算: return x * 2;             │◀═╝       │ ║   │
  │ │ └────────────────────────────────┘          │ ║   │
  │ │                                             │ ║   │
  │ │ 3. 拿到核心计算结果 res                        │ ║   │
  │ │ 4. 执行 innerWrap 后置逻辑 (如修改结果)         │ ║   │
  │ │ 5. return 结果 ═══════════════════════════╗  │ ║   │
  │ └──────────────────────────────────────────╨───┘ ║  │
  │                                                  ║  │
  │ 3. 拿到内层返回结果 res   ◀═════════════════════════╝  │
  │ 4. 执行 outerWrap 后置逻辑 (如统计耗时、包装数据)         │
  │ 5. return 最终结果                                    │
  └──────────────────────────────────────────────────────┘
```

──────

##### 11.4.4.4 现场真实编译产物验证（以 test_dec_pass.cp 为例）

在当前代码库中，我们有一段测试多层装饰器的方法：

```ts
// 4. Onion Model: Outer and Inner wrappers
dec outerWrap(next) {
  let res = next();
  return "OUTER(" + res + ")";
}

dec innerWrap(next) {
  let res = next();
  return "INNER(" + res + ")";
}

@outerWrap
@innerWrap
fn computeName(id: string): string {
  return "item-" + id;
}
```

最终执行结果完全印证了洋葱从外到内、再从内到外的顺序：

calc.computeName: OUTER(INNER(item-99))

──────

##### 11.4.4.5 为什么 CP 的洋葱模型彻底碾压 Node.js / Python？

| 对比维度       | Node.js (Koa / Express)                            | Python (Flask / FastAPI)             | CP (Concurrent Plus)                                                                     |
| -------------- | -------------------------------------------------- | ------------------------------------ | ---------------------------------------------------------------------------------------- |
| 底层实现机制   | 运行时遍历中间件数组，每次递归生成闭包 dispatch(i) | 运行时高阶函数嵌套，生成多重闭包对象 | 编译期 AST 符号展开重写                                                                  |
| 内存与堆分配   | 每次请求/调用均分配闭包与 Context 堆内存           | 依赖 Python 动态词法作用域与堆对象   | 0 堆分配（Zero Heap Allocation）                                                         |
| GC 与性能开销  | 闭包链加重 V8 垃圾回收负担，产生 STW 停顿          | 函数栈调用层级深，解释执行慢         | 0 GC 负担，LLVM 原生直接跳转                                                             |
| 极致机器码优化 | 无法跨闭包消除冗余                                 | 无法静态优化                         | LLVM 自动触发函数内联（Inlining），多层装饰器在 Release 模式下直接被抹平成单段平铺汇编！ |

因此，在 CP 语言中，您可以放心地给几万个 Agent 方法堆叠 5~10 层装饰器（权限拦截、性能追踪、参数校验、缓存预热），完全不用担心任何性能衰减与内存泄漏！

---

#### 11.4.5 与元注解（`meta`）及反射（`reflect`）的共存

##### 11.4.5.1 装饰器与元数据天然共存

```ts
// 1. 定义元注解
meta Auth(role: string);

// 2. 定义结合了元数据的装饰器
dec checkAuth(next) {
  // 编译期内省：读取当前方法的 Auth 元数据
  let requiredRole = meta.get("Auth");
  if (currentUser.role != requiredRole) {
      panic("权限不足！");
  }
  return next();
}

// 3. 业务代码中优雅组合
class AdminService {
  @checkAuth
  @Auth("SUPER_ADMIN")
  fn deleteDatabase() {
      print("Database wiped!");
  }
}
```

##### 11.4.5.2 装饰器与元数据运行时反射

```ts
import { expect_eq } from "cap:test";
import { print } from "cap:io";

// 1. 定义元注解契约
@target(method)
meta Get(path: string = "");

// 2. 定义带参 AOP 装饰器
dec tag(next, prefix: string) {
  let res = next();
  return "[" + prefix + "] " + res;
}

// 3. 定义无参计时 AOP 装饰器
dec timer(next) {
  @print("[timer] 开始执行 listUsers");
  let res = next();
  @print("[timer] 执行结束");
  return res;
}

// 4. 业务类声明（装饰器与元数据天然共存）
@reflect
class UserService {
  @timer
  @tag("AUTH")
  @Get("/users")
  fn listUsers(): string {
      return "user_list_data";
  }
}

fn main() {
  let s = new UserService();

  // 【运行态验证】：触发洋葱切面
  let res = s.listUsers();
  @print("调用返回: {s}", res);
  expect_eq(res, "[AUTH] user_list_data");

  // 【运行时反射态验证】：拿到干净的 /users 路由，且内部 _raw_ 函数安全隐藏
  let info = reflect.type<UserService>();
  let m = info.getMethod("listUsers");
  expect_eq(m != null, true);
  expect_eq(m.meta["Get"], "/users");
  @print("反射内省成功: Route = {?}", m.meta["Get"]);
}
```

- `@timer` 与 `@tag` 由编译器识别为切面逻辑并合成为包装函数；
- `@Get("/users")` 保持为元数据进入反射字典；
- `@reflect` 激活该类的运行时静态反射单例表，反射内省仅暴露公共接口 `listUsers`，内部合成的 `_raw_listUsers` 自动安全隐藏。

---

#### 11.4.6 弱类型/即席元标注（Ad-hoc Metadata：`@meta(...)`）

除了通过 `meta Name(...)` 预定义强类型元注解模式外，CP 语言原生提供了免预定义的**弱类型/即席元标注**原语 `@meta(...)`。注解的参数传递统一严格遵循函数式括号语法 `(...)`：

```cp
// 1. 纯编译期弱类型元标注（零运行时开销，不产生任何运行时反射单例表）
@meta(author: "Antigravity", timeout: 3000, version: "2.1")
class PaymentAgent {
    // 2. 字段级别弱类型标注
    @meta(column: "tx_hash", sensitive: true)
    txId: string;

    // 3. 方法级别弱类型标注
    @meta(role: "admin", audit: true)
    fn processPayment(): string {
        return "SUCCESS";
    }
}

// 4. 若需要在运行期内省该元数据，搭配 @reflect 显式授权激活（推荐使用 @[...] 批量收拢）：
@[reflect, meta(author: "Antigravity", timeout: 5000)]
class OrderService {}
```

##### 11.4.6.1 核心特性与零成本正交设计

- **免模式声明**：无需事先声明 `meta ...` 契约，直接以键值对注入编译期元数据字典，供宏（Macro）、切面装饰器（Dec）与静态分析工具读取；
- **严格零成本抽象（Zero-Cost Abstraction）**：
  - `@meta(...)` 仅负责静态附加元数据，**绝不隐式自动生成运行时反射字典**；
  - 避免无谓生成 `TypeInfo` 导致的机器码体积膨胀（保障 `cap build -s` 极致轻量产物与 Tree-Shaking 剪枝能力）；
  - 运行时反射由 `@reflect` 统一显式授权。若在未标注 `@reflect` 的类上调用 `reflect.type<T>()`，编译器会直接拦截并友好提示：
    `Compile Error: Type 'T' cannot be reflected at runtime. Add '@reflect' to the class definition to enable runtime reflection`；
- **双重内省读取路径**（在 `@reflect` 激活后）：
  1. **扁平直读**：`info.meta["author"]`、`info.meta["timeout"]` 直接返回对应属性值；
  2. **命名空间读取**：`info.meta["meta"]["author"]` 支持隔离命名空间访问；
- **全目标支持**：支持标注于类（`class`/`struct`）、字段（`field`）、方法（`method`）与参数（`param`）。

---

#### 11.4.7 批量标签收拢语法（`@[...]`）

在 CP 语言的设计哲学中，`[]` 代表**集合容器**，而非单个标注的参数语法。为了避免在类、字段或方法前堆叠冗长的 `@` 标记，CP 语言提供了高颜值的批量收拢语法 `@[...]`，用单个方括号容器一口气包裹收拢多个元注解、切面装饰器与即席标注：

```cp
dec checkAuth(next) {
    return next();
}

meta Author(name: string);
meta Timeout(ms: i32);
meta Singleton;
meta Column(name: string);
meta JsonIgnore;
meta Get(path: string);

// 1. 类级别批量收拢：一行搞定运行时反射激活 + 单例 + 自定义注解 + 即席元数据
@[reflect, Singleton, Author("Antigravity"), Timeout(3000), meta(version: "1.0")]
class UserService {
    // 2. 字段级别批量收拢
    @[JsonIgnore, Column("user_pwd")]
    secret: string;

    // 3. 方法级别批量收拢：切面装饰器与元注解天然共存
    @[checkAuth, Get("/users")]
    fn listUsers(): string {
        return "all_users";
    }
}
```

##### 11.4.7.1 语法规则与设计哲学

1. **容器与调用语法正交解耦**：
   - `@[ ... ]` 是**集合收拢容器**，用于收拢多个标签；
   - 集合内的具体标注自身（如 `meta(...)`, `Author(...)`, `Column(...)`）严格采用小括号 `(...)` 传参，语义清晰统一；
2. **零成本与高内聚**：单行 `@[reflect, meta(...)]` 既完全符合“显式开启反射”的系统语言严谨性，又彻底消除了多行堆叠的视觉繁琐；
3. **多态混排**：在同一个 `@[...]` 容器内，编译器智能自动识别：
   - 匹配 `dec` 声明的标识符 -> 编译期织入洋葱切面逻辑（AOP Wrapper）；
   - 匹配 `meta` 或即席声明的标识符 -> 录入元数据字典；
   - 包含 `reflect` -> 编译期为该类生成原生运行时单例反射表；
4. **语法容错**：容器内支持任意排列，末尾逗号可选，可写无参（`Singleton`）、位置实参（`Column("user_pwd")`）或命名实参（`meta(version: "1.0")`）。

5. **统一整洁**：单个 `@[...]` 收拢替代多行分散的 `@Tag`，代码结构更加紧凑；
6. **多态混排**：在同一个 `@[...]` 内，编译器智能自动识别：
   - 匹配 `dec` 声明的标识符 -> 编译期织入洋葱切面逻辑（AOP Wrapper）；
   - 匹配 `meta` 或即席声明的标识符 -> 编译期零开销录入反射字典；
7. **参数语法自由**：方括号内的标签可无参（`Singleton`）、位置实参（`Column("user_pwd")`）或命名实参（`meta(version: "1.0")`），末尾逗号可选容错。

---

### 11.5 运行时与编译期反射系统

#### 11.5.1 反射示例

在 CP 语言中，类型反射已经作为一等公民完全落地（见测试 test_reflect_pass.cp）：

```typescript
import { reflect } from "cap:reflect";

let info = reflect.type<UserService>();
@print("类名:", info.name); // "UserService"
@print("是否为 ref 类:", info.isRef); // false

let user = new UserService();
let info = reflect.of(user); // 动态内省实例的类型
@print("实例所属类型:", info.name); // "UserService"

for (f of info.fields) {
    print("Field:", f.name, "Type:", f.type);
}
```

- 类型视角（静态泛型内省）：
  在编译期或运行时任意逻辑中，通过类型名获取该类的静态单例元数据表：
  let info = reflect.type<UserService>();
  print("类名:", info.name); // "UserService"
  print("是否为 ref 类:", info.isRef); // false

- 实例视角（动态对象内省）：
  当你拿到一个具体对象实例变量时，直接反查它的类型元数据：
  let user = new UserService();
  let info = reflect.of(user); // 动态内省实例的类型
  print("实例所属类型:", info.name); // "UserService"

拿到 TypeInfo 结构体后，在运行时可以完整内省：

- info.fields：所有字段的名称（name）、类型名（typeName）以及字段上的元注解（meta）；
- info.methods：所有方法的名称、返回值类型、形参列表、是否 async、是否 static，以及方法上的元注解（meta）；
- info.meta：类自身挂载的所有元注解（如 @Controller、@Singleton 等）。

注意：cp中反射系统分为编译期和运行期，默认只在编译期有效，运行期抹除类型信息，除非类型前面加上`@reflect`标注。

“编译期内省（Compile-Time Introspection）是零成本的，不需要 @reflect；
只有将元数据打包存活到产物运行态（Runtime Reflection），才需要 @reflect。”

#### 11.5.2 为什么必须加 @reflect 才能反射？背后的架构哲学

为什么 CP 不学 Java、Python 或 Go 那样“默认所有类都无条件支持反射”？
这正是系统级语言中极其关键的 零开销抽象与按需付费哲学（Zero-Cost Abstraction & Pay-As-You-Go）：

##### 1. 痛点：Java / Go / Python 的全量反射灾难

- 在 Java 或 Go 中，编译器会盲目地为每一个结构体、类、字段、方法名强制生成全套符号和元数据字符串并硬编码打包进二进制产物中；
- 恶果：
  - 二进制产物体积瞬间膨胀到几十 MB；
  - 启动时占用大量常驻元数据内存堆；
  - 极易被逆向工程轻易还原出全部业务类与内部私有字段名；
  - 即使你的业务代码中 99% 的类根本不需要反射，你也必须为那 1% 的反射功能买单！

##### 2. CP 语言的终极解法：精准白名单（Explicit Opt-in）

- 未标注 @reflect 的普通类：
  编译为 Native 机器码时，类型被最大程度擦除与内联优化，内存布局是极致紧凑的纯裸数据对齐，不产生哪怕 1 个字节的冗余元数据表，可执行文件体积仅数
  KB，极其适合边缘设备与高频计算；
- 标注了 @reflect 的类：
  编译器才会精确定制生成一套专属的懒加载静态单例表（_get_typeinfo_ClassName()），单例在首次使用时初始化并全局复用。

总结：

- 不打 @reflect：编译器完全有能力拿到全部类型信息，但这些信息被且仅被用于编译期代码分析、借用检查、宏展开、AOP 装饰器合成与 Comptime计算，随后被“抹平销毁”，换取极致的机器码性能和仅数 MB 的轻量二进制产物；
- 打上 @reflect：明确指示编译器“请把这个类的元数据清单打包进最终的可执行文件”，从而赋予程序在运行时动态获取类名、字段、方法和元注解的高级内省能力。需要反射的业务（如 Web 路由/ORM）精准享受完整内省, 绝不把性能税强加给无辜的裸机业务！

这正是 CP 既能拥有 TS 的优雅高级表现力，又能达到 Zig/Rust 裸机无 GC 极致性能 的核心魔法所在！

---

#### 11.5.3 编译期与运行时的“双轨内省体系”

在 CP 语言中，类型信息的获取被严格划分在两个完全不同的时空维度：

```text
    ┌───────────────────────────────────────────────────────────────────────────────────┐
    │  一、编译期世界 (Compile-Time) —— 纯 AST 与符号表内省                                  │
    │  【无需 @reflect】零成本、零二进制体积膨胀、编译后全量抹平销毁                             │
    ├───────────────────────────────────────────────────────────────────────────────────┤
    │  • 宏代码生成 (@Json, @Table, 自定义 macro)  ────▶ 遍历类的字段，自动合成序列化代码       │
    │  • AOP 装饰器 (dec 展开)                    ────▶ 读取方法形参和返回值，自动织入切面     │
    │  • Comptime 编译期块 (!{ ... })              ────▶ 编译期常量折叠、类型特化与代码生成    │
    │  • 自动 Drop Flag 内存释放机制              ────▶ 编译期 @typeInfo 探测 release() 方法 │
    │  • 语义与借用检查 (Sema, Borrow Checker)    ────▶ 严格别名排他、NLL 生命周期安全检查      │
    └────────────────────────────────────────────────────────────────────────────────────┘
                                              │
                                编译完成，生成可执行文件
                                              ▼
    ┌──────────────────────────────────────────────────────────────────────────────────────┐
    │  二、运行期世界 (Runtime) —— 动态单例字典 (TypeInfo)                                     │
    │  【必须显式标注 @reflect】按需付费 (Pay-as-you-go)                                       │
    ├──────────────────────────────────────────────────────────────────────────────────────┤
    │  • reflect.type<T>() / reflect.of(val)      ────▶ 生产环境动态检索类名/字段/路由元数据     │
    │  • 动态插件热插拔与 IOC 依赖注入容器        ────▶ 运行时按需解析构造参数与生命周期             │
    └──────────────────────────────────────────────────────────────────────────────────────┘
```

──────

#### 11.5.4 横向对比：为什么现代语言（Rust / Zig / CP）都推崇这种设计？

| 语言与机制 | 编译期内省（无需开启标志）                                       | 运行期动态反射（通常需显式启用）                             |
| ---------- | ---------------------------------------------------------------- | ------------------------------------------------------------ |
| Rust       | derive(Serialize)、macro_rules!、proc_macro（强大，编译后擦除）  | 默认无运行时反射（仅有极其受限的 Any / TypeId）              |
| Zig        | @typeInfo(T)、@hasDecl、comptime（编译期暴力内省，0 运行时体积） | 官方不支持全量运行时反射，完全靠编译期泛型生成               |
| C#         | Source Generators（C# 9+ 引入的编译期代码生成，极速推荐）        | System.Reflection（全量运行时反射，体积大、速度慢）          |
| CP 语言    | 编译期宏 (macro)、AOP 装饰器 (dec)、Comptime Drop（零开销）      | 精细化按需启用 (@reflect + reflect.type<T>() / reflect.of()) |

---

#### 11.5.5 不标注 @reflect 时，编译期类型信息的典型应用场景

在没有标注 @reflect 时，编译器前端（cpc/）自身在内存中已经掌握了整个程序完整的抽象语法树（AST）和符号表。这些信息被广泛应用于以下场景：

##### 1. 宏代码生成与自动派生（Macro & Derive）

在 CP 语言中，类上即使不打 @reflect，也可以直接挂载宏（如 @Json、@Table、@Clone）：

```ts
// 并没有 @reflect，运行时没有反射表！
@Json
class OrderItem {
  id: i32;
  price: f64;
  title: string;
}
```

• 编译器内省过程（见 sema.rs:512）：
• 编译器在编译阶段读取 OrderItem 的结构定义，发现它有 id、price、title 三个字段及其类型；
• 宏处理器直接在编译期自动合成了 .toJson() 和 .fromJson() 的原生代码；
• 结果：生成的序列化代码和手写的一样快（硬编码高效存取），既实现了自动化，又完全不需要在运行时维护任何反射字典！
──────

##### 2. 编译期 AOP 装饰器感知（Decorators）

当您为一个方法编写装饰器时，同样不需要 @reflect：

```ts
    dec validateParams(next) {
        // 编译期内省：编译器知道目标方法有几个参数、分别叫什么名字
        let res = next();
        return res;
    }

    class PaymentService {
        @validateParams
        fn pay(account: string, amount: f64): bool {
            return true;
        }
    }
```

• 编译器内省过程：
• 编译器在重构 pay 方法时，自动通过符号表提取出 ["account", "amount"] 形参，并生成参数透传代码 self._raw_pay(account, amount)；
• 整个过程在编译期 AST 层面完成消费，编译后生成的机器码就是几个嵌套跳转，不残留任何元数据包袱。

──────

##### 3. 自动内存释放（Comptime Auto-Drop 机制）

这是我们在编译器中实现的最具代表性的零成本编译期内省。
当您在 CP 中写下：

```ts
    const t = new SharedTensor()?;
```

编译器在底层 Zig 状态机中自动注入了编译期反射代码：

```ts
defer if (!__t_moved) {
  const T = @TypeOf(t);
  // ⬅️ 这里完全是编译期内省 (Comptime Introspection)，零运行时开销！
  if (@typeInfo(T) == .pointer) {
    const Child = @typeInfo(T).pointer.child;
    if (@typeInfo(Child) == .@"struct" and @hasDecl(Child, "release")) {
      t.release(); // 探测到拥有 release 方法，自动在函数退出时执行！
    }
  }
};
```

价值：如果 t 是个普通整数或不需要释放的结构体，LLVM 在编译期就把这个 defer 整个裁掉（0 开销）；如果是大模型张量，自动精准释放。这个过程 100%
依赖类型的元数据探测，但对运行时性能完全是 0 损耗。
──────

##### 4. 严格编译期安全（借用检查与所有权分析）

编译器在语义分析阶段（Sema）：

- 检查哪个变量被 move 了（所有权转移）；
- 检查是否存在同时存在的 &（只读借用）与 *（可变借用）；
- 检查方法接收者是值接收、借用接收还是指针接收；
- 这些全都是编译期的深度类型内省，在代码生成完成的那一刻使命就已达成，不需要存留到运行期。
  ──────

### 11.6 底层控制与 `unsafe` 安全边界

CP 拥有直通硬件底层的裸指针与内存控制能力，所有非安全操作严格受限于 `unsafe` 隔离块（内部依然使用 CP 语法）：

```typescript
unsafe {
    // 裸指针声明与算术
    let raw_addr: *i32 = cp_cast_ptr(0x7FFF0000);
    let second_item: *i32 = raw_addr + 1;

    // 裸指针解引用写入
    second_item.* = 42;
}

pub unsafe fn direct_io_write(port: u16, val: u8) {
    // 裸硬件端口操作
}
```

### 11.7 Native Zig 逃逸舱（`zig { ... }`）

当需要与底层转译目标（Zig）原生交互、引入宿主项目外部命名空间或调用 Comptime 机制时，CP 提供无阻碍的 `zig { ... }` 逃逸舱：

- **顶层使用**：直接导入宿主全局命名空间（如 Bun.js 内部的 `@import("root").bun`）；
- **函数体内使用**：直接嵌入 Zig 原生高性能代码块，编译器原样 verbatim 透传；
- **兼容性**：保留对旧版 `__zig__ { ... }` 的兼容。

```typescript
// 1. 顶层穿透引入宿主头文件与定义
zig {
    const bun = @import("root").bun;
    const std = @import("std");
    const Allocator = std.mem.Allocator;
}

export class NativeBuffer {
    ptr: *any!,
    len: usize,

    fn init(*this, size: usize): void {
        this.len = size;
        // 2. 函数体内嵌入原生 Zig 操作
        zig {
            const buf = std.heap.page_allocator.alloc(u8, @intCast(size)) catch unreachable;
            self.ptr = @ptrCast(buf.ptr);
        }
    }
}
```

---

### 11.8 零成本转译抽象：按需摇树注入与原生发射

为了保证 CP 代码编译为 Zig 时的极致紧凑性与零额外运行时惩罚，CP 编译器在代码生成阶段全面实装了**零成本抽象转译引擎**：

#### 11.8.1 模块级精细化摇树按需注入 (Module Features Tree-Shaking)

编译器对每个 CP 模块进行前置静态 AST 特征扫描（`ModuleFeatures`）：

- **全域按需注入**：仅当模块显式使用了对应语法或特性时，才发射对应的运行时辅助代码。
- **纯计算模块零死代码**：若模块未引入 Actor、集合容器、反射元数据或正则，编译器彻底切除全套 `PID`、`Message`、`Map/Dict/Set`、`TypeInfo` 等运行时结构，杜绝数千行死代码膨胀。
- **示例收益**：如 `wyhash` 纯哈希模块转译代码由初始 2,401 行剧降至 256 行，极度贴近手写原生 Zig。

#### 11.8.2 标量与无析构类型 Drop Guard 彻底豁免

传统所有权语言在生成中间代码时往往会为所有局部变量添加守卫标记（Drop Flag）。CP 编译器通过编译期内省实施严格的豁免规则：

- **纯标量与原始类型**：`u8`~`u128`、`i8`~`i128`、`f32`/`f64`、`bool`、原生切片 `[]T`、原生裸指针直接声明为标准 Zig 局部变量，零守卫开销；
- **POD 结构体**：凡未显式声明 `release(*this)` 析构方法的普通结构体（如 `WyhashStateless`），完全免除 `_moved` 标志与 `defer release()` 注入；
- **独占资源精准守护**：仅对实现了 `release(*this)` 的资源类型、动态容器与胖指针自动生成安全的析构守卫。

#### 11.8.3 原生 Zig Switch 模式发射与多模式联合展开

对于字面量匹配分支及多模式联合（如 `'|' | '\\' | '{' =>`），编译器自动跳过中间嵌套块作用域，直接发射为原生紧凑的 Zig 语句：

```zig
switch (c) {
    '|', '\\', '{' => { ... },
    '-' => { ... },
    else => { ... },
}
```

完全避免标签化跳转块（`__sw_blk_*`），显著提升 Zig 编译器的增量构建速度与内联优化效率。

#### 11.8.4 原生空值比较（Null Comparison）与零成本指针/可选类型解包

当比较操作符两端任意一侧为 `null` 字面量时（如 `if (i_opt == null)`），编译器自动生成零开销的原生 Zig 条件判断 `(i_opt == null)`，消除对通用 `cp_eq` 运行时的调用；同时运行时 `cp_eq` 具备对 `@typeInfo(T) == .null` 与 `.optional` 的编译期等价性解包支持，确保多态等价性判断 100% 严谨安全。

#### 11.8.5 点路径显式错误联合与原生返回声明

在对接外部或底层系统（如 Bun 原生 Writer）时，支持声明显式点路径错误类型：

```typescript
pub fn escapeRegExp(input: string, writer: *std.Io.Writer): void ! std.Io.Writer.Error {
    ...
}
```

编译器自动将其对齐为 Zig 原生错误联合 `std.Io.Writer.Error!void`，并对 `void` 结果发射干净的 `return;`，消除多余的结构体包装开销。

#### 11.8.6 顶层函数前向引用与声明顺序解耦 (Top-Level Function Forward References)

在 CP 模块中，顶层函数支持声明顺序无关的前向引用调用（Forward References）：

- **多阶段语义扫描（Multi-Pass Sema）**：语义分析器在 Pass 1 预注册当前模块所有顶层函数的签名（`Decl::Func`）至符号表；
- **自然调用顺序**：允许顶层函数自由调用在文件下方声明的私有或辅助函数（如 `encode` 自由调用下方定义的 `encodeSlowPath`），消除传统 C 语言的前向声明负担；
- **相互递归友好**：天然支持顶层函数之间的相互递归与循环依赖调用。

#### 11.8.7 原生逃逸舱符号预扫与双向无缝集成 (Native Escape Hatch Symbol Pre-Scanning)

为了让 CP 逻辑与外部 Zig 上下文实现零成本互操作：

- **符号预检与作用域注入**：编译器在语义分析阶段自动预扫描模块顶层 `zig { ... }` 块内声明的符号（`const`, `var`, `fn`），并将标识符自动注入 CP 根作用域（`scopes[0]`）；
- **查找表与常量无感复用**：CP 原生算法可直接按名访问 `zig { ... }` 中声明的编译期查找表（如 `@setEvalBranchQuota` 构建的 `vlq_lookup_table` 或 `base64_lut`），无需重复映射或通过繁琐的外部包装访问。

#### 11.8.8 根结构体（`@root struct`）与原生 `@This()` 映射规范

在重构类似 Zig 独有的“文件即结构体”（`@This()`）底层模块时，CP 语言提供了优雅的 `@root` 装饰器机制：

- **纯正结构体建模**：使用 `export @root struct Name { ... }` 编写纯正的 CP 结构体定义，支持字段声明、默认值赋初值（如 `len: u8 = 0`）与 `@inline` 实例方法；
- **原生文件级结构体展开**：CP 编译器遇到 `@root` 结构体时，自动将其映射发射为 Zig 文件顶层根作用域：
  ```zig
  pub const Name = @This();
  bytes: [7]u8 = undefined,
  len: u8 = 0,
  pub inline fn slice(self: *const Name) []const u8 { ... }
  ```
  外部调用方直接做 `const Name = @import("./Name.zig");` 时，该模块既具备完整的实例类型定义（`[_]Name{ ... }`），又天然导出全部模块级函数与常量，彻底根除将结构体硬塞进 `zig { ... }` 逃逸舱的陈旧做法；
- **Prelude 运行时标识符隔离**：转译器内置的运行时辅助例程（Prelude helpers）内部变量命名统一采用专有前缀（如 `cp_str_slice`、`cp_slice`），严密隔离与用户公开方法（如 `pub inline fn slice(...)`）的标识符碰撞，杜绝 Zig 局部作用域遮蔽（Identifier Shadowing）报错。

#### 11.8.9 紧凑值结构体（`@value struct`）与零开销值传递语义

在底层高性能系统开发中，诸如句柄、短索引、紧凑枚举包装等小型 POD 结构体（如 `Compact { value: u14 }`）通常需要像原生标量一样按值传递，避免指针解引用的间接开销，并支持在临时右值上直接调用成员方法（例如 `Compact.from(entry).toMimeType()`）。

CP 提供了 `@value` 装饰器：

```typescript
export @value struct Compact {
    value: Table,

    static fn from(value: Table): Compact {
        return Compact { value: value };
    }

    pub fn toMimeType(this): MimeType {
        // ...
    }
}
```

- **值接收者发射**：编译器将成员方法的 `this` 接收者直接转译为值拷贝 `self: Compact`，而非指针借用 `self: *const Compact`；
- **免除多余析构与移动守卫**：`@value` 结构体天然免疫析构守卫代码生成，零开销、零冗余胶水；
- **支持右值链式调用**：在转译至 Zig 目标代码时，调用临时对象方法不会触发“taking pointer of rvalue”的 Zig 编译报错。

#### 11.8.10 原生二元比较（`==` / `!=`）与枚举/布尔/数值直接发射

在早期编译模式中，二元比较易误触发泛型比较器 `cp_eq`，引入数十行字符串与反射辅助模板。现代 CP 编译器内置智能二元分流器：

1. **数值/指针/字面量原生比较**：任一操作数为整型、浮点型、布尔型或 `null` 时，直接发射 `(a == b)` 或 `(a != b)`；
2. **枚举变体原生比较**：当操作数涉及无载荷枚举变体（如 `.text`、`.image`）或枚举类型时，直接发射原生 Zig `(self.category == .text)`，不引入任何辅助模板；
3. **摇树按需注入**：仅在两个动态字符串或未知复杂对象等值比较时，才精确激活 `needs_equality` 运行时支持。

#### 11.8.11 顶层 `string` 原生别名与跨语言零摩擦交互

CP 编译器在每个库模块头部自动注入 `const string = []const u8;`。无论是纯 CP 类型推导、还是内嵌 `zig { ... }` 块中复用的宿主代码（如 Bun 内部普遍使用的 `string`），均能天然无缝解析，彻底消除未声明标识符报错。

#### 11.8.12 函数调用约定装饰器（`@callconv`）与 C-ABI 回调兼容

在对接底层 C 库、OS 系统调用或第三方 C 动态库回调（如 BoringSSL, libuv, lsquic, JSC 回调函数）时，回调函数通常必须严格遵循 C 语言调用约定（`callconv(.c)`）。

CP 语言提供了 `@callconv` 函数级装饰器：

```typescript
@callconv("c")
pub fn onHskDone(qs: *quic.Socket, ok: c_int): void {
    // ...
}
```

- **语义**：修饰函数在转译为 Zig 时自动生成 `callconv(.c)` 函数调用约定修饰；
- **支持格式**：`@callconv("c")`、`@callconv(".c")` 或 `@callconv(c)`；
- **生成的 Zig 产物**：
  ```zig
  pub fn onHskDone(qs: *quic.Socket, ok: c_int) callconv(.c) void {
      // ...
  }
  ```
- **类/结构体内部的静态 C 回调方法**：
  若在 `class` 或 `struct` 内部定义需要传递给底层 C 运行时的回调函数，必须附加 `static` 修饰符，防止编译器自动注入实例 `self` 接收者：
  ```typescript
  export class GarbageCollectionController {
      // 静态 C-ABI 回调方法，无 self 隐式参数
      @callconv("c")
      pub static fn onGCTimer(timer: *uws.Timer): void {
          var this = timer.as(*GarbageCollectionController);
          // 处理定时器逻辑
      }
  }
  ```
  生成的 Zig 代码：
  ```zig
  pub const GarbageCollectionController = struct {
      pub fn onGCTimer(timer: *uws.Timer) callconv(.c) void {
          // ...
      }
  };
  ```

- **消除逃逸**：无需为了实现 C 回调而将整个回调函数被迫写进 `zig { ... }` 块中，使底层协议回调完全可以用 CP 现代语法书写。

#### 11.8.13 不透明类装饰器（`@opaque class`）与底层系统句柄引用语义

在对接操作系统底层资源、第三方 C/C++ 不透明句柄或宿主运行时内核对象（如 uWebSockets 的 `App`、`Loop`、`us_socket_t`、`ListenSocket`）时，CP 语言提供了专有的 `@opaque` 类级装饰器：

```typescript
export @opaque class App {
    static fn create(ssl: bool, config: SocketContextOptions): App? {
        // ...
    }
}
```

- **语义与映射**：修饰类在转译为 Zig 时直接发射为 `pub const App = opaque { ... };`；
- **引用语义自动适配**：`@opaque class` 本质上代表指针引用类型。编译器代码生成器自动豁免实例构造器 `_create` 堆分配逻辑，并在类型标注为 `App` 时生成 `*App`，返回可选类型 `App?` 时生成 `?*App`，彻底杜绝指针双重取址（`?**App`）；
- **标准库格式化兼容**：原生适配 Zig 0.15 泛型 Writer 的 `{p}` 指针格式化，消除对不透明类型解引用的编译错误。

#### 11.8.14 私有哈希前缀标识符（`#identifier`）与泛型参数多字符全大写规范

在系统级面向对象与网络协议状态机重构中，兼顾 ECMAScript 现代语法与复杂泛型推导：

1. **私有哈希标识符（`#identifier`）**：
   - 词法解析器原生支持 `#` 前缀标识符（如 `#ext: *anyopaque`）；
   - 支持类内部声明硬隔离私有字段，彻底杜绝与外部文件作用域同名符号或模块级函数的冲突。
2. **泛型类型参数多字符全大写识别规范**：
   - 在泛型类、方法或函数中，凡是全由 ASCII 大写字母组成的标识符（如 `T`、`UD`、`CTX`、`SOCKET` 等任意长度大写符号），代码生成器智能将其判定为泛型类型占位符；
   - 发射时保留原始符号名（如 `ud: UD`），杜绝误将其当成普通对象类型而强制追加指针前缀（如错误生成 `ud: *UD`），确保跨层泛型特化与参数传递 100% 精准保真。

---

### 11.9 C 语言互操作与现代系统类型映射规范 (C-ABI Interoperability & System Type Mappings)

在系统级高性能网络栈（如 Bun、uWebSockets、lsquic、BoringSSL 等）重构中，与底层 C 库、OS 系统调用的互操作至关重要。CP 语言确立了兼具“TypeScript 的极简直觉”与“底层 C-ABI 绝对保真”的互操作规范体系。

#### 11.9.1 `@c { ... }` 原生 C 符号与头文件直接吸纳（零样板代码 C-FFI）

为彻底消减传统系统编程中与 C 语言交互时沉重的胶水代码负担，CP 废弃了繁琐且易出错的手写外部声明，统一采用极简的 `@c { ... }` 块完成外部 C 库与系统头文件的原生吸纳：

##### 1. 新旧开发模式革命性对比（体现体系级方便性）

| 维度 | 传统手写绑定（旧模式） | CP 原生 `@c { ... }` 头文件吸纳（新模式） |
| :--- | :--- | :--- |
| **胶水代码量** | 每个 C 函数需逐一手写签名，几十行乃至上千行样板代码 | **仅需 1 行头文件名声明**（如 `"stdio.h"`），零样板代码 |
| **类型对齐风险** | 易因 C 指针级别、结构体 padding 误判导致段错误崩溃 | **Clang/Zig 编译器底层自动解析 ABI 与类型**，100% 内存安全对齐 |
| **维护与升级** | C 库更新头文件后需手动同步修改所有绑定函数声明 | **头文件升级时自动无缝吸收**，编译期强校验，无需人工修改胶水 |
| **命名空间隔离** | 污染全局符号表，容易引发命名遮蔽冲突 | **自动挂载在整洁的 `c` 全局安全命名空间下**（如 `c.puts`） |

##### 2. 核心语法与使用示例

###### 场景 A：直接吸纳标准库或第三方 C 库头文件
无需手写任何 C 函数声明，直接在 `@c { ... }` 块中写入头文件字符串字面量（如 `"stdio.h"`, `"sqlite3.h"`）。编译器自动解析并注入统一的 `c` 全局命名空间：

```typescript
import { print } from "cap:io";

// 一行代码直通 C 标准库与 SQLite 引擎
@c {
    "stdio.h",
    "sqlite3.h"
}

pub fn main(): void {
    // 1. 直接调用 C 标准 I/O 函数，零粘合代码
    _ = c.puts("Hello from native C stdio via @c header!");

    // 2. 直接调用复杂 C 外部库函数
    let db: *any! = null;
    let rc = c.sqlite3_open(":memory:", &db);
    if (rc == 0) {
        print("SQLite memory database initialized successfully via C-FFI!\n");
        _ = c.sqlite3_close(db);
    }
}
```

###### 场景 B：头文件与自定义 C 函数原型混合声明
同一个 `@c { ... }` 块中支持头文件引入与自定义函数声明混写，满足各类复杂底层的精细控制需求：

```typescript
@c {
    "uv.h",
    // 针对特殊私有垫片函数进行就地签名声明：
    fn custom_native_helper(fd: i32, flags: u32): i32;
}

pub fn start_native_io(fd: i32): void {
    _ = custom_native_helper(fd, 0);
}
```

##### 3. 转译与底层保障
编译器在代码生成阶段，自动为吸纳的头文件生成安全的 C-ABI 映射结构：
```zig
pub const c = struct {
    pub extern "c" fn puts(s: [*c]const u8) c_int;
    pub extern "c" fn printf(fmt: [*c]const u8, ...) c_int;
    pub extern "c" fn malloc(size: usize) ?*anyopaque;
    pub extern "c" fn free(ptr: ?*anyopaque) void;
    // ... 自动挂载头文件包含的完整外部函数集
};
```

#### 11.9.2 `@c` 单行导出与 C 调用约定修饰

当需要在 CP 模块中导出供 C 语言宿主反向调用的函数（如 Trampoline 蹦床跳板、系统回调）时，使用 `@c` 前缀修饰符：

```typescript
@c pub fn bun_vtable_on_open_trampoline(s: *any!) void {
    // 蹦床跳板实现
}
```

- **语义**：`@c` 替代冗长的 `@callconv("c") export`，直观易读且与 `@c { ... }` 块保持完全对称的设计美学。

#### 11.9.3 C-ABI 函数指针类型与可选回调 (`@c ((params): Ret)?`)

在定义系统级回调（如异步 I/O 监听、协议事件分发）时，结构体字段或函数参数常常需要接收具备 C 调用约定的函数指针。

CP 允许在类型位置直接声明 `@c ((args): Ret)?`：

```typescript
export class VTable {
    on_open: @c ((s: *any!, is_client: i32, ip: cstr, ip_len: i32): *any!)? = null,
    on_data: @c ((s: *any!, data: [*]u8!, len: usize): void)? = null,
    on_writable: @c ((s: *any!): void)? = null,
    on_close: @c ((s: *any!, code: i32, reason: *any?!): void)? = null,
}
```

- **代码生成**：生成标准的 Zig C 调用约定函数指针类型：
  `?*const fn(*anyopaque, [*]u8, usize) callconv(.c) void`。

#### 11.9.4 C 语言严格系统指针映射规范

为了根除 C 语言隐式类型转换与宽松指针带来的内存越界与悬垂野指针隐患，CP 确立了清晰严谨的专用指针体系：

| CP 原生系统指针        | 对应 Zig 底层类型                         | 语义与安全保证                                                                                                      | 核心应用场景                                        |
| :--------------------- | :---------------------------------------- | :------------------------------------------------------------------------------------------------------------------ | :-------------------------------------------------- |
| **`cstr` / `cstr?`**   | `[*:0]const u8` (const) / `[*:0]u8` (let) | **0 结尾 C 字符串**。可变性由 `let`（可变内容 `[*:0]u8`）与 `const`（不可变内容 `[*:0]const u8`）严格区分。         | 传递以 null 结尾的文件路径、主机名、环境变量。      |
| **`[*]T!` / `[*]T?!`** | `[*]T` / `?[*]T`                          | **未定长多项指针 (Many-Item Pointer)**。表示连续内存缓冲，支持切片与下标算术，提供非空与可空（`?`）双轨。           | 接收底层的 raw packet payload、IOVEC 缓冲区。       |
| **`*any!` / `*any?!`** | `*anyopaque` / `?*anyopaque`              | **通用非类型化系统裸指针 (Universal Raw Pointer)**。对齐 C 语言 `void*`，彻底废除 `anyopaque` 冗余关键字。          | 跨语言不透明句柄、用户自定义指针 `user_data`。      |
| **`*T!` / `*T?!`**     | `*T` / `?*T`                              | **单对象强类型裸指针**。在 unsafe 作用域下直接映射为 C-ABI 单对象指针，**严格禁止下标操作（如 `p[i]` 编译报错）**。 | 指向具体的单一 C 结构体实例（如 `*quic.Socket!`）。 |

> [!CAUTION]
> **绝对杜绝宽松指针 `[*c]` 与单对象越界下标**：
>
> 1. CP 坚决不引入 C 语言的 `[*c]` 宽松可空不确定指针。所有的可空性必须通过 `?` 显式显化，所有的数组遍历必须通过 `[*]` 显式显化，禁止任何隐式可空与隐式降级转换！
> 2. `*T!` 仅代表单个对象的裸指针，不支持下标访问 `p[i]`；若需要进行连续内存数组访问，必须明确使用 `[*]T!` 或切片 `[]T`。

#### 11.9.5 常量泛型类（Const Generic Classes）与定长张量/定长缓冲区

在 AI 向量计算、网络定长数据包与底层系统设计中，定长张量与定长缓冲区通常需要在编译期确定物理尺寸，以实现零堆内存分配（Zero Heap Allocation）与零开销单态化展开。

CP 完整支持**常量泛型（Const Generics）**语法，支持标量常量与数组维度常量参数：

```typescript
// 1. 定长定容网络缓冲区
pub class Buffer<const N: usize> {
    pub data: [u8; N],
    pub len: usize,

    pub fn new(): Buffer<N> {
        let b: Buffer<N> = undefined;
        b.len = 0;
        return b;
    }

    pub fn capacity(): usize {
        return N;
    }
}

// 2. 编译期多维张量定义（支持多常量参数与类型参数）
pub class Tensor<T, const shape: [usize; 2]> {
    pub rows: usize = shape[0],
    pub cols: usize = shape[1],
    pub data: [T; shape[0] * shape[1]],

    pub fn init(): Tensor<T, shape> {
        let t: Tensor<T, shape> = undefined;
        t.rows = shape[0];
        t.cols = shape[1];
        return t;
    }

    pub fn total_elements(): usize {
        return shape[0] * shape[1];
    }
}

// 3. 类型别名与静态维度单态化
type Embedding = Tensor<f32, [1, 4096]>;
type PacketBuffer = Buffer<1024>;

pub fn test(): void {
    let emb = Embedding.init();
    assert(emb.rows == 1);
    assert(emb.cols == 4096);
    assert(Embedding.rows == 1);
    assert(Embedding.cols == 4096);

    let buf = PacketBuffer.new();
    assert(Buffer<1024>.capacity() == 1024);
}
```

- **语义与约束**：
  - `<const name: Type>`：声明编译期值泛型参数（支持整数、布尔及定长数组如 `[usize; 2]`）；
  - `[T; N]` 与 `[N]T`：定长数组语法在类型与字段声明中完全互通；
  - 编译器针对具体的常量实参组合自动在编译期展开为单态化（Monomorphize）特化类型，生成确定的物理内存布局，零堆分配、零运行时抽象惩罚。

#### 11.9.5.1 Struct 嵌套与匿名代码块语法自动降级机制

针对在 CP 中书写 `@zig`、C-FFI 或匿名 `struct { ... }` 表达式的场景，CP 编译器内置 token 感知的语法自愈降级机制：
1. **`let` / `let mut` 自动降级**：在 struct 内函数体中书写的 `let` 与 `let mut` 自动转译为 Zig 的 `var`，防止 Zig 编译器因不识别 `let` 关键字而报错；
2. **函数签名冒号消除**：自动将 CP 风格函数声明中的返回值类型冒号（如 `pub fn f(): void`）消除为 Zig 标准函数签名（`pub fn f() void`），避免 `error: expected return type expression, found ':'`。

#### 11.9.5.2 运行时 Prelude 辅助函数命名空间隔离与防遮蔽机制（Anti-Shadowing）

在底层目标语言（Zig）的作用域规则中，顶层或模块级声明（如 `pub const T = enum { ... }`）会被其作用域内包含同名局部变量或常量的辅助函数判定为命名遮蔽（`local constant shadows declaration of 'T'`）。

为保障用户在 CP 业务代码中自由使用 `T`（如通用类型形参名、枚举名或结构体名）而不受转译期环境干扰，CP 编译器规范：
1. **内建辅助函数形参与局部变量私有化**：所有由代码生成器（Codegen Prelude）注入的通用类型提取辅助函数（如 `cp_iter_slice`、`is_string_like`、`CpValType`、`cp_string_from`、`cp_opt_is_null`、`cp_opt_unwrap`、`LockGuard`、`CpMutex` 等）统一采用前缀型标识符 `__T` 替代裸 `T`；
2. **自动析构（Auto-Drop）防冲突隔离**：在作用域末尾自动注入的资源释放与所有权回收代码块（`defer if (!__x_moved) { const __T = @TypeOf(x); ... }`）同样全面采用 `__T` 局部类型名，实现 100% 的作用域安全隔离与抵抗遮蔽。

```typescript
// ✅ 用户可在顶层或模块内定义任意标识符为 T，完全零冲突：
pub const T = enum {
    Alpha,
    Beta,
};

pub fn process_data(items: []const u8): usize {
    let mut total: usize = 0;
    for (items) |c| { // 底层调用 cp_iter_slice，内部使用 __T，不遮蔽顶层 T
        total = total + c as usize;
    }
    return total;
}
```

#### 11.9.6 字段级物理内存对齐注解 (`@align(N)`)

在对接硬件网卡 DMA、SIMD 矢量寄存器或特定的操作系统事件循环（如 epoll/kqueue 事件缓冲区）时，结构体字段必须满足物理内存字节对齐要求。

CP 在字段级别支持 `@align(N)` 装饰器：

```typescript
export class LoopData {
    @align(16)
    packet_buffer: u8[512],

    @align(8)
    timer_epoch: u64,
}
```

- **代码生成**：直接转译为 Zig 原生的字段对齐属性：
  ```zig
  packet_buffer: [512]u8 align(16),
  timer_epoch: u64 align(8),
  ```

#### 11.9.7 编译期能力与内省反射原语 (`reflect.hasMethod` 与 `reflect.has`)

在编写泛型网络事件调度器（如 HTTP / WebSocket Handler）时，框架需要探知用户传入的泛型处理器是否实现了特定事件方法（如 `onOpen`、`onData`、`onClose` 等）：

```typescript
export class SocketContext<Handler> {
    fn handle_open(s: *Socket): void {
        if (reflect.hasMethod(Handler, "onOpen")) {
            Handler.onOpen(s);
        }
    }

    fn check_field(): void {
        if (reflect.has(Handler, "custom_tag")) {
            // 探测字段属性
        }
    }
}
```

- **语义**：
  - `reflect.hasMethod(T, "method_name")`：在编译期静态检测类型 `T` 是否包含指定名称的成员/静态方法；
  - `reflect.has(T, "field_name")`：在编译期静态检测类型 `T` 是否包含指定属性/声明；
- **小驼峰规范**：完全契合 CP 语言与 TypeScript 生态的统一标识符风格。

#### 11.9.8 结构体与类可见性规范及全静态类型扩展

为了使系统级数据结构声明兼具 TypeScript 的简洁与底层结构体的直接透明，CP 确立了清晰的可见性与静态语义：

1. **`struct` 成员默认公开（Public by Default）**：
   - 结构体（`struct`）作为 POD 数据容器，其内部所有字段与方法**默认拥有 `pub` 可见性**，无需在每个成员前显式书写 `pub`；
   - 若需声明私有字段，可显式标注 `priv` 或使用 `#` 私有前缀。
2. **`class` 成员默认私有（Private by Default）**：
   - 类（`class`）作为面向对象封装体，其内部成员（字段与方法）**默认私有（`priv`）**；
   - 若需对外开放接口，必须显式标注 `pub` 前缀（如 `pub fn run(*this): void`）。
3. **全静态类型修饰（`static struct` / `static class`）**：
   - 当类型被标注为 `static struct` 或 `static class` 时，其内部的所有函数与常量**默认全为静态成员**；
   - 成员定义前可省略 `static` 关键字，大幅减少工具类与命名空间类型的样板代码。

---

## 十二、标准库索引与生态路线图

### 12.1 已实装官方核心标准库索引（`cap:*`）

| 标准库协议通道    | 核心导出功能                                                           | 状态           |
| :---------------- | :--------------------------------------------------------------------- | :------------- |
| `cap:collections` | `VecDeque`, `BinaryHeap`, `BTreeMap`, `BTreeSet`, `HashMap`, `HashSet` | **已全面实装** |
| `cap:io`          | `Reader`, `Writer`, `Closer`, `Seeker`, `BufferReader`, `BufferWriter`, `BufferedReader`, `BufferedWriter`, `copyStream`, `PipeReader`, `PipeWriter`, `MaxBuf`, `Pipe`, `print`, `console` | **已全面实装** |
| `cap:stream`      | 工业级零拷贝流式 I/O 管道（重导出 `cap:io` 中的统一流接口）             | **已全面实装** |
| `cap:sync`        | `Futex`, `Mutex`, `RwLock`, `Condition`, `WaitGroup`, `Channel`, `MpmcQueue`, `UnboundedQueue`, `ConcurrentMap`, `ArcSlice`, `Wyhash`, `Atomic`, `Shared` | **已全面实装** |
| `cap:json` / `cap:serde` | 零拷贝 JSON 流式解析器 `JsonParser`、`JsonValue`、内存池 `JsonBufferPool`、`@json_parse` / `@json_stringify` | **已全面实装** |
| `cap:router`      | 超高性能 Radix-Tree 路由引擎 `Router<T>`, `RadixNode<T>`, `ParamMap`, `RouteMatch` | **已全面实装** |
| `cap:http`        | `Method`, `Headers`, `ETag`, `MimeType`, `Request`, `Response`, `Server`, `fetch`, `serve`, `json` | **已全面实装** |
| `cap:path`        | 跨平台路径计算 `Path`, `PosixPath`, `WindowsPath`（`join`, `resolve`, `normalize`, `dirname`, `basename`, `extname`, `isAbsolute`） | **已全面实装** |
| `cap:url`         | WHATWG 规范 `URL`, `URLSearchParams`, `QueryParam`                     | **已全面实装** |
| `cap:encoding`    | 极速 Base64 编解码器（Standard, URL-Safe, 无填充, 内存池, 统一解码）  | **已全面实装** |
| `cap:glob`        | Unix Glob 模式匹配（递归 `**`、单级 `*`、`?`、字符集 `[a-z]`、`!`、`hasMagic`） | **已全面实装** |
| `cap:semver`      | Semver 2.0 版本解析、全语义比较排序与范围匹配（`^`, `~`, `>=`, `<=`, `-`, `||`, `*`） | **已全面实装** |
| `cap:dotenv`      | 环境变量 `.env` 极速加载器（支持单/双引号、转义符、内联注释、`export` 语法） | **已全面实装** |
| `cap:ini`         | 工业级 INI 配置文件解析器（多 Section 划分、全局/分段键值提取、默认值回退） | **已全面实装** |
| `cap:which`       | 跨平台可执行二进制定位器（支持 `PATH` 环境变量遍历、相对/绝对路径解析与执行权限校验） | **已全面实装** |
| `cap:ws`          | RFC 6455 现代 WebSocket 协议帧处理（Opcode、打包分片、4 字节极速异或掩码、控制帧） | **已全面实装** |
| `cap:crypto`      | 工业级密码学体系：CSPRNG、UUID v4、散列哈希（SHA256/512/1/MD5/流式）、HMAC 消息认证、PBKDF2 密钥派生、AEAD 现代对称加密（AES-256-GCM、ChaCha20-Poly1305） | **已全面实装** |
| `cap:event_loop`  | 工业级统一事件循环体系：侵入式二叉最小堆定时器、16 字节无锁跨线程任务、微任务防风暴去重、POSIX poll 描述符监听与调度引擎 | **已全面实装** |
| `cap:security`    | 安全防护与防跨站请求伪造子系统：CSPRNG 随机 Nonce、HMAC-SHA256 签名与篡改校验、恒定时间防时序攻击比对（`constantTimeEql`）、多格式编解码（Base64Url, Base64, Hex） | **已全面实装** |
| `cap:mem`         | `Arena`, `SharedTensor`, 全局共享堆, 原子引用计数                      | **已全面实装** |
| `cap:task`        | `spawn`, `spawn_cpu`, `join`, `Task.all`, `Task.race`                  | **已全面实装** |
| `cap:actor`       | `@spawn`, `@receive`, `@reply`, `PID`, 信箱通信                        | **已全面实装** |
| `cap:fs`          | 异步/同步文件读取、写入、目录创建、流式 I/O                            | **已全面实装** |
| `cap:net`         | TCP 套接字、UDP 广播、轻量 HTTP 协议栈                                 | **已全面实装** |
| `cap:reflect`     | 编译期与运行时 `type_of`, `fields_of`, 结构体自省                      | **已全面实装** |

---

### 12.2 完备标准容器矩阵（`cap:collections`）与借用/所有权安全体系

CP 语言基于“TS 的高阶直觉 + Rust 的内存安全 + Zig 的裸机性能”愿景，在 `cap:collections` 协议通道中原生构建了对标 Rust `std::collections` 的工业级完备容器家族，全面覆盖连续数组、双端环形缓冲、优先队列二叉堆、B 树有序表以及高性能哈希集合：

```mermaid
graph TD
    Matrix["完备标准容器矩阵 (cap:collections)"]

    Matrix --> Seq["序列与双端队列 (Sequences)"]
    Matrix --> Tree["有序树型容器 (Ordered Trees)"]
    Matrix --> Heap["优先队列堆 (Priority Heaps)"]
    Matrix --> Hash["哈希映射与集合 (Hash Tables)"]

    Seq --> V["Array&lt;T&gt; / Vec&lt;T&gt; (连续动态数组)"]
    Seq --> VD["VecDeque&lt;T&gt; (双端环形队列)"]

    Tree --> BTM["BTreeMap&lt;K, V&gt; (B树有序键值映射)"]
    Tree --> BTS["BTreeSet&lt;T&gt; (B树有序去重集合)"]

    Heap --> BH["BinaryHeap&lt;T&gt; (二叉大顶堆优先队列)"]

    Hash --> HM["Map&lt;K, V&gt; / HashMap&lt;K, V&gt; (哈希字典)"]
    Hash --> HS["Set&lt;T&gt; / HashSet&lt;T&gt; (哈希集合)"]
```

#### 12.2.1 容器全景特性与时间复杂度矩阵

| 容器名称             | 底层结构                 |  插入性能   |  查找/读取  |     删除性能     |    顺序特性     | 典型应用场景                                   |
| :------------------- | :----------------------- | :---------: | :---------: | :--------------: | :-------------: | :--------------------------------------------- |
| **`Vec<T>` / `[T]`** | 连续动态切片             | $O(1)$ 均摊 | $O(1)$ 下标 |   $O(1)$ 尾部    |     插入序      | 高性能线性数组、批处理连续内存                 |
| **`VecDeque<T>`**    | 环形缓冲区 (Ring Buffer) | $O(1)$ 两端 | $O(1)$ 随机 |   $O(1)$ 两端    | 先入先出 / 双端 | Actor 消息队列、调度器环形任务池、滑动窗口     |
| **`BinaryHeap<T>`**  | 扁平二叉大顶堆           | $O(\log N)$ | $O(1)$ 栈顶 | $O(\log N)$ 弹出 |  优先级递减序   | 任务调度优先级、定时器堆、Top-K 流式筛选       |
| **`BTreeMap<K, V>`** | 紧凑有序 B 树映射        | $O(\log N)$ | $O(\log N)$ |   $O(\log N)$    | Key 自然字典序  | 有序配置表、范围切片查询、前缀扫描、有序排行榜 |
| **`BTreeSet<T>`**    | 紧凑有序 B 树集合        | $O(\log N)$ | $O(\log N)$ |   $O(\log N)$    | Element 字典序  | 唯一递增序列、区间包含检测、有序去重过滤       |
| **`HashMap<K, V>`**  | 动态哈希映射表           | $O(1)$ 平均 | $O(1)$ 平均 |   $O(1)$ 平均    |      无序       | 高频键值缓存、字典检索、实体关系映射           |
| **`HashSet<T>`**     | 动态哈希去重集合         | $O(1)$ 平均 | $O(1)$ 平均 |   $O(1)$ 平均    |      无序       | 极速 ID 去重、成员资格测试、数学集合运算       |

#### 12.2.2 严格借用与所有权检查铁律

所有标准容器均受 CP 编译器静态借用检查器（Borrow Checker）的全面监管：

1. **方法接收者二元分立**：
   - **修改方法（Mutating Methods）**：必须声明为 `*_` 可变借用接收者（如 `push_back(*_)`, `pop_front(*_)`, `insert(*_)`, `clear(*_)`）。调用时必须持有容器的排他性可变写权限；
   - **只读查询方法（Query Methods）**：声明为 `&_` 只读借用接收者（如 `peek(&_)`, `len(&_)`, `get(&_)`, `has(&_)`）。允许并发只读借用。
2. **读写互斥检查（Aliasing XOR Mutability）**：
   - 当容器存在任何未解禁的只读借用（如 `let p = &q;`）时，编译器**静态严禁**任何形式的结构改变或写入调用；
3. **`for ... of` 迭代保护（Iterator Invalidation Guard）**：
   - 在迭代遍历容器期间，容器被自动锁定为只读借用，任何在循环体内部尝试修改该容器的指令，Sema 会在编译期直接拦截（抛出 `Cannot mutate 'x' while it is borrowed in 'for ... of' loop`）！

#### 12.2.3 容器 API 规范与实战范式

```cp
import { VecDeque, BinaryHeap, BTreeMap, BTreeSet, HashMap, HashSet } from "cap:collections";
import { expect_eq } from "cap:test";

// =========================================================================
// 1. VecDeque (双端环形队列)
// =========================================================================
let deque = new VecDeque();
deque.push_back(10);
deque.push_back(20);
deque.push_front(5); // 队列当前布局: [5, 10, 20]

expect_eq(deque.len(), 3);
expect_eq(deque.peek_front(), 5);
expect_eq(deque.peek_back(), 20);

let front_item = deque.pop_front(); // 弹出 5
let back_item = deque.pop_back();   // 弹出 20
expect_eq(front_item, 5);
expect_eq(back_item, 20);

// 双轨流式迭代遍历 (支持自动消费排出)
let sum = 0;
for (n of deque) {
    sum = sum + n;
}
expect_eq(sum, 10);
expect_eq(deque.is_empty(), true);

// =========================================================================
// 2. BinaryHeap (优先队列二叉堆)
// =========================================================================
let heap = new BinaryHeap();
heap.push(15);
heap.push(42);
heap.push(8);
heap.push(27);

// peek 查看当前最高优先级元素 (大顶堆最大值)
expect_eq(heap.peek(), 42);

// 依次弹出：严格按从大到小的优先级顺序输出
expect_eq(heap.pop(), 42);
expect_eq(heap.pop(), 27);
expect_eq(heap.pop(), 15);
expect_eq(heap.pop(), 8);
expect_eq(heap.is_empty(), true);

// =========================================================================
// 3. BTreeMap (有序键值映射)
// =========================================================================
let btree = new BTreeMap();
btree.insert("banana", 20);
btree.insert("apple", 10);
btree.insert("cherry", 30);

// O(log N) 快速检索与极值探测
expect_eq(btree.get("apple"), 10);
expect_eq(btree.min_key(), "apple");
expect_eq(btree.max_key(), "cherry");

// 键列表严格按照自然升序字典序返回
let sorted_keys = btree.keys();
expect_eq(sorted_keys[0], "apple");
expect_eq(sorted_keys[1], "banana");
expect_eq(sorted_keys[2], "cherry");

// 移除键并返回旧值
let old_val = btree.remove("banana");
expect_eq(old_val, 20);
expect_eq(btree.has("banana"), false);

// =========================================================================
// 4. BTreeSet (有序去重集合)
// =========================================================================
let bset = new BTreeSet();
bset.add(30);
bset.add(10);
bset.add(20);
bset.add(10); // 重复元素自动被去重过滤

expect_eq(bset.len(), 3);
expect_eq(bset.min(), 10);
expect_eq(bset.max(), 30);

// 元素集合天然有序
let sorted_vals = bset.values();
expect_eq(sorted_vals[0], 10);
expect_eq(sorted_vals[1], 20);
expect_eq(sorted_vals[2], 30);

// =========================================================================
// 5. HashMap 与 HashSet (Rust Parity 别名)
// =========================================================================
let map = new HashMap();
map.set("name", "CP-BEAM");
expect_eq(map.get("name"), "CP-BEAM");

let set = new HashSet();
set.add(100);
expect_eq(set.has(100), true);
```

---

### 12.3 统一高性能 I/O 流体系（`cap:io`）与 Trait 契约 【已实装】

CP 语言在 `cap:io` 中构建了对标 Go `io` 与 Rust `std::io` 的工业级、零拷贝、类型安全的流式 I/O 基础设施。通过强类型 Trait 契约解耦底层数据源（操作系统文件、TCP/UDP 套接字、内存切片、终端）与数据处理管道。

#### 12.3.1 核心 Trait 契约

```typescript
import { Reader, Writer, Closer, Seeker, SeekOrigin, IOError } from "cap:io";

// 统一读取器契约：传入目标缓冲区可变切片，返回实际读取字节数，0 表示 EOF
export trait Reader {
    fn read(mut _, *buf: []u8): usize!;
}

// 统一写入器契约：传入只读数据切片，返回实际写入字节数
export trait Writer {
    fn write(mut _, buf: []const u8): usize!;
}

// 统一资源关闭契约
export trait Closer {
    fn close(mut _): void;
}

// 统一流寻址契约
export trait Seeker {
    fn seek(mut _, offset: i64, origin: SeekOrigin): u64!;
}
```

#### 12.3.2 内存切片与缓冲流

- **`BufferReader`**：基于只读字节切片 `[]const u8` 的零拷贝内存读取器，无需堆分配，原生实现 `Reader` 与 `Seeker`。
  ```typescript
  import { BufferReader, SeekOrigin } from "cap:io";

  let reader = new BufferReader("Hello CP World");
  let buf: [5]u8 = [0 as u8; 5];
  let n = reader.read(buf[0..5])?; // n = 5, buf 为 "Hello"
  let new_pos = reader.seek(0, SeekOrigin.Start)?; // 重新寻址至起始
  ```

- **`BufferWriter`**：4096 字节定长安全写入器，原生实现 `Writer`，支持 `.bytes()` 切片提取与 `.reset()` 重置。
  ```typescript
  import { BufferWriter } from "cap:io";

  let writer = new BufferWriter();
  writer.write("CP Native Data")?;
  let data = writer.bytes(); // []const u8 切片
  ```

- **`BufferedReader<R: Reader>`**：带 256 字节内部缓存的高性能包装流，支持 `.readByte()` 单字节速读与批量块填充，消除高频细粒度系统调用。
  ```typescript
  import { BufferReader, BufferedReader } from "cap:io";

  let base_reader = new BufferReader("Stream Payload");
  let buffered = new BufferedReader<BufferReader>(base_reader);
  let b = buffered.readByte()?; // 'S'
  ```

- **`BufferedWriter<W: Writer>`**：带 256 字节聚合缓存的高性能包装流，支持满缓冲自动 flush 与手动 `.flush()`。
  ```typescript
  import { BufferWriter, BufferedWriter } from "cap:io";

  let base_writer = new BufferWriter();
  let buffered = new BufferedWriter<BufferWriter>(base_writer);
  buffered.write("Chunk 1")?;
  buffered.flush()?;
  ```

#### 12.3.3 泛型流管道操作

- **`copyStream<R: Reader, W: Writer>(mut r: R, mut w: W): usize!`**：在读取器与写入器之间执行分块流式全量搬运直至 EOF，返回总传输字节数。
- **`copyN<R: Reader, W: Writer>(mut r: R, mut w: W, limit: usize): usize!`**：精确搬运 `limit` 字节。

```typescript
import { BufferReader, BufferWriter, copyStream } from "cap:io";

let src = new BufferReader("Pipelined Data Across Streams");
let dst = new BufferWriter();
let copied = copyStream(src, dst)?; // 搬运 29 字节
```

#### 12.3.4 管道与边界保护流体系（`PipeReader`, `PipeWriter`, `MaxBuf`, `cap:stream`）

CP 标准库吸收并提炼了 Bun 的工业级管道架构，在 `cap:io` 与 `cap:stream` 中原生提供无锁/锁同步内存管道与缓冲区边界守卫：

- **`MaxBuf`（动态零拷贝切片缓冲守卫）**：限制最大内存占用（默认 16MB），支持 `ensureCapacity()` 动态几何扩容并确保绝对内存安全。
- **`PipeReader` 与 `PipeWriter`**：
  - 完整实现 `Reader, Closer` 与 `Writer, Closer` Trait 契约；
  - 提供单进程内线程/协程间高速零拷贝数据流动管道。
- **`Pipe.create(capacity?: usize)`**：
  - 便捷工厂函数，一次性创建配对的 `[PipeReader, PipeWriter]`。

```typescript
import { Pipe, PipeReader, PipeWriter } from "cap:stream";

// 创建 64KB 高性能数据管道
let [reader, writer] = Pipe.create(65536);

// 写入端流动输入
writer.write("Streaming through CP Pipe!".asBytes())?;
writer.close();

// 读取端解构消费
let mut buf = new [u8; 128];
let n = reader.read(buf[..])?;
```

---

### 12.4 工业级并发原语与无锁数据结构（`cap:sync`） 【已实装】

吸收自 Bun 高并发核心基础设施，`cap:sync` 提供跨平台底层同步原语与高性能无锁数据结构，专为百万级高频 Agent 协同与跨物理线程零锁争用设计：

#### 12.4.1 底层同步原语全览

| 原语 | 类型 / 签名 | 说明 |
| :--- | :--- | :--- |
| **`Futex`** | `struct Futex` | 跨平台轻量系统 Futex 封装（Linux `SYS_futex`、macOS `__ulock`、Windows `WaitOnAddress`），支持 `wait()`、`waitForever()`、`wake()`。 |
| **`Mutex`** | `struct Mutex` | 工业级互斥锁，支持无竞争秒级原子自旋与慢速 Futex 挂起，提供 `lock()`、`unlock()` 与 `tryLock()`。 |
| **`Condition`** | `struct Condition` | 跨平台条件变量，支持基于 Futex 的毫秒级等待 `timedWait()` 与 `signal()` / `broadcast()` 唤醒。 |
| **`WaitGroup`** | `struct WaitGroup` | 计数信号量同步屏障，支持 `add(delta)`、`done()`、`wait()`，精确控制并发批处理任务聚合。 |
| **`Channel<T>`** | `struct Channel<T>` | 线程安全有界 MPMC 通道，支持非阻塞 `trySend()` / `tryRecv()` 与超时阻塞 `send()` / `recv()`，自带 `close()` 广播。 |

#### 12.4.2 无锁并发数据结构（Lock-Free Primitives）

| 结构 | 类型 / 签名 | 说明 |
| :--- | :--- | :--- |
| **`MpmcQueue<T>`** | `struct MpmcQueue<T>` | 基于 Dmitry Vyukov 经典算法的有界多生产者多消费者（MPMC）无锁环形队列，支持 `push()`、`pop()`，高频跨线程数据吞吐零全局锁争用。 |
| **`UnboundedQueue<T>`**| `struct UnboundedQueue<T>` | 侵入式链表无锁无界并发队列，支持多线程高吞吐安全并发入队。 |
| **`ConcurrentMap<V>`** | `struct ConcurrentMap<V>` | 基于 **Wyhash** 64 位超高速哈希分片的分段并发哈希表（Striped Map），将锁竞争分散至 16 个独立桶，支持高并发读写 `put()`、`get()`、`remove()`。 |
| **`ArcSlice<T>`** | `struct ArcSlice<T>` | 原子引用计数切片（Atomic Reference-Counted Slice），提供跨线程零拷贝只读共享视图与幂等释放 `retain()` / `release()`。 |
| **`Wyhash`** | `struct Wyhash` | 工业级 64 位极速确定性哈希算法，提供 `hash(seed, data)` 与 `hash64(data)`。 |

#### 12.4.3 完整使用示例

```typescript
import { Futex, Mutex, Condition, WaitGroup, Channel, MpmcQueue, ConcurrentMap, ArcSlice } from "cap:sync";

// 1. 无锁 MPMC 环形队列
var queue = MpmcQueue(i32, 16).init();
queue.push(101);
queue.push(102);
let v1 = queue.pop().?; // 101
let v2 = queue.pop().?; // 102

// 2. 线程同步屏障 WaitGroup
var wg = WaitGroup.init();
wg.add(2);
// 工作线程完成时：
wg.done();
wg.done();
wg.wait(); // 屏障解除

// 3. 有界 MPMC 通道 Channel
var ch = Channel(i32, 8).init();
ch.send(42);
let received = ch.recv().?; // 42
ch.close();

// 4. 分片并发哈希表 ConcurrentMap
let map = ConcurrentMap([]const u8).init(allocator);
defer map.deinit();
map.put("session_id", "sess_998877");
let val = map.get("session_id"); // "sess_998877"

// 5. 原子引用计数切片 ArcSlice
const raw_data = "immutable shared agent tensor payload";
let arc = ArcSlice(u8).init(allocator, raw_data) catch unreachable;
defer arc.release();
let slice_ref = arc.retain(); // 增加引用计数
defer slice_ref.release();
```

---

### 12.5 统一零拷贝流式 I/O 管道体系（`cap:stream` / `cap:io`） 【已实装】

CP 标准库吸收并提炼了 Bun 的工业级管道架构与环形缓冲区机制，在 `cap:io` 与 `cap:stream` 中原生提供无锁/锁同步内存管道与缓冲区边界守卫：

#### 12.5.1 核心组件

- **`PipeReader`**：流式管道读取端，包装底层 `Pipe`，具备非阻塞与阻塞读取能力；
- **`PipeWriter`**：流式管道写入端，支持向内部环形缓冲区推入数据，并在缓冲区满时挂起协程或阻塞让步；
- **`MaxBuf`**：安全边界守卫，动态监控流式读写的内存上限，杜绝畸形大报文耗尽系统内存；
- **`Pipe.create(allocator, capacity)`**：原子成对初始化 `(PipeReader, PipeWriter)`。

#### 12.5.2 完整使用示例

```typescript
import { Pipe, PipeReader, PipeWriter, MaxBuf } from "cap:stream";

// 创建 4KB 缓冲管道
let pair = Pipe.create(allocator, 4096)?;
defer pair.deinit();

let writer = pair.writer;
let reader = pair.reader;

// 写入端推送数据
const message = "Agent Pipeline Stream Data";
writer.writeAll(message)?;
writer.flush()?;

// 读取端消费数据
let read_buf: [128]u8 = undefined;
const bytes_read = reader.read(&read_buf)?;
const received = read_buf[0..bytes_read]; // "Agent Pipeline Stream Data"
```

---

### 12.6 零拷贝序列化与流式 JSON 解析框架（`cap:json` / `cap:serde`） 【已实装】

CP 语言在 `cap:json` 与 `cap:serde` 中实现了业界领先的**零拷贝递归下降 JSON 解析器**与**内存池**，杜绝堆垃圾与二次字符串深拷贝：

#### 12.6.1 核心机制

1. **零拷贝切片借用（Zero-Copy Slicing）**：
   - 提取字符串时，`JsonValue.String` 直接借用原始 JSON 输入 Buffer 的只读切片（`[]const u8`），指针直接指向输入内存，零分配；
2. **内存池高速复用（`JsonBufferPool`）**：
   - 吸收 Bun 的 `HashMapPool` 机制，提供节点与哈希表的预分配内存池，解析复杂树状结构时内存复用率高达 95%+；
3. **极速解析器（`JsonParser`）**：
   - 支持基本标量（`null`、`bool`、`number`、`string`）及任意嵌套的 `Array` 与 `Object`；
   - 提供 TS 风格对象字段检索（`obj.get("field")`）与数组索引（`arr.getIndex(index)`）。

#### 12.6.2 完整使用示例

```typescript
import { JSON, JsonParser, JsonValue, JsonBufferPool } from "cap:json";

const raw_json = "{\"name\": \"Antigravity\", \"version\": 2, \"tools\": [\"cli\", \"cpc\"]}";
const root = JSON.parse(allocator, raw_json)?;
defer root.deinit(allocator);

// 1. 结构体字段零拷贝借用
if (name_val := root.get("name")) {
    const name_str = name_val.asString().?; // "Antigravity"，直接借用 raw_json 内存！
}

// 2. 数值解析
if (ver_val := root.get("version")) {
    const ver = ver_val.asNumber().?;       // 2.0
}

// 3. 数组元素访问
if (tools_val := root.get("tools")) {
    const tool0 = tools_val.getIndex(0).?.asString().?; // "cli"
    const tool1 = tools_val.getIndex(1).?.asString().?; // "cpc"
}
```

---

### 12.7 超高性能 Radix-Tree 路由引擎（`cap:router`） 【已实装】

CP 语言在 `cap:router` 中内置了对标 Bun、Fastify 与 Go Chi 的微秒级 Radix-Tree 前缀树路由分发引擎：

#### 12.7.1 核心特性

- **8 树独立动词分发**：针对 `GET`, `POST`, `PUT`, `DELETE`, `PATCH`, `HEAD`, `OPTIONS`, `ALL` 预置独立子树，消除跨方法遍历开销；
- **全类型参数抽取**：
  - 动态参数：`:param` 与 `[param]` 语法支持；
  - 通配符全匹配：`*path` 与 `[...path]` 尾部贪婪捕获；
- **零堆分配 `ParamMap`**：参数字典默认以 16 槽位栈内存就地缓存，HTTP 请求路由全链路零动态堆申请。

#### 12.7.2 完整使用示例

```typescript
import { Router } from "cap:router";

let router = Router([]const u8).init(allocator);
defer router.deinit();

// 1. 注册多动词与动态参数路由
router.add("GET", "/api/v1/users", "handleListUsers")?;
router.add("GET", "/api/v1/users/:id", "handleGetUser")?;
router.add("POST", "/api/v1/users", "handleCreateUser")?;
router.add("GET", "/static/*filepath", "handleStaticFiles")?;

// 2. 精确路由匹配与参数提取
const match1 = router.lookup("GET", "/api/v1/users/42");
if (m := match1) {
    const handler = m.handler;               // "handleGetUser"
    const user_id = m.params.get("id").?;    // "42"
}

// 3. 通配符贪婪匹配
const match2 = router.lookup("GET", "/static/images/logo.png");
if (match2) |m| {
    const handler = m.handler;               // "handleStaticFiles"
    const path = m.params.get("filepath").?; // "images/logo.png"
}
```

---

### 12.8 现代工业级 HTTP 核心协议体系（`cap:http`） 【已全面实装】

吸收自 Bun.js 经过全球数亿级并发高频检验的工业级网络协议基础设施，在 `cap:http` 中全面实装 HTTP/1.1、HTTP/2 与 HTTP/3 全协议栈客户端与服务引擎：

#### 12.8.1 模块全景与核心组件

1. **协议基础设施 (Step 1)**：
   - **`Method` 枚举**：完备覆盖 33 种 HTTP 动词，内置 `isIdempotent()`、`hasRequestBody()` 与 `fromString()` / `toString()`；
   - **`Headers` 集合**：基于 ASCII 大小写无关查找的 HTTP 报头字典，具备单缓冲内存优化，提供 `get()`, `set()`, `append()`, `delete()`, `count()`, `at()`；
   - **`Encoding`**：内容与传输编码规范（`Identity`, `Gzip`, `Deflate`, `Brotli`, `Zstd`, `Chunked`），内置 `canUseLibDeflate()` 与 `isCompressed()`；
   - **`FetchCacheMode`, `FetchRedirect`, `FetchRequestMode`**：完全遵循 Web 标准与 Fetch 规范的请求控制模式；
   - **`URLPath`**：微秒级路径、扩展名（`extname`）、首段目录（`first_segment`）与查询字符串极速切片；
   - **`SignalStore` & `Signals`**：原子化请求生命周期与取消跟踪器（`isAborted()`, `isStreaming()`, `isUpgraded()`, `abort()`）；
   - **`HeaderValueIterator`**：无堆分配的逗号分隔报头（如 `Accept-Encoding`）零拷贝 Token 迭代器；
   - **`HTTPRequestBody`**：支持零拷贝内存切片与流式传输的主体封装；
   - **`ETag` & `MimeType`**：强弱 ETag 生成、解析与 `ifNoneMatch()` 条件协商；常见扩展名与路径的 MIME 类型秒级检测。

2. **HTTP/2 与 HTTP/3 子系统 (Step 2)**：
   - **HTTP/2 二进制帧体系 (RFC 7540)**：`FrameHeader`（9字节二进制帧头编解码）、`FrameType`（Data, Headers, Priority, Settings, Ping, Goaway 等）、`ErrorCode`；
   - **HTTP/2 多路复用流会话**：`H2ClientSession` 跟踪并发流限额、管理服务端初始与对端窗口尺寸（`WindowUpdate`）、分配客户端奇数流 ID（1, 3, 5...）、处理 `Goaway`；`H2Stream` 跟踪单流状态（`Idle`, `Open`, `HalfClosedRemote`, `Closed`）；
   - **HTTP/3 Alt-Svc 协商 (RFC 7838)**：`AltSvc` 解析 `h3=":443"; ma=86400` 报头，提取目标端口与有效期限；
   - **HTTP/3 QUIC 客户端流会话**：`H3ClientSession` 分配客户端双向请求流（0, 4, 8...），`H3Stream` 跟踪流生命周期。

3. **工业级核心客户端引擎与生命周期 (Step 3)**：
   - **`HTTPClient` 引擎**：
     - 完全遵循 RFC 7230 / RFC 7231 的请求行、Host、User-Agent、Content-Length 及自定义报头序列化；
     - 分块传输编码解码器（`decodeChunkedBody`），自动识别十六进制块尺寸并完成 `0\r\n\r\n` 零拷贝拼合；
     - 响应状态码（`parseResponseStatus`）与主体切片（`parseResponseBody`）极速解析；
   - **`HTTPContext` 连接池**：内置 Keep-Alive 套接字池（`PooledSocket`），支持多域名、多端口与 SSL 隔离复用，提供死链接自动剔除与安全回收；
   - **`Decompressor` 流式解压**：无缝对接 Gzip / Deflate / Brotli / Zstd 压缩传输流；
   - **`ProxyTunnel` (RFC 7231)**：支持 HTTP CONNECT 代理握手建立与响应校验；
   - **`SendFile` 零拷贝文件上传**：基于 Linux 内核 `sendfile` 驱动的零用户态内存拷贝极速流式上传。

#### 12.8.2 完整使用示例

```typescript
import {
    Method,
    Headers,
    Encoding,
    FetchCacheMode,
    URLPath,
    SignalStore,
    HeaderValueIterator,
    FrameHeader,
    FrameType,
    H2ClientSession,
    AltSvc,
    ProxyTunnel,
    HTTPContext,
    HTTPClient,
    Decompressor
} from "cap:http";

// 1. 协议基础设施与路径解析
const path_info = URLPath.parse("/api/v2/items/archive.json?filter=active");
const is_json = path_info.extname;       // "json"
const is_root = URLPath.parse("/").isRoot(); // true

// 2. 报头 Token 迭代器 (零拷贝流式解析)
var iter = HeaderValueIterator.init("gzip, deflate, br;q=1.0, zstd");
while (iter.next()) |token| {
    // 依次遍历: "gzip", "deflate", "br;q=1.0", "zstd"
}

// 3. HTTP/2 9字节二进制帧头编解码
var h2_header = FrameHeader{
    length: 512,
    type: FrameType.Data,
    flags: 0x01, // EndStream
    stream_id: 1,
};
let wire_buf: [9]u8 = undefined;
h2_header.encode(&wire_buf);
const decoded = FrameHeader.decode(&wire_buf).?;

// 4. HTTP/2 多路复用会话与流分配
let h2_session = H2ClientSession{};
if (h2_session.canCreateStream()) {
    let stream = h2_session.newStream(); // 分配客户端奇数 Stream ID (1)
    defer stream.close();
}

// 5. HTTP/3 Alt-Svc 服务发现
if (entry := AltSvc.parse("h3=\":443\"; ma=86400")) {
    const h3_port = entry.port;       // 443
    const max_age = entry.max_age;    // 86400
}

// 6. Keep-Alive 连接池管理
let http_ctx = HTTPContext.init();
defer http_ctx.deinit();

// 从池中获取或复用长连接
if (fd := http_ctx.acquireSocket("api.github.com", 443, true)) {
    // 复用已建立的 TLS 连接...
    _ = http_ctx.releaseSocket(fd, "api.github.com", 443, true);
}

// 7. 客户端请求序列化与分块解码
let client = HTTPClient.init(Method.POST, "https://api.github.com/v1/events");
client.body = "{\"event\":\"ping\"}";

let req_buf: [1024]u8 = undefined;
const req_str = client.formatRequest(&req_buf)?;
// 自动生成: POST /v1/events HTTP/1.1\r\nHost: api.github.com\r\nContent-Length: 16\r\n...

// 8. 分块传输传输解码 (Chunked Transfer Decoding)
const chunked = "4\r\nWiki\r\n5\r\npedia\r\n0\r\n\r\n";
let out_buf: [64]u8 = undefined;
const decoded_len = HTTPClient.decodeChunkedBody(chunked, &out_buf)?; // "Wikipedia"
```

---

### 12.9 跨平台路径计算与 WHATWG URL 规范（`cap:path` & `cap:url`） 【已实装】

CP 标准库提供了完全对标 Node.js `path` 与浏览器 WHATWG `URL` 标准的系统级支持：

#### 12.9.1 跨平台路径操作（`cap:path`）

- 自动抹平 POSIX 与 Windows 差异，支持盘符（`C:`）、UNC 路径（`\\server\share`）与正反斜杠转换；
- 提供 `join()`, `resolve()`, `normalize()`, `dirname()`, `basename()`, `extname()`, `isAbsolute()`。

#### 12.9.2 WHATWG URL 解析引擎（`cap:url`）

- `URL.parse(urlString, allocator)`：结构化拆解 `protocol`, `username`, `password`, `host`, `hostname`, `port`, `pathname`, `search`, `hash`, `origin`；
- `URLSearchParams`：查询字符串增删查改与标准 URL 编码重序列化。

#### 12.9.3 完整使用示例

```typescript
import { path } from "cap:path";
import { URL, URLSearchParams } from "cap:url";

// 1. 跨平台路径运算
let path_buf: [256]u8 = undefined;
const parts = ["/home/user", "projects", "../data/config.cp"];
const joined = path.join(parts[0..], &path_buf); // "/home/user/data/config.cp"
const base = path.basename(joined, null);        // "config.cp"
const ext = path.extname(joined);                // ".cp"

// 2. WHATWG URL 解析与操作
const raw_url = "https://alice:secret@example.com:8080/api/v1/search?q=agents&page=1#results";
let parsed = URL.parse(raw_url, allocator)?;
defer parsed.deinit();

const host = parsed.hostname;                    // "example.com"
const port = parsed.port;                        // "8080"
const query_q = parsed.searchParams.get("q").?;  // "agents"
```

---

### 12.10 高性能数据编码标准库（`cap:encoding` / Base64） 【已实装】

`cap:encoding` 提供了纯 CP 编写的高性能 Base64 编解码器：

- **多模式支持**：标准 Standard Base64（带 `=` 填充）、URL-Safe Base64（`-` 与 `_` 替代 `+` 与 `/`，无填充）；
- **自适应解码引擎**：单个 `decode()` / `decodeAlloc()` 自动识别并解码标准与 URL-Safe 两种格式，自动跳过空白符；
- **零分配就地运算**：提供 `encode(dest, src)` 与 `decode(dest, src)`，杜绝多余堆内存申请。

#### 12.10.1 完整使用示例

```typescript
import { Base64 } from "cap:encoding";

// 1. 栈上零分配标准 Base64 编码
let out_buf: [64]u8 = undefined;
const encoded_len = Base64.encode(&out_buf, "Hello CP World!");
const b64_str = out_buf[0..encoded_len]; // "SGVsbG8gQ1AgV29ybGQh"

// 2. 栈上解码
let dec_buf: [64]u8 = undefined;
const decoded_len = Base64.decode(&dec_buf, b64_str)?;
const original = dec_buf[0..decoded_len]; // "Hello CP World!"

// 3. 动态内存分配与 URL-Safe 模式
const url_safe = Base64.encodeURLSafeAlloc(allocator, "data?token=xyz")?;
defer allocator.free(url_safe);
```

---

### 12.11 极速 Unix Glob 模式匹配（`cap:glob`） 【已实装】

`cap:glob` 提供了工业级高性能 Unix 文件路径与通配符模式匹配器，基于分段递归回溯架构，专为构建工具、文件监听及包管理器设计：

#### 12.11.1 语法全特性覆盖

| 语法原语 | 说明 | 示例 |
| :--- | :--- | :--- |
| `*` | 匹配单个路径段内的任意字符（不跨越目录边界 `/`） | `*.cp` 匹配 `main.cp`，不匹配 `src/main.cp` |
| `**` | 递归 Globstar，匹配 0 个或多个任意层级的目录 | `**/*.cp` 匹配 `main.cp`、`src/main.cp`、`a/b/c.cp` |
| `?` | 匹配单个任意字符 | `test?.cp` 匹配 `test1.cp`、`testA.cp` |
| `[a-z]` / `[0-9]` | 字符范围集合匹配 | `v[0-9].txt` 匹配 `v1.txt`、`v9.txt` |
| `[!...]` / `[^...]`| 字符范围集合取反 | `log_[!0-9].txt` 匹配 `log_a.txt` |
| `!` (前缀) | 整体模式取反 | `!*.tmp` 匹配除 `.tmp` 结尾以外的所有路径 |
| `hasMagic(pat)` | 极速检查模式串是否含有 Glob 通配符（跳过普通文件的遍历开销） | `hasMagic("**/*.cp") == true` |

#### 12.11.2 完整使用示例

```typescript
import { Glob } from "cap:glob";

// 1. 单级与多级递归通配
const is_match1 = Glob.match("*.cp", "main.cp");              // true
const is_match2 = Glob.match("**/*.cp", "src/core/expr.cp");  // true
const is_match3 = Glob.match("lib/**/test_*.cp", "lib/a/b/test_deep.cp"); // true

// 2. 字符集与取反
const is_match4 = Glob.match("v[0-9].json", "v3.json");        // true
const is_match5 = Glob.match("!*.tmp", "data.json");           // true (模式取反)

// 3. 语法探测
if (Glob.hasMagic("src/**/*.cp")) {
    // 启用目录递归遍历探测引擎
}
```

---

### 12.12 Semver 2.0 语义化版本与范围匹配（`cap:semver`） 【已实装】

吸收自 Bun 的包管理器核心算法，`cap:semver` 提供了完全遵循 Semver 2.0 规范的语义化版本解析器、高阶优先级排序与 npm/Cargo 范围匹配引擎：

#### 12.12.1 规范与比较规则

- **全三段式解析**：支持 `v1.2.3`、`=2.10.5-beta.1+sha.abcdef`，精准分离 `major`, `minor`, `patch`, `prerelease`, `build`；
- **Semver 2.0 优先级铁律**：
  - 正式发布版本优先级高于任意预发布版本（`1.0.0 > 1.0.0-alpha`）；
  - 预发布标识按 `.` 分割逐项比较：纯数字分量按数值大小比较（`1.0.0-beta.2 < 1.0.0-beta.11`），非纯数字按 ASCII 词法排序；
- **多维度版本范围匹配**：
  - Caret 锁版本（`^1.2.3`）：允许不改变最左侧非零主/次版本的递增（`^1.2.3` := `>=1.2.3 <2.0.0`，`^0.2.0` := `>=0.2.0 <0.3.0`）；
  - Tilde 补丁更新（`~1.2.0`）：仅允许 patch 级别递增（`~1.2.0` := `>=1.2.0 <1.3.0`）；
  - 比较符（`>=`, `<=`, `>`, `<`）与连字符范围（`1.0.0 - 2.0.0`）；
  - 逻辑或联合集（`||`）：例如 `^1.0.0 || ^3.0.0`；
  - 通配符（`*` / `x` / `X`）。

#### 12.12.2 完整使用示例

```typescript
import { Semver, Version } from "cap:semver";

// 1. 解析 Semver 2.0 版本
const v1 = Version.parse("v1.5.0-beta.2+build.101")?;
const v2 = Version.parse("1.5.0")?;

// 2. 版本比较与语义排序
const is_less = v1.lt(v2);   // true (预发布版本优先次序低于正式版)
const is_greater = v2.gt(v1); // true

// 3. 版本格式化
let buf: [64]u8 = undefined;
const formatted = v1.format(&buf); // "1.5.0-beta.2+build.101"

// 4. npm/Cargo 规范范围匹配
const ok1 = Semver.satisfiesStr("1.2.5", "^1.2.0");          // true
const ok2 = Semver.satisfiesStr("2.0.0", "^1.2.0");          // false
const ok3 = Semver.satisfiesStr("1.2.9", "~1.2.0");          // true
const ok4 = Semver.satisfiesStr("1.5.0", "1.0.0 - 2.0.0");   // true
const ok5 = Semver.satisfiesStr("3.2.0", "^1.0.0 || ^3.0.0");// true
```

---

### 12.13 环境变量 `.env` 极速加载器（`cap:dotenv`） 【已实装】

`cap:dotenv` 提供了工业级 `.env` 环境配置解析器，采用 100% 栈分配就地解析，零堆垃圾：

#### 12.13.1 核心特性

- **现代语法兼容**：支持 `KEY=VALUE`、`KEY: VALUE`，以及 Bash 风格的 `export KEY=VALUE`；
- **双引号转义解析**：自动解析 `\n`, `\r`, `\t`, `\"`, `\\`，支持多行长文本字段；
- **单引号原样保护**：单引号内内容严格保持字面量，不执行任何转义；
- **智能内联注释截断**：识别 `KEY=value # inline comment` 并自动剔除尾部注释。

#### 12.13.2 完整使用示例

```typescript
import { DotEnv } from "cap:dotenv";

const env_content =
    "# 生产环境应用配置\n" +
    "APP_NAME=Antigravity\n" +
    "PORT=8080\n" +
    "export SECRET_KEY=\"top_secret_key\"\n" +
    "GREETING=\"Hello\\nWorld\"\n" +
    "TIMEOUT=30 # 请求超时秒数\n";

const env = DotEnv.parse(env_content);

// 提取变量
const app = env.get("APP_NAME").?;        // "Antigravity"
const port = env.get("PORT").?;           // "8080"
const secret = env.get("SECRET_KEY").?;   // "top_secret_key"
const greet = env.get("GREETING").?;      // "Hello\nWorld" (转义已就地解析)

// 默认值回退与存在性探测
const debug = env.getOr("DEBUG", "false");// "false"
const has_key = env.has("APP_NAME");      // true
```

---

### 12.14 工业级 INI 配置文件解析器（`cap:ini`） 【已实装】

`cap:ini` 提供了纯 CP 实现的高性能 INI 配置解析器：

#### 12.14.1 核心特性

- **多段（Section）划分**：支持全局配置区与 `[section_name]` 命名段空间隔离；
- **双注释符兼容**：无缝支持 `;`（经典 Windows INI）与 `#`（Unix 风格）注释行及行尾注释；
- **双向快速检索**：提供 `get(section, key)`、`getGlobal(key)`、`getOr()` 以及 `hasSection()`。

#### 12.14.2 完整使用示例

```typescript
import { Ini } from "cap:ini";

const ini_content =
    "; 全局应用基础配置\n" +
    "title = Agent System\n" +
    "version = 1.0.0\n\n" +
    "[server]\n" +
    "host = 127.0.0.1\n" +
    "port = 3000\n" +
    "enable_ssl = false\n\n" +
    "[database]\n" +
    "name = \"production_db\"\n" +
    "max_connections = 100 ; 最大连接池大小\n";

const conf = Ini.parse(ini_content);

// 1. 全局配置提取
const title = conf.getGlobal("title").?;            // "Agent System"

// 2. 区分 Section 提取
const host = conf.get("server", "host").?;          // "127.0.0.1"
const port = conf.get("server", "port").?;          // "3000"
const db_name = conf.get("database", "name").?;     // "production_db"

// 3. Section 存在性探测与默认值回退
const has_srv = conf.hasSection("server");          // true
const workers = conf.getOr("server", "workers", "4");// "4"
```

---

### 12.15 跨平台可执行程序定位器（`cap:which`） 【已实装】

吸收自 Bun 的 `which` 高速寻径算法，`cap:which` 提供了工业级跨平台可执行二进制文件定位与可执行权限检测支持：

#### 12.15.1 核心特性

- **跨平台多模式解析**：
  - 绝对路径检测：若为绝对路径，直接校验目标文件是否存在且具备可执行权限；
  - 相对路径检测：若路径包含 `/` 或 `\`，根据指定的 `cwd` 目录就地拼接并校验；
  - `PATH` 环境变量多段解析：按操作系统分隔符（POSIX 为 `:`，Windows 为 `;`）分词并按优先级遍历；
- **Windows 扩展名智能推导**：未显式提供后缀时自动轮询 `.exe`、`.cmd`、`.bat`；
- **零动态堆分配**：支持传入栈缓冲区 `out_buf: []u8`，就地完成路径拼接与切片返回。

#### 12.15.2 完整使用示例

```typescript
import { which, isExecutable, WhichOptions } from "cap:which";

// 1. 查找系统二进制程序（自动遍历 PATH）
let buf: [512]u8 = undefined;
const sh_path = which("sh", .{}, &buf);
if (p := sh_path) {
    // 命中 "/bin/sh" 或 "/usr/bin/sh"
    const can_run = isExecutable(p); // true
}

// 2. 自定义 PATH 与工作目录
let custom_buf: [512]u8 = undefined;
const node_path = which("node", .{
    .path = "/opt/homebrew/bin:/usr/local/bin",
    .cwd = "/app/server",
}, &custom_buf);
```

---

### 12.16 RFC 6455 现代 WebSocket 协议帧处理（`cap:ws`） 【已实装】

吸收自 Bun 的底层网络帧模型，`cap:ws` 提供了纯 CP 编写的高性能 WebSocket 帧协议编解码、掩码变换与控制帧构建库：

#### 12.16.1 核心组件

- **`Opcode` 枚举**：完备支持 `Continue (0x0)`、`Text (0x1)`、`Binary (0x2)`、`Close (0x8)`、`Ping (0x9)`、`Pong (0xA)`，提供 `isControl()` 与 `isData()`；
- **`CloseCode` 状态码**：标准 RFC 6455 关停状态码（`NormalClosure=1000`、`GoingAway=1001`、`ProtocolError=1002` 等）；
- **`FrameHeader`**：支持 7 位基础载荷长度、16 位中型载荷扩展与 64 位超大型载荷扩展，完整支持客户端 4 字节掩码 `mask_key` 编解码；
- **`Mask.apply(data, key)`**：RFC 6455 标准 4 字节 XOR 掩码编解码；
- **`FrameBuilder`**：提供 `buildText()`、`buildBinary()`、`buildPing()`、`buildPong()` 与 `buildClose()`。

#### 12.16.2 完整使用示例

```typescript
import { Opcode, CloseCode, FrameHeader, FrameBuilder, Mask } from "cap:ws";

// 1. 构建带掩码的客户端文本帧
let frame_buf: [256]u8 = undefined;
const mask_key = [0x12, 0x34, 0x56, 0x78];
const frame_len = FrameBuilder.buildText(&frame_buf, "Hello WebSocket!", mask_key)?;

// 2. 解码服务端收到的数据帧头部
const header = FrameHeader.decode(frame_buf[0..frame_len])?;
const is_text = header.opcode == Opcode.Text; // true
const is_masked = header.masked;              // true
const payload_len = header.payload_len;       // 16

// 3. 解除掩码还原数据
let payload: [64]u8 = [0 as u8; 64];
@memcpy(payload[0..payload_len], frame_buf[header.header_len .. frame_len]);
Mask.apply(payload[0..payload_len], header.mask_key);
// payload 即为原始字符串 "Hello WebSocket!"

// 4. 构建心跳 Ping 与优雅断开 Close 帧
let ping_buf: [64]u8 = undefined;
const ping_len = FrameBuilder.buildPing(&ping_buf, "heartbeat_ping", null)?;

let close_buf: [64]u8 = undefined;
const close_len = FrameBuilder.buildClose(&close_buf, .NormalClosure, "server shutting down", null)?;
```

---

### 12.17 文件系统工业级增强能力（`cap:fs` 扩充） 【已实装】

吸收自 Bun 的系统文件高可用与零拷贝设计，`cap:fs` 补充了工业级临时文件管理与原子写入能力：

- **`tmpDir()`**：跨平台临时目录自动嗅探（按序解析 `TMPDIR`, `TEMP`, `TMP`，POSIX 回退 `/tmp`，Windows 回退 `C:\Temp`）；
- **`tmpPath(prefix, suffix, dest_buf)`**：结合单调时钟纳秒时间戳与高并发原子递增计数器，生成绝对无冲突的临时文件路径；
- **`writeAtomic(dest_path, content)`**：先向同目录下隐秘临时文件写入全部数据，完成后通过操作系统级原子重命名（Atomic Rename）覆盖目标文件，杜绝写入中断导致的文件数据损毁。

#### 12.17.1 完整使用示例

```typescript
import { tmpDir, tmpPath, writeAtomic, readFileSync, remove } from "cap:fs";

// 1. 嗅探系统临时目录并生成唯一临时文件路径
const temp_dir = tmpDir(); // "/tmp"
let path_buf: [512]u8 = undefined;
const unique_file = tmpPath("agent_task", ".json", &path_buf);

// 2. 崩溃安全的原子写入（避免写入半途断电破坏旧文件）
writeAtomic("/data/app/config.json", "{\"status\": \"active\"}");

// 3. 读取已就绪的配置并清理
const data = readFileSync("/data/app/config.json");
```

---

### 12.18 工业级现代密码学体系（`cap:crypto`） 【已实装】

CP 语言基于底层操作系统高熵随机源（Linux `getrandom`）与硬件加速加密引擎，在 `cap:crypto` 模块中提供了开箱即用、内存安全的现代化工业级密码学基础设施：

#### 12.18.1 核心组件与特性矩阵

1. **安全真随机数与 UUID 生成 (`random.cp`)**：
   - `randomBytes(size: usize): []const u8`：直通操作系统 CSPRNG 内核熵池；
   - `randomUUID(): []const u8`：RFC 4122 v4 标准全局唯一标识符（如 `f47ac10b-58cc-4372-a567-0e02b2c3d479`）；
   - `randomInt(min: i64, max: i64): i64`：无偏高精度加密随机整数。
2. **密码学安全散列（SHA-256 / SHA-512 / SHA-1 / MD5，`hash.cp`）**：
   - 极速单次计算：`sha256(data)`、`sha512(data)`、`sha1(data)`、`md5(data)` 直接返回标准十六进制字符串；
   - 流式增量散列：`Hash` 类，支持多阶段 `.update(chunk)` 与最终 `.digest()` 输出。
3. **HMAC 消息认证码 (`hmac.cp`)**：
   - `hmacSha256(key, data)`：标准化 SHA-256 HMAC 摘要；
   - 流式 `Hmac` 类：支持流式消息签名校验。
4. **PBKDF2 口令密钥派生 (`pbkdf2.cp`)**：
   - `pbkdf2Sync(password, salt, iterations, keylen, algo)`：输出十六进制格式派生密钥；
   - `pbkdf2Raw(password, salt, iterations, out_key, algo)`：零拷贝直接写入目标原生切片。
5. **现代 AEAD 对称加密与解密 (`cipher.cp`)**：
   - `Aes256Gcm`：NIST SP 800-38D 标准 Galois/Counter Mode，兼具保密性与密文完整性校验；
   - `ChaCha20Poly1305`：RFC 8439 高性能移动端/跨平台认证加密，抗侧信道时序攻击。

#### 12.18.2 完整使用示例

```typescript
import {
    randomBytes,
    randomUUID,
    randomInt,
    sha256,
    sha512,
    createHash,
    hmacSha256,
    createHmac,
    pbkdf2Sync,
    Aes256Gcm,
    ChaCha20Poly1305
} from "cap:crypto";

// 1. 安全随机数与 UUID v4 生成
const rand_token = randomBytes(16);
const trace_id = randomUUID();
const secret_pin = randomInt(100000, 999999);

// 2. 密码学哈希计算（单次与流式）
const hash_hex = sha256("hello cp crypto");

// 流式计算大文件 / 数据流哈希
const hasher = createHash("sha256").?;
hasher.update("chunk_1_data");
hasher.update("chunk_2_data");
const stream_digest = hasher.digest();

// 3. HMAC 消息签名与校验
const secret_key = "cp-super-secret-key";
const signature = hmacSha256(secret_key, "request_payload");

// 4. PBKDF2 密码派生安全密匙
const derived_key = pbkdf2Sync("user_password", "random_salt", 1000, 32, "sha256");

// 5. 现代 AEAD 加密与认证解密 (AES-256-GCM / ChaCha20-Poly1305)
let key: [32]u8 = undefined;
let nonce: [12]u8 = undefined;
for (i of 0..32) key[i] = 0x42;
for (i of 0..12) nonce[i] = 0x01;

const plaintext = "Sensitive Mission Data";
let ciphertext: [64]u8 = undefined;
let tag: [16]u8 = undefined;

// 加密并生成 16 字节完整性认证标签 (Tag)
const ct_slice = Aes256Gcm.encrypt(&key, &nonce, plaintext, "", ciphertext[0..plaintext.len], &tag);

// 解密并校验认证标签（若密文被篡改则立即返回 false，杜绝填充攻击）
let decrypted: [64]u8 = undefined;
const ok = Aes256Gcm.decrypt(&key, &nonce, ct_slice, &tag, "", decrypted[0..ct_slice.len]);
```

---

### 12.19 工业级统一事件循环体系（`cap:event_loop`） 【已全面实装】

CP 语言事件循环移植并融合了 Bun.js 与 Node.js 极速内核，提供基于侵入式二叉最小堆的高性能定时器、16 字节跨线程无锁并发任务队列、基于上下文指针去重的微任务队列（Deferred Microtasks），以及高并发系统文件描述符轮询引擎（`PollEngine`），由 `EventLoop` 类实现统一生命周期驱动。

#### 12.19.1 核心组件与设计特性

1. **侵入式二叉最小堆定时器 (`Timer` & `TimerHeap`)**：
   - 节点内嵌堆索引（`heap_idx: usize`），实现 $O(\log N)$ 插入与到期弹出，以及 $O(\log N)$ 任意定时器即时注销（`remove`），彻底消灭扫描查找开销。
   - 区分单次定时器（`setTimeout`）与高精度循环定时器（`setInterval`），支持毫秒级到期批量排出（`drainExpired`）。
2. **16 字节跨线程无锁并发任务队列 (`ConcurrentTask` & `ConcurrentQueue`)**：
   - 任务节点仅占 16 字节（包含紧凑的回调指针与上下文指针），支持 Worker 线程与 I/O 线程极速向主事件循环投递跨线程任务（`postTask`）。
   - 采用互斥自旋锁守卫单向链表，支持批量单轮排出（`drain`），支持任务执行后自动回收（`auto_delete`）。
3. **微任务队列与上下文去重 (`DeferredTaskQueue`)**：
   - 严格遵循现代 JavaScript/Bun 微任务语义，在每轮 I/O 睡眠前后优先排空微任务。
   - 具备唯一的上下文指针去重能力（Deduplication）：针对相同上下文的重复排队请求，原地更新回调并抑制重复插入，消除事件风暴。
4. **I/O 文件描述符轮询引擎 (`PollEngine`)**：
   - 基于原生 `std.posix.poll` 封装，提供 `Readable`, `Writable`, `Error`, `Hup` 事件位掩码。
   - 具备动态动态扩容缓冲区，支持描述符注册、更新与 $O(1)$ 尾部交换注销。
5. **统一事件循环驱动 (`EventLoop`)**：
   - 单轮步进模式：`tick(timeout_override)`，按阶段顺序执行跨线程任务 -> 微任务 -> 计算定时器最短等待 -> I/O 轮询睡眠 -> 到期定时器派发 -> 尾部微任务排空。
   - 常驻运行模式：`run()` 与 `stop()`，自动监测是否有存活句柄（`hasActiveHandles()`）。

#### 12.19.2 完整使用示例

```typescript
import {
    EventLoop,
    Timer,
    TimerHeap,
    ConcurrentQueue,
    DeferredTaskQueue,
    PollEngine,
    PollEvents
} from "cap:event_loop";

// 1. 初始化统一事件循环
let loop = EventLoop.init();

class AppState {
    pub count: i32 = 0;
    pub static fn create(): *AppState {
        let p = std.heap.page_allocator.create(AppState) catch unreachable;
        p.count = 0;
        return p;
    }
}

let state = AppState.create();
const state_ptr: *anyopaque = @ptrCast(state);

// 2. 投递微任务 (自动按 state_ptr 去重)
loop.deferTask((ctx: ?*anyopaque) => {
    let s: *AppState = @ptrCast(@alignCast(ctx.?));
    s.count += 10;
}, state_ptr);

// 3. 注册定时器 (10ms 后触发)
const t = loop.setTimeout((ctx: ?*anyopaque) => {
    let s: *AppState = @ptrCast(@alignCast(ctx.?));
    s.count += 1;
}, 10, state_ptr);

// 4. 注册文件描述符 I/O 监听
let pipe_fds: [2]i32 = undefined;
__zig__ {
    _ = std.os.linux.pipe(&pipe_fds);
}
loop.watch(pipe_fds[0], PollEvents.Readable, (fd: i32, events: u32, ctx: ?*anyopaque) => {
    let s: *AppState = @ptrCast(@alignCast(ctx.?));
    s.count += 100;
}, state_ptr);

// 5. 单步轮转驱动
_ = loop.tick(0); // 触发微任务: count -> 10
TimerHeap.sleepMs(15);
_ = loop.tick(0); // 定时器到期触发: count -> 11

// 6. 清理退出
_ = loop.clearTimeout(t);
_ = loop.unwatch(pipe_fds[0]);
```

---

### 12.20 安全防护与防跨站请求伪造子系统（`cap:security`） 【已全面实装】

针对现代 Web 与分布式智能体微服务的跨站请求伪造（CSRF）防护需求，CP 原生实装高可靠、零时序漏洞的 `cap:security` 安全库。

#### 1. 核心架构设计与防护原理
1. **密码学 Nonce 与时间戳**：结合 Linux 熵池 CSPRNG 生成 16 字节不可预测随机数，并在载荷内嵌入纳秒/毫秒时间戳与失效阈值，严格防御重放攻击；
2. **HMAC-SHA256 签名与防篡改**：载荷经由应用密钥实施 HMAC-SHA256 签名，服务端验证时若密文被篡改 1 比特即刻拒绝；
3. **恒定时间比对（Constant-Time Eql）**：校验过程通过 `constantTimeEql` 逐字节异或累加比较，执行时长与输入匹配程度无关，彻底免疫时序侧信道攻击；
4. **多格式编解码支持**：原生支持 `Base64Url`（URL 安全、免 Cookie 转义）、`Base64` 与 `Hex` 十六进制三种主流工业形态。

#### 2. 核心 API 契约与使用示例
```typescript
import { CSRF, TokenFormat, DEFAULT_EXPIRATION_MS, constantTimeEql } from "cap:security";

const secret = "app_secret_signing_key_2026";
const allocator = std.heap.page_allocator;

// 1. 生成 Base64Url 格式 CSRF Token
const token = CSRF.generate(secret, DEFAULT_EXPIRATION_MS, TokenFormat.Base64Url, allocator)?;
defer allocator.free(token);

// 2. 校验传入的 Token
const is_valid = CSRF.verify(token, secret, DEFAULT_EXPIRATION_MS, TokenFormat.Base64Url, allocator);
if (!is_valid) {
    return error.ForbiddenCSRF;
}

// 3. 恒定时间缓冲区对比
const equal = constantTimeEql(token_a, token_b);
```

---

### 12.21 生态演进与路线图（Roadmap）

```mermaid
timeline
    title CP-Nest 生态演进全景路线图
    Milestone 1 : 核心语言基座 (Completed) : 所有权与 NLL 借用检查 : 3-State 原生错误体系 : 通用通配符 * 与解构 ... : 双轨派生 spawn / spawn.cpu : 对象解构 ...rest 剩余绑定
    Milestone 2 : 编译器前端强化 (In Progress) : 泛型多重 Trait 约束强化 : 管道运算符 * 语法糖增强
    Milestone 3 : Nest 容器原生融合 (Next) : 动态 .so NIF 热加载 : 分布式智能体跨节点信箱路由 : SharedTensor 跨 Agent 零拷贝直通
    Milestone 4 : 工业级生态成熟 (Future) : 完整异步网络驱动 : 宏展开卫生性校验 (Hygienic Macros) : 包管理中心 Registry
```

---

## 十三、编译器工具链与 CLI 指令规范（`cpc` & `cap`）

### 13.1 `cpc` 独立编译器前端

`cpc`（CP Compiler）作为原生静态编译器前端，默认执行编译动作，无需多余的子命令：

```bash
# 1. 默认编译为 Zig 源码（直接指定输出）
cpc src/url.cp -o src/url.zig

# 2. 编译为 Zig 独立模块/库（省略 main 入口，所有 export 自动赋予 pub）
cpc src/url.cp -o src/url.zig --lib

# 3. 极速语法与借用安全检查（5ms 静态拦截，不生成中间文件）
cpc check src/url.cp

# 4. 生成 .d.cp 类型声明契约文件
cpc emit-types src/url.cp -o src/url.d.cp
```

### 13.2 原生逃逸舱类型映射与参数引用优化

在与 Zig 原生代码深度混编时：

1. **原生类型声明自动识别**：在 `zig { ... }` 块中声明的 `pub const T = ...` 或 `const T = ...` 常量别名，编译器会自动将其纳入值类型白名单。在 CP 函数签名中引用时直接发射原生类型标识符 `T`，而不是强加堆指针 `*T`；
2. **未使用参数精确抑制**：函数参数在函数体内或内嵌 `zig { ... }` 原生块中已被引用时，编译器不再发射冗余的 `_ = &param;` 取地址操作；仅在参数真正未被任何代码使用时发射符合 Zig 官方规范的 `_ = param;`。

### 13.3 结构体扩展逃逸舱、多级命名空间类型与标准 TS 静态结构体字面量

1. **类与结构体内部内嵌 `zig { ... }`**：
   允许在 `class` 或 `@value struct` 作用域内部直接内嵌 `zig { ... }` 原生逃逸舱，用于声明紧密绑定的内部引用计数器（如 `RefCount`）、常量查找表或底层 JSC 回调（如 `pub const toJS = ...;`）。编译器自动提取并在类结构体内无缝发射。
2. **标准 TS 静态结构体字面量（`{}` / `{ key: val }`）与符号职责清晰划分**：
   - **静态结构体**：遵循纯正 TypeScript 现代语法，直接使用 `{}`（默认空结构体）或 `{ key: val }`（匿名紧凑结构体），转译发射为原生紧凑的 Zig 结构体字面量（`.{}` / `.{ .key = val }`），零语法心智负担；
   - **点前缀 `.`**：**严格专属于上下文主体推断与成员省略**（如 `.Variant` 枚举标签推断、`.method()` 管道主体调用），严禁作为结构体标记，杜绝与上下文主体推断产生语义二义性；
   - **动态字典**：使用 `#{ key: val }`（专属于堆分配哈希字典）。
3. **多级命名空间点路径类型（`Dotted Path Types`）**：
   支持在类型注解中使用多级命名空间类型（如 `jsc.ZigString.Slice`、`std.mem.Allocator` 等），编译器自动识别其为外部原生值类型并保留完整路径，杜绝错误附加 `*` 指针前缀。
4. **推导错误联合后缀语法（`T!`）**：
   函数返回推导错误联合时，统一采用后缀语法 `T!`（如 `fn parse(): usize!`、`fn write(): void!`），正常返回类型 `T` 永远在最前，错误由编译器自动收集，与 `T?`（可空）及 `T ! ErrorSet`（显式错误集）构建完全自洽统一的后缀类型修饰符体系。
5. **`void!` 正交省略语法（`fn sum()!` 与 `fn sum(): !`）**：
   对于无成功返回值且可能报错的函数，允许在函数签名处省略 `void`，直接保留 `!` 标记：`fn sum()!` 或 `fn sum(): !`，等价于 `fn sum(): void!`，转译为 Zig 的 `!void`。严禁无标记隐式推导，必须在签名处显式保留 `!` 锁定失败契约，保障 API 强安全与编译器 O(1) 极速解析。
6. **全量确立纯后缀修饰符体系，彻底废除前缀 `?T`**：
   可选类型统一采用纯正 TypeScript 现代风格后缀 `T?`（如 `let s: string? = null;`）；当函数返回同时可能为 `null` 且可能发生错误时，统一采用纯正交链式后缀 `T?!`（如 `fn find(): User?!`、`fn get(): string?!`），底层自动映射为 Zig 原生错误联合 `!?T`；带显式错误集时写作 `T? ! ErrorSet`。编译器已在语义与语法层面彻底废除前缀 `?T`，静态拦截任何前置问号声明并明确报错，杜绝三明治前后包夹带来的混乱，确立“类型主体永远居首、修饰状态纯后缀链式向右展开”的全局统一规范。
7. **原生 Zig 错误集（`error{ ... }`）互操作规范**：
   在与底层 Zig 强类型错误集深度互操作时，Zig 的 `error{ ... }` 属于原生的错误枚举集合（Error Sets），与 CP 默认发射的 Tagged Union（`union(enum)`）存在语义差异。当需要向下游 Zig 消费方导出严格兼容的错误联合（如 `InitError!void`）时，可通过在 `zig { ... }` 原生逃逸舱中声明 `pub const InitError = error{ ... };`，确保与底层系统调用的类型系统与错误传播 100% 契合。

### 13.4 `--lib` 静态库模式死代码消除（DCE）与零开销原生转译

在通过 `cpc src/foo.cp -o src/foo.zig --lib` 构建供 Zig 宿主模块调用的库时，编译器全面启用模式感知的死代码消除与原生直通机制：

1. **原生操作符直通**：
   在 `--lib` 模式下，二元运算符（`==`、`!=`、`<`、`+`）与逻辑非（`!`）直接发射为 Zig 原生操作符，绝不发射未调用的通用运行时辅助函数（`cp_eq`、`cp_lt`、`cp_not`、`cp_add`），从源头上杜绝数十至上百行的 Prelude 模板代码膨胀。
2. **切片原生遍历（Native Slice Iteration）**：
   针对 `for (item of container)` 循环，若容器为已知切片属性（如 `.items`、`.values()`），编译器自动生成零开销的 Zig 原生 `for (container) |item|` 循环，无需注入 `cp_iter_slice` / `cp_mut_slice` 迭代器封装。
3. **空值合并与短路返回原生对齐**：
   针对 `a ?? b`（空值合并）与 `expr ? return null`（短路解包），在 `--lib` 模式下直接发射为标准的 Zig 原生 `(a orelse b)` 与 `((expr) orelse { return null; })`，无需任何辅助函数。
4. **辅助函数精细按需摇树**：
   将切片索引查找辅助（`indexOf`、`lastIndexOf`、`includes`）与切片迭代解耦，仅在实际存在该方法调用时才按需发射，保证生成的 `.zig` 源码紧凑、纯粹，膨胀率严格受控在 10% 以内，部分模块代码量甚至少于原始手写 Zig。

---

## 十四、 符号总表

### 14.1 基础标点符号

| 符号 | 名称   | 官方用途                                                     |
| ---- | ------ | ------------------------------------------------------------ |
| `()` | 小括号 | 函数定义/调用、参数包裹、运算优先级提升、类型强制转换        |
| `[]` | 中括号 | 数组定义、数组下标取值、切片声明；`@[]` 注解标记             |
| `{}` | 大括号 | 代码块包裹、结构体/对象字面量、作用域界定                    |
| `;`  | 分号   | 语句结束终止符                                               |
| `,`  | 逗号   | 参数分隔、数组元素分隔、多变量定义分隔                       |
| `.`  | 点号   | 对象属性访问、结构体成员调用；管道内部隐式值                 |
| `:`  | 冒号   | 类型标注、键值对分隔、函数返回类型声明                       |
| `_`  | 下划线 | 方法内 `this` 的简写代称；标识符命名分隔；**不是模式通配符** |
| `@`  | at符号 | 装饰器、编译期函数调用前缀；注解。 `@[]`聚合                 |

### 14.2 运算符体系

#### 14.2.1 算术运算符

| 符号 | 用途                                              |
| ---- | ------------------------------------------------- |
| `+`  | 加法、字符串拼接                                  |
| `-`  | 减法、负数标识                                    |
| `*`  | 乘法；可变借用类型标记；裸指针前缀；switch 通配符 |
| `/`  | 除法                                              |
| `%`  | 取模、余数运算                                    |

#### 14.2.2 赋值运算符

| 符号 | 用途     |
| ---- | -------- |
| `=`  | 基础赋值 |
| `+=` | 自增赋值 |
| `-=` | 自减赋值 |
| `*=` | 自乘赋值 |
| `/=` | 自除赋值 |
| `%=` | 自模赋值 |

#### 14.2.3 比较运算符

```text
| 符号     | 用途                |
| -------- | ------------------- |
| `==`     | 绝对相等（值+类型） |
| `&#33;=`     | 绝对不相等          |
| `>` `&#62;=` | 大于、大于等于      |
| `<` `<&#60;=` | 小于、小于等于      |
```

#### 14.2.4 逻辑运算符

| 符号 | 用途                             |
| ---- | -------------------------------- |
| `&&` | 逻辑与（短路）                   |
| `    |                                  |
| `!`  | 逻辑非；错误联合类型分隔符 `T!E` |

#### 2.5 位运算符

| 符号      | 用途                 |
| --------- | -------------------- |
| `&`       | 按位与；只读借用标记 |
| `^`       | 按位异或             |
| `~`       | 按位取反             |
| `<<` `>>` | 左移位、右移位       |

#### 2.6 CP 特色语法运算符

| 符号  | 官方名称 | 唯一用途                                                              |
| ----- | -------- | --------------------------------------------------------------------- |
| `=>`  | 箭头符   | 匿名函数、闭包定义；switch 分支箭头                                   |
| `     | >`       | 管道符                                                                |
| `?.`  | 可选链   | null 安全属性访问                                                     |
| `?`   | 问号     | 1. 类型后缀 `T?`：可选类型，可存 null 2. 表达式尾部 `expr?`：错误冒泡 |
| `??`  | 双问号   | null 兜底，**只处理 null，不捕获错误**                                |
| `...` | 扩展符   | 数组/对象解构、参数展开                                               |

### 14.3 代码块特殊标记符

| 语法        | 用途规范                                 |
| ----------- | ---------------------------------------- |
| `unsafe {}` | 安全逃逸块：关闭借用检查，允许裸指针操作 |
| `@c {}`     | C-ABI 声明块：声明外部 C 符号与库函数    |
| `zig {}`    | 原生Zig内嵌块：直接编写原生Zig代码       |
| `//`        | 单行注释                                 |
| `/* */`     | 多行块注释                               |

### 14.4 所有权、借用、裸指针、错误体系符号

| 符号写法        | 核心语义                                                    |
| --------------- | ----------------------------------------------------------- |
| `&T`            | 只读借用，只读、共享                                        |
| `*T`            | 可变借用，独占修改（主推；`&mut T` 保留为兼容备选，不推荐） |
| `*T!`           | 一级裸指针类型，仅在 unsafe 内合法                          |
| `**T!`          | 二级裸指针类型，仅在 unsafe 内合法                          |
| `*any!`         | 通用非类型化裸指针 (映射至 `*anyopaque`)                    |
| `*any?!`        | 可空通用非类型化裸指针 (映射至 `?*anyopaque`)               |
| `[*]T!`         | 未定长多项指针 (映射至 Zig `[*]T`)                          |
| `cstr`          | 0 结尾 C 字符串 (映射至 `[*:0]const u8`)                    |
| `cstr?`         | 可空 0 结尾 C 字符串 (映射至 `?[*:0]const u8`)              |
| `T!E`           | 错误联合类型：正常为 T，错误为 E                            |
| `Err(...)`      | 构造错误值，用于返回、switch 错误分支匹配                   |
| `*`（switch内） | switch 模式通配符                                           |
| `null`          | 字面量，匹配可选类型空值                                    |

### 14.5 类型与泛型符号

| 符号              | 用途                              |
| ----------------- | --------------------------------- |
| `<T>`             | 泛型类型参数包裹                  |
| `<const name: T>` | 常量泛型参数包裹 (编译期值单态化) |
| `[]`              | 动态数组 / 切片类型               |

### 14.6 编译期、装饰器、注解

| 符号写法    | 用途                               |
| ----------- | ---------------------------------- |
| `@name`     | 注解、装饰器 / 编译期函数调用      |
| `@c`        | C-ABI 函数导出与函数指针类型修饰符 |
| `@align(N)` | 字段级物理内存字节对齐注解         |
| `@[...]`    | 聚合                               |

---

### 14.7 符号使用强制规范（新增订正条目）

1. 管道符固定为 `|>`（竖线 + 大于号）。
2. 裸指针规则：`!` 固定放在类型最后，一级 `*T!`，二级 `**T!`，末尾只一个 `!`。
3. 借用优先使用 `&T`（只读）、`*T`（可变借用）。
4. `_` 不是通配符！`_` 在方法内等价于 `this`；switch 通配符固定为 `*`。
5. `?` 存在两个语义位置：类型后缀 `T?` = 可选（null）；表达式后缀 `?` = 错误冒泡。
6. `??` 只处理 null，不处理错误；错误兜底不能用 `??`。
7. 错误联合 `T!E`，错误值使用 `err(...)` 构造；switch 里匹配错误分支写 `err(xxx)`。
8. 模式匹配只使用 `switch`。
9. switch 分支：普通值直接写、`null` 匹配空、`error(xxx)` 匹配错误分支。
10. 单项指针 `*T!` 严禁指针算术（`p + 1` 编译期报错）；未定长多项内存连续操作统一使用 `[*]T!`。

---

### 14.8 C-FFI 声明与 C 指针原生映射规范

> [!IMPORTANT]
> **外部 C 函数声明与引用铁律**：
>
> 1. CP 语言中原生内嵌 Zig 代码块必须使用 `zig {}`，**严禁使用带有下划线前缀的 `__zig__ {}`**；
> 2. **CP 中不应该使用 `zig {}` 声明外部 C 函数**，而应该直接使用原生的 `@c`、`@c("alias")` 与 `@c { ... }` 原语；
> 3. `@c` 具备强大的符号重命名功能，可在导入时直接赋予符合 CP 命名规范的别名；
> 4. CP 中的 unsafe 裸指针（`*T!`、`[*]T!`、`cstr`、`*any!`、`*T!?`）专门用于 C 原生指针体系的高性能无损映射。

#### 1. 极简 C 符号声明 `@c` 与别名重命名

通过 `@c` 消除多重关键字冗余，直接对应 C-ABI 调用约定与外部链接：

```typescript
// 1. 单个外部 C 函数声明（无别名）
@c fn getpid(): i32;

// 2. 带别名的 C 库符号重命名：C 符号原名为 phr_parse_request，在 CP 中重命名为 parse_http_request
@c("phr_parse_request")
fn parse_http_request(
    buf: cstr,
    len: usize,
    method: *cstr!,
    method_len: *usize!,
    path: *cstr!,
    path_len: *usize!,
    minor_version: *i32!,
    headers: [*]phr_header!,
    num_headers: *usize!,
    last_len: usize,
): i32;

// 3. 批量 C 符号声明块 @c { ... }
@c {
    fn malloc(size: usize): *any!;
    fn free(ptr: *any!?): void;
    fn us_create_loop(hint: *any!?, wakeup_cb: *any!?, pre_cb: *any!?, post_cb: *any!?, ext_size: u32): *us_loop_t!?;
    fn us_loop_free(loop: *us_loop_t!?): void;
}

// 4. 导出为 C-ABI 函数供外部或宿主链接
@c export fn cp_add_three(x: i32): i32 {
    return x + 3;
}
```

#### 2. C 内存布局结构体与不透明类型声明

与 C 库交互时，数据布局必须与 C 原生 ABI 严格一致，CP 提供原生的 `@extern struct` 与 `@opaque struct`：

```typescript
// 1. 声明严格遵循 C ABI 内存布局的外部结构体（直接发射为 extern struct）
@c struct phr_header {
    name: cstr,
    name_len: usize,
    value: cstr,
    value_len: usize,
}

// 2. 声明 C 内部隐藏不透明句柄类型（直接发射为 opaque {}）
@opaque struct us_loop_t {}
```

#### 3. C 原生指针体系映射与安全铁律

CP 设计了一套与 C/Zig 物理 ABI 100% 对齐的 unsafe 裸指针体系，且严格遵循纯后缀链式修饰：

| CP 裸指针类型 | 底层物理映射 (Zig) | 语义说明与安全约束                                                  |
| :------------ | :----------------- | :------------------------------------------------------------------ |
| `cstr`        | `[*:0]const u8`    | 以 `\0` 结尾的只读 C 字符串指针。支持 `.len`，可零开销转 `string`   |
| `cstr?`       | `?[*:0]const u8`   | 可空 C 字符串指针                                                   |
| `*T!`         | `*T`               | 一级 C 裸指针（仅在 unsafe/FFI 中合法，**严禁指针算术**，杜绝越界） |
| `**T!`        | `**T`              | 二级 C 裸指针                                                       |
| `[*]T!`       | `[*]T`             | 未定长连续内存多项指针，支持下标 `ptr[i]` 与切片 `ptr[a..b]`        |
| `*any!`       | `*anyopaque`       | 通用非类型化裸指针（等价于 C 的 `void*`）                           |
| `*T!?`        | `?*T`              | 可空一级 C 裸指针（等价于 C 中可为 NULL 的句柄指针）                |
| `*any!?`      | `?*anyopaque`      | 可空通用非类型化裸指针（等价于 C 的 nullable `void*`）              |

```typescript
// 示例：裸指针切片与转换
let cs: cstr = "hello cp";
let n: usize = cs.len; // 8
let s: string = string.from(cs); // "hello cp"

let arr: [4]u8 = [65, 66, 67, 68];
let many: [*]u8! = arr as [*]u8!;
let item = many[1]; // 66
let slice = many[1..3]; // []u8, len = 2
```

---

### 14.9 物理内存对齐注解 `@align(N)`

支持在结构体/类级别以及字段级别直接指定物理内存字节对齐边界：

```typescript
@align(16)
struct AlignedBlock {
    @align(16) val: usize,
    other: u32,
}
```

---

### 14.10 常量泛型类与类型别名

支持在类/结构体定义中引入常量泛型参数（Const Generics），编译器自动在编译期将其展开为单态化（Monomorphize）特化展开类型，彻底消灭动态类型生成逃逸舱：

```typescript
class App<const ssl: bool> {
    is_ssl: bool = ssl;

    fn get_ssl(this): bool {
        return this.is_ssl;
    }
}

// 使用 type 声明类型别名
type AppTCP = App<false>;
type AppSSL = App<true>;

let tcp: AppTCP = AppTCP{};
let ssl_app: AppSSL = AppSSL{};
```

---

### 14.11 编译期反射探测与 `@if` 表达式

支持在编译期直接探测类型的方法与字段存在性，配合 `@if` 编译期三元表达式完成静态跳板函数与适配：

```typescript
// 探测类型是否存在指定方法或字段
let has_open = reflect.hasMethod(Handler, "onOpen"); // true / false
let has_field = reflect.hasField(AlignedBlock, "val"); // true / false
let has_decl = reflect.has(Handler, "onOpen");        // 综合探测

// @if 编译期三元表达式 (cons 与 alt 分支按条件编译期选取)
let branch = @if (reflect.hasMethod(Handler, "onOpen")) 100 else 200;
```

---

### 14.12 枚举内省与双向类型转换

#### 1. 内置内省属性

枚举变量或变体直接支持 `.name` 与 `.value`：

- `e.name`：获取变体的字面量名称字符串（发射为 Zig `@tagName(e)`）。
- `e.value`：获取枚举的数值（发射为 Zig `@intFromEnum(e)`）。

#### 2. 枚举双向转换

- 数值转枚举：支持构造器语法 `TaskState(2)` 或强制转换 `1 as TaskState`。
- 字符串转枚举：支持静态方法 `TaskState.fromName("Idle")`，返回可选类型 `TaskState?`。

```typescript
enum TaskState {
  Idle = 0,
  Running = 1,
  Completed = 2,
}

let st = TaskState.Running;
let name = st.name; // "Running"
let val = st.value; // 1

let from_int = TaskState(2); // TaskState.Completed
let from_as = 1 as TaskState; // TaskState.Running
let from_str = TaskState.fromName("Idle"); // TaskState?, .? 为 TaskState.Idle
```

---

### 14.13 多态字符串转换 `string.from(val)`

提供通用的强类型多态字符串转换原语 `string.from(...)`，统一处理 `cstr`、`enum`、`bool`、数字与带 `toString()` 对象的字符串化转换：

```typescript
let s1 = string.from(TaskState.Running); // "Running"
let s2 = string.from(cs); // cstr -> string
let s3 = string.from(true); // "true"
let s4 = string.from(12345); // "12345"
```

---

## 十五、关键字清单

> 说明：
>
> 1. 关键字是保留字，不可用作变量、函数、结构体、类型名；
> 2. 分为六大类：声明类、类型与值、所有权/安全控制、流程控制、编译期与元编程、特殊块；
> 3. 附带简要语义，方便放进规范、给编译器词法分析器做保留词表。

### 15.1 声明关键字

| 关键字   | 含义说明                                                |
| -------- | ------------------------------------------------------- |
| `let`    | 可变变量绑定声明                                        |
| `const`  | 编译期常量，不可修改                                    |
| `fn`     | 函数定义                                                |
| `struct` | 结构体定义                                              |
| `class`  | 类定义                                                  |
| `enum`   | 枚举类型定义                                            |
| `type`   | 类型别名声明                                            |
| `import` | 导入模块、导入类型/函数                                 |
| `exort`  | 导出模块、导出类型/函数                                 |
| `pub`    | 公开导出，外部模块可访问                                |
| `priv`   | 私有，仅当前模块内可见                                  |
| `Err`    | 错误构造函数，生成错误联合里的错误变体，switch 匹配错误 |

### 15.2 类型与字面关键字

| 关键字  | 含义说明                           |
| ------- | ---------------------------------- |
| `void`  | 空返回类型，可省略                 |
| `bool`  | 布尔类型                           |
| `true`  | 布尔真字面量                       |
| `false` | 布尔假字面量                       |
| `null`  | 空值，配合可空类型 `T?`            |
| `this`  | 方法内当前实例（也可用下划线 `_`） |

### 15.3 所有权、安全与内存控制关键字

| 关键字   | 含义说明                                                                 |
| -------- | ------------------------------------------------------------------------ |
| `own`    | 极简 3 字母独占所有权转移标记（替代繁琐的 `move unique`，杜绝并发别名）  |
| `move`   | 所有权转移标记（与 `own` 等价并兼容）                                     |
| `unsafe` | 不安全代码块，关闭借用检查，允许裸指针操作                               |
| `zig`    | 内嵌 Zig 代码块，直接执行原生 Zig 代码                                   |
| `&`      | 只读借用                                                                 |
| `*`      | 可变借用                                                                 |

### 15.4 流程控制关键字

| 关键字     | 含义说明                           |
| ---------- | ---------------------------------- |
| `if`       | 条件判断                           |
| `else`     | 条件分支，配合 if                  |
| `for`      | 循环（主循环，统一遍历/计数循环）  |
| `while`    | 循环（主循环，统一遍历/计数循环）  |
| `switch`   | 模式匹配分支（CP主推，替代多层if） |
| `break`    | 跳出循环 / switch                  |
| `continue` | 跳过本次循环，进入下一轮           |
| `return`   | 函数返回                           |

### 15.5 异步关键字

| 关键字  | 含义说明       |
| ------- | -------------- |
| `async` | 异步函数       |
| `await` | 等待异步返回值 |

### 15.6 rait 关键字

| 关键字  | 含义说明       |
| ------- | -------------- |
| `trait` | 类似Rust trait |
| `impl`  | 实现 trait     |

### 15.7 编译期 & 元编程关键字

| 关键字      | 含义说明                                      |
| ----------- | --------------------------------------------- |
| `@comptime` | 编译期执行标记；修饰函数/变量，在编译阶段求值 |

### 15.8 保留预留关键字（暂未实现，预留语法扩展，禁止占用）

> 为未来扩展预留，现在不实现，但同样属于保留字，不能作为标识符

- `yield`

---

## 十六、高性能 C 网络反应堆集成：`uSockets` 与 `picohttpparser` 原生架构 (High-Performance C Network Integration)

### 16.1 架构愿景与 Bun.js 级性能底座

CP 语言定位于专为 AI Agent 与海量并发设计的高性能系统语言。在网络 I/O 底层，CP 借鉴了 Bun.js 的极致性能哲学：**废弃庞大笨拙的通用网络抽象，直接在底层集成最顶级的工业级 C 语言网络反应堆与协议解析内核**：

1. **`uSockets`**：超轻量、极速异步事件循环库（uWebSockets 底层内核，也是 Bun.js 网络驱动的核心）。单机可支撑百万级长连接，内存开销比 libuv 更轻、延迟更低；
2. **`picohttpparser`**：基于 SIMD 向量化指令优化的超高速 HTTP/1.x 协议解析器，单核每秒解析超百万次 HTTP 请求，零动态内存分配（Zero Allocation）。

```mermaid
flowchart TD
    subgraph CP_App_Layer ["CP 业务与 Agent 层"]
        CP_Agent["AI Agent / HTTP Service"]
    end

    subgraph CP_FFI_Bridge ["CP 原生 C-ABI 映射层 (零抽象开销)"]
        AtC["@c('phr_parse_request') & @c { ... }"]
        Pointers["*T! / [*]T! / cstr / *any!? (纯后缀裸指针)"]
        ExternStruct["@extern struct phr_header & @opaque struct us_loop_t"]
    end

    subgraph Native_C_Core ["底层 C 原生静态库 (zig-out/lib/libusockets.a)"]
        uSockets["uSockets (Loop / Socket / Epoll)"]
        PicoHTTP["picohttpparser (SIMD HTTP Parser)"]
        Stubs["cp_stubs.c (Bun 原生适配桩)"]
    end

    CP_Agent --> CP_FFI_Bridge
    CP_FFI_Bridge --> Native_C_Core
```

---

### 16.2 Zig 0.17 现代构建管线：根目录 `build.zig` 配置规范

为了让 Zig 构建系统完全掌控底层 C 编译管线，CP 在项目根目录下提供现代化的 `build.zig`，适配 Zig 0.17 的 Module 构建规范：

```zig
// build.zig (放置在项目根目录)
const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    // 1. 创建底层 C 静态模块
    const usockets_mod = b.createModule(.{
        .target = target,
        .optimize = optimize,
    });
    usockets_mod.addIncludePath(b.path("vendor/usockets/src"));
    usockets_mod.addIncludePath(b.path("vendor/picohttpparser"));
    usockets_mod.addCSourceFiles(.{
        .files = &.{
            "vendor/usockets/src/bsd.c",
            "vendor/usockets/src/context.c",
            "vendor/usockets/src/loop.c",
            "vendor/usockets/src/socket.c",
            "vendor/usockets/src/eventing/epoll_kqueue.c",
            "vendor/usockets/src/cp_stubs.c", // 消除 Bun 运行时钩子未定义符号
            "vendor/picohttpparser/picohttpparser.c",
        },
        .flags = &.{
            "-DLIBUS_NO_SSL",
            "-O3",
            "-fno-exceptions",
        },
    });

    // 2. 构建并安装静态库 libusockets.a 与 C 头文件
    const lib = b.addLibrary(.{
        .linkage = .static,
        .name = "usockets",
        .root_module = usockets_mod,
    });
    lib.linkLibC();
    b.installArtifact(lib);
    b.installFile("vendor/usockets/src/libusockets.h", "include/libusockets.h");
    b.installFile("vendor/picohttpparser/picohttpparser.h", "include/picohttpparser.h");

    // 3. 支持 -Dcp-entry 快速编译运行 CP 转译产物
    const cp_entry = b.option([]const u8, "cp-entry", "Path to CP transpiled Zig entry file");
    if (cp_entry) |entry_path| {
        const exe_mod = b.createModule(.{
            .root_source_file = b.path(entry_path),
            .target = target,
            .optimize = optimize,
        });
        exe_mod.addIncludePath(b.path("vendor/usockets/src"));
        exe_mod.addIncludePath(b.path("vendor/picohttpparser"));

        const exe = b.addExecutable(.{
            .name = "cp_app",
            .root_module = exe_mod,
        });
        exe.linkLibrary(lib);
        exe.linkLibC();

        const run_cmd = b.addRunArtifact(exe);
        if (b.args) |args| run_cmd.addArgs(args);
        const run_step = b.step("run", "Run the CP executable with uSockets");
        run_step.dependOn(&run_cmd.step);
    }
}
```

---

### 16.3 CP 原生 `@c` 导入规范（彻底废除 `zig {}` 声明外部 C 符号）

> [!IMPORTANT]
> **语法规范铁律**：
>
> 1. **严禁在 CP 中使用 `zig {}` 声明外部 C 符号与函数原型**；
> 2. **严禁使用 `__zig__`**；
> 3. 所有 C 函数原型必须使用 CP 原生 `@c`、`@c("alias")` 或批量 `@c { ... }` 导入；
> 4. 使用 `@c struct`（或语义等价的 `@fix struct`，向下兼容 `@extern struct`）保证与 C 物理结构体对齐，使用 `@opaque struct` 保证不透明句柄安全；
> 5. 统一使用 CP unsafe 裸指针（`*T!`、`[*]T!`、`cstr`、`*any!`、`*T!?`、`*any!?`）完成精准映射。

#### 1. 结构体与 C 不透明类型声明

- **`@c struct`**：直观声明该结构体严格遵循 C-ABI 内存物理对齐规则（映射为 Zig 的 `extern struct`）；
- **`@fix struct`**：等价别名，突出“固定内存排布、禁止重排（Fixed Layout）”语义；
- **`@extern struct`**：完全保留向下兼容；
- **`@opaque struct`**：用于声明 C 原生指针所指向的无大小不透明结构体（映射为 Zig 的 `opaque {}`）。

```typescript
// C 内存对齐外部结构体（推荐使用 @c struct 或 @fix struct）
@c struct phr_header {
    name: cstr,
    name_len: usize,
    value: cstr,
    value_len: usize,
}

// C 不透明事件循环句柄 (opaque {})
@opaque struct us_loop_t {}
```

#### 2. C 函数原生导入与符号重命名

```typescript
// 1. 通过 @c("alias") 导入并赋予 CP 风格别名：
// C 链接符号为 phr_parse_request，在 CP 中直接调用 phr_parse_req
@c("phr_parse_request")
fn phr_parse_req(
    buf: cstr,
    len: usize,
    method: *cstr!,
    method_len: *usize!,
    path: *cstr!,
    path_len: *usize!,
    minor_version: *i32!,
    headers: [*]phr_header!,
    num_headers: *usize!,
    last_len: usize,
): i32;

// 2. 通过 @c { ... } 批量导入 C 函数原型（参数使用后缀可选裸指针 *T!?）
@c {
    fn us_create_loop(
        hint: *any!?,
        wakeup_cb: *any!?,
        pre_cb: *any!?,
        post_cb: *any!?,
        ext_size: u32,
    ): *us_loop_t!?;

    fn us_loop_free(loop: *us_loop_t!?): void;
}
```

---

### 16.4 工业级端到端集成测试（`cap/test/test_usockets_picohttp.cp`）

```typescript
// cap/test/test_usockets_picohttp.cp
// 端到端验证 CP 原生 @c / @c("alias") 与 unsafe 裸指针互操作体系

@c struct phr_header {
    name: cstr,
    name_len: usize,
    value: cstr,
    value_len: usize,
}

@opaque struct us_loop_t {}

@c("phr_parse_request")
fn phr_parse_req(
    buf: cstr,
    len: usize,
    method: *cstr!,
    method_len: *usize!,
    path: *cstr!,
    path_len: *usize!,
    minor_version: *i32!,
    headers: [*]phr_header!,
    num_headers: *usize!,
    last_len: usize,
): i32;

@c {
    fn us_create_loop(
        hint: *any!?,
        wakeup_cb: *any!?,
        pre_cb: *any!?,
        post_cb: *any!?,
        ext_size: u32,
    ): *us_loop_t!?;

    fn us_loop_free(loop: *us_loop_t!?): void;
}

fn test_pico_http(): void {
    @print("Testing picohttpparser HTTP parsing in pure CP via @c(\"phr_parse_request\")...");
    let req: cstr = "POST /api/v1/agent/chat HTTP/1.1\r\nHost: 127.0.0.1:8080\r\nContent-Type: application/json\r\nContent-Length: 18\r\n\r\n{\"message\":\"ping\"}";
    let req_len: usize = req.len;

    unsafe {
        let method: cstr = "";
        let method_len: usize = 0;
        let path: cstr = "";
        let path_len: usize = 0;
        let minor_version: i32 = 0;
        let header_buf: [16]phr_header = undefined;
        let num_headers: usize = 16;

        let consumed = phr_parse_req(
            req,
            req_len,
            *method!,
            *method_len!,
            *path!,
            *path_len!,
            *minor_version!,
            header_buf as [*]phr_header!,
            *num_headers!,
            0,
        );

        if (consumed > 0) {
            @print("  ✓ Parse success! Consumed: {d} bytes", consumed);
            @print("  ✓ Method: {s}", method[0..method_len]);
            @print("  ✓ Path: {s}", path[0..path_len]);
            @print("  ✓ Version: HTTP/1.{d}", minor_version);
            @print("  ✓ Headers parsed: {d}", num_headers);
            let h0 = header_buf[0];
            @print("  ✓ Header[0]: {s}: {s}", h0.name[0..h0.name_len], h0.value[0..h0.value_len]);
        } else {
            @panic("phr_parse_req failed");
        }
    }
}

fn test_usockets_event_loop(): void {
    @print("Testing uSockets Event Loop in pure CP via @c block...");
    let loop = us_create_loop(null, null, null, null, 0);
    if (loop != null) {
        @print("  ✓ Native uSockets event loop created successfully!");
        us_loop_free(loop);
        @print("  ✓ Native uSockets event loop freed successfully!");
    } else {
        @panic("us_create_loop returned null");
    }
}

export fn main(): void {
    test_pico_http();
    test_usockets_event_loop();
    @print("ALL PURE CP USOCKETS & PICOHTTPPARSER TESTS PASSED!");
}
```

#### 编译与执行验证

```bash
# 步骤 1：使用 CP 编译器转译为 Zig
cpc compile cap/test/test_usockets_picohttp.cp --output .cap/zig/cap/test/test_usockets_picohttp.zig

# 步骤 2：通过 Zig 根构建管线内联静态库编译并执行
zig build -Dcp-entry=.cap/zig/cap/test/test_usockets_picohttp.zig run
```

---

### 16.5 纯 CP 零 Unsafe 架构：面向对象分层封装（Zero-Unsafe Architecture）

在底层系统编程中，直接暴露裸指针和 `unsafe {}` 块不仅侵蚀语言的安全保证，还会增加业务团队的心智负担。借鉴 Bun 项目底层 `uws_sys` 的成熟工程实践，CP 推荐采用**四层架构**实现纯 Safe 的网络系统构建：

```mermaid
flowchart TD
    A["① C-ABI 映射层 (@c, @c struct, @opaque)"] --> B["② 局部蹦床适配层 (Trampoline Adapters)"]
    B --> C["③ 面向对象 Safe 封装层 (class App, class WebSocket)"]
    C --> D["④ 纯 Safe 业务与 Agent 逻辑层 (100% 零 unsafe)"]
```

1. **第 ① 层：C-ABI 映射层**：
   通过 `@c { ... }` 导入纯净的 C 函数原型；通过 `@opaque struct` 声明不透明外部句柄（如 `us_loop_t`、`us_socket_t`）；通过 `@c struct` / `@fix struct`（兼容 `@extern struct`）精确映射 C 结构体内存对齐。
2. **第 ② 层：局部蹦床适配层（Trampoline Adapters）**：
   在 C 回调（C Callback）处定义轻量私有静态适配器，将 C 风格的裸指针参数转换为 CP 强类型参数，集中处理指针边界判定。
3. **第 ③ 层：面向对象 Safe 封装层**：
   将底层句柄私有封装进安全的 `class`，例如 `class WebSocket` 或 `class HttpServer`。资源获取与释放绑定到类的生命周期，对外部暴露纯 Safe 的事件回调与方法。
4. **第 ④ 层：业务与 Agent 应用层**：
   业务与 AI Agent 逻辑只需使用标准的面向对象 API，在整个上层应用开发中 **100% 零 `unsafe {}` 块**，享受绝对的内存安全与舒适编码体验。

#### 16.5.1 局部蹦床函数（Trampoline）深度剖析：为何能消除 `unsafe`？

许多开发者常困惑：“底层明明是 C 原生回调与裸指针，为什么放进局部结构体中就能做到纯 Safe？”

##### 1. 痛点：C 语言回调与高级闭包的物理矛盾

- C 语言的事件循环（例如 uSockets 的 `on_data` 回调）底层仅接受一个**纯函数指针（`void (*)(us_socket_t*, char*, int)`）**；
- 高级语言中的回调通常是**带有外部上下文环境的动态闭包（Fat Pointer）**。C 语言根本无法识别闭包的内存结构，直接将动态闭包传给 C 必然崩溃。

##### 2. 解决方案：局部结构体充当“安全隔离舱”

在底层封装库内部，我们在函数内声明一个无实例的局部结构体，并将其静态方法冠以 `@c` 属性：

```typescript
// 位于底层网络库封装层内部（对业务开发者完全不可见）：
struct Trampoline {
    @c pub fn on_socket_data(socket: *us_socket_t!, buf: [*]u8!, len: usize) void {
        // 1. C 语言通过 user_data 通道将绑定的面向对象实例安全还回
        let server = us_socket_context(socket) as *HttpServer!;

        // 2. 原地将裸字节指针重构为 Safe 的 string 切片（零拷贝）
        let chunk: string = buf[0..len];

        // 3. 分发给 Safe 的面向对象实例事件处理器
        server.dispatch_data(chunk);
    }
}
```

##### 3. 为何业务层 100% 零 `unsafe`？

- **纯原生 C 签名**：`Trampoline.on_socket_data` 是一个不携带任何外部捕获环境的纯原生 C 函数指针，其物理签名与 C-ABI 完全契合；
- **安全隔离封口**：所有的裸指针类型转换均在局部结构体内部完成封闭，业务开发者最终调用的 API 是完全受所有权和借用检查保护的 `server.on("data", (chunk: string) => { ... })`；
- **架构哲学**：**“底层封装库在隔离舱内负重前行，上层业务逻辑岁月静好”**。

---

### 16.6 C 类型透明兼容与底层栈缓冲区安全规范

为了在维持严格内存安全的前提下，提供与原生 C 库媲美的极限性能与易用性，CP 编译器在底层放行了以下语义与代码生成模式：

#### 16.6.1 `cstr` 与 `string` 零拷贝转换物理真相

> **物理本质**：UTF-8 编码的底层物理格式，本身就是无符号字节序列（`u8 bytes`）。

许多开发者以为 UTF-8 类似于 UTF-16 或某种紧凑压缩包，需要“解压转换”。但在物理内存中：

1. **ASCII 字符（0~127，如 HTTP 动词 `GET`、`POST`、标头字段）**：
   其 UTF-8 编码值与传统 ASCII **100% 绝对一致**，每个字符占用 1 字节（`0x00`~`0x7F`），高位固定为 0；
2. **多字节字符（如中文“你好”）**：
   在 UTF-8 中被编码为 2 到 4 个字节，每个字节的值均在 `128`~`255`（`0x80`~`0xFF`）之间；
3. **物理铁律：多字节 UTF-8 序列中绝对永远不可能出现 `0x00`（NUL 字节）！**
4. **C 语言字符串对文本的真实处理方式**：
   C 语言的 `const char*` 根本不管也不校验字符编码。在 C 语言看来，`char` 就是裸字节（byte），C 语言唯一认定的规则是：**从首地址向后顺序扫描，直到遇到数值为 `0` 的字节（`0x00`，即 `\0`）为止**。
   因为 UTF-8 的所有字符字节均大于 0，直接把 UTF-8 字节流传给 C 语言，C 语言不仅不会提前截断，还能原封不动地进行网络发送与哈希计算。

##### 零拷贝转换机制：

- **CP `string` -> C `cstr`（零开销传参）**：
  CP 字符串字面量（如 `"POST /api/chat HTTP/1.1\r\n"`）存放在二进制文件的 `.rodata` 只读数据段中，末尾天然自带 `\0` 字节。向 C 函数传递时，编译器底层直接传递其首地址指针（`.ptr`），耗时 0 纳秒，拷贝 0 字节；
- **C `cstr` -> CP `string`（零开销重构）**：
  当 C 函数返回 `(ptr, len)`（如 picohttpparser 解析出的 method、path），CP 直接通过内存切片语法 `ptr[0..len]` 原地组装为双字长胖指针 `{ ptr, len }`。该操作仅在 CPU 寄存器中完成赋值，**底层指向的依然是 C 缓冲区同一块物理内存**，拷贝 0 字节数据！

#### 16.6.2 未初始化栈输出缓冲区传递（Stack Output Buffer Pattern）

许多高性能 C 库（如 `picohttpparser`）要求调用者在栈上预分配一块数组内存作为输出缓冲区（Output Parameter）：

```typescript
// 1. 在栈上声明未初始化的定长数组缓冲区
let header_buf: [16]phr_header = undefined;
let num_headers: usize = 16;

// 2. 通过 header_buf as [*]phr_header! 强转为多项裸指针传递给 C
let consumed = phr_parse_req(
    req, req_len,
    *method!, *method_len!,
    *path!, *path_len!,
    *minor_version!,
    header_buf as [*]phr_header!,
    *num_headers!,
    0,
);
```

CP 编译器的未初始化变量检查器（Definite Assignment Checker）针对此类强制转换为 `[*]T!` 的标识符进行了特殊放行识别：**允许作为未初始化底层 C 输出缓冲安全传参，避免了运行期强制填充零字节带来的多余 CPU 内存开销**。

#### 16.6.3 统一后缀裸指针与可选裸指针规范表

| 指针类型 | 含义与底层映射                          | 允许的操作与适用场景                          |
| :------- | :-------------------------------------- | :-------------------------------------------- |
| `*T!`    | 确定非空单对象裸指针 (`*T`)             | 必须在 `unsafe` 中解引用或赋值，不可为 null   |
| `[*]T!`  | 确定非空多元素连续缓冲指针 (`[*]T`)     | 支持索引访问 `ptr[i]` 与切片操作              |
| `*any!`  | 确定非空泛型不透明指针 (`*anyopaque`)   | 等价于 C 的 `void*`                           |
| `*T!?`   | 后选可空裸指针 (`?*T`)                  | 可赋值为 `null`；经 `if != null` 收窄为 `*T!` |
| `*any!?` | 后选可空泛型指针 (`?*anyopaque`)        | 适合接受可选 context 的 C API 回调参数        |
| `cstr`   | C 风格以 0 结尾字符串 (`[*:0]const u8`) | 直接与 C 字符串接口无缝对接                   |

---

### 16.7 CP 标准库网络与密码学生态拓扑规划（自主可控 cap:* 体系）

为了让 CP 具备完整的网络通信能力（底层 I/O、TLS 加密、HTTP 解析、WebSocket、高并发事件循环），同时摆脱对 Bun 内部私有命名空间（如 `bun.BoringSSL`、`bun.sha`、`bun.uws`）的耦合，CP 标准库（`cp/lib`）设立了以下四大网络与密码学顶层子系统：

```mermaid
flowchart TD
    subgraph CP_Stdlib ["CP 官方原生标准库体系 (cap:*)"]
        subgraph CryptoModule ["cap:crypto (密码与传输层安全)"]
            BoringSSL["lib/cap/crypto/boringssl.cp<br/>(BoringSSL C 原生绑定)"]
            Hash["lib/cap/crypto/hash.cp<br/>(SHA-1/256/512 / HMAC)"]
            TLS["lib/cap/crypto/tls.cp<br/>(TLSContext / TLSSocket)"]
        end
        subgraph NetModule ["cap:net (网络驱动与事件循环)"]
            Loop["lib/cap/net/loop.cp<br/>(uSockets EventLoop 封装)"]
            Socket["lib/cap/net/socket.cp<br/>(Non-blocking Socket / TcpServer)"]
        end
        subgraph HttpModule ["cap:http (高性能超文本服务)"]
            Pico["lib/cap/http/parser.cp<br/>(picohttpparser 极速解析)"]
            HttpServ["lib/cap/http/server.cp<br/>(HttpServer / Request / Response)"]
        end
        subgraph WsModule ["cap:ws (实时双向通信与广播)"]
            WS["lib/cap/ws/websocket.cp<br/>(WebSocketClient / WebSocketServer / PubSub)"]
        end
    end
```

#### 16.7.1 模块职责与目录结构矩阵

| 导入命名空间     | 物理源码路径      | 底层依赖 C/Zig 引擎                | 职责与对外暴露核心能力                                                                   |
| :--------------- | :---------------- | :--------------------------------- | :--------------------------------------------------------------------------------------- |
| **`cap:crypto`** | `lib/cap/crypto/` | `vendor/boringssl`                 | 提供 `SHA256`、`Hmac`、数字签名，以及面向对象的 `TLSContext` 与 `TLSSocket` 证书加密通道 |
| **`cap:net`**    | `lib/cap/net/`    | `vendor/uSockets`                  | 提供原生异步事件驱动 `EventLoop`、非阻塞 `Socket`、`TcpServer` 与 DNS 解析               |
| **`cap:http`**   | `lib/cap/http/`   | `vendor/picohttpparser` + uSockets | 提供微秒级零拷贝 HTTP 解析器、流式 `HttpServer`、`Request` 与 `Response` 封装            |
| **`cap:ws`**     | `lib/cap/ws/`     | `vendor/uSockets`                  | 提供符合 RFC 6455 规范的 `WebSocket` 服务端/客户端，支持 Pub/Sub 主题广播                |

#### 16.7.2 纯 Safe 业务调用体验示例

上层业务开发者与 AI Agent 无需关心底层 uSockets 裸指针或 C 语言回调，直接享受 100% 纯 Safe 的现代 TypeScript 风格编程体验：

```typescript
import { HttpServer, Request, Response } from "cap:http";
import { SHA256 } from "cap:crypto";

const server = new HttpServer({ port: 8080 });

server.post("/api/v1/agent/chat", (req: Request, res: Response) => {
  let body = req.body_string();
  let digest = SHA256.digest(body);
  res.json({
    status: "ok",
    hash: digest.to_hex(),
    reply: "Hello from Pure Safe CP!",
  });
});

server.listen();
```

---

## 第十七章：AI-First 智能可自愈诊断系统与 E-Code 规范体系 (AI-Friendly Self-Healing Diagnostics)

### 17.1 核心设计理念：专为 AI Agent 打造的 0 秒自愈编译器

传统系统级编程语言（如 C/C++、Zig 早期或未调优的 Rust）在遇到编译错误时，往往抛出冷冰冰的语法错误（`SyntaxError: unexpected token`）或内部断言栈（`panic at ...`）。AI Agent 面对此类报错往往陷入信息孤岛，必须耗费多轮对话翻阅编译器前端源码，极大拖慢了自动化研发流。

CP 语言编译器从底层确立了 **AI-First 诊断原则**：

1. **彻底根除 Rust Panic 栈回溯污染**：所有借用检查、语法分析与类型错误全部转换为结构化 `Diagnostic`；
2. **工业级三段式自愈诊断规范**：
   - **错误主体与代码切片定位**：精确指出错误文件、行号、列号与 `^^^^` 下划线；
   - **`= cause:` 根本原因剖析**：直白解释为何当前代码违反了语言设计语义；
   - **`= help:` 规范与规则释义**：阐述 CP 语言的核心准则；
   - **`= suggestion:` 精准可采纳建议**：提供可以直接单轮采纳应用的正确代码写法，实现 0 秒闭环自愈！

```text
error[E0201]: cannot perform mutable operation on read-only parameter 'outBuf'
  --> src/boringssl.cp:58:24
   |
58 |     outBuf[0] = 0;
   |     ^^^^^^
   |
   = cause: parameter 'outBuf' is a read-only borrow (immutable by default in CP).
   = help: in CP, function parameters are read-only borrows by default. To modify arguments in-place, declare them as exclusive mutable borrows using '*' prefix.
   = suggestion: parameter 'outBuf' is a read-only borrow. Add '*' to declare it as mutable borrow: '*outBuf: OutIPBuf'
```

---

### 17.2 官方标准错误码（E-Code）全景矩阵

CP 编译器的诊断错误码分为三大正交区间：

| 错误码区间  | 所属编译阶段                                    | 涵盖核心领域与典型场景                                                                                           |
| :---------- | :---------------------------------------------- | :--------------------------------------------------------------------------------------------------------------- |
| **`E01xx`** | **词法与语法分析 (Lexer & Parser)**             | 条件圆括号强制、废除 TS 关键字自愈（`interface` -> `trait`, `try/catch` -> `?`, `mut` -> `let/*`）、循环语法     |
| **`E02xx`** | **语义分析与借用检查 (Sema & Borrow Checker)**  | 单写多读借用冲突、修改只读参数自愈、移动后访问、未定义符号（编辑距离智能联想与标准库导入提示）、类型不匹配与解包 |
| **`E03xx`** | **模块解析与包管理 (Resolver & Module System)** | 模块缺失、内置 `cap:*` 协议标准库检索推荐                                                                        |

#### 17.2.1 典型 E-Code 规则与 AI 自愈建议速查表

| E-Code      | 错误分类与诊断名称                     | 触发场景示例                                                        | AI Agent 自动化自愈方案 (`suggestion`)                                   |
| :---------- | :------------------------------------- | :------------------------------------------------------------------ | :----------------------------------------------------------------------- |
| **`E0101`** | **条件与循环圆括号缺失**               | `if x > 5 { ... }`、`while true { ... }` 或 `for i of 0..n { ... }` | 包装为圆括号：`if (...)`、`while (...)`、`for (...)`                     |
| **`E0102`** | **TS `interface` 关键字**              | `interface User { id: i32 }`                                        | 替换为行为契约 `trait`：`trait User { ... }`                             |
| **`E0103`** | **传统 `try/catch` 异常块**            | `try { ... } catch (e) { ... }`                                     | 改为错误传播符 `?` 或行内 `catch` 表达式                                 |
| **`E0104`** | **多余的 `mut` 关键字**                | `mut a = 1;`                                                        | 移除 `mut`，使用默认可变的 `let a = 1;`                                  |
| **`E0105`** | **`any` 弱类型**                       | `let val: any;`                                                     | 声明具体类型、泛型 `<T>` 或 `*anyopaque`                                 |
| **`E0106`** | **C 风格三段式 for 循环**              | `for (let i = 0; i < n; i++)`                                       | 替换为 `while (i < n)` 或 `for (x of list)`                              |
| **`E0201`** | **修改只读借用参数**                   | `fn f(buf: [16]u8) { buf[0] = 0; }`                                 | 参数前添加 `*` 声明为独占可变借用：`*buf: [16]u8`                        |
| **`E0202`** | **借用冲突 (Aliasing XOR Mutability)** | 在不可变借用存活期间发生可变借用                                    | 结束不可变借用作用域或使用克隆值                                         |
| **`E0203`** | **移动后使用 (Use After Move)**        | 变量经 `move` 所有权移交后再次读取                                  | 移除 `move` 或改用引用传参 `&x`                                          |
| **`E0204`** | **常量重新赋值**                       | `const x = 1; x = 2;`                                               | 改为可变变量声明：`let x = 1;`                                           |
| **`E0205`** | **非 async 函数中使用 await**          | 同步函数内调用 `await task;`                                        | 函数签名添加 `async fn`                                                  |
| **`E0206`** | **未定义符号（拼写错误/缺少导入）**    | 调用未导入的 `print(...)` 或拼写错误 `addr_srt`                     | 自动给出 `import { print } from "cap:io";` 或 `did you mean 'addr_str'?` |
| **`E0207`** | **类型不匹配 (Type Mismatch)**         | 期待 `T` 却传入了 `T?` 或 `T!E`                                     | 提示解包：`val ?? default` 或错误传播 `val ?`                            |
| **`E0208`** | **运行时反射未启用**                   | 对未注解类调用 `Reflect.typeOf()`                                   | 在类定义前添加 `@reflect class ...`                                      |
| **`E0209`** | **Unsafe 操作缺失**                    | 在普通函数内执行裸指针解引用                                        | 将操作包裹在 `unsafe { ... }` 块中                                       |
| **`E0210`** | **循环中修改容器**                     | 在 `for (item of list)` 遍历中调用 `list.push()`                    | 克隆容器后遍历或改用下标 `while` 循环                                    |
| **`E0107`** | **字段访问前冗余解引用 `.*.`**         | `in6.*.family = AF.INET6;`                                          | 移除 `.*.` 直接使用点号访问：`in6.family = AF.INET6;`                    |

---

### 17.3 现代系统级语法规范与指针类型字典

#### 17.3.1 禁止字段访问前冗余解引用 `.*.`

在 CP 语言中，值类型、引用（`*T`）和裸指针（`*T!`）在访问成员字段与方法时**统一采用点号 `.` 直接访问**（编译器自动完成寻址与解引用）。严禁书写 Zig 遗留的 `ptr.*.field` 冗余解引用语法：

```typescript
// ❌ 错误（编译器 E0107 拦截并提示修正）：
in6.*.family = AF.INET6;

// ✅ 正确：
in6.family = AF.INET6;
```

#### 17.3.2 `pub` 与 `_` 极简规范

- **导出声明统一使用 `pub`**：全面废弃繁琐的 `export`，统一使用 `pub class`、`pub @c struct`、`pub enum`、`pub const`、`pub fn`。
- **接收者统一使用 `_` 替代 `this`**：在结构体与类方法中，接收者声明与访问推荐使用更简洁的 `_`（等同于 `this`），例如 `fn count(*_): u32`、`_.release()`。
- **`_` 为接收者保留标识符**：`_` 不能作为普通变量名或编译期虚位形参使用（若需忽略编译期参数可使用具体命名如 `_lookup` 并配合 `_ = _lookup;`）。

#### 17.3.3 RAII 自动析构 `release(*_)`

CP 类与结构体资源释放统一实现 `pub fn release(*_): void`，由编译器 Comptime RAII 自动管理析构；同时可提供 `pub fn deinit(*_): void { _.release(); }` 供外部 Zig 调用方平滑兼容。

#### 17.3.4 标准系统类型与后置修饰符语法字典

CP 全面废止 C 风格类型关键字与前置问号指针语法，统一使用现代强类型与后置修饰符：

| 传统/C 风格语法        | CP 现代标准语法                 | 编译转译语义 (Zig)          | 内存安全与语义说明                     |
| :--------------------- | :------------------------------ | :-------------------------- | :------------------------------------- |
| `c_int` / `c_uint`     | `i32` / `u32`                   | `i32` / `u32`               | 明确位宽整数，消除平台歧义             |
| `c_ushort` / `c_short` | `u16` / `i16`                   | `u16` / `i16`               | 规范 16 位整数                         |
| `*anyopaque`           | `*any?!`                        | `?*anyopaque`               | 可选不透明裸指针                       |
| `?[*:0]u8`             | `[*:0]u8?!`                     | `?[*:0]u8`                  | 可选哨兵以 0 结尾字符串指针            |
| `?[*][*:0]u8`          | `[*]([*:0]u8!)?!`               | `?[*][*:0]u8`               | 指向非空字符串数组的可选多项指针       |
| `?[*:null]?[*:0]u8`    | `[*:null]([*:0]u8?!)?!`         | `?[*:null]?[*:0]u8`         | null 结尾的可选字符串数组可选指针      |
| `cstr`                 | `const s: cstr` / `let s: cstr` | `[*:0]const u8` / `[*:0]u8` | 纳入借用检查体系的原生 C 字符串        |
| `[*c]T`                | `[*c]T` / `[*c]T?`              | `[*c]T` / `?[*c]T`          | C-ABI 兼容指针，与 `&buf` 隐式无缝互通 |

#### 17.3.5 C-ABI 兼容指针与守卫三剑客发散控制流

1. **C-ABI 指针 (`[*c]T`)**：
   - 专为 C 语言头文件 FFI 绑定设计，在底层转译为 Zig 的 `[*c]T`；
   - 自动提升可选语义：`[*c]T?` 与 `?[*c]T` 等价，规范化为 `Optional(CPtr(T))`；
   - 支持将局部借用引用（如 `*buf` 可变借用或 `&buf` 只读借用）隐式传入接收 `[*c]T` 的 C-ABI 函数，极大减轻系统库绑定的心智负担。

2. **守卫三剑客（发散控制流表达式）**：
   - 空值守卫：`val ?? return null;`、`entry ?? continue;`、`item ?? break;`；
   - 布尔守卫：`valid || return;`、`ready || continue;`、`found || break;`；
   - 错误守卫：`res ? return;`、`step ? continue;`、`op ? break;`。
     在遇到 null、false 或 error 时提前发散退出，消除深层嵌套 if，保持主干代码绝对扁平。

---

### 17.4 底层系统级互操作与高级表达式扩展规范

为了彻底消除底层系统库（如网络解析、DNS、安全密码学等模块）中残留的 `zig { ... }` 桥接块，CP 语言提供了四项关键底层互操作与控制流扩展特性：

#### 17.4.1 C-ABI 联合体声明与内存重叠 (`@c union`)

在底层系统编程（如网络栈、协议报文解析）中，经常需要同一段内存在不同类型之间物理重叠（如 IPv4 `in_addr` 与 IPv6 `in6_addr`）。CP 语言原生提供 `@c union` 与普通 `union` 声明支持：

```typescript
// 声明 C-ABI 兼容的 extern union
pub @c union union_unnamed_3 {
    addr4: struct_in_addr,
    addr6: struct_ares_in6_addr,
}

// 声明紧凑内存对齐的 packed union
pub @packed union PackedBits {
    raw: u32,
    flag: u8,
}

// 声明普通安全联合体
pub union ValueUnion {
    int_val: i32,
    byte_val: u8,
}
```

- **转译语义**：`@c union` 转译为 Zig 的 `extern union`；`@packed union` 转译为 `packed union`；普通 `union` 转译为 `union`；
- **内存安全保障**：`union` 类型自动跳过堆分配构造器（`_create`）和自动内存析构注入，直接与底层 C 内存布局 100% 同构对齐。

#### 17.4.2 `switch` 整数与常量标识符范围匹配模式

在状态码解析与网络错误映射中，需要匹配连续范围的整数或枚举（例如 `1...ARES_ENOSERVER` 或负数范围 `-ARES_ENOSERVER...-1`）。CP 放宽了范围匹配端点的限制：

```typescript
const ARES_SUCCESS = 0;
const ARES_ENOSERVER = 10;

fn classify_ares_code(rc: i32): i32 {
    return switch (rc) {
        0 => 0,
        1...ARES_ENOSERVER => 100,      // Zig 风格端点常量标识符
        -ARES_ENOSERVER...-1 => -100,   // 支持负数常量与负整数字面量
        100..=200 => 200,               // 现代 CP 闭区间范围
        * => 999,                        // 通配兜底
    };
}
```

- **端点支持**：端点支持字面量整数、常量标识符、负号表达式（如 `-ARES_ENOSERVER`）；
- **范围符号**：全面支持 Zig 风格 `start...end` 以及 CP 风格 `start..=end`（闭区间）与 `start..end`（半开区间）。

#### 17.4.3 `@if` 编译期平台条件类型别名与深层命名空间类型注解

在编写跨平台网络和系统库时，特定系统类型往往随目标操作系统变化（如 Windows 的 `SOCKET` 对应 `ws2_32.SOCKET`，POSIX 对应 `i32`），或者直接引用标准库/外部命名空间类型（如 `std.posix.sockaddr`）。CP 原生支持条件类型别名与点号深层路径注解：

```typescript
import { Environment } from "bun";

// 1. 根据编译期平台环境条件切换类型定义
pub type ares_socket_t = @if (Environment.isWindows) ws2_32.SOCKET else i32;
pub type hostent_int = @if (Environment.isWindows) i16 else i32;

// 2. 深层命名空间系统类型别名与直接类型注解
pub type struct_sockaddr = std.posix.sockaddr;
let addr: struct_sockaddr = undefined;
let p_addr: &std.posix.sockaddr = &addr;
```

- **转译语义**：条件别名转译为 Zig 的 `pub const Name = if (cond) TypeA else TypeB;`，深层类型直接映射为底层命名空间符号路径；
- **语义分析（Sema）透明展开**：CP 编译器会在借用检查和类型推导时自动递归解开别名，当目标为 `@if` 条件类型时，只要与任意一侧分支类型兼容即可安全赋初值或传参。深层路径类型直接纳入借用与生命周期检查体系。

#### 17.4.4 带值块级作用域表达式与标记块 (Labeled Block Expressions)

当需要经过复杂多步局部计算、或穿透多层嵌套循环才能计算得到目标值时，使用带值的**具名标记块（Labeled Block）**或**匿名表达式块**是 CP 语言中极为优雅且高安全的惯用法。

##### 1. 一击穿透多层嵌套循环并直接赋值给不可变变量

在传统编程语言中，从多层嵌套循环中寻找目标项时，通常必须：

1. 先在外层声明一个可变变量 `let target: Item? = null;`；
2. 再额外引入一个布尔标志位 `let found = false;`；
3. 在内层找到目标后，设置 `target` 和 `found`，并在每一层循环判断 `if (found) break;` 逐层跳出；
4. 或者不得不将循环逻辑强行拆分成一个单独的辅助函数以便使用 `return`。

在 CP 中，使用**标记块（Labeled Block）**，可以直接一击穿透所有嵌套循环，将结果带值返回并赋给外层的 `const` 常量：

```typescript
// search 是定义在循环外部的具名标记块
const target: Item? = search: {
    for (row of matrix) {
        for (col of row) {
            if (check(col)) {
                // 瞬间连跳两层循环，直接将 col 赋给外层的 target 常量，并即刻结束 search 块！
                break :search col;
            }
        }
    }
    break :search null; // 如果全都没搜到，返回默认的 null
};
```

##### 2. 标记块赋值的两大工程与架构价值

1. **从根源确立 `const` 不可变性（Immutable by Construction）**：
   - 传统命令式写法必须先声明可变变量（如 `let target: Item? = null;`），随后在循环内重新赋值。这不仅丧失了编译期对常量的强类型不可变保证，而且在外层后续庞大的业务流程中，该变量极易被误写、被竞态覆盖；
   - 通过标记块表达式直接赋值，`target` **在定义诞生的那一刻起就是绝对的 `const` 常量**！后续任何代码试图对其重新赋值，编译器前端将即刻拦截，构筑坚不可摧的不可变状态安全。

2. **彻底避免中间过程变量污染外层上下文（Zero Scope Pollution）**：
   - 搜索与计算过程中引入的矩阵行变量 `row`、列变量 `col`、临时累加器或中间标志位，**生命周期完全限定在标记块大括号内部**；
   - 块执行完毕后，所有中间临时变量被即刻弹出销毁，**外层作用域零污染、零命名冲突**，上下文保持 100% 极致纯净！

##### 3. 具名标签块与匿名表达式块用法总览

```typescript
// 1. 具名标签块表达式：适合需要多点分支跳出或跨控制流返回的复杂构造
fn get_host_pointer(host: string, host_buf: *[256]u8): cstr?! {
    const host_ptr: cstr?! = brk: {
        let len = Math.min(host.len, host_buf.len - 1);
        Memory.copy(host_buf[0..len], host[0..len]);
        host_buf[len] = 0;
        break :brk &host_buf[0]; // 带值跳出并赋值给外层常量
    };
    return host_ptr;
}

// 2. 匿名表达式块：适合快速收敛单流程的局部复杂初始化
fn compute_result(): i32 {
    let result: i32 = ({
        let base = 50;
        let factor = 2;
        break base * factor; // 自动绑定到匿名块并返回值
    });
    return result;
}
```

- **底层转译映射**：`label: { ... break :label expr; }` 直接生成 Zig 原生标签块；匿名表达式块 `({ ... break expr; })` 转译为 `(blk: { ... break :blk expr; })`，在机器码层面直接编译为寄存器直接传递，零函数调用开销、零栈帧分配！

---

#### 17.5 高性能网络套接字与异步流式 I/O 设计模式 (High-Performance Sockets & Asynchronous Stream I/O Patterns)

在编写跨平台、高吞吐的底层网络套接字（Socket）服务时，CP 语言提供了以下规范与最佳实践：

##### 17.5.1 接收者 `_` 在高频事件回调中的应用

在事件循环（Event Loop）触发的密集网络回调中，统一使用 `_` 作为接收者（`this`）的紧凑代称，代码更清晰易读：

```typescript
export class TCPSocket {
    socket: SocketHandle;
    bytes_written: u64 = 0;

    pub fn onWritable(_: *TCPSocket): void {
        if (_.socket.isDetached()) return;
        _.internalFlush();
    }

    pub fn onData(_: *TCPSocket, data: []const u8): void {
        _.processPayload(data);
    }
}
```

##### 17.5.2 泛型与参数化类工厂模式

通过编译期布尔或类型参数单态化生成不同协议层（如 TCP / TLS）的高性能网络抽象：

```typescript
pub fn NewSocket(comptime ssl: bool): type {
    return class {
        const This = @This();
        socket: SocketHandle;
        flags: SocketFlags = .{};

        pub fn memoryCost(_: *This): usize {
            const ssl_cost: usize = if (ssl and !_.flags.bypass_tls) 40 * 1024 else 0;
            return @sizeOf(This) + ssl_cost;
        }

        pub fn release(_: *This): void {
            _.socket.close();
            bun.destroy(_);
        }

        pub fn deinit(_: *This): void {
            _.release();
        }
    };
}

pub const TCPSocket = NewSocket(false);
pub const TLSSocket = NewSocket(true);
```

##### 17.5.3 表达式块在安全状态转换中的应用

在套接字协议原地升级（如 TCP -> TLS 协商）或复杂选项提取时，使用表达式块 `({ ... break val; })` 消除未初始化可变变量，配合 CP 规范的 `:=` 条件解包与 `?` 错误传播：

```typescript
const secure_context: JSValue = ({
    if (v := opts.getTruthy(globalObject, "secureContext")?) break v;
    if (t := opts.getTruthy(globalObject, "tls")?) {
        if (t.isObject()) {
            if (v := t.getTruthy(globalObject, "secureContext")?) break v;
        }
    }
    break .zero;
});
```

#### 17.6 现代 HTTP/WebSocket 服务端架构与请求上下文设计模式 (Modern HTTP/WebSocket Server Architecture & Request Context Patterns)

在构建高并发、低延迟的 HTTP 与 WebSocket 服务端运行时（如 `src/runtime/server/`），CP 语言推荐以下架构与内存安全设计模式：

##### 17.6.1 统一接收者 `_` 与高频请求分发

在请求处理器与事件回调密集调用的核心上下文类型（如 `RequestContext`、`WebSocketServerContext`）中，使用 `_` 作为统一接收者，消除冗余解引用并确保单写多读借用安全：

```typescript
export class RequestContext {
    arena: MemoryArena;
    request_headers: Headers;
    response_body: ResponseStream?;

    pub fn respondWith(_: *RequestContext, response: JSValue): void {
        if (_.response_body != null) return;
        _.processResponse(response);
    }

    pub fn release(_: *RequestContext): void {
        _.arena.deinit();
        _.request_headers.release();
    }
}
```

##### 17.6.2 表达式块安全驱动的路由决策与状态迁移

通过带值块级表达式 `({ ... break val; })` 封装路由匹配（文件路由、静态预编译路由、通配路由）逻辑，保持上下文变量的不变性：

```typescript
const match_result: RouteResult = ({
    if (entry := static_routes.find(path)?) break entry;
    if (file_entry := file_router.match(path)?) break file_entry;
    break RouteResult.notFound;
});
```

##### 17.6.3 静态文件路由与零拷贝流式分发

对于文件响应（`FileResponseStream`、`RangeRequest`），通过 RAII 自动析构 `release(*_)` 与 Comptime 借用分析，实现从磁盘到网络缓冲区的高效管道传输，并在流生命周期终止时自动释放文件描述符与分片切片资源：

```typescript
export class FileResponseStream {
    fd: FileDescriptor;
    offset: u64 = 0;
    length: u64;

    pub fn release(_: *FileResponseStream): void {
        if (_.fd.isValid()) {
            _.fd.close();
        }
    }
}
```

#### 17.7 高性能事件循环与多线程任务调度设计模式 (High-Performance Event Loop & Multi-Threaded Task Scheduling Patterns)

在构建低延迟、高并发的异步事件驱动内核（如 `src/event_loop/`）时，CP 语言提供了以下关键设计范式：

##### 17.7.1 统一接收者 `_` 在高频事件排空（Drain）与轮询中的应用

在事件循环高频流转的每个周期中，`_` 作为接收者（`this`）的紧凑代称，消除了冗余的方法调用解引用，并保障单写多读并发安全：

```typescript
export class MiniEventLoop {
    tasks: Queue = undefined;
    concurrent_tasks: ConcurrentTaskQueue = .{};
    loop: *uws.Loop = undefined;

    pub fn tickOnce(_: *MiniEventLoop, context: *any!): void {
        if (_.tickConcurrentWithCount() == 0 and _.tasks.count == 0) {
            defer _.onAfterEventLoop();
            _.loop.inc();
            _.loop.tick();
            _.loop.dec();
        }

        while (task := _.tasks.readItem()) {
            task.run(context);
        }
    }

    pub fn release(_: *MiniEventLoop): void {
        _.tasks.deinit();
        bun.assert(_.concurrent_tasks.isEmpty());
    }
}
```

##### 17.7.2 紧凑指针打包（Pointer Packing）与硬件级原子并发队列

利用指针低位对齐位编码元数据（如 `auto_delete` 标志），配合 `@atomicLoad` / `@atomicStore` 实现跨线程零开销无锁并发任务调度：

```typescript
export class ConcurrentTask {
    task: Task = undefined;
    next: PackedNextPtr = .none;

    pub const PackedNextPtr = enum(usize) {
        none = 0,
        auto_delete = 1,
        _,

        pub inline fn init(ptr: ?*ConcurrentTask, auto_del: bool): PackedNextPtr {
            const ptr_bits = if (ptr) |p| @intFromPtr(p) else 0;
            return @enumFromInt(ptr_bits | @intFromBool(auto_del));
        }

        pub inline fn getPtr(_: PackedNextPtr): ?*ConcurrentTask {
            const addr = @intFromEnum(_) & ~@as(usize, 1);
            return if (addr == 0) null else @ptrFromInt(addr);
        }
    };
}
```

##### 17.7.3 确定性析构与微任务后延迟队列（Deferred Task Queue）

利用 Comptime RAII 自动析构 `release(*_)` 管理哈希表与队列生命周期，杜绝跨事件循环轮询造成的内存残留：

```typescript
export class DeferredTaskQueue {
    map: std.AutoArrayHashMapUnmanaged(*any?!, DeferredRepeatingTask) = .{};

    pub fn release(_: *DeferredTaskQueue): void {
        _.map.deinit(bun.default_allocator);
    }

    pub fn deinit(_: *DeferredTaskQueue): void {
        _.release();
    }
}
```

#### 17.8 高精度定时器与 URL 规范解析架构模式 (High-Precision Timers & URL Specification Parsing Patterns)

在构建跨平台运行时的时间调度子系统（如 `src/runtime/timer/`）与 URL 解析处理内核（如 `src/url/`）时，CP 语言提供了以下经典架构范式：

##### 17.8.1 统一接收者 `_` 在高频定时器堆操作中的应用

在最小堆定时器树和微秒级延迟监控（`EventLoopDelayMonitor`、`DateHeaderTimer`）中，使用 `_` 作为统一接收者，结合 Comptime RAII 自动析构 `release(*_)`，彻底根除高频定时任务的句柄遗漏与内存残留：

```typescript
export class DateHeaderTimer {
    date_header_bytes: [DateHeaderLength]u8 = undefined;
    has_registered_timer: bool = false;

    pub fn release(_: *DateHeaderTimer): void {
        if (_.has_registered_timer) {
            _.unregister();
        }
    }

    pub fn deinit(_: *DateHeaderTimer): void {
        _.release();
    }
}
```

##### 17.8.2 带值块级表达式驱动的 URL 状态机解析

WHATWG URL 规范解析器（`URL.parse`）中存在大量复杂的条件分支与层级回退。CP 语言推荐使用表达式块 `({ ... break val; })` 替代标签跳转，确保局部不可变性与解析状态单向流动：

```typescript
export class URL {
    href: string = "";
    origin: string = "";
    pathname: string = "/";

    pub fn displayProtocol(_: *const URL): string {
        if (_.protocol.len > 0) return _.protocol;
        if (port := _.getPort()) {
            if (port == 443) return "https";
        }
        return "http";
    }
}
```

##### 17.8.3 零分配查询参数扫描器与哈希索引表 (Zero-Allocation QueryStringMap)

通过 `MultiArrayList` 结构体切片布局与紧凑字符串指针（`StringPointer`），结合自动 RAII 释放，实现亚微秒级参数查询：

`````typescript
export class QueryStringMap {
    allocator: std.mem.Allocator;
    slice: string;
    buffer: []u8;
    list: Param.List;

    pub fn release(_: *QueryStringMap): void {
        if (_.buffer.len > 0) _.allocator.free(_.buffer);
        if (_.list.len > 0) _.list.deinit(_.allocator);
    }
}

#### 17.9 C-ABI 回调跳板工厂与局部函数单态化架构规范 (C-Trampoline & Local Callback Factory Patterns)

在系统级编程及网络异步库（如 `c-ares` DNS 解析、`libuv`、`uSockets`）开发中，C 底层函数接口通常需要传递符合 C 符号约定（`callconv(.c)`）的无状态回调函数指针，并通过 `ctx: *!?`（或 `*any!?`）传递弱类型上下文。CP 语言通过**编译期单态化（Comptime Monomorphization）+ 局部 `@c fn` 声明提升 + 极简指针缩写**提供了零开销、类型安全的 C 回调跳板（C-Trampoline / C-Closure）支持：

##### 17.9.1 局部 `@c fn` 声明与跳板生成
开发者可在工厂函数内部直接声明局部 `@c fn`，并直接返回该函数的只读借用（`&handle` 获取静态 C 函数指针）。编译器会在转译阶段自动将其单态化提升为 Zig 局部静态结构体 `struct { pub fn handle(...) callconv(.c) void { ... } }.handle`，并自动豁免局部变量借用逃逸检查：

```typescript
// 省略参数名、省略 ': void'、使用极简 '*!?' 通用裸指针
pub type ares_host_callback = @c fn (*!?, i32, i32, *!?);

// 使用统一的 @comptime 元编程注解修饰编译期参数，使用标准 auto 自动推导类型（兼容历史 anytype）
pub fn hostCallbackWrapper(@comptime Type: type, @comptime callback: auto): ares_host_callback {
    @c fn handle(ctx: *!?, status: i32, timeouts: i32, hostent: *!?): void {
        unsafe {
            // 使用 CP 规范的 'as' 显式强转，编译器自动安全注入底层 @ptrCast(@alignCast(...))
            let this_ptr = ctx.? as *Type;
            callback(this_ptr, status, timeouts, hostent);
        }
    }
    // &handle 获取静态 C-ABI 函数指针（只读不可变符号借用）
    return &handle;
}
```

##### 17.9.2 为什么必须是 `@comptime callback: auto`？（编译期单态化 vs 运行期 any）

在系统级 C 跳板中，`@comptime callback: auto`（编译期自动类型推导，兼容 `anytype`）具有极其严密且不可替代的架构意义：

1. **C-ABI 裸函数指针的根本矛盾**：
   C 原生接口（如 `ares_host_callback`）仅接受无状态、无环境闭包的静态纯函数指针（`*const fn(...) void`），它**无法像 JS 箭头函数或高阶闭包那样隐式捕获外层变量**；
2. **零开销编译期单态化（Zero-Cost Monomorphization）**：
   - 如果不使用单态化，开发者就必须为每一种回调类型手写一个专有静态跳板；
   - CP 语言引入 `@comptime callback: auto`（标准语法，语义即“自动推导回调类型”，向下兼容 Zig 历史习惯的 `anytype`），让编译器在编译期捕获具体的函数符号；
   - 当调用 `hostCallbackWrapper(MyContext, my_user_callback)` 时，编译器就地为该特定类型和回调具现化出一套**专有的局部静态结构体与函数**；
   - 在生成的 `@c fn handle` 内部，对 `callback` 的调用是**静态直接调用（Direct Call）甚至内联展开**，不存在任何虚表查找或间接函数指针调用的性能损耗；
3. **与运行期弱类型 `any` 的本质区别**：
   - 这里的 `auto` **不是运行期动态类型**，它没有类型标签，不产生任何装箱（Boxing）或动态 GC 追踪；
   - 它在编译期直接绑定为实参的具体强类型，**100% 享有静态类型与借用安全检查**。

##### 17.9.3 裸指针双轨语法：原生 `as` 强转与类型镜像同构取指针语法糖

CP 语言针对底层 C-FFI 提供了双轨并行且心智完全统一的指针操作语法：

1. **类型与表达式的镜像同构（Isomorphism）**：
   - 在 CP 中，`*` 为可变借用，`!` 为裸指针标记：
     - **类型体系**：`*T`（可变借用），`*T!`（可变 C 裸指针，弱类型通用缩写为 `*!`），`*T!?` / `*!?`（可选弱类型 C 裸指针，对应 Zig `?*anyopaque`，即 C `void*`）；
     - **表达式体系**：`*x`（取可变借用），`*x!`（直接取可变 C 裸指针），`*x!?`（直接取可选弱类型 C 裸指针）；
     - **只读体系**：`&x`（取只读借用），`&x!`（取只读 C 裸指针），`&x!?`（取只读可选 C 裸指针）；
   - **零二义性保证**：CP 语言废除了 Rust 风格的单独后缀 `expr!` 运行期解包算子（CP 统一使用 `?` 错误传播与 `.?` 安全解包），因此 `*expr!` 与 `*expr!?` 具有 100% 独立正交的语法空间。

2. **双轨使用范例（语法糖 vs 显式 as 强转）**：
   ```typescript
   // 方式 A (极简语法糖，类型与表达式完美同构):
   let raw_ctx = *my_ctx!?;   // 直接取得 ?*anyopaque 可选 C 弱类型裸指针
   let read_ctx = &my_ctx!?;  // 取得 ?*const anyopaque 只读可选 C 弱类型裸指针

   // 方式 B (正统显式强转):
   let raw_ctx = *my_ctx as *!?;
   let read_ctx = &my_ctx as *!?;
   ```
3. **函数类型签名极简化**：支持省略形参名（如 `@c fn (*!?, i32, i32, *!?)`），且当返回值为 `void` 时可完全省略 `: void`。

##### 17.9.4 c回调跳板函数完整使用范例

```typescript
import { print } from "cap:io";

// 省略参数名、省略 ': void'、使用极简 '*!?' 通用弱类型指针
pub type ares_host_callback = @c fn (*!?, i32, i32, *!?);

pub struct MyContext {
    pub received_status: i32,
    pub received_timeouts: i32,
}

fn my_user_callback(ctx: *MyContext, status: i32, timeouts: i32, hostent: *!?) {
    ctx.received_status = status;
    ctx.received_timeouts = timeouts;
}

// 使用统一的 @comptime 元编程注解修饰编译期参数，使用标准 auto 语法
pub fn hostCallbackWrapper(@comptime Type: type, @comptime callback: auto): ares_host_callback {
    @c fn handle(ctx: *!?, status: i32, timeouts: i32, hostent: *!?) {
        unsafe {
            // 使用 CP 规范的 'as' 显式强转，编译器自动安全注入底层 @ptrCast(@alignCast(...))
            let this_ptr = ctx.? as *Type;
            callback(this_ptr, status, timeouts, hostent);
        }
    }
    // &handle 获取静态 C-ABI 函数指针
    return &handle;
}

fn main(): void {
    // 遵循 CP 哲学：'let' 变量默认可变，废除冗余的 'mut'
    let my_ctx = MyContext {
        received_status: 0,
        received_timeouts: 0,
    };

    // 自动类型推断，无需冗余类型注解
    let cb = hostCallbackWrapper(MyContext, my_user_callback);

    unsafe {
        // 双轨支持：可直接使用镜像语法糖 '*my_ctx!?' 或显式 'as *!?'
        let raw_ctx = *my_ctx!?;
        cb(raw_ctx, 200, 15, null);
    }

    print(`Status: ${my_ctx.received_status}\n`);
    print(`Timeouts: ${my_ctx.received_timeouts}\n`);
}
```

#### 17.10 JavaScriptCore 虚拟机事件循环与任务桥接架构模式 (JSC Event Loop & Task Bridging Architecture Patterns)

在连接 JavaScriptCore (JSC) 虚拟机与系统事件循环（如 `src/jsc/event_loop.cp` 与 `src/jsc/Task.cp`）时，CP 语言提供了以下经典高并发与防泄漏架构范式：

##### 17.10.1 统一接收者 `_` 在高频任务队列排空（Drain）中的应用

在主线程任务轮询与微任务深度控制（`enter` / `exit`）中，使用 `_` 作为统一接收者，彻底切除冗余解引用并确保单写多读借用安全：

```typescript
export class EventLoop {
    tasks: Queue = undefined;
    entered_event_loop_count: isize = 0;

    pub fn enter(_: *EventLoop): void {
        _.entered_event_loop_count += 1;
        _.debug.enter();
    }

    pub fn exit(_: *EventLoop): void {
        const count = _.entered_event_loop_count;
        defer _.debug.exit();
        if (count == 1 and !_.virtual_machine.is_inside_deferred_task_queue) {
            _.drainMicrotasksWithGlobal(_.global, _.virtual_machine.jsc_vm) catch {};
        }
        _.entered_event_loop_count -= 1;
    }
}
```

##### 17.10.2 标记指针联合体（TaggedPointerUnion）类型分发

通过紧凑无额外内存分配的指针标记联合体封装 90+ 种系统、文件、网络、进程任务，在 `tickQueueWithCount` 中结合 `try` 错误传播与异常防护：

```typescript
pub fn tickQueueWithCount(_: *EventLoop, virtual_machine: *VirtualMachine, counter: *u32): bun.JSTerminated!void {
    while (task := _.tasks.readItem()) {
        defer counter.* += 1;
        switch (task.tag()) {
            @field(Task.Tag, @typeName(CppTask)) => {
                let any: *CppTask = task.get(CppTask).?;
                any.run(_.global) catch err => reportErrorOrTerminate(_.global, err)?;
            },
            else => {},
        }
        _.drainMicrotasksWithGlobal(_.global, _.global.vm())?;
    }
}
```

##### 17.10.3 跨线程工作池任务（WorkTask / ConcurrentPromiseTask）与 RAII 自动回收

在后台计算与 Promise 响应桥接中，利用 Comptime RAII 自动析构 `release(*_)` 释放强引用并从宿主循环解注册：

````typescript
pub fn WorkTask(comptime Context: type): type {
    return struct {
        ctx: *Context;
        ref: Async.KeepAlive = .{};

        pub fn release(*_): void {
            _.ref.unref(_.event_loop.virtual_machine);
            bun.destroy(_);
        }

        pub fn deinit(*_): void {
            _.release();
        }
    };
}

#### 17.11 多线程系统并发原语与工作窃取线程池架构模式 (Multi-threaded System Concurrency Primitives & Work-Stealing ThreadPool Patterns)

在构建高吞吐底层系统软件（如 `src/threading/` 模块）时，CP 语言提供了以下多线程系统级并发、无锁侵入式队列与工作窃取线程池架构范式：

##### 17.11.1 跨平台 Futex 与用户态轻量互斥原语抽象
统一封装 Linux `sys_futex`、Darwin `__ulock_wait2`、Windows `RtlWaitOnAddress` 等平台原语，并利用原子变量内存序（`.acquire` / `.release` / `.monotonic`）保障同步性能与零额外堆分配：

```typescript
export class Futex {
    pub const Word = u32;

    pub fn wait(ptr: *const Atomic(Word), expect: Word, timeout_ns: u64?): FutexError!void {
        if (comptime Environment.isWindows) {
            return waitWindows(ptr, expect, timeout_ns);
        } else if (comptime Environment.isMac) {
            return waitDarwin(ptr, expect, timeout_ns);
        } else if (comptime Environment.isLinux) {
            return waitLinux(ptr, expect, timeout_ns);
        }
    }

    pub fn wake(ptr: *const Atomic(Word), max: u32): void {
        if (comptime Environment.isWindows) {
            wakeWindows(ptr, max);
        } else if (comptime Environment.isMac) {
            wakeDarwin(ptr, max);
        } else if (comptime Environment.isLinux) {
            wakeLinux(ptr, max);
        }
    }
}
`````

##### 17.11.2 侵入式无锁并发队列（UnboundedQueue）与缓存行对齐

在 MPSC/MPMC 场景中，将链表指针侵入式嵌入任务节点，并利用平台缓存行大小（`cache_line_length`）对齐原子头尾指针，消除伪共享（False Sharing）：

```typescript
pub fn UnboundedQueue(comptime T: type, comptime next_field: meta.FieldEnum(T)): type {
    return struct {
        const Self = @This();
        pub const queue_padding_length = cache_line_length / 2;

        back: std.atomic.Value(?*T) align(queue_padding_length) = .init(null);
        front: std.atomic.Value(?*T) align(queue_padding_length) = .init(null);

        pub fn push(_: *Self, item: *T): void {
            _.pushBatch(item, item);
        }

        pub fn pop(_: *Self): *T? {
            let first = _.front.load(.acquire) ?? return null;
            // 环形原子替换与 CAS 检索...
            return first;
        }
    };
}
```

##### 17.11.3 工作窃取线程池（Work-Stealing ThreadPool）与本地环形缓冲区

线程池工作线程维护本地单生产多消费（SPMC）有界环形任务缓冲区（`Buffer`），本地队列为空时按需从全局队列或其他线程缓冲区窃取（Steal）半数任务，通过 `.acq_rel` 消除高争用场景下的调度瓶颈：

```typescript
export class ThreadPool {
    sync: Atomic(Sync) = .init(.{});
    idle_event: Event = .{};
    join_event: Event = .{};
    run_queue: Node.Queue = .{};
    wait_group: WaitGroup = .init();

    pub fn schedule(_: *ThreadPool, batch: Batch): void {
        _.scheduleImpl(batch, false);
    }

    pub fn release(*_): void {
        _.shutdown();
        _.join();
        bun.memory.deinit(&_.sync);
        bun.memory.deinit(&_.idle_event);
        bun.memory.deinit(&_.join_event);
        bun.memory.deinit(&_.run_queue);
        bun.memory.deinit(&_.wait_group);
    }

    pub fn deinit(*_): void {
        _.release();
    }
}
```

#### 17.12 CP 与 Zig 深度互操作、混合类型系统与跨界所有权安全规范 (CP-Zig Deep Interop, Hybrid Type System & Cross-Boundary Ownership Safety)

在从 Zig/Bun 迁移底层系统组件或构建高性能原生系统库时，CP 语言提供了零样板代码、零胶水摩擦的深度互操作支持，彻底废除繁琐冗余的 `zig { ... }` 块包装。本节系统阐述 CP 在跨语言边界下的语法表现、混合类型体系、两级检查模型以及坚不可摧的变量所有权与借用安全保障。

##### 17.12.1 顶层常量与变量直接声明

CP 允许在模块顶层直接使用 `const`、`let` 以及 `pub const`、`pub let` 声明模块级常量与可变变量，并支持深层命名空间别名与带属性的类型引用。CP 语言**彻底废除 `var` 关键字**，变量体系统一遵循现代标准：`let` 声明可变变量，`const` 声明常量。转译时，`cpc` 编译器会自动将顶层 `let` / `pub let` 映射为底层的 Zig `var` / `pub var`：

```typescript
// 直接引用导入模块的命名空间与深层属性，无需 zig {} 包装
pub const Environment = bun.Environment;
pub const FD = bun.FD;
pub const HashedString = bun.HashedString;
pub const Logger = bun.logger;
pub const default_allocator = bun.default_allocator;
pub const api = bun.schema.api;

// 顶层类型别名定义与对外导出
pub type PathString = bun.PathString;

// 模块级可变状态与线程局部变量（统一使用 let，严禁使用 var）
pub let global_counter: i32 = 100;
pub threadlocal let worker_id: u32 = 0; // 亦支持 @tls let worker_id: u32 = 0;
```

> [!NOTE] 顶层常量与变量编译期求值规范 (Top-Level Comptime Initializer Semantics)
> 在转译目标 Zig 中，模块顶级作用域中声明的所有常量（`const`）与变量（转译为 Zig `var`）初始值表达式均在**编译期（Comptime Context）**进行静态求值。
> CP 编译器（`cpc`）在生成顶层 `Stmt::Var`（无论是 `let` 还是 `const`）初始值代码时，统一处于 `is_comptime = true` 模式：
> 1. 杜绝在顶层初始值中生成嵌套的 `if (comptime @typeInfo(...))`，避免触发 Zig 的 `error: comptime expression cannot be inside comptime context`；
> 2. 支持在顶层初始值中直接执行类型转换、结构体内联初始化与编译期常量计算；
> 3. 初始值生成完毕后即刻恢复运行时状态，确保函数体内的局部变量保持正确的运行时/编译期分流逻辑。


##### 17.12.2 透明 Zig 模块导入与默认导出绑定 (Default Specifier Import)

CP 支持通过现代 ES 模块风格直接透明引入 `.zig` 源代码模块，并映射为命名空间对象：

```typescript
import DirInfo from "../resolver/dir_info.zig";
import Options from "../bundler/options.zig";
import URLPath from "../http_types/URLPath.zig";
import Fs from "../resolver/fs.zig";

const FileSystem = Fs.FileSystem;
pub type LocalDirInfo = DirInfo;
```

##### 17.12.3 跨语言混合类型体系与“外柔内刚”两级检查模型 (Two-Level Safety Model)

在处理 `const bun = @import("bun");` 或 `import Sub from "./sub.zig"` 时，程序员常常产生顾虑：**“CP 中是否会失去类型？还是说会解析 Zig 代码获得类型？CP 的类型检查是否会失去作用？”**

###### 1. 前端阶段（`cpc` 编译期）：透明外部类型 (`Type::Any`)

- **设计权衡**：Zig 是一门深度依赖 **Comptime（编译期图灵完备宏计算）** 的系统语言（如 `std.ArrayList(u8)`、`bun.FD`、以及各种由 comptime 泛型函数动态计算的结构体）。如果在 `cpc` 前端内置一套庞大的 Zig Comptime 解释器，会导致编译器体积剧增并产生版本漂移；
- **底层表现**：在 `cpc` 词法与语义分析（Sema）中，被导入的外部 Zig 命名空间对象（如 `bun`、`DirInfo`）被赋予具有透明穿透能力的 **`Type::Any`（外部不透明类型）**。在 CP 阶段，编译器不会武断地因为“尚未展开 Zig AST”而报错阻碍开发，消除了繁琐的 FFI 胶水摩擦。

###### 2. 后端阶段（Zig 编译期）：绝对强类型重塑

- `cpc` 转译生成的 Zig 源码完整保留了原生的引用链：
  ```zig
  const bun = @import("bun");
  const Environment = bun.Environment;
  ```
- 当 `cap` 调用 Zig 编译器进行第二阶段 AOT 编译时，Zig 编译器会启动全量类型推导和 Comptime 计算：
  - 如果 `bun` 中根本不存在 `Environment`，Zig 编译器立即报错拦截；
  - 如果函数调用传参类型不匹配（例如将字符串传给了期望 `bun.FD: i32` 的函数），Zig 编译器会立即报 `expected type 'FD', found '[]const u8'` 并拒绝编译。
- **全景结论**：**整个编译流水线绝不丢失任何类型安全性，类型检查只是分流到了最擅长解析该类型的 Zig 原生编译器中。**

###### 3. `@import("bun")` 成员类型的 3 种标准工程姿态

- **姿态 A：深层类型别名透传（最常用、最自然）**
  ```typescript
  pub type PathString = bun.PathString;
  pub type LocalFD = bun.FD;

  let fd: LocalFD = 3; // 强类型标注，底态透传至 Zig Comptime
  ```
- **姿态 B：显式类型收束（在边界处为 CP 赋予静态类型）**
  ```typescript
  let env_name: string = bun.Environment.name;
  let max_conn: u32 = bun.getMaxConnections() as u32;
  ```
  通过显式标注或 `as` 运算符将边界值固定为 CP 原生强类型，后续代码即刻享有 100% 的 CP 本地静态检查。
- **姿态 C：防腐层包装（标准库统一规范）**
  以 `_` 开头的底层文件（如 `_boringssl.zig`, `_usockets.cp`）负责私有原生绑定；面向用户的标准库文件（如 `socket.cp`, `fs.cp`）将其收束封装为具备完整字段与方法签名的 CP `class` 或 `struct`。

###### 4. “外柔内刚”两级安全防护网

CP 类型检查不仅没有失去作用，反而构成了双层防御体系：

1. **CP 级（宽进）**：允许外部 Zig 属性安全穿透，但在赋值给本地强类型变量后，本地类型检查（如 `i32` 与 `string` 赋值冲突）**严密生效**；
2. **Zig 级（严出）**：第二阶段由 Zig 兜底校验真实内存布局与函数签名，**绝不放过任何隐患**。

##### 17.12.4 跨界变量所有权（Move）与借用安全（Borrow Checker）法则

面对来自 Zig 的外部资源，**CP 的所有权追踪与借用检查规则绝对没有失效，依然 100% 严密生效！**

###### 1. 正交性原理：生命周期与符号拓扑独立于类型

CP 的借用检查器（Borrow Checker）与所有权移动追踪（Move Analysis）是基于**变量符号（Symbol）、控制流图（CFG）与生命周期拓扑**运行的。它只关心：“当前作用域谁借用了该变量？不可变借用期间是否有可变借用发生？该变量是否已被 `move`？”——而完全不在乎底层变量是 CP 原生对象还是来自 Zig 的资源句柄。

###### 2. 跨界 Use-After-Move 静态拦截（依然 100% 拦截）

无论资源句柄来自 Zig 还是 CP，一旦移动后再次读取，CP 编译器在编译期即刻报出致命错误：

```typescript
const bun = @import("bun");

fn test_move() {
    let handle = bun.openResource("data.bin"); // 来自 Zig 的原生资源

    let consumer = move handle; // 所有权转移

    // ❌ CP 编译器在 cpc 前端直接拦截并报编译错误：
    // error: Use of moved value 'handle'
    let invalid = handle.read();
}
```

###### 3. 只读/可变借用排他性检查

```typescript
fn test_borrow() {
    let ctx = bun.createContext();

    let r1 = &ctx; // 只读借用
    let r2 = *ctx; // ❌ 编译期拦截：同一作用域已存在共享借用，无法进行可变借用！
}
```

###### 4. 跨语言 Comptime Auto-Drop (RAII 自动析构黑魔法)

当来自 Zig 的结构体离开作用域时，CP 是如何保证内存安全且杜绝 Double-Free 的？
CP 代码生成器通过 Zig 的 **Comptime 反射机制**在底层注入了影子移动标记与自适应释放钩子：

```zig
var __handle_moved = false; _ = &__handle_moved;
// ...
defer if (!__handle_moved) {
    const T = @TypeOf(handle);
    if (@typeInfo(T) == .pointer) {
        const Child = @typeInfo(T).pointer.child;
        if (@typeInfo(Child) == .@"struct" and @hasDecl(Child, "deinit")) {
            handle.deinit(); // 自动识别并调用 Zig 对象的 deinit()！
        } else if (@typeInfo(Child) == .@"struct" and @hasDecl(Child, "release")) {
            handle.release(); // 自动识别并调用 release()！
        }
    }
};
```

- 一旦用户在 CP 代码中书写 `move handle`，编译器自动发射 `__handle_moved = true;`；
- 原作用域的 `defer` 在运行时被自动旁路，而在新的所有者作用域内接管析构；
- **架构价值**：CP 不仅未对 Zig 变量妥协安全，反而通过编译器静态分析，**为原本不具备借用安全与所有权语义的裸 Zig 资源戴上了类似 Rust 的坚固内存安全护甲**！

##### 17.12.5 上下文软关键字机制 (Contextual Soft Keywords)

为了兼顾 Zig 风格的操作符（如 `and`、`or`、`orelse`）与面向对象 API（如 `Atomic.and()`、`BitSet.or()`）的并存，CP 编译器在词法分析与语法解析阶段采用**上下文软关键字**机制：

1. **作为二元运算符**：当 `and`、`or` 处于中缀表达式位置时，自动作为逻辑与、逻辑或运算符解析；当 `orelse` 处于表达式后时，作为空值合并/解包兜底操作符解析；
2. **作为标识符/成员名**：当作为结构体字段、方法名称或普通变量标识符出现时（例如 `calc.and(val)`、`flags.or(mask)`），保留标识符语义；
3. **保留字自动转义**：CP Codegen 自动检测生成的 Zig 代码，当方法或属性名与 Zig 保留字（如 `and`, `or`, `type`）冲突时，底层自动发射 `@"and"`、`@"or"` 转义，无需开发者手动处理。

```typescript
let active = (count > 0 and ready) or forced;
let final_name = opt_name orelse "default";

// 正常调用以 and/or 命名的方法，编译器自动进行底层保留字安全转义
let res = calc.and(mask);
```

##### 17.12.6 带哨兵切片 (Sentinel Slice)

CP 原生支持带哨兵的切片语法 `[start..end :sentinel]`，直通 Zig 底层零结尾字符串与哨兵数组，保障 C-ABI 兼容性与高性能切片安全：

```typescript
let buf: [16]u8 = "Hello, World!\x00";
// 截取以 0 结尾的哨兵切片
let slice = buf[0..13 :0];
```

---

### 第十八章：AI-First 机器自愈生态与无别名单写者保障 (Machine-Readable Diagnostics & Alias-Free Protection)

#### 18.1 结构化 JSON 诊断与毫秒级机器自愈闭环 (`cpc check --json`)

为了让 AI Agent（如 Antigravity、Bun AGY）及现代 IDE/LSP 能够以确定性方式瞬间修复代码问题，`cpc` 编译器提供了专用的 `--json` 机器可读诊断接口：

```bash
cpc check --json src/main.cp
```

1. **输出 Envelope 规范**：
   - 检查通过：`{"status":"ok","diagnostics":[]}`（Exit Code 0）；
   - 包含错误：`{"status":"error","diagnostics":[...]}`（Exit Code 1）。
2. **Machine-Readable Edits**：
   每个诊断对象不仅包含精确的 `file`、`line`、`col`、`span_start`、`span_end`，更包含由编译器静态计算生成的 `edits` 数组（包含 `offset`, `delete_len`, `insert`），AI Agent 读取后可直接执行无损打补丁：
   ```json
   {
     "status": "error",
     "diagnostics": [
       {
         "code": "E0101",
         "severity": "error",
         "message": "CP Syntax Error: 'if' condition requires parentheses.",
         "file": "src/main.cp",
         "line": 12,
         "col": 8,
         "span_start": 180,
         "span_end": 184,
         "cause": null,
         "help": "in CP, control flow conditions must be enclosed in parentheses to avoid ambiguity with struct literals.",
         "suggestion": "wrap the condition in parentheses: if (condition) { ... }",
         "edits": [
           { "offset": 180, "delete_len": 0, "insert": "(" },
           { "offset": 184, "delete_len": 0, "insert": ")" }
         ]
       }
     ]
   }
   ```

#### 18.1.1 编译器语句级与顶层项 Panic-Mode 语法容错恢复与多错误批量收集 (Multi-Error Recovery)

在传统单错误中断模式下，编译器在遇到第一个语法歧义或符号不匹配时即提前终止退出，导致在大型模块开发或迁移时，开发者或 AI 必须经历痛苦的“修改单处 -> 重新编译 -> 暴露下一处”的反复单轮往返。

`cpc` 编译器内置了**双层 Panic-Mode 语法容错同步状态机**，单次编译即可尽最大努力收集并呈现所有独立语法错误：

1. **语句级容错同步 (`recover_stmt`)**：
   - 当在代码块内解析单条语句发生语法异常时，记录当前 `Diagnostic` 错误并压入诊断集合；
   - 维持嵌套大括号深度（Brace Depth）平衡，向前跳过无效 Token，直到遇到分号 `;`、块结束大括号 `}` 或新的语句起始关键字（`let`、`if`、`while`、`for`、`return`、`switch`、`break`、`continue`、`defer`、`errdefer`）；
   - 保证 Token 指针严格单向递增，彻底消除语法自愈死循环风险，平滑继续解析后续语句。

2. **顶层项容错同步 (`recover_module_item`)**：
   - 当模块顶级声明（函数、结构体、类、枚举等）出现解析错误时，解析器自动跳至下一个顶层关键词（`pub`、`export`、`fn`、`class`、`struct`、`let`、`const`、`import`、`@`）或平衡的块末尾；
   - 确保单个顶层定义的语法缺陷不会污染后续函数的编译解析。

3. **双模式批量聚合输出**：
   - **CLI 终端模式 (`cpc check`)**：一次性按源码行号顺序渲染所有语法错误，完整附带三段式源码切片、`cause`、`help` 与 `suggestion`；
   - **机器可读模式 (`cpc check --json`)**：统一输出 `{"status":"error","diagnostics":[...]}`，单次往返返回所有错误的精确定位与机器可采纳 `edits` 补丁，AI Agent 单轮交互即可完成批量修正。

#### 18.2 极简 `own` 关键字与无别名单写者保护 (Alias-Free Single Ownership)

在百万级并发 Agent/Actor 通信场景下，共享堆大对象（如 `SharedTensor`、大缓存、Socket 连接）的传递必须从物理上杜绝并发别名读写冲突。CP 语言正式确立极简的 3 字母关键字 **`own`**，替代并超越了业界繁琐的 `move unique`。

##### 1. `own` 与传统 `move` 的核心区别

| 维度 | 传统 `move` | 现代 `own`（无别名单写者） |
| :--- | :--- | :--- |
| **设计哲学** | 侧重于单线程作用域内的**“所有权移动动作（Action）”** | 侧重于跨边界的**“排他单写者契约（Exclusive Ownership Contract）”** |
| **并发别名防护** | 仅做普通变量移出，不主动约束跨 Actor 的无别名（Alias-Free）语义 | **严格约束单写者**，保证接收方是全局唯一的排他持有者，杜绝并发竞争 |
| **函数参数修饰** | `fn process(move x: T)` 语义偏向动作，语法略显突兀 | **`fn process(own x: T)`** 优雅明确表达“该形参要求独占所有权” |
| **语法精炼度** | 历史上跨并发独占需写为 `move unique`，冗长繁琐 | **统一收敛为 3 个字母 `own`**，语法极简有力，无冗余心智负担 |
| **向后兼容性** | 继续保留，作为 `own` 的动作等价写法向下兼容 | **作为 CP 语言推荐的标准所有权关键字** |

##### 2. `own` 核心三大应用场景

###### 场景一：Actor 间跨线程/微进程零拷贝传递共享堆大对象（Tensor、大缓存）
在 Actor 消息传递模型中，物理值拷贝大数组开销巨大，而多指针别名又容易引发数据竞争。使用 `own` 既实现零拷贝原子指针传递，又确保接收端是排他的“单写者（Single-Writer）”：
```typescript
import { spawn } from "cap:actor";
import { SharedTensor } from "cap:mem";

// 工作微进程：要求必须独占传入的 Tensor
fn compute_worker(own tensor: SharedTensor): void {
    // 绝对单写者保证，无需互斥锁，就地高性能矩阵变换
    tensor.inplace_matmul();
}

pub fn main(): void {
    let weights = SharedTensor.alloc([4096, 4096]);
    // 移交排他所有权给 worker 微进程
    compute_worker(own weights);

    // ❌ 编译期静态拦截：error[E0203]: use of moved value 'weights'
    // 绝对防止当前 Actor 在权重计算期间进行并发读写踩踏！
    print("{}", weights.shape);
}
```

###### 场景二：系统 RAII 核心资源的排他转移与自动析构接管
对于包装了底层文件描述符、网络 Socket 或显存资源的结构体，通过 `own` 移交所有权时，编译器自动织入 Drop Flag（`__{var}_moved = true;`），使原作用域的自动释放（`release`）安全旁路，彻底消除双重释放（Double Free）与悬挂指针：
```typescript
struct SafeSocket {
    pub fd: i32,
    pub fn release(this): void {
        if (this.fd >= 0) {
            c.close(this.fd);
            this.fd = -1;
        }
    }
}

fn register_epoll(own sock: SafeSocket): void {
    // sock 的所有权在此被接管，离开此函数时才会触发 sock.release()
}

pub fn main(): void {
    let s = SafeSocket { fd: 10 };
    register_epoll(own s);
    // main 结束时，编译器检测到 s 已被 own 移交，自动跳过 main 作用域的 release()，杜绝二次释放！
}
```

###### 场景三：无锁并发队列与环形缓冲区任务提取
在事件循环（Event Loop）与任务调度池中，消费任务时使用 `own` 能够以确定性方式表明该任务已被消费线程认领并转移，杜绝多 Worker 重复执行。

##### 3. 编译期使用已移动变量的智能自愈拦截（`E0203`）
当变量被 `own` 移交后，后续任何读取或借用均被编译器静态语义分析直接阻断，并输出三段式自愈诊断：
```text
error[E0203]: use of moved value 'r1'
  --> src/main.cp:23:17
   |
23 |     print("{}", r1.id);
   |                 ^^
   = cause: value of 'r1' was moved and its ownership was transferred.
   = help: variables moved via 'own' or 'move' transfer ownership and cannot be accessed afterwards.
   = suggestion: avoid moving 'r1' if you need to use it later, or pass by borrow '&r1'
```

---

### 第十九章：原生系统级底层能力扩展与编译期元编程规范 (System-Level Extensions & Comptime Metaprogramming)

为了支撑高性能系统软件（如 Bun.js 底层 I/O 内核、WebAssembly 运行环境、密码学与网络协议栈）从底层 Zig 向纯正 CP 语言的平滑迁移，CP 编译器引入了原生的系统级能力扩展与编译期元编程工具集。

#### 19.1 原生内联汇编表达式 (`@asm(...)` / `asm volatile (...)`)

在微架构调优、WebAssembly 运行时代数运算、原子同步等待（如 Futex `memory.atomic.wait32`）或特定 CPU 指令调用场景下，CP 原生支持内联汇编表达式：

##### 1. 语法规范
- **简易内联指令**：`@asm("nop")` 或 `@asm volatile ("nop")`；
- **带输入/输出寄存器约束的汇编表达式**：
  支持直接书写汇编字符串与操作数约束，使用 `: [ret] "=r" (-> Type)` 指定输出寄存器，使用 `: [param] "r" (val)` 指定输入寄存器及 clobber 列表：
  ```typescript
  // 1. 无操作数的基本汇编指令
  @asm("nop");
  asm volatile ("nop");

  // 2. 携带寄存器绑定的内联汇编表达式
  let result = asm volatile (
      "mov %1, %0"
      : [ret] "=r" (-> usize),
      : [input] "r" (val)
  );

  // 3. WebAssembly 环境下的原子 Futex 等待（Futex.cp 场景）
  let rc = asm volatile (
      "local.get %[ptr]\nlocal.get %[expected]\nlocal.get %[timeout]\nmemory.atomic.wait32 0\nlocal.set %[ret]"
      : [ret] "=r" (-> u32),
      : [ptr] "r" (&ptr.raw),
        [expected] "r" (@as(i32, @bitCast(expect))),
        [timeout] "r" (to),
  );
  ```

##### 2. 编译器代码生成映射
- CP AST 中将该语法节点解析为 `Expr::InlineAsm { is_volatile: bool, content: String }`；
- Codegen 层精确映射并发射 Zig 原生的 `asm volatile (...)` / `asm (...)` 语法。

---

#### 19.2 线程局部存储属性修饰符 (`@tls let` / `threadlocal let`)

在高性能网络服务与 URL 路径解析等高并发场景下，避免高频内存分配的经典方案是使用线程局部静态缓冲区（Thread-Local Storage）。

##### 1. 核心语法约束
CP 语言**彻底废除了容易引起混乱的 `var` 关键字**，变量声明体系只有 `let`（默认可重新赋值/修改）与 `const`（不可变常量）。因此，TLS 变量严格遵循 CP 风格：
- 推荐现代宏修饰符写法：`@tls let name: Type = init;`
- 系统关键字兼容写法：`threadlocal let name: Type = init;`
- **严禁使用 `var`**：若书写 `threadlocal var`，编译器在语法解析阶段将直接报错并提示修正为 `let`。

```typescript
// 顶层声明线程局部复用缓冲区（URLPath.cp 场景）
@tls let temp_path_buf: [1024]u8 = undefined;
threadlocal let big_temp_path_buf: [16384]u8 = undefined;

pub fn format_path(input: []const u8): []const u8 {
    // 线程独立访问，零数据竞争，零全局锁竞争开销
    temp_path_buf[0] = '/';
    return temp_path_buf[0..1];
}
```

##### 2. 编译器代码生成映射
CP Codegen 会将顶层 `@tls let` / `threadlocal let` 变量发射为 Zig 的 `threadlocal var name: Type = init;`，实现零开销线程私有全局变量。

---

#### 19.3 辨识联合（Tagged / Discriminated Union）

Zig 中的 `union(enum)` 在 CP 语言中天然映射为 **携带载荷的强类型枚举（Payload Enums）**。

##### 1. CP 纯正定义与映射契约
在 CP 中，用户无需使用冷冰冰的 Zig 式宏定义，直接以现代面向对象联合枚举书写：
```typescript
// 带载荷的强类型辨识联合枚举（Header.cp:60 场景）
pub enum Class {
    forbidden,
    host,
    regular: u16,
}

// 复杂应用任务状态辨识联合
pub enum Task {
    Idle,
    Text: string,
    Move: [i32, i32],
}
```

- **底层发射**：CP 编译器检测到枚举变体携带类型载荷（`tuple_type` 或匿名结构体字段）时，自动发射为 Zig 原生的：
  ```zig
  pub const Task = union(enum) {
      Idle,
      Text: []const u8,
      Move: [2]i32,
  };
  ```

##### 2. 模式匹配与变量捕获解构
配合 CP 原生的 `match` 表达式，可以直接安全地解构辨识联合的变体载荷：
```typescript
let t = Task.Text("hello tagged union");

match (t) {
    Task.Idle => print("idle"),
    Task.Text(s) => print("text payload: {}", s),
    Task.Move(pos) => print("coords: x={}, y={}", pos[0], pos[1]),
    _ => print("unknown"),
}
```

---

#### 19.4 编译期静态查找表与内置枚举映射体系 (`@map`, `.get()`, `.has()`, `.name`, `@refCount`)

在处理网络协议、HTTP MIME 类型解析（如 `MimeType.cp`）、S3 ACL 权限列表（如 `acl.cp`）时，运行时的字符串哈希查找存在显著性能损耗。CP 引入了编译期静态查找表与枚举内置映射体系：

##### 1. 编译期静态查找表 `@map`
`@map` 在编译期（Comptime）将键值对映射生成完美最小哈希表（Perfect Hash Map），运行时享受 O(1) 甚至近似 O(0) 的内联静态查表性能：

- **显式指定枚举类型（推荐）**：首参数声明目标类型，内部享受极简的点语法（`.variant`）：
  ```typescript
  enum ACL {
      Private,
      PublicRead,
      PublicReadWrite,
  }

  // 显式声明类型，映射外部非标字符串至强类型枚举变体
  const ACL_MAP = @map(ACL, {
      "private": .Private,
      "public-read": .PublicRead,
      "public-read-write": .PublicReadWrite,
  });
  ```
- **自动推导类型**：首个键值对作为锚点标明类型，后续自动享受极简点语法：
  ```typescript
  const MIME = @map({
      "text/html": MimeType.html,
      "application/json": .json,
      "image/png": .png,
  });
  ```
- **查询 API**：
  - `MAP.get("key")`：返回可选类型 `?ValueType`（查无则为 `null`）；
  - `MAP.has("key")`：返回布尔值 `bool`。

##### 2. 内置枚举映射自洽体系（替代繁琐的 `fromName`）
枚举类型本身是天然的“标识符名称 -> 枚举变体”静态映射表。CP 语言正式确立以 `.get()` / `.has()` / `.name` 统一的极简自洽体系：
```typescript
enum Method {
    GET,
    POST,
    PUT,
    DELETE,
}

// 1. 编译期自动生成的极速静态获取：代替冗长的 fromName/fromString
let m = Method.get("GET");      // 返回 ?Method: Method.GET
let invalid = Method.get("FOO");// 返回 null

// 2. 存在性快速判定
let exists = Method.has("POST");// 返回 true

// 3. 极速获取枚举变体标识符名称（零开销字符串）
let val = Method.POST;
let str_name = val.name;         // 返回 "POST"
```

##### 3. 类顶层引用计数自动管理装饰器 (`@refCount`)
在包装底层 C 资源、网络套接字或共享内存对象时，生命周期需要通过侵入式引用计数（Reference Counting）进行精准管理。CP 提供了 `@refCount` 类顶层装饰器（credentials.cp 等场景）：

```typescript
@refCount(cleanup) // 指定归零时触发的析构回调函数名（缺省默认为 deinit）
class SharedResource {
    pub ref_count: u32;
    pub cleaned: bool;

    pub fn init() {
        this.ref_count = 1;
        this.cleaned = false;
    }

    pub fn cleanup(mut _) {
        this.cleaned = true;
    }
}

pub fn main() {
    let res = new SharedResource();
    _ = res.ref();   // 引用计数递增：ref_count = 2
    res.deref();     // 引用计数递减：ref_count = 1
    res.deref();     // 归零时自动触发 cleanup()，执行安全释放
}
```
- **编译器支持**：如果当前环境处于 Bun 宿主中，编译器自动桥接对接 `bun.ptr.RefCount`；如果处于独立 CP 环境，则自动生成原子递增/递减与条件析构的纯正原生方法。

---

#### 19.5 整型标量与枚举类型的显式双向智能转换 (`as EnumType` / `as i32`)

在网络底层协议（如 Socket 类型 `ConnectingSocket.cp`、状态字 `SocketKind.cp`）中，底层网络系统调用往往返回原始整数（如 `c_int` / `i32`），需高频与高级强类型枚举相互转换。

CP 编译器在语义分析器（`sema.rs`）与代码生成器（`codegen.rs`）中全面打通了整型标量与枚举类型的显式 `as` 转换：

##### 1. 显式双向转换规范
- **枚举转换为整型**：`kind as i32`、`kind as usize`，编译器自动发射底层原生 `@intFromEnum(kind)`；
- **整型转换为枚举**：`raw_int as SocketKind`、`0 as SocketKind`，编译器自动发射底层原生 `@enumFromInt(raw_int)`。

```typescript
pub enum SocketKind {
    Stream = 0,
    Datagram = 1,
    Raw = 2,
}

pub fn handle_socket(raw_fd_type: i32): void {
    // 1. 整数智能转换为枚举类型
    let kind = raw_fd_type as SocketKind;

    // 2. 枚举类型智能转换为整型
    let code = kind as i32;

    // 3. 字面量直接强制转换
    let default_kind = 0 as SocketKind;
}
```
- **架构价值**：开发者再也无需在 CP 代码中嵌入 Zig 的 `@enumFromInt` / `@intFromEnum` 代码块，彻底消除了此类转换对 `zig {}` 内联块的依赖，保持代码库 100% 纯正优雅。

---

#### 19.6 函数风格类型构造转换 (`Type(val)`) 与通用 `.toString()`

在系统级编程与高级业务逻辑中，除了通过 `as` 运算符进行显式类型强转外，CP 语言全面支持类似主流现代语言（TypeScript / Rust / Python）的**函数调用风格类型构造与转换**（Functional Casts / Constructors），并原生提供统一的 `.toString()` 方法。

##### 1. 函数风格类型转换规范
开发者可以直接将类型名称作为函数调用，将源值转换为目标类型：

- **基础标量数值转换**：
  - 整型：`i8(x)`, `i16(x)`, `i32(x)`, `i64(x)`, `i128(x)`, `isize(x)`
  - 无符号整型：`u8(x)`, `u16(x)`, `u32(x)`, `u64(x)`, `u128(x)`, `usize(x)`
  - 浮点型：`f32(x)`, `f64(x)`
  - 布尔型：`bool(x)`（将数值非零转换为 `true`，`0` 转换为 `false`，布尔值保持原样）
- **字符串转换与构造**：
  - `string(x)` 与 `String(x)`：支持将标量数值、枚举或布尔值格式化为字符串。
- **枚举构造转换**：
  - `EnumType(int_val)`：将整型标量直接构造为枚举变体，与 `int_val as EnumType` 完全对等且更具面向对象构造质感。

```typescript
pub enum SocketKind {
    Stream = 0,
    Datagram = 1,
    Raw = 2,
}

pub fn demo_casts() {
    // 1. 枚举与整型函数构造互转
    let kind = SocketKind(1);       // 整型构造为 SocketKind.Datagram
    let code = i32(kind);           // 枚举变体转换回 i32: 1
    let u_code = usize(kind);       // 枚举变体转换为 usize: 1

    // 2. 标量类型之间函数构造
    let pi = 3.14159;
    let int_pi = i32(pi);           // 截断为 3
    let f_val = f64(42);            // 提升为 42.0
    let is_ok = bool(1);            // 转换为 true
    let is_zero = bool(0);          // 转换为 false

    // 3. 字符串转换
    let s1 = string(12345);         // "12345"
    let s2 = String(SocketKind.Raw);// "Raw"
}
```

##### 2. 强类型安全防线：坚决摒弃模糊的 `num(x)` / `Number(x)`
> [!IMPORTANT]
> **拒绝 JS 弱类型泥潭**：JavaScript 中的 `Number()` / `num` 模糊了整数、无符号数、指针位宽与浮点数的物理边界，这在系统级高并发与高性能 Agent 通信场景中是致命隐患。
> CP 语言坚决要求精确标量类型（Precise Scalar Types）。若在代码中调用 `num(x)` 或 `Number(x)`，编译器将在前端直接硬性拦截，并输出 AI 友好的自愈诊断：
> ```text
> error[E0204]: invalid cast: 'num()' is ambiguous in CP.
>   --> src/demo.cp:12:15
>    |
> 12 |     let val = num(x);
>    |               ^^^^^^
>    = cause: CP requires explicit scalar types.
>    = help: replace 'num()' with a precise scalar cast such as 'i32(x)', 'u32(x)', 'usize(x)', or 'f64(x)'.
> ```

##### 3. 内置 `.toString()` 统一方法
所有基础标量类型与枚举类型均原生支持 `.toString()` 方法：

- **枚举类型**：`val.toString()` 自动零开销展开为枚举标签标识符字符串（底层发射 Zig 原生 `@tagName(val)`，**零堆分配、零内存损耗**，等价于 `val.name`）；
- **基础标量类型**：`num.toString()` 与 `flag.toString()` 自动格式化输出对应文本字符串；
- **自定义类/结构体**：若类中定义了 `toString()` 方法则直接调用；若未定义则通过系统运行时反射进行安全格式化输出。

```typescript
let state = SocketKind.Datagram;
let name_str = state.toString();   // "Datagram"（零堆分配，底层编译期静态展开）

let count = 42;
let count_str = count.toString();  // "42"

let flag = true;
let flag_str = flag.toString();    // "true"
```

---

### 第二十章：CP 语言与 `cpc` 编译器工业级生产化演进路线图 (Production-Grade Roadmap)

为了使 CP 语言及其核心编译器 `cpc` 从当前的“功能验证与语法完备阶段”全面跃升至**满足企业级高并发、微秒级延迟、操作系统级系统软件与百万级 AI Agent 生产集群要求的工业生产级别（Production-Grade）**，CP 架构设计委员会制定本演进路线图规范。

```mermaid
flowchart TD
    subgraph CP_Production_Ecosystem ["CP 工业生产级别编译器架构全景"]
        direction TB
        subgraph Core_Engine ["一、 运行时与内存内核"]
            A1["可插拔 Allocator 体系<br/>(Scoped Arena / 零堆逃逸分析)"]
            A2["抢占式调度器自动织入<br/>(Reduction 循环计数 / 跨 await 安全)"]
        end
        subgraph Type_Safety ["二、 类型系统与借用检查"]
            B1["泛型 Trait 契约静态拦截<br/>(避免泄露底层 Zig 模板报错)"]
            B2["多字段独立借用<br/>(Disjoint Field Borrowing)"]
        end
        subgraph Observability ["三、 调试与可观测性"]
            C1["DWARF / Source Maps 映射<br/>(GDB/LLDB 单步断点调试 .cp)"]
            C2["生产 Panic 还原与 Profiling<br/>(带行号堆栈还原 / perf 符号)"]
        end
        subgraph Toolchain ["四、 开发者工具链与 AI 闭环"]
            D1["完整 LSP (跳转/引用/重构)"]
            D2["官方权威格式化器 (cpc fmt)"]
            D3["100% 覆盖的 AI 结构化自愈"]
        end
    end
```

#### 20.1 源码映射、可观测性与生产级调试 (Source Maps, DWARF Debuginfo & Demangling) 【已实装】

##### 1. 痛点场景与设计考量
`cpc` 将 `.cp` 转译为 `.zig` 并由底层 LLVM 生成机器码二进制。由于 Zig 语言原生不提供 C 语言风格的 `#line` 预处理宏指令（遇到 `#line` 会直接作为语法错误拒绝），直接在转译文件中写 `#line` 无法通过 Zig 解析。
因此，CP 采用现代工业级架构方案：**前端 AST Span 精确映射 + 自动伴随生成 JSON Source Map (`.zig.map`) + 工具链原生堆栈反解器 (`cpc demangle` / `cap demangle`)**。

##### 2. 演进技术规范与落地架构
1. **源码映射 (Source Map) 自动伴随生成**：
   - 编译器 `Codegen` 在生成每一个函数定义和语句（`generate_stmt` / `generate_function_decorated`）时，自动根据 AST `Span` 将 CP 源码行号、列号与生成的 Zig 代码行号精准记录；
   - 转译输出 `<target>.zig` 时，自动在同一目录下伴随生成 `<target>.zig.map` 标准映射文件：
     ```json
     {
       "version": 1,
       "cp_source": "cap/test/test_source_map_demangle.cp",
       "zig_target": ".cap/zig/cap/test/test_source_map_demangle.zig",
       "mappings": [
         { "zig_line": 23, "zig_col": 1, "cp_file": "cap/test/test_source_map_demangle.cp", "cp_line": 3, "cp_col": 1, "symbol": "trigger_crash" },
         { "zig_line": 24, "zig_col": 1, "cp_file": "cap/test/test_source_map_demangle.cp", "cp_line": 4, "cp_col": 5, "symbol": "trigger_crash" }
       ]
     }
     ```
2. **生产崩溃堆栈反解工具 (CP Demangler - `cpc demangle` / `cap demangle`)**：
   - 当运行时发生 panic 产生堆栈跟踪（如 `.cap/zig/cap/test/test_xxx.zig:24:5`）时，运行 `cap demangle [trace_file]` 或管道输入 `cat trace.txt | cap demangle`：
   - Demangler 会自动加载关联的 `.zig.map`，瞬间将堆栈重构反解为原生 `.cp` 文件名、行号与列号，并输出上下文代码切片与指针 `^`：
     ```text
     /data/dev/cp/cap/test/test_source_map_demangle.cp:4:5: 0x1034f72 in trigger_crash (test_source_map_demangle)
         panic("explicit crash test");
         ^
     /data/dev/cp/cap/test/test_source_map_demangle.cp:10:26: 0x103513a in test_0 (test_source_map_demangle)
         let res = trigger_crash();
                                  ^
     ```
3. **原生性能分析与 eBPF 集成**：
   - 标准化函数符号修饰规则（Symbol Mangling），使 Linux `perf`、`pprof` 生成的火焰图清晰识读 CP 模块名与函数。

---

#### 20.2 可插拔内存分配器体系与区域作用域 (Pluggable Allocators & Scoped Arena) 【已完成】

##### 1. 痛点场景
系统级高并发场景（如高吞吐 Web 服务、百万 Agent 工具调用流）严禁对每一次请求都走全局堆 `malloc`，否则将诱发严重的堆锁竞争与内存碎片。

##### 2. 技术规范与已落地实现
1. **一等公民 Allocator 注入契约**：
   - 标准库容器与大型对象支持显式传入自定义 `Allocator`。
2. **作用域区域分配器（`arena { ... }` 原生语法）**：
   - 支持**语句级**与**表达式级**作用域 Arena：
     ```typescript
     // 1. 语句块作用域 Arena
     arena {
         let temp = new Dict();
         temp.set("key", "val");
         let msg = `Processing: ${100 + 23}`;
         print(msg);
     } // 👈 离开作用域时，内部 Arena 整块 O(1) 瞬时清空释放！零碎片、零逐个 free 损耗！

     // 2. 表达式作用域 Arena（支持返回值向外传递）
     let computed_val: i32 = arena {
         let a = 10;
         let b = 20;
         let temp_node = new BufferNode(a + b, "temp");
         temp_node.val * 2 // 👈 纯标量值/结果安全返回，临时对象随块销毁
     };

     // 3. 原生底层动态分配支持
     arena {
         let raw_ptr = @alloc(64); // 👈 自动绑定当前活跃的作用域 Arena 分配器
         raw_ptr[0] = 77;
     }
     ```
3. **状态机与生命周期无缝集成**：
   - 编译器前端在 `extract_locals_simple` 与 `extract_local_vars_to_str` 完整支持 `ArenaBlock` 与 `Expr::Arena` 嵌套提取，确保异步协程/状态机上下文字段安全注入；
   - 底层精准映射为 Zig `std.heap.ArenaAllocator.init(std.heap.page_allocator)` 并自动注入 `defer __arena.deinit()`，兼顾极致性能与强异常安全。
4. **静态逃逸分析与栈分配提升 (Escape Analysis)**：
   - 编译器前端自动检测局部对象生命周期，未逃逸出当前栈帧的对象自动提升为栈分配。

---

#### 20.3 抢占式调度与高可用并发引擎 (Auto-Reduction Injection & Structured Concurrency)

##### 1. 痛点场景
在海量 Agent 共存运行环境下，单个 Agent 一旦因提示词异常陷入死循环（如 `while (true)`）或执行本地长耗时正则/数学运算，极易将底层工作线程卡死，导致同线程其他 Agent 饥饿。

##### 2. 演进技术规范
1. **自动抢占式计数器注入（Auto-Reduction Injection）**：
   - 在 `-nest` 模式下，`cpc` 在循环回边（Loop Backedge）与函数序言（Function Prologue）中自动织入指令递减逻辑：
     ```zig
     // 编译器自动注入的抢占检测代码
     if (__ctx.dec_reductions(1) <= 0) {
         return __ctx.yield(); // 达到预算后强制出让执行权，重新排队
     }
     ```
   - 无论业务代码写得多深，达到 2000 个 Reductions 阈值强制出让，彻底根治死循环与饥饿问题。
2. **跨越 `await` 的静态借用安全拦截 (Send / Sync 约束)**：
   - 跨越 `await` 挂起点时，协程可能被不同的工作线程唤醒。
   - 编译器前端静态拦截：**严禁跨越 `await` 持有非线程安全裸指针、不可重入互斥锁或临时局部栈借用**。
3. **结构化并发作用域 (Structured Concurrency Nursery)**：
   - 原生提供并发作用域，确保派生任务统一监管，支持超时令牌（Cancellation Token）与树状生命周期级联取消。

---

#### 20.4 模块化增量编译与工程扩展性 (Modular Incremental Transpilation & Caching)

##### 1. 痛点场景
大型工程包含上百个模块、数十万行代码时，全量单体 AST 解析与重复转译会导致编译耗时激增。

##### 2. 演进技术规范
1. **基于内容哈希的模块级增量转译缓存**：
   - 引入 Content-Addressable Cache：根据 AST 语法指纹缓存模块产物，仅转译发生变更的源码及其受影响的下游依赖。
2. **跨模块循环引用解耦 (Two-Pass Declaration Resolution) 【已实装】**：
   - **两阶段解析机制**：符号收集与类型分析两阶段严格分离（Pass 1 预扫描并收集所有模块的 `class`、`trait`、`enum` 与类型别名声明；Pass 2 建立模块拓扑图；Pass 3 深度进行函数体与方法体的类型推导与借用检查）；
   - **拓扑解耦与死锁消除**：`resolver.rs` 采用规范路径追踪（`HashSet<PathBuf>`）杜绝循环导入时的递归栈溢出，底层 Zig 代码发射无序依赖结构体定义，彻底解决模块 A 和模块 B 互相包含类型时的推导死锁；
   - **示例**：
     ```typescript
     // circ_a.cp
     import { NodeB } from "./circ_b";
     export class NodeA {
         id: i32;
         b: ?NodeB;
     }

     // circ_b.cp
     import { NodeA } from "./circ_a";
     export class NodeB {
         tag: string;
         a: ?NodeA;
     }
     ```
3. **稳定 C-ABI 导出与动态插件热加载 (Dynamic Plugins)**：
   - 支持 `@export("c")` 规范与 `.so` / `.dylib` 运行时安全动态加载，为 Agent 容器的热插拔插件提供稳定接口。
4. **编译器核心代码生成器模块化架构 (Modular Codegen Architecture) 【已全面实装】**：
   - 彻底重构拆解 1.7 万行巨石单体文件，全面升级为 8 个高内聚职责子模块：
     - `codegen/mod.rs`：上下文环境、`Codegen` 核心生命周期、缩进派发与模块级顶层项生成；
     - `codegen/prelude.rs`：`ModuleFeatures` AST 特性扫描与运行时环境感知；
     - `codegen/types.rs`：类型推导、Zig 原生类型映射与常量泛型单态化；
     - `codegen/borrow.rs`：NLL 借用检查与跨作用域冲突拦截；
     - `codegen/pattern.rs`：模式匹配、穷尽性分析与 `switch` 展开；
     - `codegen/actor.rs`：CPS 状态机切分、异步闭包变量捕获与 Actor 协程逻辑；
     - `codegen/stmt.rs`：控制流语句生成（`let`、`if`、`while`、`for`、`return`、`defer`、`errdefer`）；
     - `codegen/expr.rs`：表达式求值、运算符重载、方法调用与内联闭包。
   - **多子智能体并行维护**：单文件规模严格控制在 1,000 ~ 4,000 行以内，彻底消除了跨 800 行分段读取往返延迟，具备极高并发迭代能力；
   - **作用域防护增强**：Trait 虚函数表严格限定为局部 `@This().VTable`，杜绝用户代码顶层同名 `VTable` 符号引起的作用域歧义冲突。

---

#### 20.5 高级类型系统与借用检查器深度升级 (Advanced Type System & Borrow Checker)

##### 1. 痛点场景
目前泛型报错有时会穿透到底层 Zig 模板展开阶段；借用检查器若锁定整个结构体实例，会导致独立字段并发读写误报冲突。

##### 2. 演进技术规范
1. **泛型 Trait 静态契约拦截 (Generic Bounds)**：
   - 在 CP 前端直接校验泛型约束（如 `fn sort<T: Comparable>(*arr: [T])`），类型不满足时立即给出精准自愈建议，彻底避免底层晦涩报错。
2. **多字段分离借用 (Disjoint Field Borrowing) 【已实装】**：
   - **细粒度字段借用跟踪**：借用检查器将结构体借用追踪升级为 `(root_var, field_path)` 元组，不再一刀切锁定整个根对象；
   - **正交字段无冲突并发借用**：只要两个借用指向同一对象的不同字段（如 `*data.q_len` 与 `*data.active_timer`，或多级嵌套 `*ctx.header.content_len` 与 `*ctx.body_read`），编译器判定为 Disjoint 独立访问，允许同时存在多个可变借用；
   - **整体验证与前缀覆盖保护**：借用整个结构体（`*data`）依然与任何活动中的子字段借用保持互斥；借用父字段（`*data.header`）依然与子字段（`*data.header.len`）保持互斥；同一字段的可变借用仍然严格保证排他性；
   - **示例**：
     ```typescript
     class InternalLoopData {
         q_len: i32;
         active_timer: i32;
         max_batch: i32;
     }

     fn process_loop(*data: InternalLoopData): void {
         // 多字段分离借用：无需打散结构体即可自由借用不同字段（CP 可变借用前缀为 *）
         let q = *data.q_len;
         let t = *data.active_timer;
         let b = *data.max_batch;

         q.* = 42;
         t.* = 100;
         b.* = 1024;
     }
     ```
3. **常量泛型与定长张量 (Const Generics)**：
   - 原生支持参数化常量类型，如 `Tensor<f32, [1, 4096]>`、`Buffer<1024>`，使张量与网络缓冲区在编译期确定物理尺寸。

---

#### 20.6 生产级零拷贝标准库基石 (Production Standard Library Ecosystem)

##### 1. 统一零拷贝流抽象 (`Reader` / `Writer` / `Closer` / `Seeker`) 【已全面实装】
- 详见第 12.3 节《统一高性能 I/O 流体系（`cap:io`）与 Trait 契约》。在 `cap:io` 中已完备落地 `Reader`、`Writer`、`Closer`、`Seeker` 四大核心契约，并配套提供 `BufferReader`、`BufferWriter`、`BufferedReader`、`BufferedWriter` 与 `copyStream` 泛型管道。文件系统、TCP/UDP 套接字、TLS 安全连接与内存 Buffer 统一实现标准流契约，数据流转零多余深拷贝。

##### 2. 高性能编译期序列化框架 (Zero-Copy Serde)
- 对标 `simdjson`，实现零拷贝编译期 JSON / 结构化二进制序列化与反序列化，从原始字节切片直接零成本映射至强类型数据容器。

##### 3. 无锁并发原语库 (Lock-free Primitives)
- 内置 MPMC 无锁环形队列、并发哈希表与原子切片，为跨线程高频信箱通信提供极速基础设施。

---

#### 20.7 开发者体验与工具链全家桶 (DevEx & Toolchain)

##### 1. 工业级 LSP (Language Server Protocol)
- 完善 `cpc lsp`：支持精确定义跳转（Go to Definition）、全局引用查找（Find All References）、实时语义波浪线、全局安全重命名（Rename Symbol）与智能导入补全。

##### 2. 官方权威代码格式化工具 (`cpc fmt` / `cap fmt`)
- 官方内置统一步伐的代码格式化工具，杜绝团队风格分歧与提交冲突。

##### 3. 原生测试与基准评测套件 (`cap test` / `cap bench`)
- 内置 `test "name" { ... }` 与 `benchmark "name" { ... }` 语法，一键输出耗时、每秒操作数（ops/sec）以及内存分配统计。

##### 4. C 头文件自动化绑定生成工具 (`cap bindgen`)
- 自动解析复杂 C/C++ 库头文件并生成现代强类型的 `.cp` 绑定声明，一键吸纳庞大 C 语言成熟生态。

---

#### 20.8 AI-First 机器自愈全闭环与确定性构建 (Autonomic Evolution & Reproducible Builds)

##### 1. 100% 覆盖的 Machine-Readable Edits 自愈闭环
- 将 `--json` 结构化修正从语法层深入扩展至类型错误与借用冲突：
  - 遇到只读参数就地修改时：自动输出插入 `*` 或转换为独占 `own` 的修改偏移量；
  - 遇到遗漏模块导入时：自动计算正确的 `import` 插入位置；
  - 遇到模式匹配未穷尽时：自动追加缺失的分支骨架。

##### 2. 确定性与可复现构建 (Deterministic Builds)
- 保证相同源码在任何操作系统与时间转译出的 Zig 产物和二进制散列值（SHA-256）100% 完全一致。

---

#### 20.9 实施优先级规划与推进里程碑 (Priority Milestones)

| 阶段 | 重点领域 | 核心任务 | 交付价值与验收标准 |
| :--- | :--- | :--- | :--- |
| **P0 阶段（已全面攻坚完成）** | **调试与工程稳定性** | 1. 源码行号指示与 DWARF 映射注入 (`.zig.map` / Source Map & Demangle 反解器)【已完成】<br/>2. 泛型 Trait 前端契约校验（消除底层报错泄露）【已完成】<br/>3. Scoped Arena 区域内存池语法与运行时实现【已完成】 | 极大降低大型项目与 Bun 迁移调试门槛，实现断点与崩溃堆栈精准映射到 `.cp` 源码，消灭高频小对象内存碎片 |
| **P1 阶段（中期生产化）** | **并发调度与工程规模** | 1. 循环回边 Auto-Reduction 抢占计数自动注入<br/>2. 跨 await 线程安全静态借用拦截<br/>3. 基于内容哈希的模块级增量编译与缓存 | 真正实现百万 Agent 抗死循环与防单点饥饿，十万行以上超大工程秒级极速构建 |
| **P2 阶段（生态成熟期）** | **工具链与标准库基座** | 1. 完整生产级 LSP（跳转/引用/重命名/补全）<br/>2. 官方权威代码格式化工具 `cpc fmt`<br/>3. 统一零拷贝 I/O 流与无锁并发容器标准库 | 开发者与 AI 协同体验全面媲美 TS/Rust，具备承载国家级超高并发与关键系统服务的能力 |

---

_本规范由 CP 架构设计委员会制定，作为全体编译器研发、运行时对接与开发者生态的唯一最高准则。_
