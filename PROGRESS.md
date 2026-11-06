# CP-Lang 项目核心进展与交接文档 (PROGRESS.md)

> **文档定位**：本文件为当前会话达成 45,000+ 行交互后的**核心技术资产与上下文交接文档**。新开启的 Antigravity (agy) 智能体在阅读本文件后，可完全无缝接续当前 CP-Lang 项目的架构演进、编译器开发与测试验证工作。
> 该会话历史ID：agy --conversation=ebe8932a-593c-40d2-a931-f9ce1d4de5b9
> **基线时间戳**：2026-11-06
> **当前仓库分支**：`main`
> **当前最新 Git Commit**：`3ab7809 feat(cpc): add C-FFI decls, cstr, raw pointers, const generics, reflection, and enum introspection`

---

## 一、 项目总览与核心设计哲学

### 1. 项目愿景与架构定位

**CP (Concurrent Programming) 语言**专为百万级 AI Agent 高并发而设计，融合了：

- **TypeScript 的优雅**：极简现代语法、流式类型推断、可选链与解构；
- **Rust 的内存安全**：所有权转移（`move`）、借用检查、Fail-Fast 显式解包；
- **Zig 的原生控制力**：无 GC 运行、CPS 状态机重构、编译期反射（Comptime）、零开销 C-FFI；
- **Erlang/OTP BEAM 的并发韧性**：Actor 模型、抢占式微调度、进程隔离与自愈（"Let it crash"）。

### 2. 核心架构拓扑

```text
cp-lang/cp
├── cpc/              # CP 编译器前端正式目录 (Rust) [发布 Release 版，由 cap/bin/cpc 消费]
├── cpc_dev/          # CP 编译器并行开发调试沙盒 (Rust) [日常高频秒级验证 Debug 版]
├── cap/              # 包管理器与构建运行工具 (Zig)
│   ├── bin/          # 构建产物与全局二进制 (cap, cpc)
│   └── test/         # 长期回归集成测试套件 (test_*.cp)
├── cp_ref.md         # 官方权威语言设计规范手册（权威真理来源）
├── GEMINI.md         # CLI 驱动开发指南与双智能体并行规范
├── SESSION.md        # 四线文档：全量用户交互与执行记录（严禁删除或覆盖历史）
├── CHANGELOG.md      # 四线文档：文件级变更追踪轨迹
├── DEVLOG.md         # 四线文档：架构推演、底层原理与设计日志
└── PROGRESS.md       # 本交接文档
```

---

## 二、 铁律规范（新 Agent 必须 100% 遵守）

1. **双缓冲并行开发机制（`cpc_dev` vs `cpc`）**：
   - 另一个智能体正在并行开展 Bun.js 迁移工作，会直接向 `cpc/src/` 提交改动；
   - **日常开发一律在 `cpc_dev/` 中进行**（秒级 `cargo check` 与 `cargo build`）；
   - `cap` 已通过环境变量 `CPC_BIN` 或 `.cap/cpc_bin` 绑定至 `/data/dev/cp/cpc_dev/target/debug/cpc`；
   - **向正式版 `cpc/` 合并代码必须先取得用户明确确认**，严禁擅自覆盖。
2. **工具路由规范**：
   - 修改/创建文件：**必须**调用内置 `edit_file` / `write_file`；
   - 查看文件：**必须**调用内置 `read_file`；
   - 代码搜索：**必须**调用内置代码搜索工具；
   - Shell 命令：仅用于编译构建（`cargo`、`cap`、`zig`）与 Git。**严禁**使用 `cat`、`head`、`sed`、`awk`、`grep`、`find` 处理文件。
3. **终端单步执行规范**：
   - **严禁使用 `&&`、`;`、`|`、`||` 拼接多条命令**，终端执行器只匹配单条原子指令；
   - `cd` 与后续命令必须拆分为独立的工具调用（但注意终端每个会话独立，推荐直接使用绝对路径或相对路径，无需切换目录）。
4. **四线文档同步规范**：
   - 每次任务完成后，必须同步追加更新：`SESSION.md`、`CHANGELOG.md`、`DEVLOG.md`，并在涉及语言特性时更新 `cp_ref.md`；
   - **绝对禁止遗漏、截断或覆盖任何历史内容**。

---

## 三、 近期核心里程碑与特性落地详解

### 1. 控制流类型智能收窄 (Flow-Sensitive Type Narrowing)

- **语法形态**：
  ```typescript
  if (res != null) {
    // 离开外层可选包裹，res 自动由 Type? 收窄为 Type
    res.doSomething();
    print(res.field);
  }
  ```
