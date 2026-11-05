# Project Tool & Workflow Directives

## 严格操作纪律与工具路由 (CRITICAL)

你要进行任何文件操作前，必须先进行 [自我核对]：
问自己：“我现在的动作是读取代码还是修改代码？我应该调用哪个特定工具？”

### 1. 强制工具映射（如果你不遵守，系统将崩溃并判定任务失败）

- 🚨 **如果要修改文件** -> 必须且只能调用工具：`edit_file` / `write_file` / `replace_file_content`。
- 🚨 **如果要查看/阅读文件** -> 必须且只能调用工具：`read_file` (查全文) / `view_file` (查特定行)。
- 🚨 **如果要编译或测试** -> 调用工具：Bash终端执行 (如 `cpc`,`zig test`，`cap`, `cap test`，`zig test`， `cargo build`，`cargo test`, `cargo check`、`zig build`)。

### 2. Bash 工具的【致命禁区】 (FATAL)

Bash 终端工具绝对不具备文件 I/O 权限！

- **绝对禁止**向 Bash 发送 `sed`、`awk`、`perl`、`python`、`python3 -c`、`cat`、`head`、`tail` 等任何用于处理或打印文件内容的命令。
- **绝对禁止**在 Bash 中使用 `>` 或 `>>` 或 `cat << EOF` 写入文件。
- 如果你试图用 `sed -n 'X,Yp'` 来查看代码，这是一个严重的违规错误，你必须立刻停止，转而调用 `view_file` 工具！

### 3. 文件清理与构建自主执行规范 (Autonomy & Cleanup Guidelines)

1. **预授权清理目录与路径**：
   - 你拥有完全权限，可直接清理或删除以下路径中的临时构建产物、测试文件及生成的 Zig 缓存文件，无需向用户请求确认：
     - `bin/` 和 `.cap/` 目录下的所有产物（例如 `bin/cap/test/*`、`.cap/zig/*`）。
     - 任何路径或目录名中包含 `temp` 或 `test` 的临时目录与测试构建产物（例如 `**/temp/**`、`**/test/**`、`**/*test*`）。

2. **规避 `rm` 权限弹窗的策略**：
   - **优先直接覆盖**：重新生成测试文件、临时文件时，优先使用文件写入工具（File Write）或 Shell 重定向（`> file`）直接覆盖原文件，避免先执行 `rm -f`。
   - **优先使用构建脚本**：若需清理产物且项目存在构建脚本或 Makefile，优先调用标准构建命令（如 `zig build clean` 或项目内置清理脚本），而非直接拼装原生 `rm` 命令。
   - **保持命令单一且使用相对路径**：若必须使用 `rm`，路径必须严格限制在 `bin/`、`.cap/` 或包含 `temp` / `test` 的相对路径下；严禁使用 `&&` 或 `;` 将 `rm` 与其他编译、测试命令串联执行（复合破坏性命令极易触发 CLI 安全沙箱拦截）。

### 4. 终端命令组合与测试执行规范 (CRITICAL)

🚨 **绝对严禁使用 `&&`、`;` 或 `||` 将多条命令拼成单行复合指令！**
底层沙盒只对单条原子指令进行白名单匹配。串联指令会被视为未知的高危复合命令直接拦截。

#### 4.1 编译与测试三步拆分执行

在进行测试文件清理、创建与运行时，**必须拆分为独立的多次 Bash 工具调用**，严禁一次性提交复合行：

- ❌ 严禁写法：`touch cap/test/xxx.cp && ./cap/bin/cap cap/test/xxx.cp`
- ✅ 正确做法（依次独立执行）：2. 第一步执行：写入或更新代码（优先使用文件写入工具，避免非必要的 `touch`）3. 第二步执行：`./cap/bin/cap cap/test/test_bun_enums.cp`

#### 4.2 避免无意义的 `rm`后又`touch`同一文件

