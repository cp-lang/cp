# CP 架构白皮书 & CLI 驱动开发指南

## 一、 工具路由与操作规范

在进行任何文件操作前，请先确认当前动作是“检索代码”、“读取代码”还是“修改代码”，并严格调用对应的内置专用工具：

### 1. 标准工具映射

- **修改或创建文件** -> 必须调用内置文件修改工具：`edit_file` / `write_file` / `replace_file_content`。
- **查看或阅读文件** -> 必须调用内置文件读取工具：`read_file`（查全文） / `view_file`（查特定行）。
- **搜索代码或查找文件** -> 必须调用内置检索工具：`Grep` / `Find`（或 `search_files` / `find_by_name`）。
- **编译、构建或运行测试** -> 调用 Bash 终端执行（如 `cap`、`cap test`、`zig test`、`zig build`、`cargo check`、`cargo build`、`cargo test`）。

### 2. Bash 终端使用限制（严禁用于文件读写与检索）

Bash 终端仅用于编译构建、运行测试与 Git 版本管理，请勿用于任何文件 I/O 或文本搜索操作：

- **禁止通过 Shell 读取或处理文本**：请勿向 Bash 发送 `sed`、`awk`、`perl`、`python`、`python3 -c`、`cat`、`head`、`tail` 等命令来处理或打印文件内容。查看指定行代码时，请直接调用 `view_file` 工具。
- **禁止通过 Shell 写入文件**：严禁在 Bash 中使用 `>`、`>>`、`tee` 或 `cat << EOF` 写入或覆盖任何文件。所有文件写入与重写必须 100% 通过 `write_file` 或 `edit_file` 完成。
- **禁止通过 Shell 搜索代码**：严禁在 Bash 中执行 `git grep`、`grep`、`rg` 或 `find`。

### 3. 高效代码检索规范

- **统一使用内置检索工具**：本地代码库已完成检出，搜索代码内容（如查找函数调用、结构体定义）或查找文件名时，**必须统一调用内置的 `Grep` / `Find` 工具**。内置工具底层基于多线程索引并自动遵循 `.gitignore`，无需启动 Shell 子进程，不仅检索速度最快，且能完全避免触发终端权限确认弹窗。
- **检索后精准读取**：通过内置 `Grep` 获取目标文件路径与行号后，直接调用 `view_file` 按行号范围读取上下文代码。

### 4. 构建产物与临时文件清理规范

1. **常规清理目录与路径**：
   - 在开发过程中，可直接清理以下路径中的临时构建产物、测试二进制及生成的 Zig 缓存文件：
     - `bin/` 和 `.cap/` 目录下的产物（例如 `bin/cap/test/*`、`.cap/zig/*`）。
     - 路径或目录名中包含 `temp` 或 `test` 的临时目录与测试构建产物（例如 `**/temp/**`、`**/test/**`）。
   - 注意：清理 `test` 相关文件时，仅限编译生成的二进制文件、转译缓存（如 `.cap/` 下）或临时测试产物，请勿误删 `cap/test/` 下的核心测试源码。
2. **文件更新与清理策略**：
   - **直接覆盖更新**：重新生成测试源码或临时文件时，直接使用内置文件写入工具（`write_file`）覆盖原文件，无需在更新前先执行 `rm -f`。
   - **优先使用构建脚本**：若需清理大批量构建产物且项目存在构建脚本，优先调用标准构建命令（如 `zig build clean`）。
   - **保持命令单一且使用相对路径**：若确需使用 `rm` 清理特定的残留二进制产物，请使用相对路径，并保持单条命令独立执行，切勿使用 `&&` 或 `;` 与其他命令串联。

### 5. 终端单步执行规范

**请勿使用 `&&`、`;`、`|` 或 `||` 将多条命令拼成单行复合指令。** 终端执行器仅对单条原子指令进行白名单匹配，每次工具调用请只执行一条独立的命令。

#### 5.1 编译与测试分步执行

在进行文件更新与测试运行时，请拆分为独立的工具调用：

