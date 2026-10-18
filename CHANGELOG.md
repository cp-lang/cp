## 2026-10-06
- **Doc**: Refactored `cp_ref.md` to incorporate recently developed CP features, including Move semantics, Context Lifting (CPS Await), OOP destructors (`init`/`release`), automatic `defer`, cross-thread workers, and ES6 template strings.
- **Doc**: Added future architecture proposals (Typed Mailboxes, Pattern Matching Guards, Pipeline Operator, Comptime Macros) to `cp_ref.md`.

## 2026-10-06 (Re-Execution)
- **Doc**: Safely appended recent CP language implementations (Move semantics, CPS Context Lifting, RAII with auto-defer, OOP init/release) to `cp_ref.md` using compliant file editing tools.
- **Doc**: Appended architectural evolution proposals (Typed Mailboxes, Guard Clauses, Pipeline Operator, Comptime Macros) to `cp_ref.md` without destructive file operations.

## 2026-10-06 晚间 - 清理根目录残留实验代码
- **删除**: 删除了根目录下约 54 个 `test_*.zig` 实验性草稿文件（如 `test_linux_accept.zig`, `test_channel.zig` 等）。
- **删除**: 删除了上述 `.zig` 文件对应的同名无后缀 ELF 编译二进制可执行文件。
- **删除**: 删除了残留的测试日志与输出文件 `test_data.txt` 和 `test_io.txt`。
- **删除**: 删除了废弃的历史工作区 `test_project` 目录（其源文件早已迁入 `cap/test/`）。
- 2026-10-06: Added `RegExp` class and ES6 regular expression methods (`.test`, `.match`, `.exec`, `.replace`). Fixed and implemented all String methods (`indexOf`, `slice`, `toUpperCase`, etc) using Zig native functions. Modified `cap/src/main.zig` to automatically link libc (`-lc`) for POSIX regex support.
- 2026-10-06: Fully encapsulated and implemented the global `Math` object, mapping it to Zig's `std.math` functions. Fixed CP Parser to correctly handle `FloatLiteral` inside expressions. Fixed `cp_eq` operator to properly evaluate float equality. Added `cap/test/test_math.cp`.
- 2026-10-06: Implemented `Map` and `Set` advanced data structures. Extended the `Message` union to include `boolean`. Integrated custom hash contexts in Zig to dynamically hash and equate `Message` types allowing mapping of strings, floats, ints, and bools seamlessly. Fixed a critical pointer stability panic in `Expr::New` object allocation where memory was erroneously reset to `undefined` after proper initialization.

- 2026-10-11: 
  - **cpc/src/lexer.rs, ast.rs, parser.rs, sema.rs**: Added `mut` and `move` keywords and tokens, removed `let mut`. Added Semantic errors for borrowing and moving conflicts.
  - **cpc/src/codegen.rs**: Enhanced State Machine generation to track variable ownership (Move semantics). Injected `__var_moved` flags and `defer release()` for Automatic Resource Management (Auto-Drop).
  - **cap/test/test_ownership_pass.cp**: Added positive test case for RAII / Auto-Drop execution.
  - **cap/test/test_ownership_fail.cp**: Added negative test case ensuring Use-After-Move triggers a compile-time panic.
  - **cp_ref.md**: Updated memory safety specification guidelines.

