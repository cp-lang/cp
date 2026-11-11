# CP 变更日志 (CHANGELOG) - 2026-11-12 (热数据)

> 历史变更日志已完整归档至 [CHANGELOG_H.md](file:///data/dev/cp/CHANGELOG_H.md)。本项目采用冷热分离架构，本文件仅保留当日活跃记录。

## [2026-11-12] - 方案 A 工业级统一 I/O 流体系与 C 变量外部绑定支持

### 新特性 (Added)
- [`lib/io.cp`](file:///data/dev/cp/lib/io.cp):
  - 新增统一流错误类型 `IOError`（`EndOfStream`, `UnexpectedEof`, `Closed`, `BufferFull`, `ShortWrite`, `InvalidSeek`）；
  - 新增寻址基准枚举 `SeekOrigin`（`Start`, `Current`, `End`）；
  - 新增统一流 Trait 契约：`Reader`、`Writer`、`Closer`、`Seeker`；
  - 新增零拷贝内存切片只读流 `BufferReader`，支持无堆分配读取与 Seek；
  - 新增内存定长安全写入器 `BufferWriter`（4096 字节），支持字节提取与重置；
  - 新增高性能泛型缓冲读取器 `BufferedReader<R: Reader>`（256 字节内部缓存，支持 `readByte()` 单字节速读）；
  - 新增高性能泛型缓冲写入器 `BufferedWriter<W: Writer>`（256 字节内部缓存，支持自动/手动 `flush()`）；
  - 新增流式全量传输管道 `copyStream<R: Reader, W: Writer>` 与定额流搬运 `copyN`；
  - 100% 保持已有 `Console`、`console`、`Stdio`、`stdio`、`print` 向后兼容。
- [`cap/test/test_io_streams.cp`](file:///data/dev/cp/cap/test/test_io_streams.cp):
  - 包含 6 大端到端用例套件：`BufferReader` 切片读取与 Seek 验证、`BufferWriter` 写入与字节校验、`BufferedReader` 缓冲与单字节速读、`BufferedWriter` 批量写入与 flush、`copyStream` 管道全量传输、`copyN` 定额截断传输。
- [`cap/test/test_extern_var.cp`](file:///data/dev/cp/cap/test/test_extern_var.cp):
  - 验证 `@c extern const` 与顶层 `extern var` 的 C 外部符号绑定。

### 优化与修复 (Fixed)
- [`cpc/src/parser.rs`](file:///data/dev/cp/cpc/src/parser.rs) / [`cpc_dev/src/parser.rs`](file:///data/dev/cp/cpc_dev/src/parser.rs):
  - 扩展 `parse_decl` 支持在 `extern` 之后解析 `const`/`var`/`let`，正确标记 `is_extern` 与 `is_c`。
- [`cpc/src/sema.rs`](file:///data/dev/cp/cpc/src/sema.rs) / [`cpc_dev/src/sema.rs`](file:///data/dev/cp/cpc_dev/src/sema.rs):
  - 增加对定长数组切片引用的元素类型匹配检查；
  - 在 `Expr::New` 类型检查时剔除泛型参数 `<...>`，精准定位基类定义与方法签名。
- [`cpc/src/codegen.rs`](file:///data/dev/cp/cpc/src/codegen.rs) / [`cpc_dev/src/codegen.rs`](file:///data/dev/cp/cpc_dev/src/codegen.rs):
  - 对 `Pattern::Ident("_")` 及通配符模式生成 `_ = expr;` 丢弃语句，消除 Zig 下非法 `var _` 错误；
  - 在异步状态机闭包上下文提取中跳过 `_` 变量，防止生成非法字段名；
  - 泛型类的 `_create` 构造函数统一采用 `@This()` 申请内存，解决单态化类型缺失 `_create` 的问题；
  - `lookup_method` 剥离类型中的泛型参数后缀，正确解析泛型实例的方法返回值类型。