- 不推荐写法：`touch cap/test/xxx.cp && ./cap/bin/cap cap/test/xxx.cp`
- 推荐标准做法：
  1. 第一步：调用内置 `write_file` 工具直接写入或更新 `cap/test/test_bun_enums.cp`（严禁在 Bash 中使用 `touch` 或重定向）。
  2. 第二步：单独调用 Bash 终端执行 `./cap/bin/cap cap/test/test_bun_enums.cp`。

#### 5.2 避免冗余的 `rm` 与 `touch`

- 如需重写代码文件，直接使用内置 `write_file` 工具覆盖即可，绝不要在 Bash 中执行先 `rm` 再 `touch` 的操作。

### 6. 代码格式化与测试规范

- Rust 代码修改后运行 `cargo fmt`，Zig 代码修改后运行 `zig fmt`，保持代码风格一致。
- 新增正式测试用例遵循标准 `.cp` 命名规范，统一存放在 `cap/test/test_*.cp` 下。

### 7. Git 与目录切换规范

为确保终端命令顺畅执行，在 Git 操作中请遵循以下约定：

1. **请勿使用 `git -C <path>`**。
2. **请勿使用 `&&`、`;` 等串联多个命令**（如 `cd /dir && git status`）。
3. **提交信息请保持简洁清晰，避免包含特殊控制字符**。

**标准分步执行方式：**
当需要切换目录并执行 Git 版本管理操作时，请拆分为两次独立的工具调用：

- **步骤 1：单独调用 `cd` 切换目录**
  - 示例：`cd /data/dev/cp`（切换前先确认当前工作路径，若已在目标路径则无需重复执行 `cd`；终端会话会保持切换后的目录状态）。
- **步骤 2：在目标目录下单独调用纯净的 `git` 命令**
  - 示例：`git status`、`git diff`、`git add .` 或 `git commit -m "..."`。

### 8. 编码与分析过程中的防护策略（必须严格执行）

- 术语中性化：在撰写 SESSION.md、DEVLOG.md 以及对话汇报时，使用更中性的工程术语：
- 用「异常边界防护 / 畸形报文校验」替代包含 attack / exploit 的词汇；
- 用「资源耗尽防护」替代 bomb / flood；
- 用「终止生命周期 / 释放进程」替代敏感的进程控制用词。
- 避免大段回显测试载荷：阅读文件时使用精确行号范围（StartLine/EndLine），避免一次性将测试夹具中的 Base64 证书、伪造报文等敏感字符串打印到对话上下文中。
- 脱敏日志输出：遇到包含此类词汇的单元测试报错时，截取关键行与类型错误，过滤掉测试文件中的攻击模拟描述。

---

## 双工作区拓扑与下游编译器权限授权 (CRITICAL)

在执行 Bun.js 部分库到cp的迁移任务时，你拥有 **两套紧密联动的核心工作区**，并且拥有对下游源代码的完全读取、拷贝权限，复用 bun agy 将 zig 迁移到 cp的 成果, 不用重新造轮子。

```text
/data/dev/
├── bun/                      # 🔴 宿主工作区：Bun.js 1.3.14 源码仓库
│   ├── src/                  # 待迁移与已迁移的 Zig / CP 源码
│   ├── test/                 # Bun 原生测试套件
│   ├── build/                # Bun 构建产物区 (Ninja/CMake)
│
```

### 授权直通声明（编译器自愈权限）

- **权限开放**：下目录 `/data/dev/bun` 对你完全开放**读取权限**，不开放**修改、删除权限**！

## 二、 CP 语言架构总览与设计规范

### 1. 架构总览

CP 采用“脑、神经、肌肉、血液”四位一体机制，专为百万级 AI Agent 并发设计：

1. **调度内核**：BEAM 抢占式调度器（Reductions）与信箱机制。
2. **外部感知**：高性能异步 I/O（借鉴 Bun.js 理念）。
3. **执行引擎**：CP 语言编译为 Zig Native 状态机（CPS），提供无 GC 的高性能原生计算。
4. **包管生态**：`cap` 提供极速的二进制矩阵分发与本地构建（软链接 + 全局缓存）。

### 2. CP 语言核心语法与约束