- **支持场景**：
  - 正向判断：`if (v != null)`；
  - 反向判断：`if (null != v)`；
  - 复合短路：`if (v != null && v.len > 0)`；
  - 分支互逆：`if (v == null) { return; } else { /* v 自动收窄为非空 */ }`；
  - 嵌套 `if` 与 `while` 控制流递归收窄。
- **底层编译器实现**：
  - **`cpc_dev/src/sema.rs`**：
    - 注入 `extract_not_null_vars` 与 `extract_is_null_vars` 提取控制流约束；
    - 在 `Stmt::If`、`Stmt::While`、`Expr::If`、`BinaryOp::LogicalAnd` 分支内将符号表中的 `Type::Optional(T)` 替换为 `T`；
    - 将 `infer_type(&mut self, expr)` 改造为可变借用，流敏感维护分支局部类型映射。
  - **`cpc_dev/src/codegen.rs`**：
    - 引入 `narrowed_null_vars: HashSet<String>` 跟踪当前控制流收窄变量；
    - `Expr::Ident`：被收窄变量自动发射 `({}.?)`（非状态机 `(v.?)`，Lambda 闭包状态机 `(ctx.v.?)`），实现对字段访问、方法调用、可变借用方法调用零样板穿透；
    - **左值赋值安全防御**：在 `Stmt::Expr(BinaryOp::Assign)` 和 `Expr::Bin(BinaryOp::Assign)` 中，为被收窄变量赋值时临时剥离收窄，防止生成非法的 `(ctx.count.?) = 100`；若赋右值为 `null`，永久注销收窄；
    - **显式解包幂等保护**：在 `Expr::Unwrap` 中识别已收窄变量，仅发射单层 `.?`，彻底杜绝 `(u.?).?` 二次解包报错；
    - **Lambda 闭包局部变量完整收集**：修复 `extract_local_vars_to_str` 对 `Stmt::If`、`Stmt::While` 与 `Stmt::TryCatch` 的递归扫描，保证测试块与闭包中分支内局部变量（`sum`、`unwrapped`）正确进入 `lambda_N_Context`。
