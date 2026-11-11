# CP 协同开发执行记录 (SESSION) - 2026-11-12 (热数据)

> 历史执行记录已完整归档至 [SESSION_H.md](file:///data/dev/cp/SESSION_H.md)。本项目采用冷热分离架构，本文件仅保留当日活跃记录。

## 指令 1：推进标准库方案 A（工业级统一 I/O 流体系）与 Bun AGY 协同热修

- **用户需求**：
  1. 保持后台任务监听 `watch_task.py --creator bun_agy` 常驻，随时接收 Bun AGY 提交的任务；
  2. 优先完成标准库方案 A（工业级统一 I/O 流体系：`Reader` / `Writer` / `Closer` / `Seeker`，`BufferReader` / `BufferWriter`，`BufferedReader` / `BufferedWriter`，`copyStream` / `copyN`）；
  3. 保持与已有 `print` / `console` / `stdio` 100% 向后兼容；
  4. 遇到 Bun AGY 的热修任务立即通过极速通道修复并 Write-Through 同步；
  5. 完成后更新四线演进文档并执行 Git 提交，随后在 `cpc_dev` 中启动 `codegen/` 模块化分拆。

- **执行结果**：
  1. **协同热修结单 (`task-extern-const-var`)**：
     - 支持 `@c extern const/var` 与顶层 `extern const/var` 全语法链路；
     - 在 `cpc/src/parser.rs` 中扩展 `parse_decl` 支持在 `extern` 后解析 `const`/`var`/`let`；
     - 在 `cpc/src/sema.rs` 中将其标记为外部导入符号；
     - 在 `cpc/src/codegen.rs` 中生成 Zig 原生 `pub extern "c" const/var ...;` 无初始化表达式语句；
     - 新增回归测试 [`cap/test/test_extern_var.cp`](file:///data/dev/cp/cap/test/test_extern_var.cp)（100% PASS，601ms）；
     - 结单 `task-extern-const-var` 并由常驻守护进程持续监听。
  2. **方案 A 工业级统一 I/O 流体系（`lib/io.cp`）**：
     - 定义统一流错误集 `IOError` 与寻址基准 `SeekOrigin`；
     - 定义核心 Trait 契约：`Reader`、`Writer`、`Closer`、`Seeker`；
     - 实现零拷贝内存切片只读流 `BufferReader` 与 4096 字节栈/堆写入流 `BufferWriter`；
     - 实现泛型高性能包装流 `BufferedReader<R: Reader>`（内部 256 字节缓存）与 `BufferedWriter<W: Writer>`（自动/手动 flush）；
     - 实现泛型管道操作 `copyStream<R: Reader, W: Writer>` 与定额流搬运 `copyN`；
     - 保持现有 `Console`、`console`、`Stdio`、`stdio`、`print` 完整向后兼容。
  3. **编译器泛型类与下划线模式匹配强化**：
     - 解决 `let _ = ...;` 生成 Zig 非法 `var _` 的缺陷，修正为 Zig 规范的 `_ = expr;`；
     - 解决泛型类实例化的 `_create` 缺省缺陷，统一采用 `@This()` 执行构造；
     - 解决带泛型参数类型（如 `BufferedReader<BufferReader>`）的方法签名查找匹配。
  4. **全套回归测试通过**：
     - `cap cap/test/test_extern_var.cp`：100% PASS；
     - `cap cap/test/test_io_streams.cp`：6 大测试用例套件全部 100% PASS。

## 阶段里程碑 Git 提交 1

- **提交说明**：`feat(io): implement industrial-grade unified I/O stream system and support extern const/var`
- **变更范围**：
  1. `lib/io.cp`：方案 A 工业级统一流体系核心实现；
  2. `cap/test/test_extern_var.cp`：`extern const/var` 变量绑定回归套件；
  3. `cap/test/test_io_streams.cp`：6 大 I/O 流完整端到端测试套件；
  4. `cpc/src/ast.rs`, `cpc/src/parser.rs`, `cpc/src/sema.rs`, `cpc/src/codegen.rs`：`extern const/var`、泛型类 `@This()` 构造、下划线丢弃模式修复；
  5. `cpc_dev/src/*`：Write-Through 全量同步；
  6. `cp_ref.md`：第 12.3 节与第 20.6 节同步更新标准流体系契约；
  7. `SESSION.md` / `CHANGELOG.md` / `DEVLOG.md`：记录 2026-11-12 热日志。

## 指令 2：合并 codegen/ 模块化至 cpc 正式版本，同步更新 cp_ref.md 等四线文档

- **用户需求**：
  1. 将沙盒 `cpc_dev` 中验证通过的 `codegen/` 模块化分拆合并至正式编译器目录 `cpc/src`；
  2. 同步更新最高语言规范 `cp_ref.md`、`SESSION.md`、`CHANGELOG.md`、`DEVLOG.md` 等四线文档；
  3. 保持与 Bun AGY 的后台常驻双向协同。

- **执行结果**：
  1. **无损合流与正式版本构建**：
     - 将原 1.7 万行巨石 `cpc/src/codegen.rs` 彻底废除，替换为 8 个子模块构成的目录 `cpc/src/codegen/`（`mod.rs`, `prelude.rs`, `types.rs`, `borrow.rs`, `pattern.rs`, `actor.rs`, `stmt.rs`, `expr.rs`）；
     - 经过 `cargo check` 零错误验证；
     - 调用原子脚本 `bash scripts/sync_cpc.sh` 完成秒级正式构建与发布部署至 `/data/dev/cp/cap/bin/cpc`。
  2. **Trait 局部作用域防护修复**：
     - 在 `cpc/src/codegen/mod.rs` 中，将 Trait 与 Interface 的虚函数表指针类型从 `vtable: *const VTable` 升级为精确类型作用域 `vtable: *const @This().VTable`，彻底杜绝与用户顶层 `class VTable` 产生的 Zig 作用域符号歧义。
  3. **正式回归测试全量通过**：
     - `test_io_streams.cp`（6/6 PASS）；
     - `test_extern_var.cp`（100% PASS）；
     - `test_const_generics.cp`（2/2 PASS）；
     - `test_mutex_atomic.cp`（7/7 PASS）；
     - `test_c_ffi_pointers.cp`（11/11 PASS）；
     - `test_struct_let.cp`（100% PASS）；
     - `test_prelude_t_shadowing.cp`（100% PASS）。
  4. **四线演进文档全量对齐**：
     - `cp_ref.md`：第 20.4 节新增第 4 款《编译器核心代码生成器模块化架构》；
     - `SESSION.md` / `CHANGELOG.md` / `DEVLOG.md`：完成当日热数据归档。

## 阶段里程碑 Git 提交 2

- **提交说明**：`refactor(cpc): modularize codegen into 8 domain submodules and scope trait vtable`
- **变更范围**：
  1. `cpc/src/codegen/`：拆分为 `mod.rs`, `prelude.rs`, `types.rs`, `borrow.rs`, `pattern.rs`, `actor.rs`, `stmt.rs`, `expr.rs`；
  2. `cpc/src/codegen.rs`：删除旧单体文件；
  3. `cp_ref.md`：第 20.4 节记录代码生成器模块化演进与作用域防护；
  4. `SESSION.md` / `CHANGELOG.md` / `DEVLOG.md`：当日热日志同步更新。

## 指令 3：完成标准库第二阶段 2.1、2.2、2.3、2.4 基础设施吸收与无锁并发原语收编

- **用户需求**：
  1. 同意推进 2.1、2.2、2.3，并一并完成 2.4（无锁并发原语库：MPMC 无锁环形队列、并发哈希表、原子引用计数切片等）；
  2. 100% 优先吸收复用 Bun 已经验证成熟的资产（从 `/data/dev/bun/src/` 中迁移），避免重复造轮子；
  3. 完成后合并至 `cpc` 正式版本，同步更新四线演进文档，执行单次 Git 提交；
  4. 完成后分析下一阶段需要从 Bun 中迁移的功能并规划落地。

- **执行结果**：
  1. **2.1 并发基础设施与同步原语库 (`cap:sync`) 落地**：
     - `lib/sync/futex.cp`：跨平台系统级 Futex 封装（Linux `SYS_futex`、macOS `__ulock`、Windows `WaitOnAddress`），支持纳秒级单调时钟 `Deadline` 与超时等待；
     - `lib/sync/mutex.cp`：吸收自 Bun 的自旋与 Futex 混合互斥锁（`tryLock`, `lock`, `unlock`, `withLock`）；
     - `lib/sync/condition.cp`：跨平台条件变量（`wait`, `timedWait`, `signal`, `broadcast`）；
     - `lib/sync/wait_group.cp`：信号量并发任务计数器（`add`, `done`, `wait`, `finish`）；
     - `lib/sync/channel.cp`：线程安全有界 MPMC 管道，支持阻塞与非阻塞通信及广播关闭；
     - 解决了顶层模块同名结构体扁平合并命名冲突（将底层私有实现嵌套收束至外层导出类型作用域内）。
  2. **2.2 统一高性能 I/O 流体系与管道扩充 (`cap:io` / `cap:stream`)**：
     - `lib/io.cp`：扩充 `MaxBuf`（边界防护动态缓冲区）、`PipeReader`（实现 `Reader, Closer`）、`PipeWriter`（实现 `Writer, Closer`）、`Pipe.create()` 工厂；
     - `lib/stream.cp`：标准库 `cap:stream` 对外统一出口通道；
     - 测试用例覆盖 8 大端到端套件：`BufferReader`、`BufferWriter`、`BufferedReader`、`BufferedWriter`、`copyStream`、`copyN`、`MaxBuf`、`Pipe` 线程间数据流。
  3. **2.3 零拷贝高性能序列化框架 (`cap:json` / `cap:serde`)**：
     - `lib/json/pool.cp`：移植 Bun 的内存池机制 `JsonBufferPool`，实现多轮解析内存零二次分配；
     - `lib/json/value.cp`：零拷贝 AST 树 `JsonValue`，直接借用原始输入 Buffer 的只读切片（`&str`），消除二次字符串堆分配；
     - `lib/json/parser.cp`：极速流式递归下降 JSON 解析器，支持任意嵌套 Object/Array 与字段索引；
     - `lib/json/index.cp`、`lib/json.cp`、`lib/serde.cp`：模块导出与对外入口。
  4. **2.4 无锁并发原语库 (Lock-Free Primitives)**：
     - `lib/sync/ring_buffer.cp`：基于 Dmitry Vyukov 经典算法的定长 MPMC 无锁环形队列 `MpmcQueue<T>`；
     - `lib/sync/unbounded_queue.cp`：侵入式无锁并发链表队列 `UnboundedQueue<T>`；
     - `lib/sync/wyhash.cp`：从 Bun 移植的 64 位超高速无状态非加密哈希算法 `Wyhash`；
     - `lib/sync/concurrent_map.cp`：基于 Wyhash 16 桶分片的高并发分段哈希映射表 `ConcurrentMap<V>`；
     - `lib/sync/arc_slice.cp`：原子引用计数切片 `ArcSlice<T>`，支持跨线程零拷贝不可变只读视图与幂等释放。
  5. **编译器关键缺陷修复与无损同步**：
     - `cpc/src/codegen/mod.rs`：修复 Trait 虚表中返回错误联合类型（`!T`）时未加上 `anyerror` 前缀的非法 Zig 函数指针语法；
     - `cpc/src/codegen/stmt.rs`：对 `move` 后的 `__{var}_moved = true;` 增加 `drop_flag_vars` 存在性守卫，避免未声明标识符错误；
     - 修改同步至 `cpc_dev` 并运行 `sync_cpc.sh` 编译部署正式发布版。
  6. **全套回归测试 100% 通过**：
     - `cap/test/test_sync_primitives.cp`：8 大同步与无锁原语用例全部 PASS；
     - `cap/test/test_mutex_atomic.cp`：7 大互斥与原子操作全部 PASS；
     - `cap/test/test_io_streams.cp`：8 大统一 I/O 流与管道用例全部 PASS；
     - `cap/test/test_zero_copy_json.cp`：3 大零拷贝 JSON 解析用例全部 PASS；
     - `cap/test/test_json.cp`：原生 `@json_parse` 与 `@json_stringify` 用例 PASS。

## 阶段里程碑 Git 提交 3

- **提交说明**：`feat(stdlib): port sync primitives, lock-free queue, pipe streams, and zero-copy json from bun`
- **变更范围**：
  1. `lib/sync/`（`futex.cp`, `mutex.cp`, `condition.cp`, `wait_group.cp`, `channel.cp`, `ring_buffer.cp`, `unbounded_queue.cp`, `wyhash.cp`, `concurrent_map.cp`, `arc_slice.cp`, `index.cp`）及 `lib/sync.cp`；
  2. `lib/io.cp`（扩充 `MaxBuf`, `PipeReader`, `PipeWriter`, `Pipe`）及 `lib/stream.cp`；
  3. `lib/json/`（`pool.cp`, `value.cp`, `parser.cp`, `index.cp`）及 `lib/json.cp`, `lib/serde.cp`；
  4. `cpc/src/codegen/mod.rs`, `cpc/src/codegen/stmt.rs`：Trait 虚函数表错误类型修复与 moved 标志防护；
  5. `cap/test/test_sync_primitives.cp`, `cap/test/test_zero_copy_json.cp`, `cap/test/test_io_streams.cp`, `cap/test/test_json.cp`；
  6. `cp_ref.md`：第 12.1、12.3、12.4、12.5 节全面更新标准库索引与技术手册；
  7. `SESSION.md` / `CHANGELOG.md` / `DEVLOG.md`：当日热日志同步更新。

## 指令 4：完成标准库第三阶段 3.1、3.2、3.3（Radix-Tree 路由、HTTP 协议栈升级、跨平台路径与 WHATWG URL、Base64 编码）

- **用户需求**：
  1. 按次序完成 3.1、3.2、3.3；
  2. 完成后合并至 `cpc` 正式版本，同步更新四线演进文档，完成 Git 提交；
  3. 通过全量功能测试后，分析下一阶段需要从 Bun 中迁移的功能并规划落地。

- **执行结果**：
  1. **3.1 超高性能 Radix-Tree 路由引擎 (`cap:router`) 与 HTTP 协议栈升级 (`cap:http`)**：
     - `lib/router/param_map.cp`：纯栈存储 16 项定长零堆分配参数映射字典 `ParamMap` 与 `Param`；
     - `lib/router/route.cp`：泛型路由匹配结果 `RouteMatch<T>`；
     - `lib/router/radix.cp`：8 树 HTTP 动词前缀树 `Router<T>` 与 `RadixNode<T>`，支持静态路径、动态参数（`:param` 与 `[param]`）、通配符（`*path` 与 `[...path]`）与 `ALL` 方法回退；
     - `lib/router/index.cp` 与 `lib/router.cp`：对外官方出口 `cap:router`；
     - `lib/http/method.cp`：标准化 `Method` 枚举与 `isIdempotent()`、`hasRequestBody()`、`toString()`、`fromString()`；
     - `lib/http/headers.cp`：大小写无关 ASCII 匹配 HTTP `Headers` 集合，支持 `get`, `set`, `append`, `delete`, `count`；
     - `lib/http/etag.cp`：`ETag` 生成与强弱 `ifNoneMatch` 条件请求验证；
     - `lib/http/mime.cp`：`MimeType` 扩展名与文件路径极速映射；
     - `lib/http/index.cp` 与 `lib/http.cp`：更新对外出口。
  2. **3.2 跨平台路径规范与 WHATWG URL 解析引擎 (`cap:path` & `cap:url`)**：
     - `lib/path/posix.cp`：POSIX 标准路径计算（`basename`, `dirname`, `extname`, `isAbsolute`, `normalize`, `join`）；
     - `lib/path/windows.cp`：Windows 驱动器盘符、UNC 路径与反斜杠处理；
     - `lib/path/index.cp` 与 `lib/path.cp`：跨平台统一 `Path`（以及 `path.posix`、`path.win32`）；
     - `lib/url/query.cp`：`URLSearchParams` 查询字符串解析、修改与序列化；
     - `lib/url/url.cp`：WHATWG 规范 `URL` 对象（解析 `protocol`, `username`, `password`, `host`, `hostname`, `port`, `pathname`, `search`, `hash`, `origin`, `searchParams`）；
     - `lib/url/index.cp` 与 `lib/url.cp`：对外官方出口 `cap:url`。
  3. **3.3 数据编码标准库 (`cap:encoding` / Base64)**：
     - `lib/encoding/base64.cp`：工业级纯 CP Base64 编解码器（标准 Standard 模式、URL-Safe 模式、无填充模式、统一解码自适应 `+`/`-` 与 `/`/`_`、无堆分配与内存分配两种模式）；
     - `lib/encoding/index.cp` 与 `lib/encoding.cp`：对外官方出口 `cap:encoding`。
  4. **全套回归测试 100% 通过**：
     - `cap cap/test/test_router.cp`：3 大路由与参数提取用例全部 PASS；
     - `cap cap/test/test_http_features.cp`：4 大 Method、Headers、ETag、MimeType 用例全部 PASS；
     - `cap cap/test/test_path_url.cp`：4 大 POSIX/Windows 路径与 URL/SearchParams 用例全部 PASS；
     - `cap cap/test/test_encoding.cp`：3 大 Base64 标准、URLSafe 与动态分配用例全部 PASS；
     - 现有测试（`test_sync_primitives.cp`, `test_io_streams.cp`, `test_zero_copy_json.cp`）全量保持 100% 绿色。

## 阶段里程碑 Git 提交 4

- **提交说明**：`feat(stdlib): implement radix-tree router, http types, path/url parser, and base64 encoding`
- **变更范围**：
  1. `lib/router/` 及 `lib/router.cp`（`cap:router`）；
  2. `lib/http/`（`method.cp`, `headers.cp`, `etag.cp`, `mime.cp`, `index.cp`）及 `lib/http.cp`；
  3. `lib/path/` 及 `lib/path.cp`（`cap:path`）；
  4. `lib/url/` 及 `lib/url.cp`（`cap:url`）；
  5. `lib/encoding/` 及 `lib/encoding.cp`（`cap:encoding`）；
  6. `cap/test/test_router.cp`, `cap/test/test_http_features.cp`, `cap/test/test_path_url.cp`, `cap/test/test_encoding.cp`；
  7. `cp_ref.md`：第 12.1、12.6、12.7、12.8 节全面更新标准库文档；
  8. `SESSION.md` / `CHANGELOG.md` / `DEVLOG.md`：当日热日志同步更新。
