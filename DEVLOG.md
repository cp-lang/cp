# CP 研发日志 (DEVLOG) - 2026-11-12 (热数据)

> 历史研发日志已完整归档至 [DEVLOG_H.md](file:///data/dev/cp/DEVLOG_H.md)。本项目采用冷热分离架构，本文件仅保留当日活跃记录。

## 2026-11-12: 落地编译器核心代码生成器 8 大领域子模块重构与 Trait 作用域强化

- **任务背景与工程动因**：
  - 编译器代码生成器 `codegen.rs` 代码量膨胀至 17,081 行，单体巨石文件导致工具往返读取 Token 消耗巨大，并阻碍多智能体并行开发与细粒度增量维护；
  - 依据 GEMINI.md 架构规划，全面将 `codegen.rs` 按职责拆解为 8 个子模块，单文件控制在 1,000 ~ 4,000 行。

- **模块拆解拓扑与方法映射设计**：
  1. `cpc/src/codegen/mod.rs`（3,398 行）：
     - 宿主 `Codegen` 结构体，将原私有字段开放为 `pub(crate)` 供子模块安全访问；
     - `generate_module` 顶层总入口与顶级 AST 项生成（`generate_trait`, `generate_impl`, `generate_enum`, `generate_interface`, `generate_class`, `generate_reflection_tables`）；
     - 文本发射底层辅助方法（`indent`, `dedent`, `writeln`, `write_to_case`）与独立字符处理函数（`strip_redundant_comptime`, `lower_struct_body`）。
  2. `cpc/src/codegen/prelude.rs`（614 行）：
     - `ModuleFeatures` 结构体及 `scan`、`scan_decl`、`scan_stmt`、`scan_expr` 等全套语法特性静态扫描器。
  3. `cpc/src/codegen/types.rs`（2,601 行）：
     - 类型转换与映射核心：`map_type`、`map_type_name`、`infer_zig_type`、`infer_cp_type`、`infer_cp_type_with_mock`；
     - 泛型单态化参数提取：`split_generic_args_str`、`map_generic_args_str`；
     - 错误类型映射与兼容校验：`map_error_type`、`verify_error_type_compatibility`；
     - 方法与字段查找：`lookup_method`、`is_class_field`。
  4. `cpc/src/codegen/borrow.rs`（960 行）：
     - 生命周期与活跃度分析：`compute_last_uses_block`、`compute_last_uses_stmt`；
     - 借用检查与冲突拦截：`check_active_borrows_before_move`、`check_borrow_conflict`、`check_mutation_allowed`、`check_read_allowed`、`extract_expr_borrow_info`。
  5. `cpc/src/codegen/pattern.rs`（1,587 行）：
     - 模式匹配状态机：`generate_switch`、`generate_pattern_match`、`format_simple_pat`；
     - 分支穷尽性校验：`check_switch_exhaustiveness`；
     - 转义与字符串辅助：`escape_zig_string`、`escape_zig_fmt_string`。
  6. `cpc/src/codegen/actor.rs`（1,577 行）：
     - 函数定义与逻辑生成：`generate_function`、`generate_function_logic`、`generate_function_decorated`；
     - 异步与 Actor CPS 状态机切分：`emit_current_case`、`next_state`、`extract_locals_simple`、`extract_local_vars_to_str`。
  7. `cpc/src/codegen/stmt.rs`（2,114 行）：
     - 控制流与语句派发：`generate_stmt`（变量声明、赋值、`if`、`while`、`for`、`return`、`defer`、`errdefer`、`unsafe` 块）。
  8. `cpc/src/codegen/expr.rs`（4,332 行）：
     - 表达式生成：`generate_expr`（字面量、操作符、函数调用、构造、管道、HTTP 路由生成）。

- **关键技术发现与边界缺陷修复**：
  - **Trait 虚函数表与同名 Class 作用域冲突**：
    - `test_c_ffi_pointers.cp` 定义了 `class VTable { on_data: ... }`；
    - 当编译包含 Trait 时，Zig 代码生成为 `pub const Reader = struct { ptr: *anyopaque, vtable: *const VTable, pub const VTable = struct { ... } };`；
    - Zig 编译器在解析 `vtable: *const VTable` 时因同名导致 `ambiguous reference` 编译崩溃；
    - **修复**：将 Trait 和 Interface 生成中的虚表指针类型升级为 `vtable: *const @This().VTable`，利用 Zig `@This()` 精确限定为当前 Trait 内部的 `VTable`，彻底杜绝外层作用域符号遮蔽。

- **效果验证**：
  - 编译器代码库完成编译与秒级发布，全套回归套件（`test_io_streams`、`test_extern_var`、`test_const_generics`、`test_mutex_atomic`、`test_c_ffi_pointers`、`test_struct_let`、`test_prelude_t_shadowing`）全量 **100% PASS**。

## 2026-11-12: 攻坚标准库方案 A（工业级统一 I/O 流体系）与 extern const 编译器支持

- **任务背景与工程目标**：
  1. 响应下游 Bun AGY 在系统底层与网络迁移中对统一 I/O 抽象的迫切诉求；
  2. 实现 CP 工业级标准库基石方案 A：基于 Trait 的统一零拷贝流式 I/O 体系，对标 Go `io` 与 Rust `std::io`；
  3. 协同解决 Bun AGY 提报的 `task-extern-const-var`：支持在 `@c` 块中声明外部全局常量与变量；
  4. 保持 `cap:io` 现有接口（`print`、`console`、`stdio`）100% 向后兼容。

- **核心技术挑战与根因剖析**：
  1. **丢弃变量 `let _ = ...;` 与 Zig 0.17 命名规范冲突**：
     - 在 CP 中广泛使用 `let _ = writer.write(slice)?;` 显式丢弃不需要的结果；
     - `codegen.rs` 此前将所有 `Pattern::Ident(id)` 一律转译为 `var {id} = ...;`，导致 Zig 报出 `'_' used as an identifier without @"_" syntax`；
     - 此外，在闭包/状态机提取局部变量生成环境上下文结构体时，也将 `_` 包含进去，生成了非法字段定义 `_: ty,`。
  2. **泛型类单态化实例构造缺失 `_create`**：
     - 在 Zig 中，泛型类以函数返回类型的形式表达（`fn (comptime R: type) type`）；
     - `codegen.rs` 此前在为类生成堆分配辅助函数 `_create` 时带有 `if !is_generic` 守卫，导致泛型类实例调用 `new BufferedReader(reader)` 时，Zig 报告 `struct 'BufferedReader(BufferReader)' has no member named '_create'`；
     - 且在非泛型构造中直接使用了类名符号，而单态化类在自身内部只有 `@This()` 能够准确指示当前具体类型。
  3. **泛型实例的方法签名解析回退**：
     - `buffered.readByte() catch 0` 表达式中，`readByte()` 返回 `u8!`，但 `lookup_method` 携带了完整类型名 `BufferedReader<BufferReader>`，由于未剥离泛型后缀 `<...>`，无法匹配到基类定义，导致类型推导回退为 `i32`，引发类型不匹配。

- **实施方案与架构优化**：
  1. **下划线丢弃模式精准转译**：
     - 在 `codegen.rs` 的变量绑定中，对 `Pattern::Ident(id) if id.sym == "_"` 以及 `Pattern::Wildcard` 统一发射 `_ = {expr};`；
     - 在状态机上下文捕获时显式过滤掉 `_` 变量。
  2. **泛型类 `@This()` 统一构造支持**：
     - 移除 `_create` 的 `!is_generic` 限制，使所有类（无论是否泛型）均生成 `pub fn _create(...) *@This()`；
     - 内部内存分配统一采用 `allocator.create(@This())`，使 Zig 单态化后能安全分配结构体内存。
  3. **泛型基类方法索引增强**：
     - 在 `lookup_method` 与 `sema.rs` 的 `Expr::New` 校验中，利用 `ident.split('<').next()` 剥离泛型实参，精确命中泛型类的定义与方法签名。
  4. **全套工业级 I/O 流架构落地**：
     - 实现 `Reader`、`Writer`、`Closer`、`Seeker` 核心契约；
     - 零堆分配只读流 `BufferReader` 与定长栈/堆写入流 `BufferWriter`；
     - 高性能泛型缓冲封装流 `BufferedReader<R: Reader>`、`BufferedWriter<W: Writer>` 与全量管道传输 `copyStream`。

- **效果验证**：
  - `cap cap/test/test_extern_var.cp`：100% 通过（601ms）；
  - `cap cap/test/test_io_streams.cp`：6 大套件全面覆盖（888ms，100% 通过）。