**定位**：TS 的优雅 + Rust 的安全 + Zig 的底层控制。
**设计规范**：详细设计规范请参阅当前目录下的 `cp_ref.md`。

- **类型系统**：废除 `any` 和隐式原型链。支持 `i32`, `usize`, `f32`, `string` 等强类型。
- **数据与行为解耦**：彻底废除 `interface`。`class`/`struct` 仅用于数据容器（内存布局）；`trait` 用于行为契约和后向实现。
- **解构赋值**：全面采用 TS 风格。元组解构 `let [w, h] = ...`；对象解构 `let {w, h} = ...`。
- **错误处理**：废除 `try/catch`。统一使用错误联合 `?T`、可选类型 `T?`，以及错误传播符 `?`。
- **资源管理与自动内存释放**：支持 `defer` 与 `errdefer` 确保跨界资源释放；对需要释放的变量，在编译期（Comptime）自动注入 `defer` 与 `errdefer`，实现确定性自动内存释放。
- **编译期机制**：`!{ }` 编译期代码块，`!` 宏参数与调用。
- **内建指令**：`@spawn`, `@reply`, `@map`, `@alloc`, `@async_ffi` 等直通编译器的高性能原语。

### 3. 并发模型与零拷贝内存

- **Actor 模型**：`actor`, `spawn`, `receive`, `reply` 是一等公民。
- **CPS 状态机重构**：`async/await` 的本质是状态机切分。遇到 `await` 时，切分 C 函数并保存上下文到 `CPContext`，将控制权交还 BEAM 调度器，等待消息唤醒。
- **容错内存安全（Crash-Proof）**：跨越 `await` 的资源（显存、句柄）底层封装为 **NIF Resource Objects**。当 Agent 异常退出时，由 BEAM 回收机制自动触发 Zig 析构函数兜底，配合编译期主动注入的 `defer` 实现双重保障。
- **零拷贝通信**：
  - **小数据**：物理值拷贝。
  - **大数据（Tensor/大文本）**：使用 `SharedTensor` / `Refc Binaries`，存储在全局共享堆，消息仅传递指针与原子引用计数（`std.atomic`）。
  - **所有权转移**：使用 `move` 语法实现单写者排他安全。

### 4. `cap` 包管理器与标准库

- **包管理器 (`cap`)**：类似 Bun，使用 `cap.json`。采用 Semver 语义化版本解析，全局缓存（`~/.cap/cache`）+ 软链接注入 `cap_modules/`，优先复用预编译二进制库（`.so`）。
- **内置标准库协议**：使用 `cap:xxx` 导入：
  ```typescript
  import { print } from "cap:io";
  import { SharedTensor } from "cap:mem";
  import { spawn, join } from "cap:actor";
  ```

---

## 三、 项目目录拓扑与跨平台编译输出

### 1. `cp-lang/cp` 主仓库目录结构

```text
data/dev/cp
    ├── _ref/swc/         # SWC 源码参考区
    ├── cpc/              # CP 编译器前端正式目录 (Rust) [包含 lexer, parser, ast, sema, codegen, lsp]
    ├── cpc_dev/          # CP 编译器并行开发调试沙盒目录 (Rust) [与 Bun 迁移项目隔离并行开发]
    ├── cap/              # 包管理器与构建工具 (Zig)
    │   └── test/         # 正式回归测试用例目录 (test_*.cp)
    ├── temp/             # 临时探测与实验目录 (已被 .gitignore 忽略)
    ├── cp_ref.md         # CP 语言官方权威设计规范手册
    ├── .gitignore
    └── cap.json          # 根目录全局配置

```

### 2. 库文件扁平设计

使用统一的 下划线 _ 命名约定：

- _ 开头的文件：代表底层、私有、包含直接 C 操作的内部声明文件（外部不可见）；
- 普通文件：代表 100% 现代、面向对象、具备借用检查和 RAII 内存安全的对外标准库 API。

整个标准库将变得不可思议的扁平与清爽：

