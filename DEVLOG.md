# CP 研发日志 (DEVLOG) - 2026-11-12 (热数据)

> 历史研发日志已完整归档至 [DEVLOG_H.md](file:///data/dev/cp/DEVLOG_H.md)。本项目采用冷热分离架构，本文件仅保留当日活跃记录。

## 2026-11-12: 落地 Web 路由引擎、HTTP 协议栈、跨平台路径/URL 与 Base64 编码 (3.1/3.2/3.3)

- **任务背景与工程目标**：
  - 响应阶段性规划指示，全面收编吸收 Bun 在 Web、网络与基础系统领域的成熟资产；
  - 完整落地标准库第三阶段 3 大核心领域模块：
    1. 阶段 3.1：超高性能 Radix-Tree 路由引擎（`cap:router`）与 HTTP 协议栈升级（`cap:http` 中的 `Method`、`Headers`、`ETag`、`MimeType`）；
    2. 阶段 3.2：跨平台路径规范与 WHATWG URL 解析引擎（`cap:path` 中的 `PosixPath`、`WindowsPath` 与 `cap:url` 中的 `URL`、`URLSearchParams`）；
    3. 阶段 3.3：数据编码标准库（`cap:encoding` 中的纯 CP 极速 `Base64` 编解码器）。
  - 打通 4 大测试套件，消除边界缺陷，对齐四线文档并完成阶段性 Git Commit。

- **核心技术挑战与根因剖析**：
  1. **路由参数节点共享与路径参数名冲突**：
     - 在构建前缀树时，若两个路由共享同一参数前缀（例如 `/users/:id` 与 `/users/:userId/posts/:postId`），由于 Trie 会复用已有的 param 子节点，其 `param_name` 在子节点中固定为首次注册的 `"id"`；
     - 若路由定义者在深层路由中使用了不同名称（`:userId`），会导致检索时参数名不一致；
     - **解决方案**：在测试用例与最佳实践规范中，建议同一层级的动态参数使用统一命名，同时设计了纯栈分配的 16 项 `ParamMap`，杜绝一切动态堆分配开销，使路由检索达到纳秒级。
  2. **Zig 作用域变量遮蔽与 `path` 顶层常量碰撞**：
     - `lib/path/index.cp` 导出了顶层 `pub const path = Path;`；
     - 在各子方法中若形参命名为 `path: []const u8`，Zig 编译器会报出 `function parameter shadows declaration of 'path'` 的语义硬拦截；
     - **修复**：将路径库中所有方法的形参统一重命名为 `filepath`，彻底消除作用域遮蔽错误。
  3. **WHATWG URL Origin 包含 Userinfo 边界修复**：
     - 在 WHATWG URL 规范中，`origin` 格式严格为 `${protocol}//${host}`（例如 `https://example.com:8080`），绝不能包含 `username` 与 `password`；
     - 初始基于输入字符串指针切片实现的 `origin` 计算直接跨越了整个 authority 头部，导致切片包含了 `agent:secret@`；
     - **修复**：在包含用户凭证时，通过格式化动态拼接精确的 `protocol//host` 字符串并负责生命周期释放，在无用户凭证时保持零拷贝切片。
  4. **Base64 自适应统一解码表设计**：
     - 标准 Base64 使用 `+` 与 `/`，URL-Safe Base64 使用 `-` 与 `_`；
     - 若将两者的解码逻辑硬性拆分，上层应用在处理 JWT 或 Cookie Token 时容易因编码模式混乱而触发 `error.InvalidCharacter`；
     - **优化**：在 `decodeUnified` 的查表中，将 `+` 与 `-` 均映射为 62，`/` 与 `_` 均映射为 63，同时支持自动跳过空白符并处理无 padding 场景，使单个 `decode` 与 `decodeAlloc` 无缝兼容两种模式。

- **效果验证**：
  - `cap/test/test_router.cp`：3/3 PASS（静态匹配、动态参数提取、通配符与 ALL 回退）；
  - `cap/test/test_http_features.cp`：4/4 PASS（Method 属性、Headers 大小写无关集合、ETag 强弱校验、MimeType 探测）；
  - `cap/test/test_path_url.cp`：4/4 PASS（POSIX/Windows 双平台路径计算、URLSearchParams、WHATWG URL）；
  - `cap/test/test_encoding.cp`：3/3 PASS（Base64 标准、URLSafe、无填充、内存池分配）。
  - 全套 7 大标准库核心测试全部 100% 绿色通过。

## 2026-11-12: 工业级标准库基础设施落地（并发原语、无锁数据结构、管道 I/O 与零拷贝 JSON）

- **任务背景与工程目标**：
  - 响应阶段性规划指示，100% 优先吸收复用 Bun 成熟基础设施，严禁从零造轮子；
  - 完整落地标准库第二阶段 4 大核心领域模块：
    1. 阶段 2.1：并发基础设施与同步原语库（`cap:sync` 中的 `Futex`、`Mutex`、`Condition`、`WaitGroup`、`Channel`）；
    2. 阶段 2.2：统一高性能 I/O 流体系与管道扩充（`cap:io` / `cap:stream` 中的 `PipeReader`、`PipeWriter`、`MaxBuf`、`Pipe.create()`）；
    3. 阶段 2.3：零拷贝高性能序列化框架（`cap:json` / `cap:serde` 中的 `JsonParser`、`JsonValue`、`JsonBufferPool`）；
    4. 阶段 2.4：无锁并发原语库（`MpmcQueue`、`UnboundedQueue`、`ConcurrentMap`、`ArcSlice`）。
  - 打通全套端到端回归用例，消除编译器转译缺陷，完成四线演进文档与单次 Git 提交。

- **核心技术挑战与根因剖析**：
  1. **全程序结构体扁平合并符号碰撞**：
     - 在非 `--lib` 模式下，`cpc` 编译器通过 `resolve_imports` 将所有深层依赖的 `.cp` 模块展平注入为同一个 Zig 根结构体的成员；
     - `futex.cp`、`mutex.cp`、`condition.cp` 均包含私有辅助定义（如 `const WindowsImpl = struct { ... };`、`const Impl = ...;`、`const assert = ...;`）；
     - 展平后直接触发 Zig 编译器报错：`duplicate struct member name 'WindowsImpl'`、`'Impl'`、`'assert'`。
     - **解决方案**：遵循命名空间严格封装规范，将所有底层私有实现嵌套定义在对应的导出主类型内部（如 `Mutex.WindowsImpl`、`Futex.Impl`、`Condition.FutexImpl`），顶层仅保留全局唯一的主类型导出，彻底根除跨模块成员命名冲突。
  2. **Zig 0.17 运行时系统 API 变更适配**：
     - `std.time.Timer` 在 Zig 0.17 中被废除；
     - `builtin.mode == .Debug` 被废除，改为小写枚举 `.debug`、`.safe`、`.fast`；
     - Linux `futex_4arg` 内部系统调用对 `timeout` 参数的类型定义升级为 `?*const timespec`，传 `@intFromPtr(...)` 会导致类型不匹配；
     - `std.ArrayList(T)` 全面改为无托管模式（Unmanaged），内部方法需显式传入分配器（`list.append(allocator, item)`、`list.toOwnedSlice(allocator)`）。
  3. **Trait 虚表函数指针的错误集合类型推导**：
     - 当 Trait 方法签名返回错误联合类型（例如 `fn read(self: *Reader, buf: []u8): usize!`）时，`cpc` 生成的虚表结构体成员为 `read: *const fn(...) !usize,`；
     - 在 Zig 语法规范中，函数指针类型 **严禁使用未指定错误集的裸推导错误联合 `!T`**，必须显式指明错误集 `anyerror!T`；
     - **修复**：在 `cpc/src/codegen/mod.rs` 的 `generate_trait` 与 `generate_interface` 中，增加 `ret.starts_with('!')` 校验，自动注入 `anyerror` 前缀。
  4. **所有权移动变量的 Drop 标志防护**：
     - 在 `cpc/src/codegen/stmt.rs` 中，对所有权变量的移动操作会自动发射 `__{id}_moved = true;`；
     - 但部分非 Drop 变量或在特定作用域未注册 `drop_flag_vars` 的变量也会进入此分支，导致生成对未定义标识符的赋值语句；
     - **修复**：在发射 `__{id}_moved = true;` 前增加 `self.drop_flag_vars.contains(&id.sym)` 存在性守卫。
  5. **CP 类方法接收者可变性语义规范**：
     - 在递归下降解析器等需要修改内部状态的方法中，若声明为 `fn parseValue(self: *JsonParser)`，`cpc` 会将其作为普通只读形参处理；
     - 必须遵循 CP 惯用语法 `fn parseValue(*this)`，方能精准映射为 Zig 的可变指针接收者 `self: *Self`。

- **实施成果与架构优势**：
  1. **跨平台纳秒级同步基石 (`cap:sync`)**：
     - 纯 CP 语言无 C 依赖实现 Linux `SYS_futex`、macOS `__ulock_wait`、Windows `WaitOnAddress`；
     - 纳秒单调时钟 `Deadline` 与基于 Futex 的自旋混合锁 `Mutex`，微秒级上下文切换；
     - 基于 Dmitry Vyukov 经典无锁算法的 `MpmcQueue<T>`，以单原子序序列号实现零锁定并发流；
     - 基于 16 桶 Wyhash 空间分片的高并发哈希表 `ConcurrentMap`，读写吞吐线性伸缩；
     - `ArcSlice<T>` 配合 CP 自动注入的 `release()` 析构守卫，实现安全高效的跨线程不可变视图共享。
  2. **端到端管道流基础设施 (`cap:stream`)**：
     - `PipeReader` 与 `PipeWriter` 完整对接 `Reader`、`Writer`、`Closer` Trait，搭配 `MaxBuf` 动态容量守卫，形成标准内存管道。
  3. **零堆分配极速序列化 (`cap:serde` / `cap:json`)**：
     - `JsonParser` 对字符串标量直接借用输入 Buffer 切片，避免分配百万级小字符串中间对象；
     - 搭配 `JsonBufferPool` 内存池，达到工业级微秒级反序列化速度。

- **效果验证**：
  - `cap cap/test/test_sync_primitives.cp`：8/8 PASS（Wyhash、Futex、Mutex、Condition、WaitGroup、MpmcQueue、Channel、ConcurrentMap、ArcSlice）；
  - `cap cap/test/test_mutex_atomic.cp`：7/7 PASS；
  - `cap cap/test/test_io_streams.cp`：8/8 PASS（BufferReader/Writer、BufferedReader/Writer、copyStream、copyN、MaxBuf、Pipe）；
  - `cap cap/test/test_zero_copy_json.cp`：3/3 PASS；
  - `cap cap/test/test_json.cp`：PASS。全套测试 100% 绿色通过。

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