- **验证测试**：[`cap/test/test_if_null_narrowing.cp`](file:///data/dev/cp/cap/test/test_if_null_narrowing.cp)（5/5 测试全部通过）。

### 2. 解包原语体系与错误哲学

- **`u.?`（强制显式解包）**：
  - 断言必须非空，若为 `null` 触发 Panic / Exception；
  - 符合 BEAM Actor "Let It Crash" 设计哲学，用于不变性守卫、测试断言与 C-FFI 边界；
  - 显式解包具有幂等性：在已收窄作用域手写 `u.?` 不会产生二次解包。
- **`u?.`（可选链）**：
  - 安全短路，遇 `null` 停止求值并直接回退为 `null`，绝不中断或 Panic；
  - 表达式最终类型保持为可选类型 `T?`。
- **`u ?? def`（空值合并）**：
  - 当左侧为 `null` 时降级采用右侧默认值 `def`，返回确定性的非空类型 `T`。
- **规范章节**：详见 [`cp_ref.md`](file:///data/dev/cp/cp_ref.md) 第 8.3 与 8.3.1 节。

### 3. 裸指针与 Unsafe 内存操作矩阵

- **核心语法与类型区分**：
  - `*T`：安全可变借用（受到 Sema 借用检查器追踪，无 `!`）；
  - `&T`：安全只读借用；
  - `*T!`：底层单项可变裸指针（必须在 `unsafe` 块中解引用操作）；
  - `*const T!`：底层单项只读裸指针（指向目标内存只读，禁止 `p.* = val`）；
  - `[*]T!`：多项裸指针（支持 C 风格下标 `ptr[i]` 与切片 `ptr[a..b]`）；
  - `*any!`：不透明通用裸指针（对标 C `void*` 与 Zig `*anyopaque`）。
- **变量常量性与指针只读性的正交解耦**：
  - `const p: *T!`：指针变量 `p` 本身不可变（不可重定向），但指向的内存可修改（`p.* = 100` 合法）；
  - `let p: *const T!`：指针变量 `p` 本身可变（可指向其他内存），但指向的内存只读（`p.* = 100` 编译拦截）。
- **获取裸指针的三种标准途径**：
  1. `let p: *i32! = *num!;`（直接裸指针字面量，已在 `parser.rs` 与 `codegen.rs` 中完整实现）；
  2. `let p: *i32! = &num as *i32!;`（只读借用强转裸指针）；
  3. `let p: *i32! = *num as *i32!;`（可变借用强转裸指针）。
- **验证测试**：[`cap/test/test_unsafe_pass.cp`](file:///data/dev/cp/cap/test/test_unsafe_pass.cp)（已包含三种获取裸指针方式与解引用验证，100% 绿灯）。
- **规范章节**：详见 [`cp_ref.md`](file:///data/dev/cp/cp_ref.md) 第 6.7.1 ~ 6.7.4 节。

### 4. 既有基础架构特性

- **Top-Level C-FFI 声明**：`__zig__ { extern "c" fn ... }`，零额外包装直接映射机器码 ABI；
- **编译期 Comptime Auto-Drop (RAII)**：基于 `@typeInfo` 与 `@hasDecl(T, "release")` 在 Comptime 智能注入释放，兼顾 TS 优雅与 Rust 确定性析构；
- **错误联合与传播**：`?T` 映射 Zig `anyerror!T`，`T?` 映射可选值，后缀 `?` 自动传播错误；
- **常量泛型与反射**：支持 `@typeName`, `@fields`, `@values` 深度编译期内省。

---

## 四、 当前仓库状态与双智能体同步状态

### 1. `cpc/src` vs `cpc_dev/src` 状态

- **Bun 智能体近期改动**：
  - `cpc/src/main.rs`：Parser 错误定位增强（输出行列号与代码片段）；
  - `cpc/src/parser.rs`：`parse_ident` 支持识别 `Token::TypeKw` 为普通标识符；
  - `cpc/src/sema.rs`：预声明 `TypeAlias` 与 `Import` 符号；
  - `cpc/src/codegen.rs`：针对 `needs_unpack` 在没有 collections 时的 Message 桩代码补丁。
- **同步状态**：
  - 以上 Bun 智能体的所有变更已 **100% 无冲突合并进 `cpc_dev/src/`**；
  - `cpc_dev/` 中包含本智能体新增的全部流敏感类型收窄与安全防御代码；
  - `cpc_dev/` 当前通过 `cargo check`（0.06s）与 `cargo build`（0.06s）；
  - 测试套件全部绿灯通过。

### 2. 当前待办节点（Waiting for User Decision）

- **合并提请节点**：`cpc_dev` 中的功能已全部验证就绪，已向用户询问是否将 `cpc_dev/src/` 合并至正式版 `cpc/src/` 并构建 Release 二进制（`cargo build --release --manifest-path cpc/Cargo.toml` -> 覆盖 `cap/bin/cpc`）。
- **若用户指示合并**：执行合并流程（见下文第五节）；
- **若用户提出新特性需求**：继续在 `cpc_dev/` 中进行开发迭代。

---

## 五、 接手 Agent 快速操作指引 (Runbook)

### 1. 验证当前沙盒环境

在执行任何新改动前，运行以下指令确认沙盒工作正常：

```bash
cargo check --manifest-path /data/dev/cp/cpc_dev/Cargo.toml
```

```bash
./cap/bin/cap cap/test/test_if_null_narrowing.cp
```

```bash
./cap/bin/cap cap/test/test_unsafe_pass.cp
```

### 2. 合并 `cpc_dev` 到 `cpc` 标准流程（仅在用户明确同意时执行）

1. 检查 `cpc/src` 是否有外部新增修改：
   ```bash
   git status --porcelain cpc/src
   ```
2. 使用文件读写工具（或同步工具）将 `cpc_dev/src/` 中的文件同步覆盖到 `cpc/src/`；
3. 构建正式 Release 版编译器：
   ```bash
   cargo build --release --manifest-path /data/dev/cp/cpc/Cargo.toml
   ```
4. 将产物复制至 `cap/bin/`：
   ```bash
   cp /data/dev/cp/cpc/target/release/cpc /data/dev/cp/cap/bin/cpc
   ```
5. 运行回归测试验证正式版二进制：
   ```bash
   ./cap/bin/cap cap/test/test_if_null_narrowing.cp
   ```

### 3. Git 提交三步规范

当用户给出“git 提交”指令时：

1. **先写全记录**：将该指令追加写入 `SESSION.md`，并在 `CHANGELOG.md` 中记录涉及的文件清单；
2. **一次性原子提交**：
   ```bash
   git add .
   ```
   ```bash
   git commit -m "feat/fix(模块): 简明描述"
   ```
3. 提交后工作区保持 100% 干净，杜绝二次修改或 `--amend`。

---

_交接文档完毕，后续接手 Agent 可直接根据此文档继续高效推进！_