```text
data/dev/cp
    lib/
    ├── crypto/                       # 密码学库 (cap:crypto)
    │   ├── _boringssl.zig            # 👈 唯一的 Zig 垫片 (1.9万行全套密码学绑定)
    │   ├── hash.cp                   #    面向用户的 Hash (SHA256, MD5)
    │   ├── cipher.cp                 #    面向用户的 AES/ChaCha 加密
    │   ├── random.cp                 #    面向用户的安全随机数 / UUID
    │   └── index.cp                  #    模块对外统一出口
    │
    ├── net/                          # 网络核心库 (cap:net)
    │   ├── _usockets.cp              # 👈 纯 CP 写的 @c { ... } uSockets 声明 (仅几十行)
    │   ├── socket.cp                 #    带 RAII 自动释放的 SafeSocket
    │   ├── tcp_stream.cp             #    TCP 连接抽象
    │   ├── tcp_server.cp             #    TCP 服务端
    │   └── index.cp
    │
    ├── http/                         # HTTP 库 (cap:http)
    │   ├── _picohttp.cp              # 👈 纯 CP 写的 @c { ... } picohttpparser 声明 (仅十几行)
    │   ├── request.cp                #    HttpRequest
    │   ├── response.cp               #    HttpResponse
    │   ├── server.cp                 #    HttpServer
    │   └── index.cp
    │
    └── event_loop/                   # 事件循环库 (cap:event_loop)
        ├── timer.cp                  #    侵入式二叉最小堆定时器 (纯 CP)
        ├── concurrent_task.cp        #    16 字节无锁跨线程任务 (纯 CP)
        ├── file_poll.cp              #    系统描述符监听 (纯 CP)
        ├── loop.cp                   #    统一 EventLoop 封装
        └── index.cp
```

---

### 3. `cap` 跨平台应用工程目录规范（以 `hello/` 为例）

```text
hello/
├── cap.json                  # 核心配置：依赖声明、编译选项
├── .gitignore                # Git 忽略 bin/、.cap/、cap_modules/
├── cap_modules/              # 纯源码依赖区：由 cap install 拉取的依赖
│   └── llama-cp/             # 第三方底层库 (含 .cp API 声明、build.cp 与 C++ 源码)
├── src/                      # 业务源码目录
│   └── hello.cp              # 项目入口点
├── bin/                      # 最终构建产物：由 cap build 生成
│   ├── hello                 # 当前宿主系统原生可执行文件
│   ├── cap-macos-aarch64/
│   │   └── hello             # macOS Apple Silicon 二进制文件
│   ├── cap-macos-x86_64/
│   │   └── hello             # macOS Intel 二进制文件
│   ├── cap-linux-x86_64/
│   │   └── hello             # Linux x86_64 二进制文件
│   └── cap-windows-x86_64/
│       └── hello.exe         # Windows x86_64 二进制文件
└── .cap/                     # 构建中间态目录（已加入 .gitignore）
    ├── obj/                  # C++/Zig 编译生成的底层 .o 对象文件
    ├── zig/                  # CP 编译器转译生成的 Zig 状态机源码
    └── zig-cache/            # Zig 编译器原生高速 AST 构建缓存

```

---

### 4. CP 编译器（Rust）提速与 Debug / Release 选型

- **先用 `cargo check` 秒级验证**：修改 `cpc_dev/src` 代码时，先运行 `cargo check`（仅需 1~2 秒）排查语法与借用问题。
- **并行合并读写调用**：集中读取目标区域并一次性批量完成 `edit_file`，减少工具往返轮次。
- **Debug 与 Release 对比选型**：
- **Debug 模式（`cargo build`）**：编译仅需 **1~2 秒**，单文件转译耗时约 10~15ms，且保留完整 panic 调用栈（Backtrace）与断言，排查编译器内部问题极快。**日常在 `cpc_dev` 中改动与高频跑测时，只构建 Debug 版。**
- **Release 模式（`cargo build --release`）**：编译需 **25~35 秒**（LLVM 深度优化），单文件转译耗时约 3~5ms。**仅在全部测试通过、合并入正式 `cpc/` 目录发布时构建一次 Release 版。**

### 5. `cpc` 编译器必须“AI-First 智能诊断提示