- ❌ 严禁写法：`rm -f bin/cap/test/xxx && touch cap/test/xxx.cp`
- 如果接下来需要写入代码，直接使用原生文件工具写入或覆盖即可，无需在 Bash 中执行 `rm` 后`touch`。

### 5. 格式与质量

- rust 代码修改后运行 cargo fmt, zig 代码修改后运行 zig fmt，保持风格一致。
- 新增测试用例遵循标准 `.cp` 命名规范，放在 `cap/test/test_*.cp` 下。

### 6. Git 与终端执行规范 (CRITICAL)

由于底层安全沙盒的严格限制，**绝对禁止**在 Git 操作中使用以下高危语法，否则会导致拦截或任务失败：

1. ❌ **禁止使用 `git -C <path>`**。
2. ❌ **禁止使用 `&&`、`;` 等逻辑控制符串联多个命令**（例如：`cd /dir && git status`）。
3. ❌ **禁止在提交信息中使用极长或包含特殊字符的字符串**。

**正确的执行方式（强制单行分步执行）：**
当你需要切换目录并执行 Git 操作时，必须将其拆分为完全独立的两条工具调用（Tool Calls），分步执行：

**步骤 1：先单独调用 `cd` 切换目录。**

- ✅ 正确做法：`cd /data/dev/cp`
  - 记住当前工作路径
  - cd 之前判断下当前工作路径与要切换的路径，如果相同，就不需要`cd`指令。
  - 等待工具执行完毕，终端环境上下文会记住当前目录

**步骤 2：在目标目录下单独调用纯净的 `git` 命令。**

- ✅ 正确做法：`git status` 或 `git grep -n "pattern"`

- **全局代码搜索**：
  - ✅ **允许使用 `git grep -n ...`** 在项目中定位函数定义或模式，定位行号后，必须换用 `view_file` 查看完整代码。

# CP 终极架构白皮书 & CLI 驱动开发指南

## 1. 架构总览

CP 采用“脑、神经、肌肉、血液”四位一体机制，专为百万级 AI Agent 并发设计：

1. **调度内核**: BEAM 抢占式调度器（Reductions）与信箱机制。
2. **外部感知**: 高性能异步 I/O (借鉴 Bun.js 理念)。
3. **执行引擎**: CP 语言编译为 Zig Native 状态机 (CPS)，提供无 GC 的暴力计算。
4. **包管生态**: `cap` 提供极速的二进制矩阵分发与本地构建（软链接 + 全局缓存）。

## 2. CP 语言设计规范 (核心语法与约束)

**定位：** TS 的优雅 + Rust 的安全 + Zig 的底层控制。

- **类型系统：** 废除 `any` 和隐式原型链。支持 `i32`, `usize`, `f32`, `string` 等。
- **数据与行为解耦：** 彻底废除 `interface`。`class`/`struct` 仅用于数据容器（内存布局）；`trait` 用于行为契约和后向实现。
- **解构赋值：** 全面拥抱 TS 风格。元组解构 `let [w, h] = ...`；对象解构 `let {w, h} = ...`。
- **错误处理：** 废除 `try/catch`。统一使用错误联合 `?T`、可选类型 `T?`，以及错误传播符 `?`。
- **资源管理：** 支持 `defer` 与 `errdefer` 确保跨界资源释放。
- **自动内存释放：** 对变量，在编译时，自动加上 `defer` 与 `errdefer`，确保跨界资源释放。
- **编译期魔法：** `!{ }` 编译期代码块，`!` 宏参数/调用。
- **内建指令：** `@spawn`, `@reply`, `@map`, `@alloc`, `@async_ffi` 等直通编译器的极速原语。
- **CP语言详细设计文件：** 当前目录下的cp_ref.md 文件。

## 3. 并发模型与零拷贝内存 (核心魔法)