## 2026-10-16 - 零摩擦生命周期与栈绑定借用视图 (`ref struct`)
- **修改**: `cp_ref.md` — 完善第十章内存安全模型，新增零摩擦生命周期（NLL、三级函数返回生命周期推导、`&param` 绑定）、`Shared<T>` 与 C# 风格 `ref struct`（栈绑定借用视图）规范。
- **修改**: `cpc/src/lexer.rs` — 新增 `Token::Ref` (`ref`) 与 `Token::Struct` (`struct`) 词法标记。
- **修改**: `cpc/src/ast.rs` — `Class` 新增 `is_ref: bool` 标记；`Expr` 新增 `Borrow(Box<Expr>, bool)`；`Type` 新增 `BorrowType { inner, is_mut, lifetimes }` 与 `Shared(Box<Type>)`。
- **修改**: `cpc/src/parser.rs` — 支持 `ref struct` / `ref class` 解析（及字段含借用时自动推导为 `is_ref`），支持 `->` 与 `:` 返回类型，支持 `&expr` / `&mut expr` 表达式及 `&x T` / `&(x | y) T` 参数生命周期类型标注，支持未初始化变量声明 `let r: &T;`。
- **修改**: `cpc/src/ast_visitor.rs`, `cpc/src/dcp.rs`, `cpc/src/sema.rs` — 适配 `Expr::Borrow`、`Type::BorrowType`、`Type::Shared` 及 `Class::is_ref`。
- **修改**: `cpc/src/codegen.rs` — 实装 NLL 非词法生命周期预扫描 (`compute_last_uses_block`)、作用域深度逃逸检查 (`var_scope_depth`)、三级函数签名生命周期溯源 (`extract_expr_borrow_info`)、以及 `ref struct` 跨作用域/跨函数/跨 `@spawn` 逃逸静态拦截。
- **新增**: `cap/test/test_lifetime_pass.cp` — 生命周期、NLL、`&x T` 绑定与 `ref struct` 综合正向测试。
- **新增**: `cap/test/test_lifetime_fail_dangling.cp` — 内部短生命周期变量借用逃逸至外部变量报错测试。
- **新增**: `cap/test/test_lifetime_fail_nll.cp` — 借用尚未结束（后续仍有读取）时提前 Move 报错测试。
- **新增**: `cap/test/test_ref_struct_fail_escape.cp` — 栈绑定 `ref struct` 试图通过 `return` 逃逸出函数报错测试。

---

## 2026-10-18 (Trait、单继承 Class 与 `_`/`this` 实例所有权、借用及生命周期)