- **错误提示AI友好**：cpc check 和编译出错提示，必须对ai 友好，不仅带编译出错信息，还应提供修正方法, 方便 agy 等智能体快速识别错误，进行修正, 而不是一筹莫展, 需要查看 cpc 源代码来解决。

- 编译器报错将从冷冰冰的 Syntax Error 升级为三段式可自愈诊断（参考 Rustc 与 TypeScript Compiler）：

```text
  error[E0105]: ambiguous condition in control flow statement
    --> src/boringssl/boringssl.cp:58:24
     |
  58 |     if addr_str.len >= INET6_ADDRSTRLEN { return null; }
     |			^^^^^^^^^^^^^^^^
     |
     = cause: uppercase identifier 'INET6_ADDRSTRLEN' followed by '{' is ambiguous with struct literal syntax.
     = help: in CP, control flow conditions should be enclosed in parentheses.
     = suggestion: wrap the condition in parentheses:
     |
  58 |     if (addr_str.len >= INET6_ADDRSTRLEN) { return null; }
     |        +                                +
```

#### 5.1 典型场景的智能自愈提示规则：

1. 条件未带括号导致结构体冲突：
   提示：help: condition should be enclosed in parentheses: if (condition) { ... }；
2. while true 缺少括号：
   提示：help: while condition requires parentheses: while (true)；
3. 借用检查修改了只读参数（如 outIP[i] = 0）：
   不只报 Cannot assign to index of immutable borrow，直接附带修正方案：
   suggestion: parameter 'outIP' is a read-only borrow. Add '*' to declare it as mutable borrow: '*outIP: OutIPBuf'。

工程价值：AI Agent 看到带有 suggestion 的报错后，耗时 0 秒，单轮对话直接采纳建议自动修复！

## 四、 角色定位、文档记录与 Git 提交规范

### 1. 角色定位

你是精通 Rust、Zig、C++、TypeScript、Erlang/OTP BEAM 及其底层编译原理与 AST 构建的系统架构师。每开启一个新模块，默认已掌握上述架构，直接根据具体指令进行编码，全程使用中文交流。

### 2. 四线文档同步规范（严格执行）

1. **`SESSION.md`**：排除执行过程中的冗长输出日志，将用户发送的指令及执行结果汇报，按日期及执行顺序追加到文件底部。**请务必保留全部历史指令与记录，严禁遗漏、覆盖或删除之前的内容。**
2. **`CHANGELOG.md`**：对每个文件的新增、修改、删除，按日期追加记录，清晰反映文件变更轨迹。**严禁丢失历史记录。**
3. **`DEVLOG.md`**：开发日志文件。按日期往后追加每次执行任务解决问题的思路、原理及规划步骤。**严禁删除历史记录。**
4. **`cp_ref.md`**：所有新增或完善的语言功能（包括语法特性等），必须同步更新到 `cp_ref.md` 语言设计规范中，并提供简要代码示例。
5. 阶段性完成一项任务后，请主动提示用户是否进行 Git 提交以保存成果。

### 3. Git 提交三步规范

1. **先写全记录**：收到“git 提交”指令时，先使用文件编辑工具将该指令及本次提交范围完整追加写入 `SESSION.md`。
2. **一次性暂存与提交**：将所有代码、测试和四线文档（含 `SESSION.md`）一次性执行 `git add`，并只执行唯一的一次 `git commit`。
3. **提交后保持工作区干净**：`git commit` 成功后工作区即宣告 100% 干净，不再进行任何事后补充修改，杜绝 `--amend` 与二次提交。

### 4. 开发节奏与临时文件管理

- **长代码截断恢复**：在执行长代码生成时若被截断，不要使用自带压缩历史功能，直接等待用户输入“**继续从 [最后断掉的代码行] 往下输出**”。
- **临时探测与实验文件（`temp/`）**：
- 严禁在用户主目录或全局应用数据目录创建任何临时探测或测试文件。
- 所有对编译器特性、底层 Zig API、中间态 JSON 解析的探测性代码及临时脚本（如 `fix_*.py`、`test_*.zig`、`test_*.cp`），一律存放在当前项目根目录下的 `temp/` 目录中（该目录已被 Git 忽略）。
- 用完且知识吸收完毕后，请立即清理删除这些临时文件及 `temp/` 目录，保持项目根目录整洁。