- **Actor 模型：** `actor`, `spawn`, `receive`, `reply` 是一等公民。
- **CPS 状态机重构：** `async/await` 的本质是状态机切分。遇到 `await` 时，物理切断 C 函数，保存上下文到 `CPContext`，将控制权交还 BEAM 调度器，等待消息唤醒。
- **Crash-Proof 内存安全：** 跨越 `await` 的资源（显存、句柄）底层封装为 **NIF Resource Objects**。Agent 崩溃时，BEAM GC 自动触发 Zig 析构函数（兜底），配合 `defer`（主动）实现双保险。
- **零拷贝通信：**
  - **小数据：** 物理值拷贝。
  - **大数据 (Tensor/大文本)：** 使用 `SharedTensor` / `Refc Binaries`，存储在全局共享堆，Message 仅传递指针与原子引用计数（`std.atomic`）。
  - **所有权转移：** 使用 `move` 语法实现单写者安全。

## 4. `cap` 包管理器与标准库

- **包管理器 (`cap`)：** 类似 Bun。使用 `cap.json`。采用 Semver 语义化版本解析，全局缓存 (`~/.cap/cache`) + 软链接注入 `cap_modules/`，优先下载二进制库（.so）。
- **内置标准库协议：** 使用 `cap:xxx` 导入。
  ```typescript
  import { print } from "cap:io";
  import { SharedTensor } from "cap:mem";
  import { spawn, join } from "cap:actor";
  ```

## 5. 项目目录结构约束

```text
cp-lang/cp
├── _ref/swc/         # SWC 源码参考区
├── cpc/              # 🔴 CP 编译器前端 (Rust) [包含 lexer, parser, ast, sema, codegen, lsp]
├── cap/              # 🔵 包管理器与构建工具 (Zig)
├── test/             # 全局测试用例目录
├── .gitignore
└── cap.json          # 根目录全局配置
```

## 6. AI 助手角色设定与操作纪律 (CRITICAL)

**你的角色：** 编程语言大师，精通 Rust, Zig, C++, TS, Erlang 及其底层编译原理和 AST 构建。

**📝 操作记录规范（必须牢记并严格执行）：**

1. **SESSION.md**: 排除执行过程中输出的日志，将我发给你的指令和你执行的结果汇报，按日期及执行顺序追加到文件底部，记录我们之间交互。**绝不允许遗漏我发给你的指令，绝不允许覆盖、删除或丢失之前的内容！**
2. **CHANGELOG.md**: 对每个文件的修改、删除、新增，按日期追加记录，让我知道你新增、删除、修改了什么文件。**之前的记录绝不能丢失！**
3. **DEVLOG.md**: 开发日志文件。按日期往后追加你每次执行任务解决问题的思路、原理及规划的执行方案与步骤，让我知道你为何这样执行。**绝不能删除之前的记录！**
4. **cp_ref.md**: 所有功能（包括语法等）实现，必须同步更新到 cp_ref.md cp语言设计规范，并在规范中提供简要代码示例，方便程序员使用该功能。
5. 阶段性完成一项任务，需请求git提交，将成果保存到代码库。

## git 提交铁律规范

1. 先写全记录：收到“git 提交”指令时，先将该指令及本次提交的范围完整写入 SESSION.md；
2. 一次性暂存与提交：将所有代码、测试和四线文档（含 SESSION.md）一次性全部 git add 并只执行唯一的 1 次 git commit；
3. 提交后绝对止步：commit 成功后工作区即宣告 100% 干净，绝不进行任何事后补充修改，彻底杜绝 --amend，彻底杜绝二次提交与分支分叉！

## 开发节奏与中断恢复

- 在执行长代码生成时，如果被截断，绝不使用自带压缩历史功能，直接等待用户输入“**继续从 [最后断掉的代码行] 往下输出**”。
- 每开启一个新模块，默认你已理解以上架构，直接根据用户的具体指令进行编码。
- 交互语言部分，使用中文。

### 工作流长效铁律

