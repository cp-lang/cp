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
- [`cap/test/example_showcase.cp`](file:///data/dev/cp/cap/test/example_showcase.cp): CP 语言全特性综合示例程序，覆盖三态返回 `?T ! E`、直接返回、`null`、`Err(.Variant)`、`? default`、`? return default`、`catch` + `??`、匿名错误集、`?.` 可选链、`Box<dyn Trait>` 胖指针虚表与自动 Drop、`downcast<T>()`、声明式 `from` 错误提升、`try/catch` 自动冒泡区与 `switch` 高级模式匹配（范围、守卫、或模式）。

## [2026-10-18] 错误兜底语法精炼 (Error Fallback Refinement)

### 修改文件 (Modified)
- [`cap/test/example_showcase.cp`](file:///data/dev/cp/cap/test/example_showcase.cp):
  - 简化 `safeGetConfig` 中的错误兜底为纯粹的 `let cfg = readConfigFile(filename) ? "DEFAULT_CONFIG";`，省略冗余的 `return` 关键字，确保变量初始化与表达式级降级语义保持严格一致。
## [2026-10-19] 静态匿名对象与动态字典 (Static Anonymous Objects & Dynamic Dict)

### 修改文件 (Modified)
- [`cpc/src/lexer.rs`](file:///data/dev/cp/cpc/src/lexer.rs):
  - 新增 `Token::DotDotDot` (`...`) 与 `Token::HashLBrace` (`#{`)。
- [`cpc/src/ast.rs`](file:///data/dev/cp/cpc/src/ast.rs):
  - `ObjectField` 结构体重构：增加 `is_spread: bool` 与 `key: Option<Ident>`，同时支持 `...spread` 与常规 `key: val` / 简写 `{ key }`；
  - `Expr` 枚举新增 `Dict(Vec<ObjectField>)` 动态字典表达式与 `Index { obj: Box<Expr>, index: Box<Expr> }` 索引访问表达式；
  - `Type` 枚举新增 `Dict(Box<Type>, Box<Type>)` 与 `Set(Box<Type>)`。
- [`cpc/src/ast_visitor.rs`](file:///data/dev/cp/cpc/src/ast_visitor.rs):
  - 增加对 `Expr::Dict` 与 `Expr::Index` 节点的作用域与变量引用扫描访问支持。
- [`cpc/src/dcp.rs`](file:///data/dev/cp/cpc/src/dcp.rs):
  - 增加 `Type::Dict` 与 `Type::Set` 的类型声明格式化输出支持。
- [`cpc/src/parser.rs`](file:///data/dev/cp/cpc/src/parser.rs):
  - 支持解析 `Dict<K, V>` 与 `Set<T>` 类型注解；
  - 重构 `parse_object_fields`，支持 `...spread` 展开、`key: val` 及 `{ key }` 简写；
  - 将 `{ ... }` 识别为静态匿名对象 `Expr::Object`，将 `#{ ... }` 识别为动态字典 `Expr::Dict`；
  - 将中括号下标访问 `obj[idx]` 统一解析为 `Expr::Index`。
- [`cpc/src/sema.rs`](file:///data/dev/cp/cpc/src/sema.rs):
  - `infer_type` 支持 `Expr::Object` 字段展开与覆盖合并、`Expr::Dict`（推导为动态类型 `Type::Dict(String, Any)`）以及 `Expr::Index`；
  - 完善 `Pattern::Object` 解构赋值类型推导。
- [`cpc/src/codegen.rs`](file:///data/dev/cp/cpc/src/codegen.rs):
  - 注册 `Dict` 全局标识符；重构 Zig 运行时 `Map` 与 `Set` 底层；
  - 废除未命中时的 undefined UB，`get()` 安全返回 `?Message`，并提供 `keys()`, `values()`, `entries()`, `put_all()` 完整 ES6 集合方法；
  - 定义 `const Dict = Map;`，为 `Map` / `Set` / `Dict` 注入 `pub fn release(self: *Self) void` 实现作用域退出自动析构；
  - 注入 `CpIndexReturnType`、`cp_index_get` 与 `cp_index_assign`，实现透明的容器索引读取与赋值；
  - 注入 `CpNullCoalesceReturn` 与 `cp_null_coalesce`，优雅处理 `dict["key"] ?? fallback` 的跨类型空值合并；
  - 升级 `cp_eq` 自动解包 `?Message` 并支持多基本类型比较；
  - 增强 `infer_cp_type` 与 `infer_zig_type` 支持布尔字面量标识符 `true` / `false`、静态匿名对象展开与属性覆盖，并在 Zig 端生成零开销原生匿名结构体 `.{ .a = ... }`；
  - 支持 CPS 状态机与常规函数中的对象解构绑定，自动从右侧表达式推导字段类型；
  - 优化 `@print` 单参数安全打印，防止格式化注入错误。

### 新增文件 (Added)
- [`cap/test/test_static_object.cp`](file:///data/dev/cp/cap/test/test_static_object.cp): 静态匿名对象字面量、`.` 属性访问、解构赋值与 `...` 展开覆盖测试。
- [`cap/test/test_map_set_dict.cp`](file:///data/dev/cp/cap/test/test_map_set_dict.cp): `Map` 与 `Set` 完整 ES6 接口、`#{ ... }` 动态字典、下标索引读写、字典展开及 `??` 空值合并测试。

---

## [2026-10-21] 阶段一：MetaAnnotation 极简元注解系统 (Phase 1: MetaAnnotation Core System)

### 修改文件 (Modified)
- [`cpc/src/lexer.rs`](file:///data/dev/cp/cpc/src/lexer.rs):
  - 新增 `Token::Meta` (`meta` 关键字)。
- [`cpc/src/ast.rs`](file:///data/dev/cp/cpc/src/ast.rs):
  - 新增 `MetaDef`（元注解定义节点，含无参及参数列表）、`MetaParam`、`MetaItem` 与 `MetaArg`（支持位置参数与具名参数）；
  - `Decl` 枚举新增 `Decl::Meta(MetaDef)` 变体；
  - 为 `Class`、`Function`、`Field`、`Param`、`TraitDecl`、`EnumDecl` 统一注入 `#[serde(default)] pub meta: Vec<MetaItem>` 字段。
- [`cpc/src/parser.rs`](file:///data/dev/cp/cpc/src/parser.rs):
  - 支持解析顶层 `meta Name;`（无参免写空括号）与 `meta Name(params);`（支持默认值）；
  - 实现 `parse_meta_list` 与 `parse_meta_item`，支持解析 `@Ident` 与 `@Ident(args)`，支持参数省略键名、位置传参和具名传参；
  - 实现 `clone_lookahead` 与 `is_decl_keyword` 预测分析，精准消除顶层 `@builtin(...)` 表达式语句与元注解 `@Meta` 的文法歧义；
  - 在类成员（方法、字段）和函数参数（`parse_params`）中支持前置 `@meta` 解析并挂载至 AST。
- [`cpc/src/sema.rs`](file:///data/dev/cp/cpc/src/sema.rs):
  - 新增 `SemaError::MetaInvalidTarget`、`SemaError::MetaDuplicateOnce`、`SemaError::MetaArgNotConstant` 语义诊断错误；
  - 新增 `MetaDefInfo` 结构并在 `Analyzer::new` 预置内置元注解 `@target`、`@once`、`@reflect`；
  - 在第一遍 AST 扫描中自动收集用户定义的 `meta` 并解析提取 `@target(class, method, field, param)` 允许目标与 `@once` 唯一性约束（未声明目标时默认全通）；
  - 在声明与成员检查阶段，严密校验：
    1. 元注解参数必须为编译期常量字面量（`Expr::Lit`、常量标识符、字面量数组、字面量匿名对象）；
    2. 校验元注解应用目标（如 `@target(method)` 挂在类上精准报错拦截）；
    3. 校验 `@once` 唯一性，同名多次书写直接报错拦截；
    4. 允许多次书写同名注解，编译器天生支持自动聚合为数组，废除繁琐的 `@Repeatable`；
    5. 支持脚本及快速原型免预定义直接打注解（双轨制）。
- [`cpc/src/dcp.rs`](file:///data/dev/cp/cpc/src/dcp.rs):
  - 增加对 `Decl::Meta` 的 `.d.cp` 类型定义输出支持。
- [`cpc/src/codegen.rs`](file:///data/dev/cp/cpc/src/codegen.rs):
  - 适配 `Decl::Meta`，阶段一保持 100% 零运行时代码输出、零开销、零体积膨胀。
- [`cp_ref.md`](file:///data/dev/cp/cp_ref.md):
  - 新增第十二章《MetaAnnotation 极简元注解系统规范》，完整记录 `meta` 关键字、`@target`、`@once`、`@reflect`、三免法则与多值聚合规范。

### 新增文件 (Added)
- [`cap/test/test_meta_pass.cp`](file:///data/dev/cp/cap/test/test_meta_pass.cp): 元注解 `meta` 声明、`@target` 目标约束、`@reflect`、`@once`、无参免 `()`、单参免键名、同名注解天然多重收集为数组及 Ad-hoc 免声明即用正向综合测试。
- [`cap/test/test_meta_fail_target.cp`](file:///data/dev/cp/cap/test/test_meta_fail_target.cp): 将 `@target(method)` 元注解非法附着在类上触发编译期语义拦截反向测试。
- [`cap/test/test_meta_fail_once.cp`](file:///data/dev/cp/cap/test/test_meta_fail_once.cp): 将 `@once` 元注解重复多次标注触发编译期语义拦截反向测试。

## [Unreleased] - 2026-10-22

### 阶段二：Dual-State Reflection 双态反射模型规范与实现

### 修改文件 (Modified)
- [`cpc/src/ast.rs`](file:///data/dev/cp/cpc/src/ast.rs):
  - 在 `Expr` 枚举中新增 `ReflectType { span: Span, target_ty: Type }` 与 `ReflectOf { span: Span, expr: Box<Expr> }` 表达式节点。
- [`cpc/src/parser.rs`](file:///data/dev/cp/cpc/src/parser.rs):
  - 在后缀成员解析中支持解析 `reflect.type<T>()` 与 `reflect.of(val)`；
  - 在 `parse_ident` 中加入 `Token::Meta => "meta".to_string()`，允许 `meta` 作为合法成员属性名（如 `info.meta`）。
- [`cpc/src/ast_visitor.rs`](file:///data/dev/cp/cpc/src/ast_visitor.rs):
  - 适配 `Expr::ReflectType` 与 `Expr::ReflectOf` 的变量引用收集。
- [`cpc/src/sema.rs`](file:///data/dev/cp/cpc/src/sema.rs):
  - 新增 `SemaError::ReflectNotEnabled` 错误变体；
  - 针对 `reflect.type<T>()` 与 `reflect.of(val)` 执行编译期反射开关硬检验：若目标类/结构体未标记 `@reflect`，直接报错拦截；
  - 预置 `TypeInfo`、`FieldInfo`、`MethodInfo` 类型识别并将 `reflect` 设为合法内置标识符。
- [`cpc/src/codegen.rs`](file:///data/dev/cp/cpc/src/codegen.rs):
  - 在 Zig 运行时前导声明中引入 `FieldInfo`、`MethodInfo`、`TypeInfo`（含 `getField` 与 `getMethod` 方法）；
  - 升级 `Message` 与 `MessageContext` 支持 `.array: []Message`，支持下标直接读取多值注解数组；
  - 实现 `generate_reflection_tables`：为每个 `@reflect` 标记的类生成惰性分配的单例 `_get_typeinfo_<ClassName>()` 与 `_typeinfo_<ClassName>`；
  - 彻底复用 CP 原生动态字典 `Dict<string, any>`（`*Map`），同名多次出现的注解自动聚合为 `Message.array`，单值注解自动解构为标量；
  - 在 `generate_expr`、`infer_zig_type`、`infer_cp_type` 中打通 `Expr::ReflectType`、`Expr::ReflectOf` 及相关字段/方法类型推导。
- [`cp_ref.md`](file:///data/dev/cp/cp_ref.md):
  - 新增第十三章《Dual-State Reflection 双态反射模型规范》，详细记录反射入口、扁平反射模型、原生字典复用与零成本安全屏障。
- [`cpc/src/dcp.rs`](file:///data/dev/cp/cpc/src/dcp.rs):
  - 代码格式化适配。

### 新增文件 (Added)
- [`lib/cap/reflect.cp`](file:///data/dev/cp/lib/cap/reflect.cp): CP 标准库反射声明模块（`cap:reflect`）。
- [`cap/test/test_reflect_pass.cp`](file:///data/dev/cp/cap/test/test_reflect_pass.cp): 双态反射正向综合测试，验证 `reflect.type<T>()`、`reflect.of(val)`、`info.name`、`info.isRef`、`info.meta` 字典读取与存在性判断、`info.fields`、`info.methods`、同名注解数组收集、`getField` 与 `getMethod` 以及 `ref class` 反射。
- [`cap/test/test_reflect_fail_unmarked.cp`](file:///data/dev/cp/cap/test/test_reflect_fail_unmarked.cp): 未标记 `@reflect` 的类调用 `reflect.type<T>()` 触发编译期精准拦截反向测试。
- [`cap/test/test_reflect_fail_of_unmarked.cp`](file:///data/dev/cp/cap/test/test_reflect_fail_of_unmarked.cp): 未标记 `@reflect` 的实例调用 `reflect.of(val)` 触发编译期精准拦截反向测试。

## [Unreleased] - 2026-10-22

### 阶段三：极简装饰器系统与 AOP 静态合成规范与实现

### 修改文件 (Modified)
- [`cpc/src/lexer.rs`](file:///data/dev/cp/cpc/src/lexer.rs):
  - 注册 `Token::Dec` (`dec` 3 字母关键字)。
- [`cpc/src/ast.rs`](file:///data/dev/cp/cpc/src/ast.rs):
  - 在 `Decl` 中新增 `Decl::Dec(DecDef)` 变体；
  - 为 `Param` 新增可选参数默认值字段 `pub default: Option<Expr>`。
- [`cpc/src/parser.rs`](file:///data/dev/cp/cpc/src/parser.rs):
  - `is_decl_keyword` 与顶层解析放行 `Token::Dec`；
  - 实现 `parse_dec_def`，支持 `dec name(params) { body }`；
  - `parse_params` 支持 untyped `next` 参数（免写 `: any`）与参数默认值 `= expr` 解析；
  - `parse_primary_expr` 支持 `Token::Fn` 解析匿名函数表达式 `fn(params) { body }` 为 `Expr::Arrow`。
- [`cpc/src/sema.rs`](file:///data/dev/cp/cpc/src/sema.rs):
  - 新增 `SemaError::DecInvalidTarget` 错误变体；
  - 在 `Analyzer` 中增加 `dec_defs` 注册表，第一遍扫描记录所有 `dec` 声明并分配函数符号；
  - 严格校验装饰器挂载目标合法性（当前阶段支持方法与函数，修饰字段等非法目标时精准报错）；
  - 独立作用域分析 `dec` 内部逻辑与参数定义。
- [`cpc/src/codegen.rs`](file:///data/dev/cp/cpc/src/codegen.rs):
  - 实现 `generate_function_decorated` 与 `prepare_decorator_stmts`；
  - 建立编译期静态合成 AOP 机制：原逻辑保留为 `_raw_<name>`，逐层展开装饰器切面代码合成为 `_wrap_k_<name>` 与最终公开入口 `<name>`；
  - 实现洋葱执行模型（Onion Execution Model）：外层装饰器先进入、最后退出；
  - `next()` 调用自动透传目标函数的参数与返回值；支持直接带参风格与高阶闭包工厂风格参数绑定；
  - 在 `infer_zig_type` 与 `infer_cp_type` 中打通 `next()` 返回类型推导。
- [`cpc/src/dcp.rs`](file:///data/dev/cp/cpc/src/dcp.rs):
  - 增加对 `Decl::Dec` 的 `.d.cp` 类型声明输出支持。
- [`cp_ref.md`](file:///data/dev/cp/cp_ref.md):
  - 新增第十四章《Decorators 极简装饰器与 AOP 静态合成规范》，详细记录 `dec` 关键字、三免法则、洋葱模型、双轨参数风格与静态合成机制。

### 新增文件 (Added)
- [`cap/test/test_dec_pass.cp`](file:///data/dev/cp/cap/test/test_dec_pass.cp): 极简装饰器正向综合测试，验证无参直接装饰器 `@doubleResult`、直接带参装饰器 `@tag("SERVICE")`、高阶工厂装饰器 `@prefixFactory("GLOBAL")`、洋葱模型多装饰器堆叠 `@outerWrap` + `@innerWrap`、类方法修饰、顶层独立函数修饰及与 `@reflect` / `@Route` 元注解共存。
- [`cap/test/test_dec_fail_target.cp`](file:///data/dev/cp/cap/test/test_dec_fail_target.cp): 将方法装饰器非法附着在类字段上触发编译期语义拦截反向测试。

## [Unreleased] - 2026-10-23

### 阶段四：类型感知宏与循环安全系统 (Type-Directed Macro & Anti-Cycle System)

### 修改文件 (Modified)
- [`cpc/src/lexer.rs`](file:///data/dev/cp/cpc/src/lexer.rs):
  - 注册 `Token::Macro` (`macro` 关键字)。
- [`cpc/src/ast.rs`](file:///data/dev/cp/cpc/src/ast.rs):
  - 在 `Decl` 中新增 `Decl::Macro(MacroDef)` 变体；
  - 新增 `MacroDef` 结构体，支持包含元注解、标识符、参数列表及展开函数体。
- [`cpc/src/parser.rs`](file:///data/dev/cp/cpc/src/parser.rs):
  - `is_decl_keyword` 与顶层声明解析放行 `Token::Macro`；
  - 实现 `parse_macro_def`，支持 `macro Name(params) { body }`；
  - `parse_params` 允许 untyped `target` 参数（无类型注解直通）。
- [`cpc/src/dcp.rs`](file:///data/dev/cp/cpc/src/dcp.rs):
  - 增加对 `Decl::Macro` 的 `.d.cp` 符号声明格式化支持。
- [`cpc/src/codegen.rs`](file:///data/dev/cp/cpc/src/codegen.rs):
  - 顶层跳过 `Decl::Macro` 代码生成（宏纯编译期展开，零运行时二进制开销）；
  - 新增 `escape_zig_string` 转义工具函数，精准处理字符串字面量中既有转义与宏合成双引号；
  - 修复方法内本地变量/参数遮蔽（Shadowing）类字段的符号优先级问题；
  - 升级 `cp_add` 格式化支持 `{s}{any}` / `{any}{s}`，无缝支持字符串与多类型数据高效拼接。
- [`cpc/src/sema.rs`](file:///data/dev/cp/cpc/src/sema.rs):
  - 新增 `MacroCycleDetected`、`MacroRecursionLimitExceeded`、`MacroInvalidTarget`、`MacroProhibitedTypeCreation`、`MacroHygieneViolation` 语义错误变体；
  - 构建三阶段编译管线：Pass 1 顶层符号扫描 -> Pass 2 Phase 2 宏代码派生展开 (`expand_macros`) -> Pass 3 完整语义检查；
  - 维护 `macro_call_stack`，提供 128 层递归熔断与递归展开环路探测；
  - 严格校验宏挂载目标（仅允许挂载于 `class` 或 `struct`）；
  - 实装 `@Json` 强类型序列化派生宏（合成 `toJson(): string` 方法）、`@Table` ORM 映射派生宏（合成 `getTableName(): string` 方法）、`@ToString` 结构打印派生宏以及用户自定义宏派生方法；
  - 严守 Macro Hygiene 铁律，强校验派生产物严禁携带任何宏注解。
- [`cpc/src/main.rs`](file:///data/dev/cp/cpc/src/main.rs):
  - `analyze_module` 适配 `&mut Module` 签名，确保宏展开修改 AST 直接回传 codegen。
- [`cpc/src/lsp.rs`](file:///data/dev/cp/cpc/src/lsp.rs):
  - `analyze_module` 适配 `&mut Module` 签名。
- [`cp_ref.md`](file:///data/dev/cp/cp_ref.md):
  - 新增第十五章《类型感知宏与循环安全系统》，详细规范宏语法、内建派生宏、四大防死锁循环安全铁律。

### 新增文件 (Added)
- [`cap/test/test_macro_pass.cp`](file:///data/dev/cp/cap/test/test_macro_pass.cp): 类型感知宏与循环安全正向综合测试，验证 `@Json` 序列化派生、`@Table("t_users")` 表名映射、无参 `@Table` 默认类名映射、`@ToString` 结构打印、用户自定义 `macro Tag` 与 `macro Environment` 派生方法、以及互引类型（Tier 1 TypeDeclInfo）安全内省实测。
- [`cap/test/test_macro_fail_target.cp`](file:///data/dev/cp/cap/test/test_macro_fail_target.cp): 将类宏 `@Json` 非法附着在类方法上触发编译期语义拦截反向测试。

## [Unreleased] - 2026-10-24

### 管道操作符与主题占位符系统 (Pipeline Operator `|>` & Topic Placeholder `.`)

### 修改文件 (Modified)
- [`cpc/src/lexer.rs`](file:///data/dev/cp/cpc/src/lexer.rs):
  - 注册 `Token::Pipeline` (`|>` 操作符)，排在 `Token::Pipe` (`|`) 之前以优先匹配长符号。
- [`cpc/src/ast.rs`](file:///data/dev/cp/cpc/src/ast.rs):
  - 在 `BinaryOp` 中新增 `BinaryOp::Pipe`；
  - 在 `Expr` 中新增 `Expr::PipePlaceholder(Span)` 变体；
  - 实现 `has_pipe_placeholder`、`substitute_pipe_placeholder` 与 `transform_pipeline_rhs` 工具函数，完备支持三级降糖策略（规则 1 裸标识符、规则 2 默认首参、规则 3 显式占位符 `.`、方法调用与多字段计算）。
- [`cpc/src/parser.rs`](file:///data/dev/cp/cpc/src/parser.rs):
  - 在 `Parser` 中增加 `in_pipeline_rhs` 状态栈追踪；
  - 设定 `|>` 优先级为 3（高于比较操作符 2，低于加减法 4）；
  - `parse_bin_expr` 在解析管道右侧表达式时维护 `in_pipeline_rhs` 计数；
  - `parse_primary_expr` 区分 `.` 的上下文语义：管道右侧识别为 `Expr::PipePlaceholder` 或前置成员访问，非管道右侧拦截非法使用并报错 `Pipeline placeholder '.' can only be used on the right-hand side of a pipeline ('|>') operator`；
  - `ParseError` 新增 `Generic` 变体以规范化清晰的错误诊断。
- [`cpc/src/ast_visitor.rs`](file:///data/dev/cp/cpc/src/ast_visitor.rs):
  - `ASTVisitor` 适配 `Expr::PipePlaceholder`，防止未绑定变量误报。
- [`cpc/src/sema.rs`](file:///data/dev/cp/cpc/src/sema.rs):
  - 新增 `PipePlaceholderOutsidePipeline` 错误；
  - 适配 `BinaryOp::Pipe` 与 `Expr::PipePlaceholder` 的类型检查与类型推断。
- [`cpc/src/codegen.rs`](file:///data/dev/cp/cpc/src/codegen.rs):
  - 升级 `Expr::Ident` 生成逻辑，识别 SSA 局部管道临时变量（`__pipe_` 前缀）直通输出；
  - 实现 `generate_pipeline`，将 `left |> right` 转换为 Zig 单次求值标签块 `(blk_p_N: { const __pipe_N = <left>; break :blk_p_N <right>; })`，彻底杜绝双重求值副作用；
  - 实现 `infer_cp_type_with_mock`，建立管道推断上下文环境，精准推导流水线各阶段以及对象方法/字段访问的返回类型，消除 Zig `*anyopaque` 类型失配。
- [`cp_ref.md`](file:///data/dev/cp/cp_ref.md):
  - 新增第十六章《管道操作符与主题占位符》，规范操作符优先级、3-Tier 降糖规则、单次求值保证与 AI Agent 数据流工程范式。

### 新增文件 (Added)
- [`cap/test/test_pipe_pass.cp`](file:///data/dev/cp/cap/test/test_pipe_pass.cp): 管道与占位符完整正向综合功能测试，覆盖裸函数、默认首参、显式占位符槽位、对象成员方法流、字段访问算术、多行链式流水线、副作用单次求值（`get_seed() |> combine(., .)` 执行次数严格为 1）以及 AI Agent RAG 认知流水线实测。
- [`cap/test/test_pipe_fail_placeholder.cp`](file:///data/dev/cp/cap/test/test_pipe_fail_placeholder.cp): 在管道操作符外部非法使用主题占位符 `.` 的编译期语义拦截反向测试。

## [Unreleased] - 2026-10-25

### Unsafe 与裸指针系统规范与实现 (Unsafe & Raw Pointer System: *T!, **T!, unsafe fn, unsafe {}, Deref & Cast)

### 修改文件 (Modified)
- [`cpc/src/lexer.rs`](file:///data/dev/cp/cpc/src/lexer.rs):
  - 注册 `Token::Unsafe` (`unsafe` 关键字)。
- [`cpc/src/ast.rs`](file:///data/dev/cp/cpc/src/ast.rs):
  - 在 `Type` 中新增 `Type::RawPtr { inner: Box<Type>, depth: usize }` 变体，支持多级非托管裸指针；
  - 在 `Expr` 中新增 `Expr::Cast { span, expr, target_ty }`、`Expr::Deref(Box<Expr>)` 与 `Expr::Unsafe(BlockStmt)` 变体；
  - 在 `Stmt` 中新增 `Stmt::UnsafeBlock(BlockStmt)` 变体；
  - 在 `Function` 与 `MethodSig` 中增加 `is_unsafe: bool` 标识。
- [`cpc/src/ast_visitor.rs`](file:///data/dev/cp/cpc/src/ast_visitor.rs):
  - 适配 `Expr::Cast`、`Expr::Deref`、`Expr::Unsafe` 与 `Stmt::UnsafeBlock` 的遍历与变量引用收集。
- [`cpc/src/parser.rs`](file:///data/dev/cp/cpc/src/parser.rs):
  - 新增双 Token 前瞻缓冲区 `peeked2` 与 `peek_next`；
  - 实现裸指针类型解析（`*T!`、`**T!`、`***T!`），利用双向 lookahead 彻底消除与返回类型错误联合分隔符 `!` 的语法混淆；
  - 提取独立前缀一元/后缀解析器 `parse_postfix_expr` 与 `parse_postfix_from_left`，确保 `&user as *User!` 严谨按照 `(&user) as *User!` 进行优先级绑定，避免一元借用贪婪吞入类型转换；
  - 实现 `unsafe fn` 函数声明与 `unsafe { ... }` 表达式/语句块解析；
  - 实现显式类型强转语法 `expr as TargetType`；
  - 实现标量与多级解引用语法 `p.*` 与 `pp.*.field`；
  - 支持大写值类型结构体字面量实例化 `User{age: 20}` 并将 `struct` 正确声明为 `is_ref = true`。
- [`cpc/src/sema.rs`](file:///data/dev/cp/cpc/src/sema.rs):
  - 新增 `SemaError::UnsafeRequired(String, Span)` 编译期安全错误；
  - 在 `SymbolKind::Func` 中记录 `is_unsafe: bool`；
  - 引入 `unsafe_depth` 与 `current_fn_is_unsafe` 上下文追踪；
  - 编译期严格检查并硬性拦截在 unsafe 外部调用 unsafe 函数、强转裸指针或解引用裸指针的操作；
  - 增强 `infer_type` 对 `Expr::Cast`、`Expr::Deref`、`Expr::Unsafe` 的类型推导。
- [`cpc/src/codegen.rs`](file:///data/dev/cp/cpc/src/codegen.rs):
  - 在 `map_type` 中正确将 `self.ref_classes`（包括 `struct`）映射为值类型，多级裸指针 `Type::RawPtr` 映射为 Zig 裸指针（`*T`、`**T`）；
  - 实现 `Expr::Cast` 映射为 Zig `@as(Target, @ptrCast(@alignCast(&(...))))`；
  - 实现 `Expr::Deref` 映射为 Zig 原生解引用 `p.*`；
  - 修复状态机局部变量收集（`extract_locals_simple` 与 `extract_local_vars_to_str`），支持深入 `Stmt::UnsafeBlock` 发现变量并正确注入 `cp_main_Context`；
  - 修复函数参数字段赋值检查，允许对裸指针和可变借用参数的目标字段（pointee field）进行赋值修改。
- [`cpc/src/dcp.rs`](file:///data/dev/cp/cpc/src/dcp.rs) 与 [`cpc/src/lsp.rs`](file:///data/dev/cp/cpc/src/lsp.rs):
  - 适配 `Type::RawPtr` 穷尽模式匹配与补全。
- [`cp_ref.md`](file:///data/dev/cp/cp_ref.md):
  - 新增第十八章《Unsafe 与裸指针系统规范》，完整记录指针语法、解引用规则、与错误联合解耦、显式强转、安全拦截与生产级代码示例。

### 新增文件 (Added)
- [`cap/test/test_unsafe_pass.cp`](file:///data/dev/cp/cap/test/test_unsafe_pass.cp): 涵盖一级裸指针参数修改、二级裸指针解引用修改、标量裸指针修改、三态错误联合共存以及 switch 模式匹配的全量正向功能测试。
- [`cap/test/test_unsafe_fail_call.cp`](file:///data/dev/cp/cap/test/test_unsafe_fail_call.cp): 验证在 unsafe 块外部调用 `unsafe fn` 被编译期硬性拦截并报错的反向测试。
- [`cap/test/test_unsafe_fail_cast.cp`](file:///data/dev/cp/cap/test/test_unsafe_fail_cast.cp): 验证在 unsafe 块外部执行 `&x as *T!` 强转被编译期硬性拦截并报错的反向测试。
- [`cap/test/test_unsafe_fail_deref.cp`](file:///data/dev/cp/cap/test/test_unsafe_fail_deref.cp): 验证在 unsafe 块外部执行裸指针解引用 `p.*` 被编译期硬性拦截并报错的反向测试。


### 修改文件 (Modified)
- [`cpc/src/lexer.rs`](file:///data/dev/cp/cpc/src/lexer.rs):
  - 注册 `Token::Of` (`of`) 与 `Token::In` (`in`) 关键字。
- [`cpc/src/ast.rs`](file:///data/dev/cp/cpc/src/ast.rs):
  - 新增 `IterMode` 枚举 (`Borrow`, `MutBorrow`, `Move`)；
  - 新增 `ForStmt` 抽象语法树节点 (`span`, `mode`, `pat`, `expr`, `body`) 与 `Stmt::For(ForStmt)`；
  - 新增 `Expr::Range { span, start, end, inclusive }` 表达式节点；
  - 适配管道占位符检测与替换逻辑。
- [`cpc/src/parser.rs`](file:///data/dev/cp/cpc/src/parser.rs):
  - 实现 `parse_for_stmt`，支持三维借用修饰符 (`for x`, `for &x`, `for move x`)、可选声明关键字 (`let`/`const`)、可选括号风格 (`for (...)`) 以及区间范围语法 (`0..10` / `0..=10`)；
  - 在 `parse_ident` 中加入 `Token::Of` 与 `Token::In` 上下文关键字支持，保证 `reflect.of` 等已有代码不发生词法冲突。
- [`cpc/src/ast_visitor.rs`](file:///data/dev/cp/cpc/src/ast_visitor.rs):
  - 适配 `Expr::Range` 与 `Stmt::For` 的变量引用收集与局部变量声明提取。
- [`cpc/src/sema.rs`](file:///data/dev/cp/cpc/src/sema.rs):
  - 新增 `SemaError::MutateWhileBorrowedInFor` 错误变体；
  - 引入 `loop_borrowed_vars` 集合并在循环体内对被迭代集合施加只读借用锁，拦截任何 `push`、`clear`、`delete`、重赋值等结构性改变；
  - 针对 `IterMode::Move` 将集合登记至 `moved_vars`，禁止循环后再次使用（拦截 Use-After-Move）；
  - 适配模式解构作用域绑定与 `Expr::Range` 语义检查。
- [`cpc/src/codegen.rs`](file:///data/dev/cp/cpc/src/codegen.rs):
  - 在 Zig 运行时前导声明中注入 `cp_iter_slice` 与 `cp_mut_slice`（带 `@constCast` 支持），无缝支持切片、静态数组及不可变数组的原地安全可变切片迭代；
  - 统一区间循环为独立局部块 `{ var i: i32 = start; while (i < end) : (i += 1) { ... } }`，彻底杜绝多次区间循环的局部变量命名重定义；
  - 优化循环变量为块级原生捕获 `for (target) |item|` 与 `for (target) |*item|`，避免状态机 `ctx` 上下文全局命名冲突；
  - 引入 `mut_borrow_loop_vars` 作用域集合，在表达式和赋值中自动生成解引用操作（`x.* = ...` 与 `(x.*)`），支持标量及结构体的就地原地修改；
  - 在 `Expr::Array` 中统一输出为 `&.{ ... }`，完美兼顾静态元组解构访问与切片类型自动转换。
- [`cp_ref.md`](file:///data/dev/cp/cp_ref.md):
  - 新增第十七章《迭代系统》，详尽记录三维借用语义表、语法包容性、区间范围、模式解构与编译期安全铁律。

### 新增文件 (Added)
- [`cap/test/test_for_pass.cp`](file:///data/dev/cp/cap/test/test_for_pass.cp): 涵盖只读借用 `for x of list`、原地可变借用 `for &x of list`、所有权转移 `for move x of list`、半开区间 `0..5`、闭区间 `1..=5`、动态变量区间、对象模式解构以及可选小括号与 `let` 语法糖的正向综合功能测试。
- [`cap/test/test_for_fail_mutate.cp`](file:///data/dev/cp/cap/test/test_for_fail_mutate.cp): 在 `for x of list` 只读借用循环体内试图调用 `list.push(...)` 触发 `MutateWhileBorrowedInFor` 语义拦截反向测试。
- [`cap/test/test_for_fail_use_after_move.cp`](file:///data/dev/cp/cap/test/test_for_fail_use_after_move.cp): 在 `for move x of list` 所有权转移后试图再次读取 `list` 触发 `UseOfMovedVar` 语义拦截反向测试。

---

## 2026-10-25 (语言设计规范 `cp_ref.md` 全量同步与补全)

### 修改文件 (Modified)
- [`cp_ref.md`](file:///data/dev/cp/cp_ref.md):
  - **修正第 3 节对象模型描述**：消除旧文档“不支持动态增删属性”的表述矛盾，重构为显式的“静态匿名对象 `{ ... }`（编译期零成本）与动态字典 `#{ ... }`（运行时动态哈希）”双轨分立机制。
  - **新增第十九章《静态匿名对象与动态字典系统》**：完整定义静态匿名对象 `{ ... }` 的编译期强类型推导、模式解构、展开与覆盖语法；完整定义动态字典 `#{ ... }` / `Dict`、运行时下标索引读写 `d[k] = v`、字典展开与空值合并运算符 `??`。
  - **新增第二十章《高级集合数据结构规范 (Map & Set)》**：详尽收录 ES6 标准 `Map`（`set`, `get`, `has`, `delete`, `clear`, `size`, `keys`, `values`, `entries`、`map[k]` 下标语法糖）与 `Set`（`add`, `has`, `delete`, `clear`, `size`, `values`），以及底层的零 GC Arena 内存管理与生命周期。
  - **新增第二十一章《文本与数学核心内建对象》**：全面记录字符串方法套件（`indexOf`, `slice`, `replace`, `split`, `trim` 等 14+ API）、反引号模板字符串 `` `Hello ${name}` ``、POSIX `RegExp` 正则表达式引擎（`test`, `match`）、以及全局 `Math` 高性能数学运算对象（常量、浮点极值、三角函数、随机数）。
  - **新增第二十二章《高并发服务、多线程与网络系统》**：收录对标 Bun.js 的原生 `serve({ port, fetch })` 高性能 HTTP 反应堆、`Request`、`Response`、`json(...)`；收录轻量 Actor 协程（`@spawn`, `@receive`, `@reply`, `@self`）；收录 OS 多线程 Worker（`@worker_spawn`）与无锁跨线程环形信箱；收录 AI 大张量跨进程零拷贝共享内存 `SharedTensor`。
  - **新增第二十三章《系统标准库生态索引》**：提供系统级 `cap:*` 模块索引总表（涵盖 `cap:io`, `cap:http`, `cap:actor`, `cap:worker`, `cap:fs`, `cap:net`, `cap:crypto`, `cap:sqlite`, `cap:json`, `cap:mem`, `cap:test`, `cap:beam`）。

---

## 2026-10-25 (模板字符串系统端到端打通、转义加固与编译期常量折叠)

### 修改文件 (Modified)
- [`cpc/src/parser.rs`](file:///data/dev/cp/cpc/src/parser.rs):
  - 增强 `Token::TemplateStringRaw` 解析逻辑，支持 `\$`（字面量美元符号）、`` \` ``（字面量反引号）与 `\\`（字面量反斜杠）的转义，杜绝价格符号与格式化冲突；
  - 引入引号状态机（`in_single_quote`, `in_double_quote`, `escape`），在 `${...}` 提取内部表达式时，智能忽略字符串字面量内部的花括号，彻底修复包含 `{` / `}` 的嵌套字符串提前闭合 Bug。
- [`cpc/src/sema.rs`](file:///data/dev/cp/cpc/src/sema.rs):
  - 在 `analyze_expr` 中深度递归分析 `Expr::Template` 内部的所有插值表达式；
  - 在 `infer_type` 中将 `Expr::Template` 正确推导为 `Type::String`；
  - 在 `inner_span` 中关联 `Expr::Template` 的源码位置。
- [`cpc/src/codegen.rs`](file:///data/dev/cp/cpc/src/codegen.rs):
  - 注入 Zig 运行时格式化万能适配器 `cp_fmt(v)`，无缝支持字符串切片、布尔值、整数、浮点数、动态 `Message`、可选类型及任意结构体；
  - 实现 `Expr::Template` 编译期常量折叠（Constant Folding）：若所有插值均为常量字面量，编译期直接折叠为纯 Zig 静态切片，零运行时开销、零堆分配；
  - 实现动态模板字符串的高速格式化代码生成，利用 `escape_zig_fmt_string` 对非动态文本槽位执行 Zig 级别 `{` -> `{{`、`}` -> `}}` 转义，配合 `std.fmt.allocPrint` 达成极致原生的字符串合成；
  - 在 `infer_cp_type_with_mock` 与 `infer_zig_type` 中注册 `Expr::Template => Type::String` / `"[]const u8"`。
- [`cap/test/test_template.cp`](file:///data/dev/cp/cap/test/test_template.cp):
  - 扩充综合测试用例，覆盖基础插值、跨类型多插值、表达式计算、纯文本常量折叠、`\$` 与 `` \` `` 转义、原生 JSON 结构化负载、内嵌大括号表达式、链式方法调用（`.toUpperCase()`）及同步函数返回模板字符串全套场景。
- [`cp_ref.md`](file:///data/dev/cp/cp_ref.md):
  - 扩充第二十一章第 2 节，新增模板字符串转义规则、零转义 JSON 友好设计、常量折叠与链式调用规范说明。

---

## 2026-10-25 (`_` / `this` 首参三态借用与所有权设计全景验证与 `&_` 增强)

### 修改文件 (Modified)
- [`cpc/src/parser.rs`](file:///data/dev/cp/cpc/src/parser.rs):
  - 扩展 `parse_params` 参数解析，新增对 `Token::Ampersand`（`&_`、`&this`、`&mut _`、`&mut this` 及 `&param`）的首参修饰支持，使成员方法可变借用语法与 `for &x of list` 原地借用和 `&param` 借用参数保持绝对正交统一。
- [`cpc/src/codegen.rs`](file:///data/dev/cp/cpc/src/codegen.rs):
  - 增强 `Expr::Borrow` 中的 `is_class_ref` 识别，修复访问类字段借用（`&_.field`）产生多余二级指针的隐患。
- [`cp_ref.md`](file:///data/dev/cp/cp_ref.md):
  - 更新第 2 节与第 10.5 节规范，完整收录首参三态体系：缺省省略只读借用、`&_`（及兼容 `mut _`）可变借用、`move _`（及兼容 `move this`）所有权消费，详尽定义其语义规则、生命周期联动与编译期安全保障。

### 新增文件 (Added)
- [`cap/test/test_this_underscore_interchange.cp`](file:///data/dev/cp/cap/test/test_this_underscore_interchange.cp):
  - 涵盖缺省只读借用省略、`_` 与 `this` 属性互访、`&_` / `mut _` / `&this` / `mut this` 可变借用变异、`move _` / `move this` 所有权消费、单继承派生调用及异步状态机 `async fn` 上下文穿透综合测试用例，100% 编译运行通过。

---

## 2026-10-28 (异步任务编排 cap:parallel、事件驱动 cap:events 与跨任务借用安全检查)

### 修改文件 (Modified)
- [`cpc/src/ast.rs`](file:///data/dev/cp/cpc/src/ast.rs):
  - 为 `ArrowExpr` 新增 `is_async: bool` 字段，精确区分普通同步闭包与 `async () => { ... }` 异步任务闭包。
- [`cpc/src/parser.rs`](file:///data/dev/cp/cpc/src/parser.rs):
  - 在 `parse_primary` 中支持 `async () => { ... }`、`async fn(...) { ... }` 与 `async ident => ...` 箭头表达式语法；
  - 修复 `parse_bin_expr` 与 `parse_postfix_expr` 中将常规对象属性访问误判为 `EnumInit` 的歧义（限定仅当对象标识符与属性名均为全大写枚举命名规范时方进入枚举初始化解析）。
- [`cpc/src/sema.rs`](file:///data/dev/cp/cpc/src/sema.rs):
  - 注册 `Task`、`Event`、`EventEmitter` 内建类型与 `spawn`、`join`、`@task_spawn`、`@task_join`、`@create_pending_task`、`@complete_task` 内建函数；
  - 优化 `Pattern::Tuple` 解构语义，允许从 `Type::Any` 解构出动态元组；
  - 为 `join` / `@task_join` 与 `spawn` / `@task_spawn` 注入精准的语义返回类型推导。
- [`cpc/src/codegen.rs`](file:///data/dev/cp/cpc/src/codegen.rs):
  - 注入原生并发运行时前置基座：`Task`、`JoinBarrier`、`__cp_task_bridge`、`cp_msg_to_val`、`cp_unpack_elem` 以及泛型 `print(v: anytype)`；
  - 实现 `spawn` / `@task_spawn` 代码生成：分配独立 Arena，实例化 CPS 状态机上下文，通过 `__cp_task_bridge` 接入协程调度器；
  - 实现严密的跨任务内存借用检查网：严格禁止临时借用（`&`、`&mut`）与栈绑定 `ref struct` 跨越 `spawn` 边界，强制使用 `move` 或 `Shared<T>` 消除数据竞争；
  - 实现 `await` 多任务等待与单任务解析，支持 `let [msg1, msg2] = await join(task1, task2)` 元组智能解包与字符串自动推导；
  - 支持闭包/Lambda 对 `this` / `_` / `self` 的环境捕获与上下文自动传递；
  - 适配同步函数返回 Lambda 函数指针时的 `@ptrCast(@constCast(&lambda_X))` 自动转换；
  - 过滤标准库 `lib/cap/parallel.cp` 中 `class Task` 与 `print`、`spawn`、`join` 的重复顶层符号声明，避免 Zig 命名碰撞。
- [`lib/cap/io.cp`](file:///data/dev/cp/lib/cap/io.cp):
  - 导出 `export fn print(msg: any): void`，供非 `@print` 宏规范调用。
- [`lib/cap/parallel.cp`](file:///data/dev/cp/lib/cap/parallel.cp):
  - 实现并导出 `Task`、`spawn`、`join` 并发核心任务管理组件。
- [`lib/cap/events.cp`](file:///data/dev/cp/lib/cap/events.cp):
  - 实现并导出 `EventEmitter`（`on`, `off`, `emit`, `once`, `listenerCount`, `removeAllListeners`）与 `Event`（`isSet`, `set`, `reset`, `wait`），支持 `defer unsub()` 自动退订。
- [`cp_ref.md`](file:///data/dev/cp/cp_ref.md):
  - 扩充第二十二章，新增第 5 节《异步任务编排与并行任务池 (`cap:parallel`)》与第 6 节《事件驱动与协程同步子系统 (`cap:events`)》；
  - 更新第二十三章系统标准库总表，收录 `cap:parallel` 与 `cap:events`。

### 新增文件 (Added)
- [`cap/test/test_parallel_join.cp`](file:///data/dev/cp/cap/test/test_parallel_join.cp): 跨任务并发 `spawn`、`join` 屏障汇聚与 `let [msg1, msg2] = await join(...)` 元组解包测试，原生全绿通过。
- [`cap/test/test_parallel_fail_borrow.cp`](file:///data/dev/cp/cap/test/test_parallel_fail_borrow.cp): 验证跨 `spawn` 边界非法捕获局部临时借用触发编译期严密拦截。
- [`cap/test/test_events_pass.cp`](file:///data/dev/cp/cap/test/test_events_pass.cp): `EventEmitter` 事件监听分发与 `Event` 协程同步信号标志测试，原生全绿通过。

---

## 2026-10-28 (`@parallel` 多核 Worker 线程池、`spawn.cpu` 专用计算线程与跨任务只读/可变借用及所有权检查)

### 修改文件 (Modified)
- [`cpc/src/sema.rs`](file:///data/dev/cp/cpc/src/sema.rs):
  - 注册 `@task_spawn_cpu` 内建函数并返回 `Type::Ref("Task")`；
  - 在 `infer_type` 的 `Expr::Call` 分支识别 `spawn.cpu(...)` 属性调用语法，推导返回 `Task` 类型。
- [`cpc/src/codegen.rs`](file:///data/dev/cp/cpc/src/codegen.rs):
  - 在 `generate_module` 中解析 `main` 函数上的 `@parallel` / `@parallel(N)` / `@parallel(threads: N)` 与 `@single_thread` 元注解，并支持读取 `cap.json` 中的 `"mode": "parallel"` 配置；
  - 注入线程安全的 `GlobalTaskQueue`（带 `Spinlock` 保护的 2048 槽位环形队列）、`_WorkerPoolRunner` 工作线程池循环、`global_active_tasks` 与 `global_running_workers` 原子计数器；
  - 升级 `Task` 与 `JoinBarrier` 为自旋锁（`Spinlock`）并发安全结构，引入 `remaining = total + 1` 与 `finish_setup_and_should_suspend()` 机制消除主线程注册屏障与 Worker 线程并发完成任务之间的竞态条件；
  - 实现 `spawn.cpu(...)`（及 `@task_spawn_cpu`）代码生成，自动拉起独立 OS 计算线程执行重度 CPU 密集型闭包并通过 `__cp_task_bridge` 唤醒 `await` / `join`；
  - 强化跨任务闭包捕获的内存安全检查：只读捕获外部变量时调用 `check_borrow_conflict(r, false)` 并禁止在闭包内直接修改外部变量（报错 `Cannot mutate read-only variable or captured outer variable`）；支持在闭包内使用 `move` 转移外部变量所有权并在外层实施 Use-After-Move 拦截。
- [`lib/cap/parallel.cp`](file:///data/dev/cp/lib/cap/parallel.cp):
  - 更新 `spawn` 与 `@parallel`、`spawn.cpu` 注释说明。
- [`cap/test/test_parallel_join.cp`](file:///data/dev/cp/cap/test/test_parallel_join.cp):
  - 升级为 `@parallel(4)` 多核并行测试，覆盖 `spawn` 只读捕获、`spawn` + `move` 所有权转移、`spawn.cpu` 重度计算线程派发及 `await join(task1, task2, cpu_task)` 并发汇聚。
- [`cp_ref.md`](file:///data/dev/cp/cp_ref.md):
  - 更新第 22.5 节，完整收录 `@parallel` / `@single_thread` / `cap.json` 多核运行时配置、`spawn.cpu` 专用计算线程原语及跨任务只读借用、禁止可变修改与 `move` 所有权检查规范。

### 新增文件 (Added)
- [`cap/test/test_parallel_fail_mutate_capture.cp`](file:///data/dev/cp/cap/test/test_parallel_fail_mutate_capture.cp): 验证在 `spawn` 并发任务中直接修改捕获的外部变量时触发编译期拦截。
- [`cap/test/test_parallel_fail_use_after_move.cp`](file:///data/dev/cp/cap/test/test_parallel_fail_use_after_move.cp): 验证在 `spawn` 并发任务中 `move` 转移外部变量所有权后，外层再次访问触发 `Use of moved value` 编译期拦截。

---

## 2026-10-28 (全局统一只读借用 `&`、可变借用 `*`、Unsafe 裸指针 `*... !` 与所有权转移 `move` 符号标准)

### 修改文件 (Modified)
- [`cpc/src/parser.rs`](file:///data/dev/cp/cpc/src/parser.rs):
  - `parse_params`：将前置 `*`（如 `*x: T`、`*_`、`*this`）及 `: *T` 类型参数统一解析为可变借用（`is_mut = true`）；将前置 `&`（如 `&x: T`、`&_`、`&this`）统一解析为显式只读借用（`is_mut = false`）；
  - `parse_for_stmt`：将 `for *x of list` 统一解析为原地可变借用（`IterMode::MutBorrow`），将 `for x of list` 与 `for &x of list` 统一解析为只读借用（`IterMode::Borrow`）；
  - `parse_base_type`：为可变借用类型 `*T` 增加第 3 级参数名生命周期绑定支持（如 `: *x Tensor`、`: *(x | y) Tensor`、`: *_ Tensor`）；
  - `parse_primary_expr`：支持 `*b!` 与 `**b!` 直接裸指针获取表达式（前置 `*` 层级 + 尾置 `!`），自动映射为对应层级的 `Type::RawPtr` 转换。
- [`cpc/src/sema.rs`](file:///data/dev/cp/cpc/src/sema.rs):
  - 在 `infer_type` 的 `Expr::Cast` 分支中支持对 `*b!` / `**b!` 表达式进行自动目标裸指针类型推导并强制校验 `unsafe` 上下文。
- [`cpc/src/codegen.rs`](file:///data/dev/cp/cpc/src/codegen.rs):
  - 在 `infer_cp_type_with_mock` 与 `Expr::Cast` 代码生成中支持 `*b!` / `**b!` 的内部类型自动解析与 Zig 指针强转生成。
- [`cap/test/test_this_underscore_interchange.cp`](file:///data/dev/cp/cap/test/test_this_underscore_interchange.cp):
  - 更新并验证 `&_` 显式只读借用接收者与 `*_` / `*this` 可变借用接收者。
- [`cap/test/test_for_pass.cp`](file:///data/dev/cp/cap/test/test_for_pass.cp):
  - 更新并验证 `for &x of nums` 显式只读迭代与 `for *x of mut_list` 原地可变借用迭代。
- [`cap/test/test_unsafe_pass.cp`](file:///data/dev/cp/cap/test/test_unsafe_pass.cp):
  - 验证 `*user!`、`**p_copy!`、`*num!` 裸指针获取语法。
- [`cap/test/test_lifetime_pass.cp`](file:///data/dev/cp/cap/test/test_lifetime_pass.cp):
  - 验证 `ref struct` 中 `: *Tensor` 字段与 `*t1` 可变借用传参。
- [`cp_ref.md`](file:///data/dev/cp/cp_ref.md):
  - 在第十章新增《0. 全局借用、所有权与指针符号统一标准 (Global Symbol Matrix)》，并全面同步更新第 2.5 节、第十章、第十七章与第十八章示例与规范表。











## [Unreleased] - 2026-10-28 (Part 3)

### 可变借用变量操作 (`*v`, `p.* = 20`, `p.a = 10`, `p.push(30)`) 与容器改变操作全面纳入可变借用检查 (`Array`/`Vec`, `Map`, `Set`, `Dict`)

### 修改文件 (Modified)
- [`cpc/src/parser.rs`](file:///data/dev/cp/cpc/src/parser.rs):
  - `parse_params` 将非接收者参数的前置 `*x: T` 与 `&x: T` 统一包装为 `Type::BorrowType { inner, is_mut, lifetimes }`，与 `x: *T` / `x: &T` 保持完全等价的 AST 表示；
  - `parse_base_type` 支持 `Array<T>` 与 `Vec<T>` 泛型容器类型语法并映射为 `Type::Array(Box::new(elem))`；
  - `Token::New` 支持 `new Map<K, V>()` / `new Set<T>()` / `new Dict<K, V>()` 可选泛型实参解析。
- [`cpc/src/sema.rs`](file:///data/dev/cp/cpc/src/sema.rs):
  - `analyze_module` 注册内置 `Dict` 类符号；
  - `analyze_expr` (`Expr::Call`) 将 `"set"`、`"put_all"`、`"add"` 与 `"push"`、`"pop"`、`"delete"`、`"clear"` 共同纳入 `loop_borrowed_vars` 循环借用期间的可变拦截；
  - `types_eq` 允许 `Type::Array` 与同构 `Type::Tuple` 兼容；
  - `infer_type` (`Expr::Index`) 支持自动解包 `Type::BorrowType` 后进行下标类型推导。
- [`cpc/src/codegen.rs`](file:///data/dev/cp/cpc/src/codegen.rs):
  - 新增 `loop_borrowed_containers` 与 `anon_obj_structs` 状态跟踪；
  - 在 Zig 运行时前导中新增 `cp_array_lit`、`CpArrayElemType`、`cp_array_push`、`cp_array_pop`、`cp_container_clear`、`cp_index_of`、`cp_last_index_of`、`cp_includes`，并升级 `CpIndexReturnType`、`cp_index_get`、`cp_index_assign`、`cp_iter_slice`、`cp_mut_slice` 支持借用数组指针 `*[]const T` 与原地自动解引用修改；
  - 实现函数内同名重声明变量 Alpha-Renaming（`rename_shadowed_vars_in_block`），完美支持同一作用域连续多次声明 `let v = ...; let p = *v;` 且具有不同底层类型；
  - 将匿名对象字面量类型 `Type::Object` 规范化映射为顶层具名结构体 `const __CpObj_... = struct { ... };`，解决跨状态机/函数的 Zig 匿名结构体名义类型兼容问题；
  - 统一实现 `check_mutation_allowed` 与 `check_read_allowed`：全面覆盖 `p.* = val`、`p.a = val`、`p[i] = val`、`for *x of v` 以及容器方法 `push`、`pop`、`set`、`put_all`、`add`、`delete`、`clear` 的可变借用权限、NLL 活跃借用读写互斥与 `for` 循环借用锁定检查；
  - 修复 `compute_last_uses_stmt` 对控制流块（`If`/`While`/`For`/`Match`/`TryCatch`）的语句计数对齐，确保 NLL 最后一次使用（Last Use）语句序号与 `generate_stmt` 100% 精确同步。
- [`cp_ref.md`](file:///data/dev/cp/cp_ref.md):
  - 在第十章第 3 节补充可变借用变量的三类核心操作（`p.* = 20`、`p.a = 10`、`p.push(30)`）及 `Array`/`Vec`、`Map`、`Set`、`Dict` 容器改变操作的四重编译期可变借用检查规范与代码示例。

### 新增文件 (Added)
- [`cap/test/test_mut_borrow_ops.cp`](file:///data/dev/cp/cap/test/test_mut_borrow_ops.cp): 覆盖基础类型 `*v` (`p.* = 20`)、对象字面量 `*v` (`p.a = 100`)、数组 `*v` (`p.push(30)`、`p.pop()`、`p[1] = 25`、`for *item of v`) 以及 `Map`/`Set`/`Dict` 全套可变借用操作的正向综合测试。
- [`cap/test/test_mut_borrow_fail_readonly.cp`](file:///data/dev/cp/cap/test/test_mut_borrow_fail_readonly.cp): 验证通过只读借用 `let r = &v` 调用 `r.push(30)` 被编译器静态拦截的反向测试。
- [`cap/test/test_mut_borrow_fail_conflict.cp`](file:///data/dev/cp/cap/test/test_mut_borrow_fail_conflict.cp): 验证当 `Map` 存在活跃只读借用 `let r = &m` 时调用 `m.set("k", 1)` 被编译器静态拦截的反向测试。