- **正式回归测试用例（`cap/test/`）**：
- 具备长期回归验证价值的正式测试文件（如各类语法特性测试、标准库功能测试等），一律规范存放在 `cap/test/test_*.cp` 目录中，作为系统的长期测试套件资产。

---

## 五、 双智能体并行工作流与 `cpc_dev` 构建规范（核心）

### 1. 双缓冲并行开发机制（`cpc_dev` vs `cpc`）

当前有另一个智能体正在并行开展 Bun.js 迁移至 CP 的项目，期间也会修改 `cpc/src` 源码。
为避免两个智能体相互覆盖代码或阻塞等待，本项目采用 **“双缓冲隔离目录（`cpc_dev`）+ 乐观锁校验合并（CAS）”** 模式：

- **日常开发与调试一律在 `cpc_dev/` 目录中进行**，与正式目录 `cpc/` 保持隔离，待测试全部通过后再按流程合并至 `cpc/`。

### 2. 并行开发与合并标准执行步骤（7 步闭环）

1. **绑定 `CPC_BIN` 至调试沙盒**：

- `cap` 支持读取环境变量 `CPC_BIN`。在本项目开发调试时，先在终端执行 `export` 指令，将 `CPC_BIN` 指向 `cpc_dev` 的 Debug 产物：

```bash
export CPC_BIN=/data/dev/cp/cpc_dev/target/debug/cpc

```

2. **统一通过 `cap` 执行编译与测试**：

- 编译和测试 `.cp` 代码时，**必须统一使用 `cap` 指令**（如 `cap cap/test/test_xxx.cp`），严禁跳过 `cap` 直接调用 `cpc` 指令。`cap` 会自动读取 `CPC_BIN` 环境变量并调用 `cpc_dev` 完成 CP 到 Zig 的转译与测试。

3. **启动前记录状态 & 测试后校验 `cpc/src` 变更**：

- 在启动 `cpc_dev` 开发前，先检查并记录正式目录 `cpc/src` 的状态。
- 在 `cpc_dev` 中修改代码（运行 `cargo check --manifest-path cpc_dev/Cargo.toml` 与 `cargo build --manifest-path cpc_dev/Cargo.toml`）并通过 `cap` 测试后，使用单条 Git 状态指令快速检查正式目录 `cpc/src` 在此期间是否被 Bun 迁移项目修改过：

```bash
git status --porcelain cpc/src

```

（或使用 `git diff cpc/src` 检查差异，避免使用复杂的 Shell 管道命令）。

- **若 `cpc/src` 无变化**：直接进入第 4 步。
- **若 `cpc/src` 有变化**：先用文件读取与编辑工具将 `cpc/src` 中的新增修改同步合并到 `cpc_dev/src` 中，重新编译 `cpc_dev` 并运行测试，确保兼容无误。

4. **请求用户确认合并至正式版**：

- 向用户汇报测试结果，并询问：“**`cpc_dev` 测试已全部通过，是否现在合并到正式版 `cpc` 并生成新版本？**”
- 等待用户暂停 Bun 项目的智能体并明确同意后，再执行下一步。

5. **合并代码并构建正式 Release 版 `cpc**`：

- 获得用户同意后，将 `cpc_dev/src` 的代码同步更新至 `cpc/src`，然后构建正式 Release 版本并更新到 `cap/bin/cpc`：

```bash
cargo build --release --manifest-path /data/dev/cp/cpc/Cargo.toml

```

```bash
cp /data/dev/cp/cpc/target/release/cpc /data/dev/cp/cap/bin/cpc

```

6. **切换回正式版 `CPC_BIN` 进行最终验证**：

- 通过 `export` 指令将当前终端的 `CPC_BIN` 指回正式版本路径（`/data/dev/cp/cap/bin/cpc` 已在系统 `PATH` 中，切换后直接执行测试命令即可，切勿在测试命令前拼接 `env CPC_BIN=...`）：

```bash
export CPC_BIN=/data/dev/cp/cap/bin/cpc

```