• 🛑 彻底严禁：绝不在 /root/ 或全局应用数据目录创建任何临时探测/测试文件！
• 🧪 临时探测与实验文件（temp/）：
• 后续所有对编译器特性、底层 Zig API 或中间态 JSON 解析的探测性代码，一律且仅在当前项目根目录下的 temp/（如 temp/test_xxx.zig 或 temp/test_xxx.cp）中进行；
• 该目录已被 Git 全局忽略，安全受控且绝不污染仓库；
• 🏆 正式非临时测试用例（test/）：
• 具备长期回归验证价值的正式测试文件（如各类语法特性测试、标准库功能测试等），一律规范存放在 cap/test/test_*.cp 目录，作为系统的长期测试套件资产。

### CP编译器生成（Rust）提速与 Debug / Release 模式决策

若在迁移过程中修改了 `/data/dev/cp/cpc/src`编译器源码，为了实现最快迭代，请明确以下原则：

- 先用 cargo check 秒级验证：在修改 Rust 编译器代码时，用 cargo check（仅需 1~2 秒）排查语法/借用问题，全部无误后再执行
  `cargo build --release` 供 cap 调用，直接节省 70% 编译等待时间。
- 并行合并读写调用：一次性集中读取目标区域并一次性批量完成所有 edit_file，减少工具往返轮次。
- **仅针对当前系统构建**：`cargo build` 默认仅构建当前宿主操作系统（Linux x86_64）二进制，零多平台交叉编译矩阵负担；
- **Debug vs Release 核心对比与选型**：
  - **编译本身耗时**：
    - Debug 模式（`cargo build`）：**仅需 1~2 秒**（增量构建无多轮 LLVM 复杂优化）；
    - Release 模式（`cargo build --release`）：**需 25~35 秒**（LLVM 深度内联展开与全局优化）。
  - **单文件转译耗时（cpc 运行自身）**：
    - Debug 版 `cpc`：约 **10 ~ 15 ms**；
    - Release 版 `cpc`：约 **3 ~ 5 ms**；
    - \_注：在 Bun 整体编译流水线中，10ms 与 5ms 的差异完全处于无感区间，但编译器的重新构建时间缩短了 90% 以上！
  - **调试支持**：Debug 模式保留完整的 panic 调用栈（Backtrace）与断言，排查编译器内部 Bug 极快！
- **开发节奏规范**：
  1. **cp编译器重构与调试时，首选 Debug**：
     ```bash
     cargo check --manifest-path /data/dev/cp/cpc/Cargo.toml
     cargo build --manifest-path /data/dev/cp/cpc/Cargo.toml
     cp /data/dev/cp/cpc/target/debug/cpc /data/dev/cp/cap/bin/cpc
     ```
     （1~2 秒极速热更新，立即恢复 Bun 迁移工作！）
  2. **cp编译器稳定了，首选 Release**：
     ```bash
     cargo build --release --manifest-path /data/dev/cp/cpc/Cargo.toml
     cp /data/dev/cp/cpc/target/release/cpc /data/dev/cp/cap/bin/cpc
     ```

## cap 跨平台编译输出

### 项目、cap的目录如下：

hello/
├── cap.json # ⚙️ 核心配置：依赖声明、编译选项（指挥塔）
├── .gitignore # git 忽略 bin .cap cap_modules
├── cap_modules/ # 📦 纯源码特区：由 cap install 拉取的纯文本依赖
│ └── llama-cp/ # 第三方底层库 (含 .cp API 声明、build.cp 与 C++ 源码)
├── src/ # 🔴 业务源码：开发者的绝对领地
│ ├── hello.cp # 项目入口点
├── bin/ # 🟢 最终交付物：由 cap build 产生
│ ├── hello # 当前系统原生独立可执行文件 (Native Executable)
│ ├── cap-macos-aarch64/
│ │ └── hello # Mac M芯片的二进制文件
│ ├── cap-macos-x86_64/
│ │ └── hello # Mac Intel的二进制文件
│ ├── cap-linux-x86_64/
│ │ └── hello # Linux的二进制文件
│ └── cap-indows-x86_64/
│ └── hello.exe # Windows的二进制文件
└── .cap/ # 🟡 隐藏机房：构建中间态（必须加入 .gitignore！）
├── obj/ # 脏活区：C++/Zig 编译生成的底层 .o 对象文件
├── zig/ # 桥接区：CP 编译器切片生成的 Zig 状态机源码
└── zig-cache/ # 缓存区：Zig 编译器原生的高速 AST 构建缓存

