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

## [Unreleased] - 2026-10-28 (Part 4)

### 严禁同作用域重复声明变量 (Disallow Same-Scope Variable Redeclaration)

### 修改文件 (Modified)
- [`cpc/src/sema.rs`](file:///data/dev/cp/cpc/src/sema.rs):
  - 新增 `SemaError::DuplicateVarDecl(String, Span)`（`Compile Error: Cannot redeclare block-scoped variable '{0}' at {1:?}`）；
  - 在 `analyze_pattern` 中严格检查当前块作用域（`self.scopes.last()`）是否已存在同名变量绑定（`SymbolKind::Var`，排除通配符 `_`），若在同一 `{}` 作用域重复声明则立即报错拦截；
  - 将类方法中的类字段作用域与方法参数/方法体局部块作用域分离为双层作用域，允许方法内局部变量合法遮蔽同名类字段，同时禁止在方法顶层块内重声明方法参数或同名局部变量。
- [`cpc/src/codegen.rs`](file:///data/dev/cp/cpc/src/codegen.rs):
  - 新增 `check_and_record_scope_pat`，在 `rename_shadowed_vars_in_stmt` / `rename_shadowed_vars_in_block` 中按词法块维护独立的 `scope_vars: HashSet<String>`：对同一词法块内的重复变量声明执行编译期拦截，同时对合法的不同/平行子块（如 `{ let v = 10; } { let v = [10, 20]; }`）保留 CPS 状态机上下文字段的 Alpha-Renaming 消歧。
- [`cap/test/test_mut_borrow_ops.cp`](file:///data/dev/cp/cap/test/test_mut_borrow_ops.cp):
  - 将三段独立可变借用演示代码包裹在各自独立的 `{ ... }` 块作用域中。
- [`cp_ref.md`](file:///data/dev/cp/cp_ref.md):
  - 在第十章第 1 节新增“严禁同作用域重复声明变量（No Same-Scope Redeclaration）”规范，并同步更新第 3 节示例。

### 新增文件 (Added)
- [`cap/test/test_var_fail_redeclare.cp`](file:///data/dev/cp/cap/test/test_var_fail_redeclare.cp): 验证在同一 `{}` 块作用域内重复声明 `let v = 10; let v = [10, 20];` 时触发编译期报错拦截。

## [Unreleased] - 2026-10-28 (Part 5)

### 彻底移除 `codegen.rs` 同名遮蔽重命名代码 (`rename_shadowed_vars_*`) 并统一 `*` 模式匹配通配符同名检查

### 修改文件 (Modified)
- [`cpc/src/codegen.rs`](file:///data/dev/cp/cpc/src/codegen.rs):
  - 彻底删除此前误判同名重复 `let` 定义时引入的 `check_and_record_scope_pat`、`rename_shadowed_vars_in_pat`、`rename_shadowed_vars_in_expr`、`rename_shadowed_vars_in_stmt`、`rename_shadowed_vars_in_block`（共 420+ 行冗余代码）；
  - 在 `Stmt::Var`、`extract_locals_simple` 与 `extract_local_vars_to_str` 中完善 `Pattern::Wildcard`（`*`）支持（涵盖 `let * = expr;`、`let [*, x] = arr;`、`let {a: *, b} = obj;`）。
- [`cpc/src/sema.rs`](file:///data/dev/cp/cpc/src/sema.rs):
  - 在 `analyze_pattern` 中移除对 `_` 的豁免（在 CP 中通配符为 `*` 即 `Pattern::Wildcard`，而 `_` 是合法的变量/接收者标识符，重复绑定 `_` 同样报错 `DuplicateVarDecl`）；
  - 升级 `bind_match_pattern` 与 `bind_for_pattern` 返回 `SemaResult<()>`，对 `match` 模式与 `for` 循环解构中的同名变量（如 `[a, a]`、`[_, _]`）及循环体/分支块内的同名重声明统一执行 `DuplicateVarDecl` 报错拦截。
- [`cap/test/test_mut_borrow_ops.cp`](file:///data/dev/cp/cap/test/test_mut_borrow_ops.cp):
  - 将三段独立演示拆分为 `demo_primitive()`、`demo_object()`、`demo_array()` 三个独立函数，并增加 `let [*, second, *] = v;` 通配符 `*` 解构验证。
- [`cp_ref.md`](file:///data/dev/cp/cp_ref.md):
  - 明确说明模式匹配与解构通配符为 `*`（而非 `_`），重复绑定 `_` 或任意同名变量均触发编译期错误。

### 新增文件 (Added)
- [`cap/test/test_pat_fail_duplicate.cp`](file:///data/dev/cp/cap/test/test_pat_fail_duplicate.cp): 验证在模式解构中重复使用 `_`（`let [_, _] = [10, 20];`）时触发 `DuplicateVarDecl` 编译期报错拦截。

## [Unreleased] - 2026-10-28 (Part 6)

### 解构与模式匹配全面支持 `*`（单元素通配符）与 `...`（首/中/尾多元素忽略与零拷贝切片收集）

### 修改文件 (Modified)
- [`cpc/src/parser.rs`](file:///data/dev/cp/cpc/src/parser.rs):
  - 在 `parse_single_pattern` 中新增 `Token::DotDotDot` (`...`) 解析：当后接标识符时解析为具名剩余切片模式 `Pattern::Rest(Box::new(Pattern::Ident(id)))`（如 `...rest`、`...middle`），当为裸 `...` 时解析为多元素忽略模式 `Pattern::Rest(Box::new(Pattern::Wildcard))`（如 `let [first, ..., last] = arr;`），并在对象模式 `{ a, ... }` 中支持尾部 `...` 忽略。
- [`cpc/src/sema.rs`](file:///data/dev/cp/cpc/src/sema.rs):
  - 新增 `SemaError::MultipleRestInPattern(Span)`，严格限制同一数组/元组解构模式中最多只能包含一个 `...`；
  - 在 `analyze_pattern`、`bind_match_pattern`、`bind_for_pattern` 中全面支持 `Pattern::Rest` 位于开头（`[..., last]`）、中间（`[first, ..., last]` / `[head, ...middle, tail]`）或末尾（`[first, ...rest]`），自动计算前缀与后缀索引（反向偏移 `len - offset_from_end`），并将具名 `...rest` 推导为切片类型 `Type::Array(elem_ty)`。
- [`cpc/src/codegen.rs`](file:///data/dev/cp/cpc/src/codegen.rs):
  - 新增 Zig 运行时编译期辅助函数 `CpUnpackTailType` / `cp_unpack_tail`（按尾部偏移提取元素）与 `CpUnpackSliceType` / `cp_unpack_slice`（按前/后缀边界生成零拷贝子切片 `v[start .. v.len - suffix_len]`），并支持解包借用切片 `*[]const T`；
  - 在 `Stmt::Var`、`generate_pattern_match`、`Stmt::For`、`extract_locals_simple` 与 `extract_local_vars_to_str` 中全面支持 `Pattern::Wildcard` (`*`) 与 `Pattern::Rest` (`...` / `...rest`)。
- [`cap/test/test_mut_borrow_ops.cp`](file:///data/dev/cp/cap/test/test_mut_borrow_ops.cp):
  - 新增 `let [*, second, *] = v;`、`let [first, ..., last] = nums;` 与 `let [head, ...middle, tail] = nums;` 解构测试。
- [`cp_ref.md`](file:///data/dev/cp/cp_ref.md):
  - 补充单元素通配符 `*`、多元素剩余/忽略模式 `...`（裸 `...` 与具名 `...rest`）以及与范围符号 `..` / `..=` 正交分离的语法规范与示例。

## [Unreleased] - 2026-10-28 (Part 7)

### 全面重构《CP 语言官方设计规范与架构白皮书》（cp_ref.md）并升级编译器管道通配占位符与 Match 推导

### 修改文件 (Modified)
- [`cp_ref.md`](file:///data/dev/cp/cp_ref.md):
  - 彻底清洗重构历史遗留对话草稿（清除旧的 `Result/Option`、`let mut`、`loop` 及冲突通配符），升级为 11 篇体系完备的工业级金牌参考规范；
  - 全面收录 18 项核心要求：Nest 智能体容器（更名与定义）、双轨派生模型（`spawn` 协程池 vs `spawn.cpu` Worker 物理线程池）、`cap.json` 与 `@parallel` / `@single_thread` 并发配置、3-State 原生错误体系、容器写保护铁律（只读借用期间禁扩容与禁元素原地修改）、双轨对象模型（静态匿名结构体 vs 动态 `#{}` 字典）、`Box<dyn Trait>` 胖指针与 `downcast<T>()`、解构 `*` 与 `...`、管道占位符 `*` 与接收者 `_` 彻底解耦、`unsafe` 与裸指针、`macro` 宏系统、类继承 `init`/`release` 钩子等。
- [`cpc/src/parser.rs`](file:///data/dev/cp/cpc/src/parser.rs):
  - 在 `in_pipeline_rhs > 0` 管道表达式右侧解析中，支持 `*`（`Token::Star`）作为通用通配占位符（如 `data |> process(5, *)`），与方法接收者 `_` 彻底解耦。
- [`cpc/src/sema.rs`](file:///data/dev/cp/cpc/src/sema.rs):
  - 在 `infer_type` 中增加对 `Expr::Match`（`switch` 表达式）分支返回值的静态类型推导，避免类型退化为 `Type::Any`。

### 新增文件 (Added)
- [`cap/test/test_pipe_star.cp`](file:///data/dev/cp/cap/test/test_pipe_star.cp): 验证管道操作符配合 `*` 通配占位符与 `.` 占位符的正向编译与执行测试。

## [Unreleased] - 2026-10-29 (Part 8)

### 对象解构全链路支持 `...rest` 剩余属性变量绑定与 `...` / `...*` 忽略通配

### 修改文件 (Modified)
- [`cpc/src/ast.rs`](file:///data/dev/cp/cpc/src/ast.rs):
  - `ObjectPatProp` 新增 `is_rest: bool` 标识符，精准表示具名剩余属性（`...rest`）或匿名忽略剩余属性（`...` / `...*`）。
- [`cpc/src/ast_visitor.rs`](file:///data/dev/cp/cpc/src/ast_visitor.rs):
  - 在 `Pattern::Object` 递归提取闭包变量时，准确支持识别并注册解构对象中的具名 `...rest` 变量。
- [`cpc/src/parser.rs`](file:///data/dev/cp/cpc/src/parser.rs):
  - 重构对象模式解析：当遇到 `Token::DotDotDot` (`...`) 时，若后接标识符解析为 `ObjectPatProp { key: rest_id, val: None, is_rest: true }`；若后接 `*` 或直接为末尾/逗号，解析为通配忽略 `ObjectPatProp { key: "*", val: Some(Wildcard), is_rest: true }`。
- [`cpc/src/sema.rs`](file:///data/dev/cp/cpc/src/sema.rs):
  - 在 `bind_match_pattern` 与 `bind_for_pattern` 中支持对象解构模式中最多只能有 1 个 `...` 的合法性校验，并为具名 `...rest` 定义作用域变量；
  - 在 `analyze_pattern` 中支持剩余字段强类型计算：自动筛除已被非 rest 属性提取的键，将剩余属性构建为强类型匿名对象 `Type::Object(remaining_fields)` 并绑定到 `rest` 变量中。
- [`cpc/src/codegen.rs`](file:///data/dev/cp/cpc/src/codegen.rs):
  - 在 `Stmt::Var`（`let { a, ...rest } = obj;`）中计算剩余字段，为 `rest` 生成强类型匿名结构体 `.{ .b = _tmp_obj.b, .c = _tmp_obj.c }`；
  - 在 `generate_pattern_match`（`switch (obj) { { a, ...extra } => extra.b }`）中生成剩余字段结构体绑定；
  - 在 `Stmt::For`（`for { a, ...rest } of list`）中支持循环迭代解构绑定剩余字段；
  - 在 `extract_locals_simple` 与 `extract_local_vars_to_str` 中支持 CPS 状态机注册 `rest` 变量及其 `__CpObj` 类型。
- [`cp_ref.md`](file:///data/dev/cp/cp_ref.md):
  - 更新第 3.2 节与生态路线图，正式标记对象解构 `...rest` 剩余绑定已全面实装。

### 新增文件 (Added)
- [`cap/test/test_obj_destructure_rest.cp`](file:///data/dev/cp/cap/test/test_obj_destructure_rest.cp): 验证对象解构具名 `...rest`、裸 `...`、`...*`、`switch` 分支与 `for` 循环解构的全链路正向测试。
- [`cap/test/test_obj_rest_fail_multiple.cp`](file:///data/dev/cp/cap/test/test_obj_rest_fail_multiple.cp): 验证对象解构中包含多个 `...` 时的编译期错误拦截反向测试。

## [Unreleased] - 2026-10-29 (Part 9)

### 扩充《CP 语言官方设计规范与架构白皮书》（cp_ref.md）：新增附录《CP 在 AI 与 AI 智能体领域的独到优势与设计哲学》

### 修改文件 (Modified)
- [`cp_ref.md`](file:///data/dev/cp/cp_ref.md):
  - 目录（TOC）更新：新增《附录：CP 在 AI 与 AI 智能体领域的独到优势与设计哲学》三级索引（A.1、A.2、A.3）；
  - 全文末尾系统性增补附录 A，涵盖：
    - **A.1 主流语言物理死穴全景对比**：深度剖析 Python（GIL 锁死/asyncio 卡顿/内存暴涨）、TypeScript（单线程阻塞/无原生多核/V8 堆上限）、Go（百万 Agent 记忆引发 STW 垃圾回收停顿）、Rust（async 状态管理地狱/Arc-Mutex 泛滥/无抢占调度）、Zig（纯手工内存管理/缺乏并发信箱原语）在海量智能体场景下的致命瓶颈；
    - **A.2 CP 语言六大核心独到优势**：
      1. 独创“双轨派生模型”（`spawn` 纳秒协程池 vs `spawn.cpu` Worker 物理线程池）；
      2. 彻底无 GC（No GC）+ CPS 状态机切分（百微秒级冷启动、单机支撑百万活跃 Agent 驻留）；
      3. `SharedTensor` 全局共享堆与零拷贝原子引用穿透通信；
      4. 抢占式调度（Reductions 计数时间片，杜绝复杂 Prompt/CoT 思维链饿死集群）；
      5. 双轨对象模型（静态匿名结构体 vs 动态字典 `#{}`）与 3-State 原生错误体系（LLM Tool Calling 零阻抗与极简容错）；
      6. Crash-Proof 故障隔离与“任其崩溃”哲学（Supervisor Tree 热拉起与 NIF Resource Objects 资源兜底）；
    - **A.3 典型真实业务场景落地实测对比**：万人同域 NPC 仿真 (Virtual Town)、高频金融量化群智决策、边缘嵌入式 Agent 设备矩阵对比。
- [`.gitignore`](file:///data/dev/cp/.gitignore):
  - 添加 `*-bak*.md` 规则，防止规范文件历史备份草稿污染 git 工作区。

## [Unreleased] - 2026-10-29 (Part 10)

### 配合用户对 `cp_ref.md` 的内容更新，重新整理并同步文档目录索引 (TOC)

### 修改文件 (Modified)
- [`cp_ref.md`](file:///data/dev/cp/cp_ref.md):
  - 仅重新整理文档开头的 `## 目录索引 (Table of Contents)`，未改动任何正文内容；
  - 第一篇目录全量同步用户新增的各大章节：
    - `1.2 CP/Rust/Zig/Go/TypeScript 精简对比表`（包含 1.2.1 差异化标签与 1.2.2 优势）；
    - `1.3 为AI而生：CP在 AI/AI智能体 领域的独到优势与设计哲学`（包含 1.3.1 物理死穴对比、1.3.2 六大独到优势与核心架构解析、1.3.3 典型场景实测直观对比）；
    - `1.4 高性能服务后端（替代部分 Go/C++/C/Rust/Zig）`；
    - `1.5 CLI工具、开发工具链（替代 Go / Zig）`；
    - `1.6 嵌入式、裸机、RTOS（Zig原生主场，CP是加分版）`；
    - `1.7 游戏引擎底层、高性能引擎脚本/绑定层（替代 C++、部分Rust）`；
    - `1.8 跨平台系统软件、Daemon、系统组件（替代C / C++）`；
    - `1.9 高性能WebAssembly（Wasm）`；
    - `2.0 教育、入门系统级编程（这个是很特别的差异化赛道）`；
    - `2.1 现有Zig项目的上层迁移层（Zig生态内部的升级选择）`；
    - `2.2 边界：哪些场景CP不适合主推（心里要有数）`；
    - 以及构建与包管理器相关章节索引；
  - 移除已合入第一篇的原末尾附录目录索引条目。

## [Unreleased] - 2026-10-29 (Part 11)

### 重构《CP 语言官方设计规范与架构白皮书》（cp_ref.md）：独立升格第二篇《`cap` 统一工具链与模块生态体系》并全面对标 Bun.js

### 修改文件 (Modified)
- [`cp_ref.md`](file:///data/dev/cp/cp_ref.md):
  - 全局章节升格：将规范手册由原 11 篇系统性扩充重组为 **12 篇完整宏伟架构**，原第二篇至第十一篇依次顺延为第三篇至第十二篇，对应子章节编号（`3.1`~`12.2`）全链路对齐；
  - 第一篇定名为《第一篇：CP 语言哲学、核心愿景与全景场景矩阵》，纯粹聚焦于核心定位、竞品对比、AI/智能体独到优势与 9 大落地场景；
  - 正式独立升格设立《第二篇：`cap` 统一工具链与模块生态体系 (The `cap` All-in-One Toolchain & Ecosystem)》：
    - **2.1 架构愿景与品牌标识**：引入 CAP ($C^P$) 官方 ASCII 标志徽章，深度确立对标 Bun.js 的“全生态归一”哲学（CAP 之于 CP 犹如 Bun 之于 TS/JS）；
    - **2.2 CLI 全指令全景矩阵**：系统梳理 21 项指令，包含已实装指令（`run`, `build`, `-r`, `-f`, `-s`, `--exe`, `--obj`, `--lib`, `-b`, `add`, `-g`, `test`）、正在演进指令（`init`, `remove`, `update`, `bench`）与规划中指令（`fmt`, `lint`, `publish`, `cache clean`）；
    - **2.3 全局缓存与软链接零拷贝注入**：绘制 Mermaid 架构流向图，阐释 `~/.cap/cache/` 全局单实例存储与 `cap_modules/` 毫秒级符号软链接注入机制，终结 `node_modules` 磁盘黑洞；
    - **2.4 `cap.json` 权威清单规范**：详述工程清单、并发模式、依赖 Semver、Monorepo `workspaces`、编译控制与 Nest 容器专有参数；
    - **2.5 ES6 风格模块系统与别名权威裁决**：
      - 深度论证并裁决模块更名为何坚守 `as`（`import { a as b }`）而非 `:`，确立 `:` 作为类型注解的神圣正交性、保持 `* as math` 的纯粹自洽，并对齐主流工程直觉；
      - 详解三层向上攀升依赖查找机制（Upward Resolution）与目录缺省入口推断优先级；
    - **2.6 CAP 特性演进与对标路线图**：以对照表清晰展示 CAP 对标 Bun / Cargo 的全景落地现状与未来路线；
  - 同步全量更新全文顶部的 `## 目录索引 (Table of Contents)`。

---

## [Unreleased] - 2026-10-29 (Part 12)

### CAP ($C^P$) 统一工具链工业级重构：配置驱动构建、多模式优化矩阵、测试套件与脚手架全面落地

### 修改文件 (Modified)
- [`cap/src/project/manifest.zig`](file:///data/dev/cp/cap/src/project/manifest.zig):
  - 深度扩充 `cap.json` 清单数据模型：新增 `description`、`mode`、`dev_dependencies`、`build_config`（`opt`, `out_dir`, `target`, `targets`, `c_flags`, `link_libs`）与 `nest_config`（`mailbox_capacity`, `reduction_slice`）；
  - 引入 `OptimizeMode` 枚举（`Debug`, `ReleaseSafe`, `ReleaseFast`, `ReleaseSmall`）及字符串双向映射方法；
  - 增强 `addDependency` 支持开发依赖（`is_dev` 参数），新增 `removeDependency` 实现依赖全生命周期管理；
  - 升级 `save()` 保证清单全字段高保真格式化输出，支持数组与对象树序列化。
- [`cap/src/main.zig`](file:///data/dev/cp/cap/src/main.zig):
  - 路由中枢全面升级：实装 `build`, `run`, `init`, `add`, `remove` (`rm`), `update` (`up`), `install`, `test`, `bench`, `sniff`, `help` 全指令矩阵；
  - 构建系统打通 `cap.json` 配置解析与 CLI 参数覆盖：确立 `CLI 参数 > cap.json build 配置 > 内置默认值` 决策优先级链；
  - 完整支持 Zig 4 级优化模式切换（`-f`/`--fast`, `-s`/`--small`, `--debug`, `--safe`, `--opt=<mode>`）；
  - 支持单目标交叉编译覆盖（`-t`/`--target=<triple>`）与配置驱动的多平台发布矩阵（`-r`/`--release`）；
  - 实装 `cap test [dir]`：自动递归扫描 `test/` 发现所有测试用例，在 `Debug` 模式下秒级编译执行并输出美观的进度耗时与汇总统计面板；
  - 实装 `cap bench <file.cp>`：在 `ReleaseFast` 极致算力模式下编译并执行，使用 `std.Io.Timestamp` 进行纳秒级高精度基准计时；
  - 实装 `cap init [app|agent|lib]`：开箱即用多模板脚手架，一键初始化规范目录结构、对应 `cap.json` 与代码样例；
  - 实装 `cap remove` 与 `cap update` 依赖卸载与升级。
- [`cap/build.zig`](file:///data/dev/cp/cap/build.zig):
  - 增加对 `../bin` 的安装步骤，确保 `zig build` 能够同步生成并刷新 `cap/bin/cap` 与 `cap/zig-out/cap` 二进制文件。
- [`cp_ref.md`](file:///data/dev/cp/cp_ref.md):
  - 更新第二篇 2.2 全指令全景矩阵，将 `cap init`, `cap remove`, `cap update`, `cap bench` 状态升格为 **已全面实装**；
  - 更新 2.4 `cap.json` 规范章节，详述 Zig 4 大编译优化级别（`debug`, `safe`, `fast`, `small`）、`targets` 平台矩阵以及构建参数覆盖三层优先级决策链。

---

## [Unreleased] - 2026-10-29 (Part 14)

### 品牌标识与徽标体系重塑：确立 CP 语言专属双字母徽标与 CAP 工具链方案 A 向上跃升箭头徽标（C ^ P => CAP）

### 修改文件 (Modified)
- [`cp_ref.md`](file:///data/dev/cp/cp_ref.md):
  - 开篇正文引入 **CP 语言原生专属徽标**（`C` + `P` 经典双字母对称架构），代表 CP 语言本体（并发算力与精密掌控）；
  - 第二篇 2.1 正式装配 **CAP 工具链方案 A 专属徽标**（`C` + `^` + `P`）；
  - 彻底重塑品牌定义：明确 CAP 绝非“C 的 P 次方”，而是以代表并发算力向上跃迁、智能涌现与极速调度的向上大箭头 **`^` 代替字母 `A`**，直观牢固地连接 `C` 与 `P`，在视觉与语意上拼写出 **`CAP`**。
- [`cap/src/main.zig`](file:///data/dev/cp/cap/src/main.zig):
  - 全面更新终端 ASCII 艺术徽标为方案 A：包括 `cap help` 帮助头、`cap test` 测试套件运行头以及 `cap bench` 微基准评测运行头。

---

## [Unreleased] - 2026-10-29 (Part 15)

### 官方命名体系全景统一：确立 CP (Concurrent Plus) 与 CAP (Concurrent Application Plus) 宏大格局

### 修改文件 (Modified)
- [`cp_ref.md`](file:///data/dev/cp/cp_ref.md):
  - 规范开篇与核心愿景全量统一为 **CP (Concurrent Plus)**；
  - 第二篇 2.1 节正式定名为 **CAP (Concurrent Application Plus)**；
  - 详述 `CP` 与 `CAP` 的同源血脉与几何同构：在 `C`（Concurrent 调度基座）与 `P`（Plus 性能安全增益）之间，注入字母 `A`（`Application`），由向上跃迁大箭头 `^`（Ascend）生动连接，象征由语言规范迈向生产级应用交付的全景赋能。
- [`cap/src/main.zig`](file:///data/dev/cp/cap/src/main.zig):
  - 终端 `usage()`, `handleTest()`, `handleBench()` 欢迎横幅全量更新为 `CAP : Concurrent Application Plus v0.1.0`；
  - 重新编译输出最新原生二进制，全面通过实测。

---

## [Unreleased] - 2026-10-29 (Part 16)

### 增量编译流水线 (Incremental Build)、编译耗时度量 (Build Profiling) 与缓存清理 (cap clean) 实装

### 修改文件 (Modified)
- [`cpc/src/codegen.rs`](file:///data/dev/cp/cpc/src/codegen.rs):
  - 移除 8 处调试遗留的 `[TRACE]` 标准输出，净化前端转译终端输出。
- [`cpc/src/main.rs`](file:///data/dev/cp/cpc/src/main.rs):
  - 增加转译内容一致性预检：若输出 `.zig` 内容与已有文件一致，跳过磁盘写入以保留文件修改时间戳（`mtime`），供下游构建工具进行增量缓存判定。
- [`cap/src/main.zig`](file:///data/dev/cp/cap/src/main.zig):
  - **增量构建系统 (Incremental Build)**：检查输出二进制与源码 `.cp` 的 `mtime`，未发生变更时 0ms 极速跳过转译与 LLVM 链接，对标 Bun/Go 的即时开发体验；
  - **中间态感知复用**：若中间态 `.cap/zig/` 源码新于 `.cp` 源码，自动跳过 `cpc` 前端转译；
  - **编译耗时度量 (Build Profiling)**：在编译过程中精确度量 `cpc` 前端转译毫秒数、Zig/LLVM 原生链接秒数与全流程总耗时；
  - **强制构建参数 (`-B` / `--force`)**：支持跳过增量缓存直接强制重新编译；
  - **缓存与产物清理 (`cap clean`)**：一键安全清理 `.cap/zig/` 中间源码、`.cap/zig-cache/` 编译器缓存、`.cap/obj/` 与产物目录 `bin/`。
- [`cp_ref.md`](file:///data/dev/cp/cp_ref.md):
  - 更新 2.2 CLI 全景矩阵表格，正式收录 `cap clean` 与 `cap build -B` 选项。

---

## [Unreleased] - 2026-10-29 (Part 17)

### 官方用户门户与主自述文件 (README.md) 工业级重构与全景技术展示

### 修改文件 (Modified)
- [`README.md`](file:///data/dev/cp/README.md):
  - 引入完整的 **CP 语言双字母徽标** 与 **CAP 工具链方案 A 向上大箭头徽标**（`C • ^ • P`）；
  - 全面系统地剖析 Python、Node.js、Go、Rust、Erlang 在百万级 AI Agent 并发下的痛点与 CP 终极破局之道；
  - 阐述“脑、神经、肌肉、血液”四位一体终极架构并嵌入 Mermaid 架构图；
  - 新增 Actor 消息信箱、Bun 风格极速 HTTP 服务端与 `SharedTensor` 硬件原子零拷贝三大代码实战展示；
  - 详述 CAP 工具链从调试（`cap <file>`）、增量编译（0ms）、极限性能（`-f`）、边缘微体积（`-s`）、矩阵发布（`-r`）、测试、基准到一键缓存清理（`cap clean`）的全流程工作流；
  - 制作全方位横向对比矩阵，直观对比 Python、Node.js、Go、Rust 与 CP 在 8 大维度的核心指标。

---

## [Unreleased] - 2026-10-31 (Part 18)

### 批量标签收拢语法 (`@[...]`) 与弱类型即席元标注 (`@meta(...)`) 零成本正交设计实装

### 新增文件 (Added)
- [`cap/test/test_batch_meta_pass.cp`](file:///data/dev/cp/cap/test/test_batch_meta_pass.cp):
  - 覆盖类、方法、字段级批量收拢容器 `@[...]`，切面装饰器与元注解混排；
  - 验证弱类型即席元标注 `@meta(...)` 搭配 `@[reflect, ...]` 时属性扁平直读与命名空间嵌套读取；
  - 验证未加 `@reflect` 的纯元数据类拥有零运行时开销。
- [`cap/test/test_meta_fail_unreflected.cp`](file:///data/dev/cp/cap/test/test_meta_fail_unreflected.cp):
  - 反向测试：未标注 `@reflect` 的类在调用 `reflect.type<T>()` 时，编译器精准拦截并输出诊断引导。

### 修改文件 (Modified)
- [`cpc/src/parser.rs`](file:///data/dev/cp/cpc/src/parser.rs):
  - `parse_meta_ident` 与 `parse_meta_arg`：扩展支持 `Token::Meta`, `Token::Dec`, `Token::Macro` 等关键字作为元标注标识符，解决关键字作为标注名的语法解析问题；
  - `parse_single_meta_inner`：严格遵循函数式括号传参规范 `(...)`，纠正方括号参数误区（`[]` 专属于批量集合容器 `@[...]`）；
  - `parse_meta_list`：实装 `@[...]` 批量语法，一口气收拢声明多个元注解、装饰器与即席标注，末尾逗号自动容错；
  - `parse_module_item`：前瞻分析器支持 `@[...]` 括号深度精准匹配，确保声明关键字安全解析。
- [`cpc/src/sema.rs`](file:///data/dev/cp/cpc/src/sema.rs):
  - 注册内建 `@meta` 弱类型元注解契约（允许多目标全覆盖：类、方法、字段、参数、Trait、Enum，支持多实例重复标注）；
  - 践行“零成本抽象（Zero-Cost Abstraction）”方案 A：严格保持正交性，`@meta` 专注于元数据挂载，运行时反射必须由 `@reflect` 统一显式授权激活，未授权时精确报错。
- [`cpc/src/codegen.rs`](file:///data/dev/cp/cpc/src/codegen.rs):
  - `generate_meta_population`：为 `@meta` 实现平铺注入顶层属性（`info.meta["author"]`）与嵌套命名空间（`info.meta["meta"]["author"]`）的双重读取支持；
  - `generate_reflection_tables`：仅对显式标注 `reflect` 的类生成运行时反射单例字典，杜绝无谓的二进制代码膨胀与 Tree-Shaking 阻碍；
  - `generate_class`：增加对 `TypeInfo`、`FieldInfo`、`MethodInfo` 等内建类型的重复生成过滤，防止导入时符号重复定义冲突；
  - `cp_message_from` & `cp_index_get`：修复指针类型包装时取栈局部变量地址导致的悬垂指针缺陷，并让 `cp_index_get` 支持对 `Message.any` 动态 Map 的链式属性递归读取。
- [`cp_ref.md`](file:///data/dev/cp/cp_ref.md):
  - 完整规范化第 3.6 节《弱类型/即席元标注（Ad-hoc Metadata：`@meta(...)`）》与第 3.7 节《批量标签收拢语法（`@[...]`）》，强调 `[]` 集合语义与零成本正交设计。

---

## [Unreleased] - 2026-11-01

### 标载荷代数数据类型 (Tagged Union Enum) 与 Switch 全模式解构变量捕获体系实装

### 新增文件 (Added)
- [`cap/test/test_switch_binding_pass.cp`](file:///data/dev/cp/cap/test/test_switch_binding_pass.cp):
  - 覆盖带载荷枚举（无载荷 `Idle`、单载荷 `Text: string`、元组载荷 `Move: [i32, i32]`、结构体载荷 `Compute: { batch: usize, priority: i32 }`）；
  - 覆盖上下文推导前缀省略构造（`.Idle`, `.Text("hello")`, `.Move [15, 25]`, `.Compute { ... }`）；
  - 覆盖 `switch` 元组方括号解构模式匹配（`.Move [x, y]`）；
  - 覆盖 `switch` 结构体字段解构匹配（`.Compute { batch, priority }`）；
  - 覆盖 `switch` 结构体剩余/全捕获匹配（`.Compute { ...fields }`）；
  - 覆盖 `switch` 海象操作符区间变量绑定（`s := 90..=100`）；
  - 验证运行结果 100% 符合预期。

### 修改文件 (Modified)
- [`cpc/src/lexer.rs`](file:///data/dev/cp/cpc/src/lexer.rs):
  - 新增 `#[token(":=")] ColonAssign` 词法标记，支持 Walrus（海象操作符）变量绑定语法。
- [`cpc/src/ast.rs`](file:///data/dev/cp/cpc/src/ast.rs):
  - `Pattern` 枚举新增 `Pattern::Binding { ident: Ident, subpat: Box<Pattern> }`（变量与子模式绑定）与 `Pattern::TypeMatch { ident: Ident, ty: Type }`（动态类型匹配分支）；
  - `EnumInitExpr` 增加 `#[serde(default)] pub tuple_args: Option<Vec<Expr>>`，原生支持 `.Move [x, y]` 元组构造。
- [`cpc/src/ast_visitor.rs`](file:///data/dev/cp/cpc/src/ast_visitor.rs):
  - 在 `extract_idents_from_pat` 中适配 `Pattern::Binding` 与 `Pattern::TypeMatch`，确保变量分析与借用检查完整捕获绑定标识符。
- [`cpc/src/parser.rs`](file:///data/dev/cp/cpc/src/parser.rs):
  - `Parser` 引入 `in_switch_arm: usize` 状态追踪，精准消除 `switch` 分支中 `ident: Type` 类型分支与对象字面量键值对的文法歧义；
  - `parse_single_pattern`：支持 `.Variant` 匹配时直接接 `{ ... }`（含 `...rest` 字段解构）、`[ ... ]`（元组解构）、`(...)`（括号解构）与单个 `Ident` 载荷捕获；
  - 支持 `ident := subpat` 海象操作符绑定与 `ident: Type` 动态类型匹配；
  - 彻底移除旧版 Zig 风格的 `|fields|` 管道捕获语法；
  - `parse_enum`：支持 `Ident: Type` 语法定义枚举载荷变体；
  - `parse_primary_expr` & `parse_postfix_expr`：支持在已知类型上下文中直接使用 `.Variant { ... }`、`.Variant [ ... ]`、`.Variant( ... )` 进行隐式推导构造。
- [`cpc/src/sema.rs`](file:///data/dev/cp/cpc/src/sema.rs):
  - 在 `bind_match_pattern`、`bind_for_pattern` 与 `analyze_pattern` 中支持 `Pattern::Binding` 与 `Pattern::TypeMatch` 的作用域符号注入与类型推导；
  - 放行前缀为空的隐式推导枚举构造（推断为 `Type::Any` 或上下文目标类型）。
- [`cpc/src/codegen.rs`](file:///data/dev/cp/cpc/src/codegen.rs):
  - `generate_pattern_match`：支持带载荷枚举多元素元组解构绑定（`cp_unpack_elem`）、结构体属性提取与剩余字段全量打包绑定（`...fields`）；
  - 支持 `Pattern::Binding` 生成 `const ident = val_expr;` 绑定，以及 `Pattern::TypeMatch` 的运行时类型判断与指针安全窄化转换；
  - 修复 `Expr::Cast` 对数值类型（整数、浮点数）无脑生成 `@ptrCast(@alignCast(...))` 的底层缺陷，精确按目标类型生成 Zig `@intCast`、`@floatCast`、`@intFromFloat` 与 `@floatFromInt`；
  - 重命名内部异步任务结构为 `CpTask`，避免用户定义 `enum Task` 时与 Zig 预置上下文名称冲突。
- [`cap/test/test_traits_enums.cp`](file:///data/dev/cp/cap/test/test_traits_enums.cp):
  - 语法升级：将旧版遗留的 `match (task) { .Compute |fields| => ... }` 重构为规范的 `switch (task) { .Compute { batch } => ... }`。
- [`cp_ref.md`](file:///data/dev/cp/cp_ref.md):
  - 增补第 3.5 节《枚举与代数数据类型（Enum & Tagged Union）》，规范化定义、上下文推导与构造语法；
  - 增补第 4.5 节《枚举载荷解构、Walrus 绑定与动态类型匹配》，详细记录结构体解构、元组解构、海象绑定、类型分支匹配及废除 `|fields|` 的设计。

---

## 2026-11-01 (全量编译期元编程系统 Comptime、自动单态化与静态内省原语)

### 修改文件 (Modified)
- [`cpc/src/ast.rs`](file:///data/dev/cp/cpc/src/ast.rs):
  - 为 `IfStmt` 与 `ForStmt` 增加 `is_comptime: bool` 字段，支持静态分支修剪与循环展开；
  - 新增 `Expr::Comptime(BlockStmt)` 表达式变体，支持 `@{ ... }` 编译期求值代码块；
  - 新增 `Type::Auto` 类型变体，支持 `auto` 编译期参数类型推导。
- [`cpc/src/lexer.rs`](file:///data/dev/cp/cpc/src/lexer.rs):
  - 新增 `Token::Auto` 关键字词元。
- [`cpc/src/parser.rs`](file:///data/dev/cp/cpc/src/parser.rs):
  - `parse_params`：收敛形参约束语法，仅保留统一规范的 `@comptime param: Type`（严格契合“带 @ 为编译期”铁律），切除 bare `comptime`；
  - `parse_base_type`：收敛自动推导语法，仅保留统一规范的 `auto`，彻底切除冗余的 `anytype`；
  - `parse_stmt`：识别 `@{ ... }` 与 `!{ ... }` 产生 `Stmt::ComptimeBlock`；识别 `@if` 与 `@for` 并置 `is_comptime = true`；
  - `parse_primary_expr`：支持 `@{ ... }` 产生 `Expr::Comptime`；将非内置宏指令的 `@fn(...)` 统一解析为 `Expr::Call { is_comptime: true, ... }` 编译期函数显式调用；将 `@test`、`@expect_eq` 以及其他标准库宏指令纳入内置宏保护清单。
- [`cpc/src/sema.rs`](file:///data/dev/cp/cpc/src/sema.rs):
  - 增强 `infer_type` 对 `Expr::Comptime` 与 `Expr::Unsafe` 的类型推导容错，支持块内局部变量推导并在未解析时优雅降级；
  - 完善 `Stmt::ComptimeBlock` 与 `Expr::Comptime` 块内语义检查与作用域管理。
- [`cpc/src/codegen.rs`](file:///data/dev/cp/cpc/src/codegen.rs):
  - 将 `Type::Auto` 映射为 Zig 原生 `anytype`；
  - `Expr::Comptime` 与 `Stmt::ComptimeBlock`：生成 `comptime (blk: { ... })` 与 `comptime { ... }`，生成期间自动将 `is_state_machine` 置为 false 并置 `is_comptime` 为 true，确保局部变量直接生成为编译期变量而非状态机字段；
  - `Stmt::If`：当 `is_comptime` 时，生成 `if (comptime ...) { ... } else { ... }` 实现编译期静态分支修剪；
  - `Stmt::For`：当 `is_comptime` 时，生成 `inline for (...) |...| { ... }` 实现编译期静态循环展开；
  - 函数形参：`is_comptime` 形参生成 `comptime ` 前缀，跳过取地址操作；
  - 函数调用：`is_comptime` 函数调用生成 `comptime callee(args)`；
  - 内建编译期内省指令映射：支持 `@typeOf`、`@typeName`、`@fieldsOf`、`@field`、`@hasField`、`@sizeOf`、`@alignOf`、`@compileError`、`@compileLog`；
  - `infer_cp_type_with_mock` 与 `infer_zig_type`：递归分析 `Expr::Comptime` 与 `Expr::Unsafe` 内部局部变量推断真实返回类型；
  - `cp_add`：增强多整数类型混用的原生加法支持（自动通过 `@intCast` 适配 `i32` 与 `usize` 等混合运算）。
- [`cpc/src/dcp.rs`](file:///data/dev/cp/cpc/src/dcp.rs):
  - 支持 `Type::Auto` 格式化为 `"auto"`。
- [`cpc/src/ast_visitor.rs`](file:///data/dev/cp/cpc/src/ast_visitor.rs):
  - 支持 `Expr::Comptime` 遍历与访问。
- [`cp_ref.md`](file:///data/dev/cp/cp_ref.md):
  - 更新第 11.1 节《编译期元编程系统（Comptime Metaprogramming）》，详细记录 `@{ }` 编译期代码块、`@fn()` 编译期函数调用、`@comptime` 参数约束、`val: auto` 单态化、`@if` 静态分支修剪、`@for` 静态循环展开及 9 种内建编译期内省指令。

### 新增文件 (Added)
- [`cap/test/test_comptime.cp`](file:///data/dev/cp/cap/test/test_comptime.cp):
  - 涵盖 `@{ }` 编译期求值、`@add()` 编译期函数调用、`@comptime` 参数约束、`val: auto` 编译期推导参数、`@if / @else` 分支修剪与 `@for` 循环展开的完整自动化回归测试用例，100% 测试通过。

---

## 2026-11-01 (工业级宏元编程系统：内置派生宏、聚合派生 @derive 与第三方 AST 模板宏完整实装)

### 修改文件 (Modified)
- [`cpc/src/ast.rs`](file:///data/dev/cp/cpc/src/ast.rs):
  - 为 `MacroDef` 结构体新增 `methods: Vec<Function>`，支持宏体内定义可注入目标类型的模板方法。
- [`cpc/src/lexer.rs`](file:///data/dev/cp/cpc/src/lexer.rs):
  - 扩展 `Ident` 正则词元规则，原生支持识别 `${field}` 与 `get_${field}` 等 AST 模板插值标识符；
  - 注册 `#[token("$")] Dollar` 词元。
- [`cpc/src/parser.rs`](file:///data/dev/cp/cpc/src/parser.rs):
  - 重构 `parse_macro_def`：支持解析宏体花括号 `{ ... }` 内声明的函数（`fn` / `pub fn`）与语句，注入 `MacroDef.methods`；
  - 在 `is_known_builtin` 注册清单中补全 `@hash` 原语以及全部底层系统内置宏（`@io_*`、`@fs_*`、`@net_*`、`@http_*`、`@crypto_*` 等）。
- [`cpc/src/sema.rs`](file:///data/dev/cp/cpc/src/sema.rs):
  - 全量内置派生宏支持：
    - `@Clone`：自动合成 `fn clone(_): Self` 深拷贝方法（返回 `new Class(_.f1, _.f2, ...)`）；
    - `@Comparable`：自动合成字典序比较 `fn cmp(_, other: &Self): i32` 与等价性比对 `fn eq(_, other: &Self): bool`；
    - `@Hash`：自动合成 `fn hash(_): usize`，结合全字段级联 FNV-1a 计算稳定哈希值；
    - `@ToString`、`@Json`、`@Table`：稳定继承并支持多方法批量衍生；
  - `@derive(...)` 聚合宏与 `@[...]` 批量语法：统一展开引擎，支持任意组合派生（如 `@derive(Clone, Comparable, Hash, ToString, Json)`）；
  - 第三方 AST 模板插值宏（Template Macros）：
    - 支持宏定义形参到实参的映射绑定与 AST 变量替换（函数名 `${field}`、属性访问 `_.${field}`、返回类型及字面量文本）；
    - 保留无体空宏向后兼容性（如 `macro Tag(target) {}` 自动回退生成 `getTag(): string`）；
  - 零样板构造函数自动合成：当类中包含字段且开发者未显式编写 `fn init(...)` 时，编译期 Sema Phase 2 自动生成成员级 `init` 构造函数。
- [`cpc/src/codegen.rs`](file:///data/dev/cp/cpc/src/codegen.rs):
  - 在 Zig 运行时基座注入 `cp_lt`、`cp_gt`、`cp_lte`、`cp_gte` 与 `cp_hash_val` 强类型原生辅助函数；
  - `BinaryOp`：`<`、`>`、`<=`、`>=` 自动映射为 `cp_lt` / `cp_gt` / `cp_lte` / `cp_gte`，支持字符串与复杂结构的原生字典序比对；
  - `Expr::Builtin`：为 `@hash` 原语生成零开销 FNV-1a 编译期级联哈希内联块 `(blk: { var _h: usize = ...; break :blk _h; })`。
- [`cp_ref.md`](file:///data/dev/cp/cp_ref.md):
  - 全面更新第 11.2 节《宏元编程（Macro Metaprogramming）》，详尽记录内置派生宏清单（`@Clone`、`@Comparable`、`@Hash`、`@ToString`、`@Json`、`@Table`）、`@derive(...)` 聚合宏、`@[...]` 批量注解语法、第三方 AST 模板宏（`${param}` 插值）与 Comptime 过程宏的最佳实践与代码范式。
---

## 2026-11-01 (语法纯净化：切除遗留 @derive，全面归一为原生批量注解 @[...])

### 修改文件 (Modified)
- [`cpc/src/sema.rs`](file:///data/dev/cp/cpc/src/sema.rs):
  - 彻底切除并移除 Rust 遗留的 `@derive` 冗余宏名称与 `expand_derive_macro` 展开逻辑；
  - 消除语法冗余与表达割裂，唯一标准统一为 CP 原生一等公民的批量元注解体系 `@[Clone, Comparable, Hash, ToString, Json]`。
- [`cap/test/test_macro_pass.cp`](file:///data/dev/cp/cap/test/test_macro_pass.cp):
  - 将 `OrderItem` 上的 `@derive(...)` 替换为规范的 `@[Clone, Comparable, Hash, ToString, Json]`，全量测试 100% 绿灯。
---

## 2026-11-01 (Debug 派生宏与双轨 for...of 流式迭代器架构完整落地)

### 修改文件 (Modified)
- [`cpc/src/sema.rs`](file:///data/dev/cp/cpc/src/sema.rs):
  - 注册并实装内置派生宏 `Debug`，自动合成 `fmt_debug(&_): string` 与 `toString(_): string` 双重调试打印方法；
  - 完美打通 `@[Debug]` 与 `trait Debug` 的契约装配，支持标准 Trait 格式化约束。
- [`cpc/src/codegen.rs`](file:///data/dev/cp/cpc/src/codegen.rs):
  - `Stmt::For` 迭代引擎升级为**双轨智能分流架构**：
    - **快轨（Fast Path）**：针对连续切片与数组，保持极致的 `cp_iter_slice` 裸机遍历与 `while` 计数器循环；
    - **流式轨（Iterator Stream Path）**：针对包含 `next(*_): ?Item` 契约的对象或数据流，全自动展开为 `while (__iter.next()) |item|` 循环，当 `next()` 返回 `null` 时零开销自动终止，并支持元组/结构体解构模式。
- [`cp_ref.md`](file:///data/dev/cp/cp_ref.md):
  - 增补第 11.2.9 节《迭代器契约与双轨 `for ... of` 迭代机制》，详尽规范快轨连续内存遍历与流式迭代器实现；
  - 增补第 11.2.10 节《泛型 Trait Bound 契约约束》，阐述 `fn log(item: &Debug)` 与 `@[Debug]` 协作范式。
- [`cap/test/test_macro_pass.cp`](file:///data/dev/cp/cap/test/test_macro_pass.cp):
  - 新增测试用例 9：`@[Debug, Clone]` 自动合成 `fmt_debug` 与 `toString`；
  - 新增测试用例 10：自定义流式迭代器 `NumberStream` 经由 `for n of stream` 成功遍历求和，全量 100% 绿灯通过。

---

## 2026-11-01 (语法纯净化：全面清理 Rust 风格 &mut，统一为 CP 正统 *_ / *this 可变借用)

### 修改文件 (Modified)
- [`cap/test/test_macro_pass.cp`](file:///data/dev/cp/cap/test/test_macro_pass.cp):
  - 将流式迭代器 `NumberStream` 中的 `fn next(&mut _): ?i32` 彻底纠正为 CP 正规正统的 `fn next(*_): ?i32`；
- [`cp_ref.md`](file:///data/dev/cp/cp_ref.md):
  - 将 11.2.9 节迭代器流式契约从 `next(&mut _): ?Item` 纠正为 `next(*_): ?Item`；
  - 清理 11.2 节宏示例中的 `set_${field}(*_)` 与 `reset(*_)` 接收者修饰符；
  - 清除全篇遗留的 Rust 风格 `&mut` 痕迹，强化 CP 中 `*_`（可变借用）与 `&_`（只读借用）的正交语义。

---

## 2026-11-02 (完备标准容器矩阵构建：实装 VecDeque、BinaryHeap、BTreeMap、BTreeSet 及完整借用检查)

### 新增文件 (Added)
- [`lib/cap/collections.cp`](file:///data/dev/cp/lib/cap/collections.cp):
  - 正式发布 CP 官方标准容器库 `cap:collections`，统一导出 `VecDeque`、`BinaryHeap`、`BTreeMap`、`BTreeSet`、`HashMap`、`HashSet`、`Map`、`Set`、`Dict`；
- [`cap/test/test_collections_matrix.cp`](file:///data/dev/cp/cap/test/test_collections_matrix.cp):
  - 覆盖双端环形队列、优先队列大顶堆、B 树有序键值映射、B 树有序集合、Rust 对齐别名以及可变/只读借用参数传递的全套 7 项综合测试，100% 绿灯全通。

### 修改文件 (Modified)
- [`cpc/src/parser.rs`](file:///data/dev/cp/cpc/src/parser.rs):
  - `parse_base_type` 支持 `HashMap<K, V>`、`BTreeMap<K, V>` 映射为 `Type::Map`，`HashSet<T>`、`BTreeSet<T>` 映射为 `Type::Set`，并支持任意通用泛型类型参数消费与类型推断；
- [`cpc/src/sema.rs`](file:///data/dev/cp/cpc/src/sema.rs):
  - 注册 `VecDeque`、`BinaryHeap`、`BTreeMap`、`BTreeSet`、`HashMap`、`HashSet` 为顶层预置全局符号；
  - 在 `analyze_expr` 中将 `push_back`、`push_front`、`pop_back`、`pop_front`、`insert`、`remove` 纳入 `loop_borrowed_vars` 循环借用锁定检查，杜绝循环遍历中的迭代器失效；
- [`cpc/src/codegen.rs`](file:///data/dev/cp/cpc/src/codegen.rs):
  - 在 Zig 运行时前导中注入高性能 `VecDeque`（环形缓冲区）、`BinaryHeap`（堆排优先队列）、`BTreeMap`（有序平衡二分表）、`BTreeSet`（有序集合）及 `cp_msg_cmp` 全序比对函数；
  - 增强 `check_mutation_allowed` 与 `check_read_allowed`：全面对齐新容器的写入与只读方法，支持剥离 `BorrowType` 精准推导容器实体类型；
  - 增强 `CpAddReturn` 与 `cp_add`：支持 `Message` 与 `i32` 混合算术运算；
  - 升级 `Stmt::For`：无缝将四类新容器纳入双轨流式迭代路径 `while (it.next()) |item|`；
- [`cp_ref.md`](file:///data/dev/cp/cp_ref.md):
  - 更新 6.3 节容器写保护铁律，纳安全监管矩阵；
  - 更新 12.1 节已实装标准库索引表，增补 `cap:collections`；
  - 新增 12.2 节《完备标准容器矩阵（cap:collections）与借用/所有权安全体系》，详尽规范时间复杂度、内存布局与生产级代码示例。

---

## 2026-11-02 (编译器工具链进化：默认编译指令、--lib 模块模式与 zig {} 原生逃逸舱)

### 修改文件 (Modified)
- [`cpc/src/lexer.rs`](file:///data/dev/cp/cpc/src/lexer.rs):
  - 词法解析器新增 `Token::ZigEscape` 对 `zig` 关键字的支持，与 `__zig__` 双轨并存；
- [`cpc/src/parser.rs`](file:///data/dev/cp/cpc/src/parser.rs):
  - 在 `parse_ident` 中允许 `Token::ZigEscape` 作为上下文标识符，兼顾变量名使用与块级逃逸舱；
- [`cpc/src/codegen.rs`](file:///data/dev/cp/cpc/src/codegen.rs):
  - 新增 `pub is_lib_mode: bool`；在 `--lib` 模式或无 `main()` 函数时，跳过 `pub fn main` 入口与事件循环的生成；
  - 将所有用户定义的 `class`、`enum`、`error set`、`trait`、匿名对象等以 `pub const` 导出，确保外部 Zig 模块（如 Bun.js 原生模块）可以通过 `@import` 无缝直接访问；
- [`cpc/src/main.rs`](file:///data/dev/cp/cpc/src/main.rs):
  - 重构 CLI 解析架构：支持直接 `cpc <INPUT> [-o <OUTPUT>] [--lib]` 执行默认编译动作，废除强制写 `compile` 的冗余心智负担，同时保留 `compile` 作为向后兼容别名；
- [`cp_ref.md`](file:///data/dev/cp/cp_ref.md):
  - 新增 11.6 节《Native Zig 逃逸舱（`zig { ... }`）》；
  - 新增 13.1 节《`cpc` 独立编译器前端》CLI 指令规范。

---

## 2026-11-02 (Bun 迁移工作流指南规范发布：创建 BUN-GEMINI.md 与编译器直通授权)

### 新增文件 (Added)
- [`BUN-GEMINI.md`](file:///data/dev/cp/BUN-GEMINI.md):
  - 针对负责 Bun.js 1.3.14 迁移任务的 AGY 定制的全局指令规范与架构白皮书；
  - 开放 `/data/dev/cp` 编译器上游源码的读取与修改权限，制定遇编译问题直接修改 `cpc` 源码并热更新的自愈工作流；
  - 明确软链接路径映射（`/root/.cap/bin/cpc -> /data/dev/cp/cap/bin/cpc`），将可执行文件覆盖路径规范为 `/data/dev/cp/cap/bin/cpc`；
  - 明确单系统构建无需交叉编译，确立“日常高频迁移首选 Debug 模式（1~2秒快速热更新且带调用栈）、阶段性收尾可选 Release”的高效提速工作流；
  - 提供完整 Zig → CP 语法与内存安全映射字典、`zig { ... }` 原生逃逸舱范式及四阶段迁移路线图。

---

## 2026-11-03 (四层类型转换系统实现：无损隐式流转、显式 as 截断、裸指针 unsafe 隔离与高阶方法解析)

### 修改文件 (Modified)
- [`cpc/src/sema.rs`](file:///data/dev/cp/cpc/src/sema.rs):
  - 在 `SemaError` 中新增 `InvalidCast(String, Span)` 错误变体；
  - 新增 `can_coerce`、`int_fits_in_type`、`is_safe_widening`、`is_subclass`、`type_repr`、`is_numeric_type` 等静态类型流转与判定方法；
  - 落地第一层无损自然流转（Zero-Cost Coercion）：
    - 字面量自适应推导（Literal Fitting）：字面量数值在目标类型范围内自动适配（如 `let a: u64 = 100;`），超出范围静态拦截；
    - 数值无损拓宽（Safe Widening）：无符号小转大（`u8 -> u16 -> u32 -> u64 -> usize`）、有符号小转大（`i8 -> i16 -> i32 -> i64 -> isize`）、小整数/浮点转大浮点（`i32 -> f64`, `f32 -> f64`）自动流转，严禁要求 `as`；
    - 安全结构向上包装：具体值自动包装进可选类型 `T -> T?`、错误联合 `T -> ?T`；
    - 借用权限收紧：独占可变借用 `*T` 自动流转为只读借用 `&T`；
    - 面向对象与 Trait 多态：子类自动流转为基类，实现 Trait 的结构体借用自动流转为 `&dyn Trait`；
    - 符号跨作用域同名匹配：消除 `Ident.span` 差异对类型名相等性判定的干扰；
  - 落地第三层裸指针安全隔离：
    - `Expr::Cast` 全面检查源类型与目标类型，若任一方为裸指针且不在 `unsafe` 作用域内，静态报错 `Raw pointer casting is only allowed inside unsafe block`；
  - 落地第四层高阶语义转换约束：
    - 静态拦截 `string` 与数字之间的 `as` 强转，输出友好引导信息（提示使用 `.parse(...) ?` 或 `.to_string()`）；
  - 在变量声明 `Stmt::Var` 与赋值表达式 `BinaryOp::Assign` 中深度接入 `can_coerce` 静态校验。
- [`cpc/src/codegen.rs`](file:///data/dev/cp/cpc/src/codegen.rs):
  - 在 Zig 运行时前导代码中注入 `cp_cast_int(comptime Dest: type, val: anytype) Dest` 辅助函数，完美解决 Zig `@intCast` 运行期溢出 panic 及跨符号位 `@truncate` 限制，实现底层零开销位截断与符号位重新解释；
  - 在 `generate_expr` 的 `Expr::Cast` 中：
    - 目标为整数时：浮点源使用 `@intFromFloat`，裸指针源使用 `@intFromPtr`，枚举源使用 `@intFromEnum`，其余整数使用 `cp_cast_int`；
    - 目标为浮点数时：整数源使用 `@floatFromInt`，浮点源使用 `@floatCast`；
    - 目标为裸指针时：整数源使用 `@ptrFromInt(@as(usize, @truncate(...)))`，指针源使用 `@ptrCast(@alignCast(...))`；
    - 目标为枚举时：使用 `@enumFromInt(...)`。
- [`cpc/src/ast.rs`](file:///data/dev/cp/cpc/src/ast.rs), [`cpc/src/ast_visitor.rs`](file:///data/dev/cp/cpc/src/ast_visitor.rs), [`cpc/src/dcp.rs`](file:///data/dev/cp/cpc/src/dcp.rs), [`cpc/src/lexer.rs`](file:///data/dev/cp/cpc/src/lexer.rs), [`cpc/src/main.rs`](file:///data/dev/cp/cpc/src/main.rs), [`cpc/src/parser.rs`](file:///data/dev/cp/cpc/src/parser.rs):
  - 运行 `cargo fmt` 保持代码风格完全一致。

### 新增文件 (Added)
- [`cap/test/test_type_conversion.cp`](file:///data/dev/cp/cap/test/test_type_conversion.cp):
  - 四层类型转换系统长期回归测试套件，全面覆盖字面量适配、无损拓宽、可选包装、有损截断、符号位反转、浮点整型互转、枚举互转与裸指针在 unsafe 内外的行为验证。

---

## 2026-11-03 (Bun 迁移赋能：循环控制 break/continue、定长数组重复初始化与类型流转增强)

### 修改文件 (Modified)
- [`cp_ref.md`](file:///data/dev/cp/cp_ref.md):
  - 新增 9.3.1 节《循环控制与跳转语句 (`while`, `break`, `continue`)》；
  - 在第 8 章与第 9 章规范中新增定长数组类型 `[T; N]` / `[N]T` 与重复初始化语法 `[val; count]`（如 `let zero_buf: [u8; 256] = [0; 256];`）。
- [`cpc/src/ast.rs`](file:///data/dev/cp/cpc/src/ast.rs):
  - `Stmt` 枚举中新增 `Break` 与 `Continue` 语句变体；
  - `Expr` 枚举中新增 `ArrayRepeat(Box<Expr>, Box<Expr>)` 表达式；
  - 适配管道操作符占位符检测与代换。
- [`cpc/src/ast_visitor.rs`](file:///data/dev/cp/cpc/src/ast_visitor.rs):
  - 在变量引用遍历中适配 `Stmt::Break`、`Stmt::Continue` 与 `Expr::ArrayRepeat`。
- [`cpc/src/lexer.rs`](file:///data/dev/cp/cpc/src/lexer.rs):
  - 词法解析器注册 `break` (`Token::Break`) 与 `continue` (`Token::Continue`) 关键字。
- [`cpc/src/parser.rs`](file:///data/dev/cp/cpc/src/parser.rs):
  - 支持解析 `break;` 与 `continue;` 语句；
  - 支持解析定长数组类型声明 `[T; N]`；
  - 支持在方括号中解析重复初始化表达式 `[val; count]`。
- [`cpc/src/sema.rs`](file:///data/dev/cp/cpc/src/sema.rs):
  - 在 AST 代换、语义分析及类型推导中全面支持 `Stmt::Break`、`Stmt::Continue` 与 `Expr::ArrayRepeat`；
  - `Expr::ArrayRepeat` 推导为 `Type::FixedArray`（count 为整型字面量时）或 `Type::Array`；
  - 在 `can_coerce` 中增强对定长数组 `Type::FixedArray` 及动态数组 `Type::Array` 的递归推导支持（元素字面量自适应与重复初始化支持）。
- [`cpc/src/codegen.rs`](file:///data/dev/cp/cpc/src/codegen.rs):
  - 增强 `cp_cast_int`：完善 `comptime_int` 负数向无符号类型的底层位解释截断；
  - 为 `Stmt::Break` 生成 `break;`，为 `Stmt::Continue` 生成 `continue;`；
  - 新增 `expected_array_elem_type` 上下文推导，在 `Expr::ArrayRepeat` 中生成与 Zig 0.17 完美兼容的高性能定长广播 `@as([{count}]{elem_ty}, @splat(@as({elem_ty}, {val})))`。

### 新增文件 (Added)
- [`cap/test/test_bun_features.cp`](file:///data/dev/cp/cap/test/test_bun_features.cp):
  - 验证 `while` 循环中的 `break` / `continue` 跳转控制，以及定长数组 `[val; count]` 重复初始化的编译与运行。

---

## 2026-11-03 (编译器字段推导、空值合并字面量收敛与错误联合 Catch 增强)

### 修改文件 (Modified)
- [`cpc/src/sema.rs`](file:///data/dev/cp/cpc/src/sema.rs):
  - 增强 `can_coerce` 字面量按需收敛（Layer 1.1）：支持在 `Expr::NullCoalesce`（`??`）与 `Expr::QuestionFallback` 表达式中递归对 fallback 分支进行字面量收敛推导；
  - 修正 `infer_type` 对 `Expr::Ident` 中接收者标识符 `_` 与 `this` 的处理，使其优先通过符号表作用域解析出具体类类型 `Type::Ref`；
  - 增强 `Expr::Member` 结构体/类成员类型推断：通过遍历类字段与其继承链，精准获取访问字段的声明类型；同时对字符串与数组的 `.len` / `.length` 推导为 `Type::USize`。
- [`cpc/src/codegen.rs`](file:///data/dev/cp/cpc/src/codegen.rs):
  - 修正 `Expr::Catch` 在 `infer_cp_type` 中的解包逻辑：当被捕获表达式返回 `Type::ErrorUnion(t)` 时，正确解包为载荷类型 `*t`，杜绝变量局部推断类型残留 `anyerror!T`；
  - 增强 `Expr::Catch` 的 Zig 代码生成：识别原生 `anyerror!T` 错误联合与 `T ! E` 结果类型，生成 Zig 原生 `({exp}) catch |err| { ... }` 语法，并为 fallback 表达式注入 `@as(TargetType, val)` 显式类型收敛保护。
- [`cp_ref.md`](file:///data/dev/cp/cp_ref.md):
  - 标量类型表补充 `isize` 说明；
  - 完善第 5.3 节与 5.4 节：明确类方法中只读接收者 `_` 的缺省规则、方法体内直接访问属性/兄弟方法的无前缀语法规则（命名遮蔽时使用 `_.` 消除歧义）、以及 `!T`（极简推导错误联合，缺省 `Error`）与 `T ! ErrorSet`（强类型显式错误联合）的双轨制规范。

### 新增文件 (Added)
- [`cap/test/test_slice_calc.cp`](file:///data/dev/cp/cap/test/test_slice_calc.cp):
  - 针对 `calcSizeForSlice` 场景的长期测试用例，覆盖实例接收者省略（默认只读借用）、类方法内成员直接无前缀访问、极简 `!usize` 错误联合签名、字面量按需收敛 `pad_char ?? 0`、默认可变 `let result`（无需 mut）、可选类型安全判空 `!= null` 以及错误捕获 `catch 0`。

---

## 2026-11-03 (Bun AGY 极简 `err` 关键字、128 位整型支持与代码生成零成本抽象树摇优化)

### 修改文件 (Modified)
- [`cpc/src/lexer.rs`](file:///data/dev/cp/cpc/src/lexer.rs):
  - 词法解析器注册 `err` 关键字（作为 `Token::ErrorKw` 别名），实现 3 字母极简大小写镜像对称哲学（`err` vs `Err` vs `errdefer`）；
  - 注册 `i128` 与 `u128` 词法标记；
  - 优化 16 进制、2 进制、8 进制以及大整数字面量解析，防止 64 位整型边界溢出。
- [`cpc/src/ast.rs`](file:///data/dev/cp/cpc/src/ast.rs) & [`cpc/src/dcp.rs`](file:///data/dev/cp/cpc/src/dcp.rs):
  - 在基础类型枚举 `Type` 中新增 `I128` 与 `U128` 标量变体，适配分布式哈希与超高精度中间计算。
- [`cpc/src/parser.rs`](file:///data/dev/cp/cpc/src/parser.rs):
  - 错误集声明与返回联合支持 `err` 关键字（如 `err SetName { ... }`、`return err.Tag;`、`Type ! err`）；
  - 基础类型解析支持 `i128` 与 `u128`；
  - 模式匹配支持 `else =>` 作为后备通配分支（与 `* =>` 等价）。
- [`cpc/src/sema.rs`](file:///data/dev/cp/cpc/src/sema.rs):
  - 语义分析与类型检查适配 `i128` / `u128`；
  - 增强 `can_coerce` 与字面量收敛适配超宽整型；
  - 支持定长数组与动态数组字面量自适应（`FixedArray`/`Array` 与 `Tuple` 解构互转推导）。
- [`cpc/src/codegen.rs`](file:///data/dev/cp/cpc/src/codegen.rs):
  - 实装模块级精细化摇树按需注入（`ModuleFeatures`）：纯计算模块（如 `wyhash`）彻底消除死代码（生成体积由 2400 行剧降至 250 行）；
  - 修正入口函数 `main` 与调度器按需注入逻辑：当存在 `main` 时确保调度内核与 `PID`/`schedule`/`cp_fmt` 正常发射；
  - 标量与无析构类型 Drop Guard 彻底豁免；
  - 修复整数字面量生成，保持负数正常十进制输出；
  - 原生 Zig `switch` 模式平铺发射。
- [`cp_ref.md`](file:///data/dev/cp/cp_ref.md):
  - 标量类型表增加 `i128` 与 `u128` 规格定义；
  - 新增 8.2.1 节阐述 3 字母极简哲学（`err` vs `Err` vs `errdefer`）；
  - 新增 11.7 节《零成本转译抽象：按需摇树注入与原生发射》，详解摇树机制、Drop Guard 豁免与紧凑 Switch 生成。

### 新增文件 (Added)
- [`cap/test/test_err_features.cp`](file:///data/dev/cp/cap/test/test_err_features.cp):
  - 长期回归测试用例，覆盖 `err` 关键字声明与返回、`!T` 推导错误联合、`128` 位整型运算、以及 `switch ... else =>` 语法，经 `./cap/bin/cap` 编译执行 100% 成功。

---

## 2026-11-04 (Bun 迁移深水区支持：转义标识符、纯枚举定长整型标签、方法/嵌入逃逸舱与多模式 Switch 匹配)

### 修改文件 (Modified)
- [`cpc/src/lexer.rs`](file:///data/dev/cp/cpc/src/lexer.rs):
  - 注册 `Token::Ident` 转义标识符正则 `@"..."`，无缝支持网络协议、HTTP 标头与 Web 标准等带连字符名称（如 `@"same-origin"`、`@"no-cors"`）。
- [`cpc/src/ast.rs`](file:///data/dev/cp/cpc/src/ast.rs):
  - `EnumDecl` 增添 `methods: Vec<Function>`（实例方法）、`tag_type: Option<Type>`（底层定长整型类型）与 `extra_zig: Vec<String>`（内嵌原生 Zig 逃逸舱）；
  - `Field` 增添 `default: Option<Expr>`，支持字段默认初值赋值。
- [`cpc/src/parser.rs`](file:///data/dev/cp/cpc/src/parser.rs):
  - 实现通用 `parse_raw_zig_block`，支持在枚举体内直接内嵌 `zig { ... }` 原生逃逸舱注入编译期常量映射；
  - 支持无载荷纯枚举定义定长整型底层标签（如 `enum FetchRequestMode: u2 { ... }`）；
  - 支持在枚举体内定义实例方法与静态方法；
  - 模式匹配全面支持在 `in_switch_arm` 中以逗号 `,` 分隔多模式合并（如 `case .@"same-origin", .@"no-cors" =>`）；
  - 彻底修正 `Token::Bang` 与 `Token::Tilde` 一元运算符优先级绑定，调用 `parse_postfix_expr` 优先绑定后置方法与属性访问。
- [`cpc/src/sema.rs`](file:///data/dev/cp/cpc/src/sema.rs):
  - 适配并深度分析枚举内实例方法与静态方法，建立独立局部作用域并注入 `this`、`_`、`self` 接收者；
  - 修复匿名结构体推导中构造 `ast::Field` 时遗漏的 `default: None`。
- [`cpc/src/codegen.rs`](file:///data/dev/cp/cpc/src/codegen.rs):
  - 纯枚举直接发射为 Zig 原生 `pub const Name = enum(Tag) { ... };`；
  - 纯枚举变体带点访问（`Expr::Member` 与 `Expr::EnumInit`）统一生成 `Name.Variant`，杜绝结构体初始化语法报错；
  - 修正枚举实例方法生成逻辑，以值拷贝而非指针作为接收者（`self: EnumName`），并跳过形参列表中重复的 `self`；
  - 修复摇树中 `scan_expr` 遗漏 `Expr::Array` 导致 `cp_array_lit` 未定义的问题；
  - 智能识别纯枚举 Switch 穷尽匹配（`is_exhaustive_enum`），全覆盖时自动剔除不可达通配分支并禁止发射 `else => unreachable`；
  - 修复匿名结构体推导中构造 `ast::Field` 时遗漏的 `default: None`。
- [`cp_ref.md`](file:///data/dev/cp/cp_ref.md):
  - 新增第 3.5.4 节《特殊符号变体、底层标签与嵌套逃逸舱》，规范转义标识符 `@"..."`、定长整型底层标签（`: u2`）、枚举内嵌 `zig { ... }` 块与实例方法规范；
  - 完善第 4.3 节关于联合多模式匹配（支持 `|` 与 `,` 逗号合并）说明。

### 新增文件 (Added)
- [`cap/test/test_bun_enums.cp`](file:///data/dev/cp/cap/test/test_bun_enums.cp):
  - 涵盖转义标识符 `@"same-origin"`、定长整型底层标签 `: u2`、枚举内实例方法调用、枚举内嵌 `zig { ... }` 逃逸舱函数、逗号多模式合并匹配以及穷尽 Switch 校验的长期回归测试用例，经 `./cap/bin/cap` 编译执行 100% 成功。
- [`cap/test/test_root_inline.cp`](file:///data/dev/cp/cap/test/test_root_inline.cp):
  - 验证 `@inline` 实例方法调用与数组字面量格式化输出的长期回归测试用例，经 `./cap/bin/cap` 编译执行 100% 成功。

















---

## 2026-11-04 (原生逃逸舱类型值传递与函数未使用参数精准抑制)

### 修改文件 (Modified)
- [`cpc/src/codegen.rs`](file:///data/dev/cp/cpc/src/codegen.rs):
  - 顶层 `zig { ... }` 块符号扫描时，将所有声明的类型与别名自动加入 `ref_classes` 与 `value_classes` 白名单，确保跨模块引用时按原生值类型传递而非强加堆指针 `*T`；
  - 新增 `block_contains_ident` 辅助函数，递归扫描函数体内所有内嵌 `zig { ... }` 块是否包含指定形参标识符；
  - 重构函数形参未使用抑制逻辑：已引用的形参不生成任何抑制代码，真正未使用的形参生成符合 Zig 规范的 `_ = param;`（取代无条件取地址 `_ = &param;`）。
- [`cp_ref.md`](file:///data/dev/cp/cp_ref.md):
  - 新增第 13.2 节《原生逃逸舱类型映射与参数引用优化》，详细阐明外部原生类型直接发射与参数引用优化的工程规范。

### 新增文件 (Added)
- [`cap/test/test_escape_param_opt.cp`](file:///data/dev/cp/cap/test/test_escape_param_opt.cp):
  - 长期回归测试用例，验证逃逸舱原生结构体纯值传递、内嵌原生块中形参引用感知以及未使用参数抑制，经 `./cap/bin/cap` 编译执行 100% 成功。


---

## 2026-11-04 (结构体内嵌逃逸舱、多级命名空间类型、后缀推导错误联合与错误成员访问)

### 修改文件 (Modified)
- [`cpc/src/ast.rs`](file:///data/dev/cp/cpc/src/ast.rs):
  - `Class` 结构体新增 `extra_zig: Vec<String>`，用于存储类或结构体内部内嵌的 `zig { ... }` 原生逃逸舱源码片段。
- [`cpc/src/parser.rs`](file:///data/dev/cp/cpc/src/parser.rs):
  - 在 `parse_class` 中支持解析类与结构体作用域内的 `zig { ... }` 块并存入 `extra_zig`；
  - 在 `parse_base_type` 中支持后缀 `T!` 语法：当后置 `!` 之后无特定错误标识符或 `err` 关键字时，将类型解析为推导错误联合 `Type::ErrorUnion(Box::new(base))`。
- [`cpc/src/sema.rs`](file:///data/dev/cp/cpc/src/sema.rs):
  - 为所有内置 Mock 类（`RegExp`, `Set`, `Map`, `Dict`, `Error`, `Box` 等）补全 `extra_zig: vec![]` 初始化字段。
- [`cpc/src/codegen.rs`](file:///data/dev/cp/cpc/src/codegen.rs):
  - 扫描所有类的 `extra_zig` 声明，将 `pub const T = ...` 或 `const T = ...` 常量名称自动加入 `ref_classes` 与 `value_classes`；
  - 在结构体定义体内原样发射内嵌的 `class.extra_zig` 原生代码段；
  - `Expr::EnumInit`: 支持匿名空结构体 `.{{}}` 与字段紧凑结构体 `.{{ .field = val }}`；
  - `Expr::Member`: 支持 `err.Variant` 与 `error.Variant` 直接发射为 Zig 原生 `error.Variant`；
  - 静态对象字面量：空对象 `{}` 直接发射为 Zig 原生紧凑结构体 `.{}`；
  - `map_type`: 多级点路径类型（如 `jsc.ZigString.Slice`、`std.mem.Allocator`）按值类型直接发射完整路径，不添加 `*` 堆指针前缀。
- [`cp_ref.md`](file:///data/dev/cp/cp_ref.md):
  - 完善所有权与借用符号矩阵；
  - 新增第 13.3 节《结构体扩展逃逸舱、多级命名空间类型与标准 TS 静态结构体字面量》，详尽规范内嵌 `zig` 块、`{}` 结构体字面量、多级点路径类型与后缀 `T!` 体系。

### 新增文件 (Added)
- [`cap/test/test_class_extra_zig.cp`](file:///data/dev/cp/cap/test/test_class_extra_zig.cp):
  - 长期回归测试用例，覆盖结构体内嵌原生 `zig` 块、后缀 `T!` 推导错误联合、`err.Variant` 错误成员访问以及静态空结构体 `{}` 与匿名结构体 `{ count: 123 }`，经 `./cap/bin/cap` 编译执行 100% 成功。


---

## 2026-11-04 (海象操作符条件解包、函数缺省返回类型简写与前缀可选兼容优化)

### 修改文件 (Modified)
- [`cpc/src/ast.rs`](file:///data/dev/cp/cpc/src/ast.rs):
  - `IfStmt` 结构体新增 `capture: Option<Ident>` 字段，用于支持 `if (binding := expr)` 的条件绑定标识符。
- [`cpc/src/parser.rs`](file:///data/dev/cp/cpc/src/parser.rs):
  - 在 `parse_if_stmt` 与 `parse_while_stmt` 中全面支持海象操作符语法 `(binding := expr)`，同时对旧版 Zig 管道捕获 `|capture|` 输出清晰的迁移指引报错；
  - 函数、箭头函数与方法返回类型解析中，支持 `!` 与 `!Error` 简写形式（自动将 ok 类型补全为 `Type::Void`）；
  - 保留前缀 `?T` 向下兼容解析，杜绝历史测试用例与标准库代码被破坏性中断。
- [`cpc/src/sema.rs`](file:///data/dev/cp/cpc/src/sema.rs):
  - 在 `Stmt::If` 的语义分析中，若存在 `capture` 则自动创建局部子作用域并登记不可变绑定符号，支持后续表达式直接引用。
- [`cpc/src/codegen.rs`](file:///data/dev/cp/cpc/src/codegen.rs):
  - 在 `Stmt::If` 与 `Stmt::While` 的代码生成中，将海象操作符转译为 Zig 原生的 `if (expr) |binding|` 与 `while (expr) |binding|`；
  - 自动注入 `_ = &binding;` 抑制语句，避免 Zig 编译器在捕获未被全部分支引用时抛出 `unused capture` 错误；
  - 在 `type_to_display_string` 中将 `Type::ErrorUnion` 统一输出为现代后缀 `T!` 格式。
- [`cp_ref.md`](file:///data/dev/cp/cp_ref.md):
  - 新增第 9.3.2 节与第 9.3.3 节，详尽规范 `while (binding := expr)` 与 `if (binding := expr)` 的类型收窄、作用域隔离与零成本转译机制。

### 新增文件 (Added)
- [`cap/test/test_walrus_binding.cp`](file:///data/dev/cp/cap/test/test_walrus_binding.cp):
  - 长期回归测试用例，全量验证 `if (val := find())` 成功与 else 分支、`while (item := iter.next())` 循环解包、以及 `fn foo() !` 错误联合简写，经 `./cap/bin/cap` 编译执行 100% 成功。


---

## 2026-11-04 (全面废弃前缀 ?T 统一收敛为后缀 T?、匿名错误集前瞻消歧与测试库代码全量升级)

### 修改文件 (Modified)
- [`cpc/src/parser.rs`](file:///data/dev/cp/cpc/src/parser.rs):
  - 在 `parse_base_type` 中对前缀 `?` 实施编译期硬报错，强行拦截 `?T` 并给出迁移至 `T?` 或 `T?!` 的指导；
  - 新增 `is_error_type_rhs_start` 前瞻探测方法，精准判定匿名错误集 `!{ ... }`，消除函数体/代码块 `{` 与匿名错误集之间的语法冲突；
  - 恢复 `try/catch` 表达式中错误变量标识符的准确解析。
- [`cpc/src/codegen.rs`](file:///data/dev/cp/cpc/src/codegen.rs):
  - 引入 `cp_match_tag` 宏，支持 Zig 纯枚举 (`enum`) 与标签联合 (`union(enum)`) 在 `switch` 匹配中的无缝兼容；
  - 确保所有类/结构体始终发射 `pub fn _create(allocator)` 实例构造函数；
  - 统一 `Type::ResultType` 搭配 `Error` 时的转译逻辑为 `CpResult(ok, Error)`；
  - 在 `Expr::Builtin` 中正式支持 `@expect_eq` 与 `@test` 内建断言与运行宏。
- [`lib/cap/sqlite.cp`](file:///data/dev/cp/lib/cap/sqlite.cp):
  - 全量升级函数返回类型与字段类型为后缀 `T?` / `T?!` / `void!`。
- [`lib/cap/test.cp`](file:///data/dev/cp/lib/cap/test.cp):
  - 将 `test` 与 `expect_eq` 参数类型升级为泛型 `auto`，映射为 Zig `anytype`。
- [`cp_ref.md`](file:///data/dev/cp/cp_ref.md):
  - 清理并统一所有示例与规范表述中的可选类型，全面采用后缀 `T?` / `T?!`。
- `cap/test/` (25+ 测试文件全量更新):
  - 包含 `hello.cp`, `main.cp`, `test_actor.cp`, `test_advanced.cp`, `test_all.cp`, `test_async_tuple.cp`, `test_builtins.cp`, `test_comptime.cp`, `test_dependency.cp`, `test_dist.cp`, `test_error_box_switch_pass.cp`, `test_error_fail_non_exhaustive_switch.cp`, `test_error_fail_null_coalesce_on_err.cp`, `test_errors.cp`, `test_features.cp`, `test_fs_async.cp`, `test_macro_pass.cp`, `test_map_set_dict.cp`, `test_matmul.cp`, `test_object.cp`, `test_spawn_arrow.cp`, `test_sqlite.cp`, `test_stdlib.cp`, `test_switch_binding_pass.cp`, `test_tensor.cp`, `test_traits_enums.cp`, `test_undefined.cp`, `test_zero_copy.cp`, `example_showcase.cp`。

---

## 2026-11-05 (评估并整合 Bun AGY 编译器升级、修复未变动变量判定 Bug、支持 CPC_BIN 环境变量路由与 err/move 语法双轨完善)

### 修改文件 (Modified)
- [`cpc/src/codegen.rs`](file:///data/dev/cp/cpc/src/codegen.rs):
  - **关键修复**：回滚了导致不可变 `let` 变量在 Zig 中触发 `error: local variable is never mutated` 的误判逻辑，恢复对非状态机变量始终发射 `_ = &ident;` 取地址标记，彻底修复 `example_showcase.cp` 等测试用例编译失败问题；
  - 接入 `is_lib_mode`，优化库编译模式下 Zig 代码生成精简度；
  - 支持 `cstring` 映射为 Zig `[:0]const u8`（指针为 `[:0]u8`）；
  - 支持未注册 `@builtin` 宏透明透传至底层 Zig/C 函数调用。
- [`cpc/src/parser.rs`](file:///data/dev/cp/cpc/src/parser.rs):
  - **双轨错误值构造器与匹配**：在 `parse_primary_expr` 与 `parse_pattern` 中支持小写 `err(...)`，同时 100% 保持对大写 `Err(...)` 兼容；
  - **灵活 move 参数位置**：在 `parse_params` 中同时支持 `fn f(move u: User)`（标识符前置）与 `fn f(u: move User)`（类型位置前置）；
  - 支持声明前置 `export`（如 `export fn`、`export class`）；
  - 支持 `[N]T` 固定长度数组类型语法。
- [`cpc/src/sema.rs`](file:///data/dev/cp/cpc/src/sema.rs):
  - 注册 `ident.sym == "err"` 为内建符号标识符；
  - 对 `@packed` 与 `@extern` 结构体免除成员逐项 `init` 构造器生成，保证二进制紧凑布局与 C ABI 对齐；
  - 增加编译期整型常量折叠计算 `eval_const_int_expr`。
- [`cap/src/main.zig`](file:///data/dev/cp/cap/src/main.zig):
  - 增加 `getCpcBin()` 辅助函数，优先读取环境变量 `CPC_BIN`，未配置时安全回退至默认正式编译器路径 `/data/dev/cp/cpc/target/release/cpc`，支持多 Agent 并行开发；
  - 替换全部 5 处死绑定的编译器硬编码路径。
- [`cp_ref.md`](file:///data/dev/cp/cp_ref.md):
  - 补充 `cstring` 标量类型规范（对接 C/Bun 原生库，映射为 Zig `[:0]const u8`）；
  - 补充 `[N]T` 定长数组语法；
  - 补充 `export fn` 符号导出修饰符说明；
  - 补充 `@packed` 与 `@extern` 内存布局元属性；
  - 同步更新第 6 节（参数所有权转移语法）与第 8.2.1 节（`err(...)` 主推与 `Err(...)` 兼容）。
- [`.gitignore`](file:///data/dev/cp/.gitignore):
  - 将 `cpc_dev/` 目录加入版本忽略，防止并发沙盒本地改动污染主仓库。
- [`GEMINI.md`](file:///data/dev/cp/GEMINI.md):
  - 用户更新：明确 rm 指令针对 temp/test 目录及其文件的免申请操作权限。

### 新增文件 (Added)
- [`cap/test/test_err_and_move.cp`](file:///data/dev/cp/cap/test/test_err_and_move.cp):
  - 包含 `fn f(move u: User)`、`fn f(u: move User)`、`err(...)` 错误构造与模式匹配、大写 `Err(...)` 兼容性的全景回归测试，经 `./cap/bin/cap` 执行 100% 成功。

