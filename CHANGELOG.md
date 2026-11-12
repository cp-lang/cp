# CP 变更日志 (CHANGELOG) - 2026-11-13 (热数据)

> 历史变更日志已完整归档至 [CHANGELOG_H.md](file:///data/dev/cp/CHANGELOG_H.md)。本项目采用冷热分离架构，本文件仅保留当日活跃记录。

## [2026-11-13] - 核心里程碑：工业级现代密码学体系（cap:crypto）建设、编译器枚举方法与解构自愈

### 工业级密码学体系实装 (Added)
- **真随机数与 CSPRNG (`random.cp`)**:
  - [`lib/crypto/random.cp`](file:///data/dev/cp/lib/crypto/random.cp)：直通 Linux 内核 `getrandom` 熵池的 CSPRNG，实现 RFC 4122 v4 `randomUUID()` 与无偏高精度 `randomInt(min, max)`。
- **散列哈希与流式增量散列 (`hash.cp`)**:
  - [`lib/crypto/hash.cp`](file:///data/dev/cp/lib/crypto/hash.cp)：`sha256`, `sha512`, `sha1`, `md5` 单次快速哈希计算；面向对象的流式 `Hash` 类，支持 `.update(data)` 增量注入与 `.digest()` 最终十六进制输出。
- **HMAC 消息认证码 (`hmac.cp`)**:
  - [`lib/crypto/hmac.cp`](file:///data/dev/cp/lib/crypto/hmac.cp)：RFC 2104 `hmacSha256(key, data)` 与流式 `Hmac` 类。
- **PBKDF2 口令派生密钥 (`pbkdf2.cp`)**:
  - [`lib/crypto/pbkdf2.cp`](file:///data/dev/cp/lib/crypto/pbkdf2.cp)：`pbkdf2Sync`（返回十六进制字符串）与 `pbkdf2Raw`（直接安全填充目标字节数组）。
- **现代 AEAD 认证对称加密 (`cipher.cp`)**:
  - [`lib/crypto/cipher.cp`](file:///data/dev/cp/lib/crypto/cipher.cp)：现代工业标准 `Aes256Gcm`（NIST SP 800-38D）与 `ChaCha20Poly1305`（RFC 8439），兼具保密性加密与带 16 字节认证标签的防篡改解密校验。
- **模块重导出与单一出口 (`index.cp`)**:
  - [`lib/crypto/index.cp`](file:///data/dev/cp/lib/crypto/index.cp)：对外统一规范重导出全部密码学符号。
- **废弃平铺单文件清理 (Removed)**:
  - 彻底删除旧版平铺单文件 `lib/crypto.cp`，消除同名文件与复合目录歧义。

### 编译器增强与热修自愈 (Compiler & Hotfix)
- [`cpc/src/codegen/expr.rs`](file:///data/dev/cp/cpc/src/codegen/expr.rs) / [`cpc_dev/src/codegen/expr.rs`](file:///data/dev/cp/cpc_dev/src/codegen/expr.rs):
  - 修复枚举静态方法调用被误降级为结构体/联合体初始化的关键缺陷：在 `Expr::Call` 访问枚举成员时，严格校验 `is_method || !is_variant`，若为方法或非变体则生成标准函数调用 `Enum.method(...)`，杜绝非法 Zig 语法 `Enum{ .method = arg }`。
- [`cpc/src/codegen/actor.rs`](file:///data/dev/cp/cpc/src/codegen/actor.rs):
  - 修复类方法接收者 `self: *T` 在生成参数列表时过滤掉 `first.ident.sym == "self"` 并保留其可变指针类型（`*T` 而非 `*const T`）。
- [`cpc/src/codegen/mod.rs`](file:///data/dev/cp/cpc/src/codegen/mod.rs):
  - 适配 Zig 0.17 结构体解包，将 `.@"struct".fields` 升级为 `.field_names` 和 `.field_types`。
- [`cpc/src/sema.rs`](file:///data/dev/cp/cpc/src/sema.rs) / `task-sema-object-tuple-destructure`:
  - 修复匿名结构体位置字段在推断为 `Type::Object` 时被丢弃的缺陷，打通 `const [a, b] = pair` 对匿名对象的解构模式分析，结单归档下游任务。

### 测试验证 (Tests)
- [`cap/test/test_crypto_features.cp`](file:///data/dev/cp/cap/test/test_crypto_features.cp)：涵盖 CSPRNG/UUID、SHA256/512/1/MD5/Streaming、HMAC/PBKDF2、AES-GCM/ChaCha20-Poly1305 4 大端到端套件，100% PASS。
- [`cap/test/test_object_destructure.cp`](file:///data/dev/cp/cap/test/test_object_destructure.cp)：匿名结构体元组解构测试，100% PASS。
- 全量回归测试：`test_http_full.cp`, `test_sync_primitives.cp`, `test_zero_copy_json.cp`, `test_io_streams.cp`, `test_path_url.cp` 等全部 100% PASS。

---

## [2026-11-13] - 核心里程碑：全面收编 Bun 工业级 HTTP 全协议栈引擎（Step 1/2/3 闭环）与底层编译器自愈

### 工业级 HTTP 核心体系完整迁入 (Added)
- **Step 1: 协议类型与基础设施助手 (Types & Infrastructure)**:
  - [`lib/http/encoding.cp`](file:///data/dev/cp/lib/http/encoding.cp)：`Encoding` 枚举（`Identity`, `Gzip`, `Deflate`, `Brotli`, `Zstd`, `Chunked`）与 `canUseLibDeflate()`、`isCompressed()`；
  - [`lib/http/fetch_modes.cp`](file:///data/dev/cp/lib/http/fetch_modes.cp)：`FetchCacheMode`, `FetchRedirect`, `FetchRequestMode` 及其字符串双向映射；
  - [`lib/http/cert.cp`](file:///data/dev/cp/lib/http/cert.cp)：`HTTPCertError`, `CertificateInfo`, `InitError`；
  - [`lib/http/url_path.cp`](file:///data/dev/cp/lib/http/url_path.cp)：`URLPath` 极速路径与参数切片分析器（`extname`, `path`, `pathname`, `first_segment`, `query_string`, `isRoot()`）；
  - [`lib/http/signals.cp`](file:///data/dev/cp/lib/http/signals.cp)：`Signals` 与 `SignalStore` 原子生命周期状态存储器；
  - [`lib/http/header_iterator.cp`](file:///data/dev/cp/lib/http/header_iterator.cp)：`HeaderValueIterator` 逗号分隔报头零拷贝切片迭代器；
  - [`lib/http/request_body.cp`](file:///data/dev/cp/lib/http/request_body.cp)：`HTTPRequestBody` 与 `HTTPRequestBodyTag` 请求体封装；
  - [`lib/http/internal_state.cp`](file:///data/dev/cp/lib/http/internal_state.cp)：`HTTPVerboseLevel`, `HTTPUpgradeState`, `Protocol`, `ShouldContinue`, `Stage`, `InternalStateFlags`, `InternalState` 客户端状态机。
- **Step 2: HTTP/2 二进制帧与 HTTP/3 客户端子系统 (H2 & H3 Subsystems)**:
  - [`lib/http/h2/frame.cp`](file:///data/dev/cp/lib/http/h2/frame.cp)：RFC 7540 HTTP/2 二进制帧协议（`CLIENT_PREFACE`, `FrameType`, `ErrorCode`, `SettingsType`, `FrameHeader` 9 字节帧头编解码, `SettingsPayloadUnit`）；
  - [`lib/http/h2/client.cp`](file:///data/dev/cp/lib/http/h2/client.cp)：`H2Stream`, `H2StreamState`, `H2ClientSession` 多路复用流会话与窗口控制器；
  - [`lib/http/h2/index.cp`](file:///data/dev/cp/lib/http/h2/index.cp)：HTTP/2 子系统统一聚合导出；
  - [`lib/http/h3/alt_svc.cp`](file:///data/dev/cp/lib/http/h3/alt_svc.cp)：RFC 7838 Alt-Svc 协商报头解析器（`AltSvc`, `AltSvcEntry`）；
  - [`lib/http/h3/client.cp`](file:///data/dev/cp/lib/http/h3/client.cp)：`H3Stream`, `H3StreamType`, `H3ClientSession` QUIC 双向请求流会话；
  - [`lib/http/h3/index.cp`](file:///data/dev/cp/lib/http/h3/index.cp)：HTTP/3 子系统统一聚合导出。
- **Step 3: 工业级核心客户端引擎与生命周期 (Client Engine & Lifecycle)**:
  - [`lib/http/decompressor.cp`](file:///data/dev/cp/lib/http/decompressor.cp)：`DecompressorType`, `Decompressor` 流式解压缩适配器；
  - [`lib/http/proxy.cp`](file:///data/dev/cp/lib/http/proxy.cp)：RFC 7231 `ProxyTunnel` HTTP CONNECT 代理隧道握手与校验器；
  - [`lib/http/sendfile.cp`](file:///data/dev/cp/lib/http/sendfile.cp)：基于 Linux 内核 `sendfile` 的零用户态拷贝文件上传器；
  - [`lib/http/context.cp`](file:///data/dev/cp/lib/http/context.cp)：`HTTPContext`, `PooledSocket` Keep-Alive 跨域套接字连接池与死链接驱逐；
  - [`lib/http/client.cp`](file:///data/dev/cp/lib/http/client.cp)：吸收自 Bun 2843 行工业核心的 `HTTPClient`, `HTTPClientResult`，完整实现 RFC 7230 请求格式化、分块传输解码（`decodeChunkedBody`）与状态解析；
  - [`lib/http/index.cp`](file:///data/dev/cp/lib/http/index.cp)：更新聚合导出全套 HTTP 工业能力。

### 编译器增强与热修自愈 (Compiler & Hotfix)
- [`cpc/src/sema.rs`](file:///data/dev/cp/cpc/src/sema.rs) / [`cpc_dev/src/sema.rs`](file:///data/dev/cp/cpc_dev/src/sema.rs):
  - 增强类型系统，打通 `Type::String` 与 `Type::Array(Box::new(Type::U8))` / `Type::SentinelSlice`（`[]const u8`）的自动安全双向转换，解决字符串字面量赋值给字节切片字段的类型不匹配阻断。
- [`cpc/src/codegen/mod.rs`](file:///data/dev/cp/cpc/src/codegen/mod.rs) / [`cpc_dev/src/codegen/mod.rs`](file:///data/dev/cp/cpc_dev/src/codegen/mod.rs):
  - 增强 `lower_struct_body`：原生支持将 CP 原始指针类型 `*!?` 转换为 `?*anyopaque`、`*!` 转换为 `*anyopaque`，将函数类型 `?fn(...)` 转换为 `?*const fn(...)`、`fn(...)` 转换为 `*const fn(...)`。
- [`scripts/sync_cpc.sh`](file:///data/dev/cp/scripts/sync_cpc.sh):
  - 同步部署更新 `/data/dev/cp/cap/bin/cpc` 与 `/data/dev/cp/cpc/target/release/cpc`，保证 `cap` 工具链始终调用最新构建的编译器。
- 下游 Bun AGY 阻断结单：
  - 极速排查并解决任务 `task-struct-body-lower-rawptr-fn`，测试验证通过，结单归档。

### 测试验证 (Tests)
- [`cap/test/test_http_full.cp`](file:///data/dev/cp/cap/test/test_http_full.cp)：全新端到端全量综合测试套件，覆盖 Step 1, Step 2, Step 3 所有原语与流程，100% PASS。
- [`cap/test/test_http_features.cp`](file:///data/dev/cp/cap/test/test_http_features.cp)：现有基础套件 100% PASS。
- [`cap/test/test_sync_primitives.cp`](file:///data/dev/cp/cap/test/test_sync_primitives.cp)：现代化去除废弃 `try` 语法，8 大并发套件 100% PASS。
- 全量标准库回归：14 个标准库核心测试套件 100% PASS。

---

## [2026-11-12] - 架构清理：全面删除根目录冗余门面文件、规范单一真理来源

### 标准库清理与重构 (Refactored & Removed)
- **彻底删除 14 个根目录同名空壳门面文件**：
  - 删除 `lib/io.cp`, `lib/sync.cp`, `lib/router.cp`, `lib/http.cp`, `lib/path.cp`, `lib/url.cp`, `lib/encoding.cp`, `lib/glob.cp`, `lib/semver.cp`, `lib/dotenv.cp`, `lib/ini.cp`, `lib/which.cp`, `lib/ws.cp`, `lib/json.cp`；
  - 杜绝“既有目录又有同名门面”的双重文件形态，所有多文件复合库一律以 `lib/<module>/index.cp` 作为唯一入口，根目录保持纯正扁平清爽。
- [`lib/http/`](file:///data/dev/cp/lib/http/):
  - 新增 `request.cp`：`Request` 类定义；
  - 新增 `response.cp`：`Response` 类定义；
  - 新增 `server.cp`：`Server`、`fetch`、`json`、`serve` 服务与客户端函数；
  - 更新 `index.cp` 统一对外公开导出全部 HTTP 契约与能力。
- 清理空目录：移除 `lib/io_ext` 与 `lib/json_ext`。

### 编译器修复 (Fixed)
- [`cpc/src/codegen/expr.rs`](file:///data/dev/cp/cpc/src/codegen/expr.rs) / [`cpc_dev/src/codegen/expr.rs`](file:///data/dev/cp/cpc_dev/src/codegen/expr.rs):
  - 修复 `r#"..."#` 原生字符串中误将 `@import("std")` 写为 `\"std\"` 导致生成的 Zig 语法中带有非法反斜杠的问题。

### 规范手册更新 (Docs)
- [`cp_ref.md`](file:///data/dev/cp/cp_ref.md)：
  - 第 1.1 与 1.2 节全面更新标准库组织规范：明确复合模块由 `lib/<name>/index.cp` 独立负责，杜绝根目录堆积多余门面。

## [2026-11-12] - 阶段审查：原生 Re-export 落地、lib/io 目录化重构与全标准库规范化审计

### 新特性与编译器增强 (Added & Compiler)
- [`cpc/src/ast.rs`](file:///data/dev/cp/cpc/src/ast.rs) / [`cpc_dev/src/ast.rs`](file:///data/dev/cp/cpc_dev/src/ast.rs):
  - 在 `ModuleItem` 枚举中新增 `Export(ExportStmt)`，支持记录导出符号列表与可选来源路径 `source`。
- [`cpc/src/parser.rs`](file:///data/dev/cp/cpc/src/parser.rs) / [`cpc_dev/src/parser.rs`](file:///data/dev/cp/cpc_dev/src/parser.rs):
  - 增加 `parse_export_stmt` 与 `parse_export_star_stmt`，完整支持 `export { foo, bar } from "./path";`、`export { foo, bar };` 与 `export * from "./path";`。
- [`cpc/src/resolver.rs`](file:///data/dev/cp/cpc/src/resolver.rs) / [`cpc_dev/src/resolver.rs`](file:///data/dev/cp/cpc_dev/src/resolver.rs):
  - 提取公共 `resolve_source_path` 模块路径解析函数；在 `resolve_dependencies` 与 `resolve_imports` 中对 `ModuleItem::Export` 的来源文件递归展开解析与依赖收集。
- [`cpc/src/sema.rs`](file:///data/dev/cp/cpc/src/sema.rs) / [`cpc_dev/src/sema.rs`](file:///data/dev/cp/cpc_dev/src/sema.rs):
  - 语义分析符号表中对 `ModuleItem::Export` 导出标识符进行符号声明登记与合法性校验。
- [`cpc/src/codegen/prelude.rs`](file:///data/dev/cp/cpc/src/codegen/prelude.rs):
  - 在生成运行时预加载时扫描 `ModuleItem::Export`，确保重导出的内部能力得到正确支持。

### 标准库重构与规范化 (Refactored)
- [`lib/io/`](file:///data/dev/cp/lib/io/) & [`lib/io.cp`](file:///data/dev/cp/lib/io.cp):
  - 严格遵循 `xxx_0.cp` 演进规则，将原单文件暂存为 `lib/io_0.cp`，创建模块化目录 `lib/io/`；
  - 拆分为 `traits.cp`、`buffer_stream.cp`、`stdio.cp`、`max_buf.cp`、`pipe.cp` 以及高内聚出口 `index.cp`；
  - 100% 覆盖原功能后安全移除 `lib/io_0.cp`，重写根目录 `lib/io.cp` 门面为规范的 `export * from "./io/index";`。
- 全量标准库重导出语法统一改造：
  - 更新 `lib/sync/index.cp` & `lib/sync.cp`
  - 更新 `lib/router/index.cp` & `lib/router.cp`
  - 更新 `lib/http/index.cp` & `lib/http.cp`
  - 更新 `lib/path/index.cp` & `lib/path.cp`
  - 更新 `lib/url/index.cp` & `lib/url.cp`
  - 更新 `lib/encoding/index.cp` & `lib/encoding.cp`
  - 更新 `lib/glob/index.cp` & `lib/glob.cp`
  - 更新 `lib/semver/index.cp` & `lib/semver.cp`
  - 更新 `lib/dotenv/index.cp` & `lib/dotenv.cp`
  - 更新 `lib/ini/index.cp` & `lib/ini.cp`
  - 更新 `lib/which/index.cp` & `lib/which.cp`
  - 更新 `lib/ws/index.cp` & `lib/ws.cp`
  - 更新 `lib/json/index.cp` & `lib/json.cp` & `lib/serde.cp`
  - 更新 `lib/stream.cp`
- [`cp_ref.md`](file:///data/dev/cp/cp_ref.md)：
  - 增加第 1.1 节重导出规范与门面模式使用指南。

## [2026-11-12] - 标准库第五阶段：可执行程序定位器、WebSocket 帧协议与文件系统原子增强

### 新特性 (Added)
- [`lib/which/`](file:///data/dev/cp/lib/which/) & [`lib/which.cp`](file:///data/dev/cp/lib/which.cp):
  - `which.cp`：吸收自 Bun 的跨平台可执行程序快速定位器，支持绝对路径、相对路径以及 `PATH` 环境变量遍历；在 Windows 下自动推导 `.exe`、`.cmd`、`.bat`；基于 `std.c.access(path, 1)`（`X_OK`）检测执行权限；
  - `index.cp` & `lib/which.cp`：官方标准库导出通道 `cap:which`。
- [`lib/ws/`](file:///data/dev/cp/lib/ws/) & [`lib/ws.cp`](file:///data/dev/cp/lib/ws.cp):
  - `opcode.cp`：标准 RFC 6455 `Opcode`（`Text`, `Binary`, `Close`, `Ping`, `Pong`, `Continue`）与 `CloseCode` 状态码；
  - `mask.cp`：RFC 6455 规范 4 字节 XOR 掩码编解码；
  - `frame.cp`：`FrameHeader` 支持 7 位基础、16 位中型与 64 位超大型载荷扩展与掩码解析，`FrameBuilder` 提供便捷帧构造方法；
  - `index.cp` & `lib/ws.cp`：官方标准库导出通道 `cap:ws`。
- [`lib/fs.cp`](file:///data/dev/cp/lib/fs.cp):
  - 扩充 `tmpDir()` 跨平台系统临时目录探测；
  - 扩充 `tmpPath()` 原子序列号与单调时钟纳秒时间戳保证唯一临时文件路径；
  - 扩充 `writeAtomic()` 崩溃安全的原子覆盖写入。
- 测试用例套件：
  - [`cap/test/test_which.cp`](file:///data/dev/cp/cap/test/test_which.cp)：可执行二进制定位与 PATH 解析全部 100% PASS；
  - [`cap/test/test_ws.cp`](file:///data/dev/cp/cap/test/test_ws.cp)：4 大 WebSocket 帧协议与掩码往返套件全部 100% PASS；
  - [`cap/test/test_fs_extensions.cp`](file:///data/dev/cp/cap/test/test_fs_extensions.cp)：临时文件与原子写入套件全部 100% PASS。

### 文档更新 (Docs)
- [`cp_ref.md`](file:///data/dev/cp/cp_ref.md)：
  - 第 12.1 节总表中更新 `cap:which` 与 `cap:ws` 状态为已全面实装；
  - 新增第 12.15 节 `cap:which`、第 12.16 节 `cap:ws` 与第 12.17 节 `cap:fs` 增强能力的详细技术契约与完整代码示例。

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