### 修改文件 (Modified)
- [`cp_ref.md`](file:///data/dev/cp/cp_ref.md):
  - 更新第 1 节 `match` 模式匹配通配符为 `*`，避免与自身实例别名 `_` 冲突；
  - 更新第 2 节 `trait` 与 `class` 规范，明确支持单继承 `extends` 与 `trait` 默认方法实现；
  - 新增第 10.5 节《`trait`、单继承 `class` 与自身实例 `_` / `this` 的所有权、借用与生命周期》，完整定义 `_`/`this` 同义符号、省略 `_.` 前缀规则、首参缺省只读借用与 `mut _` / `move _` 接收者标注、方法返回借用的 Tier 1 生命周期自动穿透，以及 `class extends` 与 `trait` 静态多态的借用安全契约。
- [`cpc/src/lexer.rs`](file:///data/dev/cp/cpc/src/lexer.rs):
  - 新增 `Token::Extends` (`extends`) 与 `Token::Super` (`super`) 词法标记。
- [`cpc/src/ast.rs`](file:///data/dev/cp/cpc/src/ast.rs):
  - `TraitMethod` 新增 `body: Option<BlockStmt>` 以支持 `trait` 默认方法实现；
  - `ClassDecl` 新增 `super_class: Option<Ident>` 以支持 `class Child extends Parent` 单继承。
- [`cpc/src/parser.rs`](file:///data/dev/cp/cpc/src/parser.rs):
  - `parse_params` 支持成员方法首参 `_`、`mut _`、`move _`（以及 `this`、`mut this`、`move this`）无类型简写解析；
  - `parse_trait` 支持解析带函数体的默认方法与不带函数体的契约签名；
  - `parse_class` 支持解析 `extends Parent` 单继承子句与 `implements Trait1, Trait2`；
  - `parse_pattern` 将模式匹配通配符切换为 `*` (`Token::Star`)，使 `_` 在 `match` 分支内依然指向自身实例 `this`。
- [`cpc/src/sema.rs`](file:///data/dev/cp/cpc/src/sema.rs):
  - 完善 `TraitDecl`、`ClassDecl`、`ImplDecl` 语义遍历，注册 `this`、`_`、`super` 隐式实例上下文并跳过接收者首参的类型计数偏移。
- [`cpc/src/codegen.rs`](file:///data/dev/cp/cpc/src/codegen.rs):
  - 新增 `ReceiverMode` (`ReadOnly`、`Mut`、`Move`、`Static`) 推导与检查体系；
  - 实现 `resolve_class_hierarchy_and_traits` 预处理：单继承 `extends` 字段与方法展开、父类被重写方法生成 `__super_xxx`、子类重写接收者权限一致性检查、`trait` 默认方法自动注入，以及 `implements` / `impl Trait for Type` 契约完整性与接收者模式匹配校验；
  - 实现成员函数内 `_` / `this` 到 Zig `self` 的映射、同类字段与兄弟成员方法省略 `_.` 前缀自动解析，以及 `super.method()` 调用生成；
  - 实现只读方法（缺省 `_`）内禁止修改字段或调用 `mut _` / `move _` 方法的编译期拦截；
  - 实现调用点 `obj.mutMethod()` 的 NLL 借用冲突检查与只读借用/只读参数越权调用拦截；
  - 实现调用点 `obj.moveMethod()` 与构造器 `move` 参数的所有权转移追踪（自动标记 `__obj_moved = true` 并在后续使用时报错 `Use of moved value`）；
  - 实现成员方法返回 `&T` 或 `ref struct` 时自动绑定接收者生命周期（Tier 1 零标注穿透）并在调用点接入 NLL 借用冲突检查。

### 新增文件 (Added)
- [`cap/test/test_trait_class_pass.cp`](file:///data/dev/cp/cap/test/test_trait_class_pass.cp): `trait`、单继承 `class extends`、`_` 接收者、省略 `_.` 互调、`*` 通配符、`mut _` / `move _` 接收者与方法返回借用 NLL 正向综合测试。
- [`cap/test/test_trait_class_fail_readonly_mutate.cp`](file:///data/dev/cp/cap/test/test_trait_class_fail_readonly_mutate.cp): 缺省只读方法内修改 `_.val` 编译期拦截反向测试。
- [`cap/test/test_trait_class_fail_nll_conflict.cp`](file:///data/dev/cp/cap/test/test_trait_class_fail_nll_conflict.cp): 方法返回 `&Tensor` 借用存活期间对宿主对象调用 `mut _` 方法的 NLL 借用冲突拦截反向测试。
- [`cap/test/test_trait_class_fail_use_after_move_self.cp`](file:///data/dev/cp/cap/test/test_trait_class_fail_use_after_move_self.cp): 调用 `move _` 方法吞噬实例后再次使用原实例的编译期拦截反向测试。
- [`cap/test/test_trait_class_fail_trait_mut.cp`](file:///data/dev/cp/cap/test/test_trait_class_fail_trait_mut.cp): 通过只读 `Trait` 参数越权调用 `mut _` 成员方法的编译期拦截反向测试。

---

## 2026-10-18 (Part 2: 函数三态返回类型 `?T ! E`、错误冒泡、`? default`、`try/catch`、可选链、`Box<dyn Trait>` 胖指针与 `switch` 模式匹配)

### 修改文件 (Modified)
- [`cp_ref.md`](file:///data/dev/cp/cp_ref.md):
  - 更新第 1 节模式匹配关键字为 `switch`，通配符为 `*`；
  - 新增第 11 节《函数返回类型（三态模型）、错误冒泡与捕获、可选链、`Box<dyn Trait>` 胖指针与 `switch` 模式匹配》，完整记录三态返回类型、`?` / `? default` / `? return default` / `??` 运算符分工、`try { ... } catch` 自动冒泡块、声明式 `from` 错误转换、内置 `Error` 结构体、`Box<dyn Trait>` 胖指针与 `downcast<T>()` 规范及安全边界。
- [`cpc/src/lexer.rs`](file:///data/dev/cp/cpc/src/lexer.rs):
  - 新增 `Token::Switch` (`switch`)、`Token::Try` (`try`)、`Token::Catch` (`catch`)、`Token::Null` (`null`)、`Token::Dyn` (`dyn`)、`Token::QuestionQuestion` (`??`)、`Token::QuestionDot` (`?.`)、`Token::DotDotEq` (`..=`)。
- [`cpc/src/ast.rs`](file:///data/dev/cp/cpc/src/ast.rs):
  - 新增 `ErrorType` (`Named`, `Union`, `AnonSet`, `BoxDyn`, `DefaultError`)；
  - `Type` 新增 `ResultType { ok: Box<Type>, err: ErrorType }`、`BoxDyn(Ident)`、`BareDyn(Ident)`；
  - `EnumVariant` 新增 `tuple_type: Option<Type>` 与 `from_err: Option<Ident>`（支持 `Io(FileErr) from FileErr`）；
  - `Stmt` 新增 `TryCatch { span, try_block, catch_param, catch_body }`；
  - `Pattern` 新增 `Null`, `Err(Box<Pattern>)`, `Range { start, end, inclusive }`, `Or(Vec<Pattern>)`；`EnumPattern` 新增 `tuple_pats`；`MatchArm` 新增 `guard: Option<Expr>`；
  - `Expr` 新增 `OptMember`, `OptCall`, `QuestionFallback { expr, fallback, is_return_fallback }`, `NullCoalesce`, `Catch`, `ErrWrap`, `BoxNew`, `Downcast`。
- [`cpc/src/ast_visitor.rs`](file:///data/dev/cp/cpc/src/ast_visitor.rs):
  - 适配所有新增 `Stmt` 与 `Expr` 变体的闭包变量引用与声明扫描。
- [`cpc/src/parser.rs`](file:///data/dev/cp/cpc/src/parser.rs):
  - 支持解析 `: ?T ! E`、`: !E`、`E1 | E2`、`!{ a, b }`、`Box<dyn Trait>`、`dyn Trait`、`?&T` / `?&mut T`；
  - 支持枚举变体 `Variant(Payload) from SourceErr` 声明解析；
  - 支持 `try { ... } catch err => { ... }` 语句块解析；
  - 支持 `switch` 表达式与模式解析（`Err(...)`, `null`, `*`, `..=`, `|`, `if` 守卫，并通过 `in_match_guard` 消解 `=>` 与箭头函数的词法歧义）；
  - 支持 `?.` 可选链、`? default` / `? return default`、`??`、`catch err => ...`、`Err(...)`、`Error { ... }`、`Box.new(...)`、`.downcast<T>()` / `.downcast_mut<T>()` 解析。
- [`cpc/src/sema.rs`](file:///data/dev/cp/cpc/src/sema.rs), [`cpc/src/dcp.rs`](file:///data/dev/cp/cpc/src/dcp.rs):
  - 注册内置 `Error` 与 `Box` 类型，适配所有新增 AST 节点的语义检查与 `.d.cp` 类型签名格式化。
- [`cpc/src/codegen.rs`](file:///data/dev/cp/cpc/src/codegen.rs):
  - 注入 `CpResult(T, E)`、内置 `Error`、`cp_convert_err`、`cp_has_ok_val`、`cp_unwrap_ok_val`、`cp_is_null_val`、`CpOptType` 等 Zig 0.17 运行时辅助原语；
  - 实现编译期裸 `dyn Trait` 栈分配拦截、匿名错误集 `__AnonErr_` 与合并错误集 `__UnionErr_` 自动合成、以及枚举 `from` 自动转换生成；
  - 实现 `Trait` 自动生成 `VTable`（含 `__type_id` 与 `_drop`）、`Box<dyn Trait>` 胖指针结构体、类编译期 64 位 FNV-1a `__TYPE_ID`、`downcast<T>()` 运行时校验与自动 Drop 释放；
  - 实现 `infer_cp_type` 完整类型推导、`check_switch_exhaustiveness` 编译期穷尽检查、`generate_switch` 模式匹配生成、`try { ... } catch` 区域自动冒泡包裹，以及 `?` / `? default` / `? return default` / `??` / `catch` / `?.` 代码生成。

### 新增文件 (Added)
- [`cap/test/test_error_box_switch_pass.cp`](file:///data/dev/cp/cap/test/test_error_box_switch_pass.cp): 三态返回类型、`?` 冒泡、`? default`、`? return default`、`??`、`catch`、`try/catch` 自动冒泡块、匿名错误集、声明式 `from`、统一 `Error`、`?.` 可选链、`Box<dyn Trait>` 胖指针与 `downcast<T>()`、`switch` 模式匹配全特性正向测试。
- [`cap/test/test_error_fail_null_coalesce_on_err.cp`](file:///data/dev/cp/cap/test/test_error_fail_null_coalesce_on_err.cp): 对未处理错误的 `T ! E` 直接使用 `??` 的编译期报错反向测试。
- [`cap/test/test_error_fail_non_exhaustive_switch.cp`](file:///data/dev/cp/cap/test/test_error_fail_non_exhaustive_switch.cp): `switch` 非穷尽匹配（缺失 `null` 与错误变体分支）的编译期报错反向测试。
- [`cap/test/test_box_fail_bare_dyn.cp`](file:///data/dev/cp/cap/test/test_box_fail_bare_dyn.cp): 裸 `dyn Trait` 置于栈上的编译期报错反向测试。
- [`cap/test/test_error_fail_incompatible_from.cp`](file:///data/dev/cp/cap/test/test_error_fail_incompatible_from.cp): 跨异构错误集冒泡未声明 `from` 转换时的编译期报错反向测试。
