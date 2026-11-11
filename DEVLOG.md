# CP 研发日志 (DEVLOG) - 2026-11-12 (热数据)

> 历史研发日志已完整归档至 [DEVLOG_H.md](file:///data/dev/cp/DEVLOG_H.md)。本项目采用冷热分离架构，本文件仅保留当日活跃记录。

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
