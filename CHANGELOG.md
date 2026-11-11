# CP 变更日志 (CHANGELOG) - 2026-11-12 (热数据)

> 历史变更日志已完整归档至 [CHANGELOG_H.md](file:///data/dev/cp/CHANGELOG_H.md)。本项目采用冷热分离架构，本文件仅保留当日活跃记录。

## [2026-11-12] - 标准库第四阶段：Unix Glob 模式匹配、Semver 2.0 版本解析与范围匹配、DotEnv 与 INI 配置解析

### 新特性 (Added)
- [`lib/glob/`](file:///data/dev/cp/lib/glob/) & [`lib/glob.cp`](file:///data/dev/cp/lib/glob.cp):
  - `matcher.cp`：基于路径分段（Segment-based）递归回溯匹配算法的工业级 Glob 引擎，支持 `*` 单段任意字符匹配、`**` 多级目录递归匹配、`?` 单字符通配、`[a-z]` / `[0-9]` 字符范围集合匹配、`!` 取反以及 `hasMagic()` 极速通配符探测；
  - `index.cp` & `lib/glob.cp`：官方标准库导出通道 `cap:glob`。
- [`lib/semver/`](file:///data/dev/cp/lib/semver/) & [`lib/semver.cp`](file:///data/dev/cp/lib/semver.cp):
  - `version.cp`：Semver 2.0 规范语义版本解析器与格式化器（支持 `v1.2.3`, `=2.10.5-beta.1+sha.abcdef` 等格式），严格遵循 Semver 2.0 优先级铁律（正式发布版 > 预发布版，点分数字按数值比较，字符按 ASCII 比较）；
  - `range.cp`：支持 Caret 锁版本（`^`）、Tilde 补丁更新（`~`）、不等式（`>=`, `<=`, `>`, `<`）、连字符范围（`1.0.0 - 2.0.0`）、逻辑或（`||`）与通配符（`*` / `x` / `X`）；
  - `index.cp` & `lib/semver.cp`：官方标准库导出通道 `cap:semver`。
- [`lib/dotenv/`](file:///data/dev/cp/lib/dotenv/) & [`lib/dotenv.cp`](file:///data/dev/cp/lib/dotenv.cp):
  - `loader.cp`：100% 栈分配零堆垃圾 `.env` 解析器，支持 `KEY=VALUE`、`KEY: VALUE` 与 `export KEY=VALUE`；支持单引号原样保护、双引号转义解析（`\n`, `\r`, `\t` 等）与内联注释安全截断；提供 `get()`, `getOr()`, `has()`；
  - `index.cp` & `lib/dotenv.cp`：官方标准库导出通道 `cap:dotenv`。
- [`lib/ini/`](file:///data/dev/cp/lib/ini/) & [`lib/ini.cp`](file:///data/dev/cp/lib/ini.cp):
  - `parser.cp`：工业级 INI 解析器，支持全局配置区与 `[section_name]` 独立段空间；支持 `;` 与 `#` 双注释符及行尾注释；提供 `get(section, key)`, `getGlobal(key)`, `hasSection(section)`, `getOr()`；
  - `index.cp` & `lib/ini.cp`：官方标准库导出通道 `cap:ini`。
- 测试用例套件：
  - [`cap/test/test_glob.cp`](file:///data/dev/cp/cap/test/test_glob.cp)：4 大 Glob 通配与模式匹配套件全部 100% PASS；
  - [`cap/test/test_semver.cp`](file:///data/dev/cp/cap/test/test_semver.cp)：4 大 Semver 解析、比较、格式化与范围匹配套件全部 100% PASS；
  - [`cap/test/test_config.cp`](file:///data/dev/cp/cap/test/test_config.cp)：2 大 DotEnv 与 INI 配置解析套件全部 100% PASS。

### 文档更新 (Docs)
- [`cp_ref.md`](file:///data/dev/cp/cp_ref.md)：
  - 第 12.1 节已实装标准库总览表中新增 `cap:glob`, `cap:semver`, `cap:dotenv`, `cap:ini`；
  - 第 12.4 ~ 12.14 节全量深度扩充详细技术契约与全套完整可运行代码示例（涵盖 `cap:sync`, `cap:stream`, `cap:json`/`cap:serde`, `cap:router`, `cap:http`, `cap:path`, `cap:url`, `cap:encoding`, `cap:glob`, `cap:semver`, `cap:dotenv`, `cap:ini`）。

## [2026-11-12] - 标准库第三阶段：Radix-Tree 路由引擎、HTTP 协议栈、跨平台路径与 WHATWG URL、Base64 编码

### 新特性 (Added)
- [`lib/router/`](file:///data/dev/cp/lib/router/) & [`lib/router.cp`](file:///data/dev/cp/lib/router.cp):
  - `param_map.cp`：纯栈 16 项定长零堆分配参数映射字典 `ParamMap`，提供 `get()`, `at()`, `has()`, `append()`, `count()`, `isEmpty()`；
  - `route.cp`：`RouteMatch<T>` 泛型路由匹配结构体；
  - `radix.cp`：8 树 HTTP 动词前缀树 `Router<T>` 与 `RadixNode<T>`，支持静态路径、动态参数（`:param` 与 `[param]`）、通配符（`*path` 与 `[...path]`）与 `ALL` 方法回退；
  - 新增 `lib/router.cp` 作为 `cap:router` 官方通道出口。
- [`lib/http/`](file:///data/dev/cp/lib/http/) & [`lib/http.cp`](file:///data/dev/cp/lib/http.cp):
  - `method.cp`：标准 HTTP `Method` 枚举与 `isIdempotent()`, `hasRequestBody()`, `toString()`, `fromString()`；
  - `headers.cp`：大小写不敏感的高性能 HTTP `Headers` 集合，支持 `get()`, `set()`, `append()`, `delete()`, `count()`, `at()`；
  - `etag.cp`：`ETag` 生成与强弱 `ifNoneMatch()` 条件匹配；
  - `mime.cp`：`MimeType` 常见文件后缀与文件路径极速映射；
  - `lib/http.cp`：导出 `Method`, `Headers`, `HeaderEntry`, `ETag`, `ParsedETag`, `MimeType`。
- [`lib/path/`](file:///data/dev/cp/lib/path/) & [`lib/path.cp`](file:///data/dev/cp/lib/path.cp):
  - `posix.cp`：POSIX 标准路径计算（`basename`, `dirname`, `extname`, `isAbsolute`, `normalize`, `join`）；
  - `windows.cp`：Windows 驱动器盘符、UNC 路径与反斜杠处理；
  - `index.cp`：跨平台统一 `Path`（以及 `path.posix`、`path.win32`）；
  - 新增 `lib/path.cp` 作为 `cap:path` 官方通道出口。
- [`lib/url/`](file:///data/dev/cp/lib/url/) & [`lib/url.cp`](file:///data/dev/cp/lib/url.cp):
  - `query.cp`：`URLSearchParams` 查询字符串解析、修改与序列化；
  - `url.cp`：WHATWG 规范 `URL` 对象（解析 `protocol`, `username`, `password`, `host`, `hostname`, `port`, `pathname`, `search`, `hash`, `origin`, `searchParams`）；
  - 新增 `lib/url.cp` 作为 `cap:url` 官方通道出口。
- [`lib/encoding/`](file:///data/dev/cp/lib/encoding/) & [`lib/encoding.cp`](file:///data/dev/cp/lib/encoding.cp):
  - `base64.cp`：纯 CP 编写的高性能 Base64 编解码器（标准 Standard 模式、URL-Safe 模式、无填充模式、自适应统一解码 `+`/`-` 与 `/`/`_`、无堆分配与内存分配两种模式）；
  - 新增 `lib/encoding.cp` 作为 `cap:encoding` 官方通道出口。
- 测试用例套件：
  - [`cap/test/test_router.cp`](file:///data/dev/cp/cap/test/test_router.cp)：3 大路由套件（静态、动态、通配符与 ALL 回退全部 100% PASS）；
  - [`cap/test/test_http_features.cp`](file:///data/dev/cp/cap/test/test_http_features.cp)：4 大 HTTP 协议特性全部 100% PASS；
  - [`cap/test/test_path_url.cp`](file:///data/dev/cp/cap/test/test_path_url.cp)：4 大 POSIX/Windows 路径与 WHATWG URL 套件全部 100% PASS；
  - [`cap/test/test_encoding.cp`](file:///data/dev/cp/cap/test/test_encoding.cp)：3 大 Base64 编解码套件全部 100% PASS。

## [2026-11-12] - 标准库第二阶段：并发同步基础设施、无锁原语、管道流与零拷贝 JSON

### 新特性 (Added)
- [`lib/sync/`](file:///data/dev/cp/lib/sync/):
  - `futex.cp`：跨平台系统级用户态同步原语，封装 Linux `SYS_futex`、macOS `__ulock`、Windows `WaitOnAddress`，提供纳秒单调时钟 `Deadline` 与超时等待；
  - `mutex.cp`：吸收 Bun 成熟的自旋与 Futex 混合互斥锁（`tryLock`、`lock`、`unlock`、`withLock`）；
  - `condition.cp`：跨平台条件变量（`wait`、`timedWait`、`signal`、`broadcast`）；
  - `wait_group.cp`：并发信号量计数器（`add`、`done`、`wait`、`finish`）；
  - `channel.cp`：线程安全有界 MPMC 数据通道，支持非阻塞与超时阻塞收发；
  - `ring_buffer.cp`：基于 Dmitry Vyukov 经典算法的定长多生产者多消费者（MPMC）无锁环形队列 `MpmcQueue<T>`；
  - `unbounded_queue.cp`：侵入式无界并发无锁链表队列 `UnboundedQueue<T>`；
  - `wyhash.cp`：从 Bun 移植的 64 位极速无状态非加密哈希算法 `Wyhash`；
  - `concurrent_map.cp`：基于 Wyhash 16 桶分片的高并发分段哈希映射表 `ConcurrentMap<V>`；
  - `arc_slice.cp`：原子引用计数切片 `ArcSlice<T>`，支持跨线程零拷贝共享与自动析构；
  - `index.cp` & `lib/sync.cp`：模块导出与对外统一标准库入口。
- [`lib/io.cp`](file:///data/dev/cp/lib/io.cp) & [`lib/stream.cp`](file:///data/dev/cp/lib/stream.cp):
  - 扩充 `MaxBuf`（动态零拷贝切片缓冲守卫）；
  - 扩充 `PipeReader`（实现 `Reader, Closer`）与 `PipeWriter`（实现 `Writer, Closer`）；
  - 扩充 `Pipe.create()` 工厂函数，支持快速创建双端数据管道；
  - 新增 `lib/stream.cp` 作为 `cap:stream` 官方通道出口。
- [`lib/json/`](file:///data/dev/cp/lib/json/), [`lib/json.cp`](file:///data/dev/cp/lib/json.cp), [`lib/serde.cp`](file:///data/dev/cp/lib/serde.cp):
  - `pool.cp`：移植 Bun 内存池机制 `JsonBufferPool`，实现多轮解析内存复用；
  - `value.cp`：零拷贝 AST `JsonValue`，直接借用源输入切片；
  - `parser.cp`：极速流式递归下降 JSON 解析器 `JsonParser`；
  - 新增 `lib/serde.cp` 作为 `cap:serde` 序列化统一出口通道。
- [`cap/test/test_sync_primitives.cp`](file:///data/dev/cp/cap/test/test_sync_primitives.cp):
  - 覆盖 Wyhash、Futex、Mutex、Condition、WaitGroup、MpmcQueue、Channel、ConcurrentMap、ArcSlice 8 大用例套件。
- [`cap/test/test_zero_copy_json.cp`](file:///data/dev/cp/cap/test/test_zero_copy_json.cp):
  - 验证基础标量、源缓冲区指针直通零拷贝借用及嵌套对象/数组检索。

### 优化与修复 (Fixed)
- [`cpc/src/codegen/mod.rs`](file:///data/dev/cp/cpc/src/codegen/mod.rs) / [`cpc_dev/src/codegen/mod.rs`](file:///data/dev/cp/cpc_dev/src/codegen/mod.rs):
  - 在生成 Trait 虚表结构体时，若方法返回类型以 `!` 开头（错误联合类型），将其转译为 `anyerror{ret}`，彻底消除 Zig 函数指针类型无法使用裸 `!T` 的语法错误。
- [`cpc/src/codegen/stmt.rs`](file:///data/dev/cp/cpc/src/codegen/stmt.rs) / [`cpc_dev/src/codegen/stmt.rs`](file:///data/dev/cp/cpc_dev/src/codegen/stmt.rs):
  - 在生成 `__{var}_moved = true;` 标志位赋值时，增加 `self.drop_flag_vars.contains(&id.sym)` 守卫，避免对未注册 drop 标志的变量生成非法标识符引用。
- [`cap/test/test_json.cp`](file:///data/dev/cp/cap/test/test_json.cp):
  - 修复测试用例中将 `const conf` 进行成员字段修改的语义错误，改为 `let conf`。

## [2026-11-12] - 编译器核心 Codegen 模块化分拆与 Trait 作用域强化

### 架构演进 (Changed)
- [`cpc/src/codegen/`](file:///data/dev/cp/cpc/src/codegen/):
  - 彻底废除 1.7 万行巨石单体文件 `codegen.rs`，重构分拆为 8 个高内聚子模块：
    - `mod.rs`（3,398 行）：上下文派发中心、`Codegen` 结构体、代码缩进派发与模块级顶层项（Class/Trait/Enum/Impl/Interface/Reflection）；
    - `prelude.rs`（614 行）：`ModuleFeatures` AST 特性扫描与运行时环境感知；
    - `types.rs`（2,601 行）：类型推导、Zig 原生类型映射与常量泛型单态化；
    - `borrow.rs`（960 行）：NLL 借用检查与跨作用域冲突拦截；
    - `pattern.rs`（1,587 行）：模式匹配、穷尽性分析与 `switch` 展开；
    - `actor.rs`（1,577 行）：CPS 状态机切分、异步闭包变量捕获与 Actor 协程逻辑；
    - `stmt.rs`（2,114 行）：控制流语句生成（`let`、`if`、`while`、`for`、`return`、`defer`、`errdefer`）；
    - `expr.rs`（4,332 行）：表达式求值、运算符重载、方法调用与内联闭包。
  - 单文件规模降至 1,000 ~ 4,000 行，消除跨 800 行分段读取开销，全面赋能多子智能体并行开发。

### 优化与修复 (Fixed)
- [`cpc/src/codegen/mod.rs`](file:///data/dev/cp/cpc/src/codegen/mod.rs):
  - **Trait 虚表局部作用域强化**：在 Trait 与 Interface 声明生成中，将虚表指针类型升级为 `vtable: *const @This().VTable`，彻底杜绝与业务代码中的顶层 `class VTable` 发生 Zig 符号歧义。

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