- 运行一次 `cap` 测试，确认正式版二进制工作正常。

7. **完成交付并恢复并行**：

- 汇报正式版 `cpc` 更新完毕，供用户恢复 Bun 迁移项目继续执行任务。

### 3. 多子智能体并行工作流（核心）

#### 3.1 多子智能体并行开发cpc机制（`cpc_dev` vs `cpc_dev/agent_1`）

cpc 源代码几万行，修改、编译、测试非常耗时，需要修改、测试cpc 源代码实现自愈功能时，需将多个功能分派给多个子智能体在各自拷贝的沙箱目录中修改、测试，加快cpc研发速度。

为避免多个智能体相互覆盖代码或阻塞等待，本项目采用 **“多缓冲隔离目录（`cpc_dev/agent_1(1-9)`）+ 乐观锁校验合并（CAS）”** 模式：

- 子智能体创建自己的cpc的拷贝目录，已经准备了9个： `cpc_dev/agent_1`、`cpc_dev/agent_2`。
- **子智能体开发与调试一律在自己的 `cpc_dev/agent_*` 目录中进行**，与正式目录 `cpc/`、`cpc/cpc_dev` 保持隔离，待测试全部通过后再按流程合并至 `cpc/cpc_dev`。

### 2. 并行开发与合并标准执行步骤

1. **使用 `env` 至调试沙盒**：

- `cap` 支持命令行`env CPC_BIN`设置cpc运行目录，来运行cpc， 你也可以直接使用你自己目录的cpc进行测试
- 通过注入 CPC_BIN 运行官方测试用例，其中cpc_dev_2为子智能体拷贝的工作目录：

```bash
   env CPC_BIN=/data/dev/cp/cpc_dev/agent_*/target/debug/cpc ./cap/bin/cap cap/test/test_err_and_move.cp
```

3. **启动前记录状态 & 测试后校验 `cpc_dev/src` 变更**：

- 启动拷贝工作区时，记录 `cpc_dev/src`的状态。
- 在 `cpc_dev/agent_*` 中修改代码（运行 `cargo check --manifest-path cpc_dev/agent_*/Cargo.toml` 与 `cargo build --manifest-path cpc_dev/agent_*/Cargo.toml`）并通过测试后，使用单条 Git 状态指令快速检查正式目录 `cpc_dev/src` 在此期间是否被其他子智能体修改过：

```bash
git status --porcelain cpc_dev/src
```

（或使用 `git diff cpc/src` 检查差异，避免使用复杂的 Shell 管道命令）。

- 3.1 **若 `cpc_dev/src` 无变化**：直接将工作区`cpc_dev/agent_*`代码合并到`cpc_dev/src`。
- 3.2 **若 `cpc_dev/src` 有变化**：先用文件读取与编辑工具将 `cpc_dev/src` 中的新增修改同步合并到 `cpc_dev/agent_*/src` 工作区中，重新编译 `cpc_dev/agent_*` 并运行测试，确保兼容无误。
- 重复 3.2，直至`cpc_dev/src` 无变化，按 3.1 合并子工作区代码到'cpc_dev/src'。

**务必注意**：

1. 以上 `cpc_dev_*`的`*`，表示各个子智能体创建的cpc 拷贝目录的数字编号，必须避免不同子智能体同时在一个工作区工作，同时修改同一个代码文件，导致代码混乱。
2. 每个 智能体占用一个 `cpc_dev/agent_*` 目录， 必须同步`cpc_dev/src` 到占用`cpc_dev/agent_*` 目录，并在该目录下的`work.json`（没有则创建）写入 状态为占用（state: 'busy'）。
3. 每个 智能体使用完 `cpc_dev/agent_*` 目录， 必须在该目录下的`work.json`（没有则创建）写入 状态为空闲（state: 'idle'）。
4. 每个 子智能体使用 `cpc_dev/agent_*` 目录时，需检查 该目录下的`work.json`状态为空闲（state: 'idle'）， 没有文件则创建，并直接占用，状态为‘busy’。

以上并发工作方案，目的是多子智能体并行工作，同时互不干扰，提高工作效率，缩短工作时间。