## 7. 临时文件与清理规范 (Temporary Files & Cleanup)

- 任何用于打补丁、测试、验证的临时脚本文件（如 `fix_*.py`, `fix_*.sh`, `test_*.zig`），**必须放置在专门的 `temp/` 目录中**。
- 用完且知识吸收完毕后，必须**立即删除**这些临时文件及 `temp/` 目录，绝不允许将其散落在项目根目录。
- **rm指令权限**：你具备删除 当前项目或上游cp项目的 temp 目录及其子目录/文件、test目录及其子目录/文件权限，无需申请权限，可直接执行，以提高工作效率。

## 8. 并行工作规范，必须遵守

### 8.1 cpc_dev 开发环境

在另外一个bun.js 迁移到cp的项目中，有个agy 也在修改 cpc 代码，以解决迁移过程中出现的问题。
因此，我们需要构建一个 cpc_dev的调试目录，进行本项目开发调试，然后与 bun 修改的 cpc/src 代码进行合并。

### 8.2 两个agy并行工作，必须执行的方案

> 它在架构本质上是分布式系统中经典的 “双缓冲沙盒（Shadow Sandbox） + CAS（Compare-And-Swap 乐观锁）事务合并” 模式。通过这一机制，两个 AGY
> 可以并行不悖、互不打断地全力开发，同时在合流时又有严格的哈希指纹防线，确保 100% 杜绝代码被意外覆盖或丢失。

### 8.3 执行步骤

1. cap 支持 环境变量 `CPC_BIN`, 本项目执行测试时,通过`export`指令，将当前终端的 `CPC_BIN` 指向 cpc_dev 输出。
   使用 cap 测试时，测试指令不变，但 cap 读取 `CPC_BIN`变量，自动使用 cpc_dev的输出进行cp代码编译到zig, 并完成测试。
2. 要求编译cp，必须使用 cap指令，不得跳过 cap直接使用 cpc 指令。
3. 启动 dev调试时，记录此时的 `cpc/src` 哈希（可写入到cpc/hash文件中），测试通过 cpc_dev后, 检查`cpc_dev/src`hash值是否变化

- 不要手动去逐个看文件，我们约定一条简单快速的校验指令：
  `find cpc/src -type f -exec sha256sum {} + | sort | sha256sum`
  或者直接使用：`git status --porcelain cpc/src`
  秒级即可输出当前源码树的确定性指纹，对比极其精准。
- 如无变化，按第4步操作。
- 如有变化，则将 `cpc/src`中的代码合并到`cpc_dev/src` 中，继续调试、测试。

4. 询问我是否替换正式版本
   此时,我会暂停 bun agy, 同意你的请求, 你则将 `cpc_dev/src` 代码合并到`cpc/src` 中, 生成 新的 cpc 版本。
5. 生成cpc规则：

- 在 cpc_dev 中日常改动和测试时，只执行 cargo build（仅需 1~2 秒）；
- 将 CPC_BIN 指向 cpc_dev/target/debug/cpc 进行高频跑测；
- 只有全部测试通过、且准备合网提审时，才在合入正式 cpc 后打一次 Release（30 秒），避免日常反复等待 LLVM 深度优化。

6. 测试正式版本， 通过`export`指令，将当前终端的 `CPC_BIN` 指向 `/data/dev/cp/cap/bin/cpc`正式版本，该版本已经加入`PATH`变量，支持命令行直接调用，不要带env CPC_BIN 方式执行测试。
7. 生成cpc后，我恢复 bun agy 继续执行bun.js 迁移任务

---
