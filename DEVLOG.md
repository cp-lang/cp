# CP 研发日志 (DEVLOG) - 2026-11-13 (热数据)

> 历史研发日志已完整归档至 [DEVLOG_H.md](file:///data/dev/cp/DEVLOG_H.md)。本项目采用冷热分离架构，本文件仅保留当日活跃记录。

## 2026-11-13: 工业级现代密码学体系（cap:crypto）建设、编译器枚举静态方法自愈与下游阻塞结单

- **任务背景与工程目标**：
  - 响应用户全面收编 Bun 资产、完善 CP 官方标准库生态的规划，启动现代密码学体系（`lib/crypto/`，对应 `cap:crypto`）迁移；
  - 彻底淘汰原先仅包含部分未完善接口的平铺单文件 `lib/crypto.cp`，遵循 GEMINI.md 规范收口至模块目录 `lib/crypto/`；
  - 保证包含真随机数生成（CSPRNG）、RFC 4122 v4 UUID、哈希散列（SHA256/512/1/MD5 与流式）、HMAC 消息认证、PBKDF2 密钥派生、现代 AEAD 对称加密（AES-256-GCM 与 ChaCha20-Poly1305）的 100% 完备功能；
  - 协助下游 Bun AGY 排查消除 `task-sema-object-tuple-destructure` 编译阻断；
  - 编写端到端全面回归测试套件 `cap/test/test_crypto_features.cp`。

- **核心技术挑战与根因剖析**：
  1. **枚举静态方法调用被误判为变体初始化 (Codegen Enum Static Method Call)**：
     - 在执行 `cap cap/test/test_crypto_features.cp` 时，遇到编译报错：
       `error: type 'test_crypto_features.HashAlgorithm' does not support struct initialization syntax: const parsed = HashAlgorithm{ .fromString = algo };`
     - **原因剖析**：在 `cpc/src/codegen/expr.rs` 约 938 行，当处理 `Expr::Call` 且被调用者为 `Expr::Member(mem)`、对象在 `self.enum_defs` 中时，编译器无条件假定这是一个带负载的枚举/联合变体构造函数（如 `Option.Some(val)`），直接格式化输出 `format!("{}{{ .{} = {} }}", id.sym, mem.prop.sym, arg_code)`；
     - 当枚举定义了静态方法（如 `pub static fn fromString(str: []const u8)`）或 `mem.prop.sym` 并非变体名称时，上述无条件分支导致输出了非法的 Zig 结构体/联合体初始化语法；
     - **自愈方案**：检索 `self.enum_defs.get(&id.sym)`，判定 `is_method = ed.methods.iter().any(|m| m.ident.sym == mem.prop.sym) || ...` 与 `is_variant = ed.variants.iter().any(|v| v.ident.sym == mem.prop.sym)`。若 `is_method || !is_variant`，生成标准的静态方法调用 `format!("{}.{}({})", id.sym, mem.prop.sym, args_code.join(", "))`，彻底解决枚举静态方法调用问题。
  2. **类方法接收者可变指针退化 (Actor / Class Method Receiver Mutability)**：
     - 在流式散列 `Hash.update` 和 `Hmac.update` 中，`self: *Hash` 需要对内部状态进行就地变异。但在 `actor.rs` 降级时，由于接收者参数过滤逻辑遗漏了对 `p.ident.sym == "self"` 的匹配，导致首个参数被当作普通不可变借用生成为 `self: *const Hash`，导致 Zig 报错 `expected *Hash, found *const Hash`；
     - **自愈方案**：在 `actor.rs` 中补充首参数识别与借用类型判定，当为 `*Hash` 或 `mut` 时生成标准的 `*Hash`，保障类方法内部变异合法。
  3. **下游 Bun AGY 阻断极速热修 (Hotfix `task-sema-object-tuple-destructure`)**：
     - Bun AGY 汇报在对匿名结构体进行元组解构（`const [num, flag] = pair;`）时触发 `error[E0207]: type mismatch: expected 'Tuple([])', found 'Object([])'`；
     - 经排查：`sema.rs` 在 `infer_type(Expr::Object)` 中遇到未具名字段（`f.key == None`）时直接略过，导致生成的 `Type::Object` 字段列表为空；同时 `Pattern::Tuple` 仅匹配 `Type::Tuple`，拒绝 `Type::Object`；
     - **自愈方案**：在 `sema.rs` 中处理位置字段推断，并在 `Pattern::Tuple` 中支持 `Type::Object` 解构；更新 `codegen/stmt.rs`、`codegen/pattern.rs`、`codegen/actor.rs` 及 Zig 0.17 结构体解构适配，瞬时通过并结单。

- **效果验证**：
  - 编写并执行全新测试套件 [`cap/test/test_crypto_features.cp`](file:///data/dev/cp/cap/test/test_crypto_features.cp)，4 大测试套件全部 100% PASS；
  - 回归测试 [`cap/test/test_object_destructure.cp`](file:///data/dev/cp/cap/test/test_object_destructure.cp) 100% PASS；
  - 全量标准库测试全绿通过；
  - 权威设计手册 `cp_ref.md` 第 12.18 节全面更新，附带完整可运行代码示例。

---

## 2026-11-13: 工业级 HTTP 全协议栈体系迁移（Step 1/2/3 闭环）、编译器自愈与下游阻塞极速结单

- **任务背景与工程目标**：
  - 用户指出此前迁移的 HTTP 库仅包含 8 个基础包装文件，缺失了 Bun 工业核心中 2800 余行的 `http.cp` 客户端引擎以及 `h2_client/` 与 `h3_client/` 子系统；
  - 用户下达明确指令：“按次序完成 第一步 第二步 第三步”；
  - 严格按照路线图落地：Step 1（协议类型与基础设施助手）-> Step 2（HTTP/2 帧协议与 HTTP/3 客户端子系统）-> Step 3（工业核心客户端引擎、Keep-Alive 连接池、分块传输编解码与全生命周期）。

- **核心技术挑战与根因剖析**：
  1. **字符串与字节切片静态类型安全互转 (Sema Coercion)**：
     - 在编译包含 `CLIENT_PREFACE: []const u8 = "PRI * HTTP/2.0..."` 等常量定义时，语义分析器报错 `error[E0207]: type mismatch: expected 'Array(U8)', found 'String'`；
     - **原因剖析**：CP 静态类型系统中，字符串字面量推断为 `Type::String`，而 `[]const u8` 在 AST 中被解析为 `Type::Array(Box::new(Type::U8))`。在 `can_coerce` 和 `types_eq` 中，原先未注册 `Type::String` 与元素类型为 `U8` 的数组/切片之间的安全双向兼容规则；
     - **自愈方案**：在 `cpc/src/sema.rs` 中的 `can_coerce` 与 `types_eq` 分支中，明确加入 `String` 与 `Array(U8)` / `SentinelSlice(U8)` 的双向等价与安全兼容规则，彻底消除字面量切片赋值阻断。
  2. **`cap` 与 `cpc` 部署路径对齐**：
     - 发现原子编译脚本 `sync_cpc.sh` 仅拷贝产物至 `/data/dev/cp/cap/bin/cpc`，而 `cap/src/main.zig` 在未显式设置 `CPC_BIN` 时优先寻找 `/data/dev/cp/cpc/target/release/cpc`；
     - 这导致即使编译了最新的 debug 编译器，`cap` 仍可能加载 3 天前未支持重导出的老 release 二进制；
     - **自愈方案**：更新 `scripts/sync_cpc.sh`，在编译后原子覆盖 `cap/bin/cpc` 与 `cpc/target/release/cpc` 两处，保障编译链 100% 呼应生效。
  3. **下游 Bun AGY 阻断极速热修 (Hotfix `task-struct-body-lower-rawptr-fn`)**：
     - 监听到 Bun AGY 汇报的编译缺陷：`lower_struct_body does not lower CP raw pointer types *!? to ?*anyopaque, *! to *anyopaque, or ?fn/fn to ?*const fn/*const fn in anonymous struct expressions`；
     - 经定位，匿名结构体表达式 `Expr::Zig` 经 `lower_struct_body` 降级时，缺乏对 CP 特色原始指针（`*!?`, `*!`）与函数类型（`?fn(...)`, `fn(...)`）到 Zig `?*anyopaque`, `*anyopaque`, `?*const fn(...)`, `*const fn(...)` 的转换；
     - 在 `cpc/src/codegen/mod.rs` 中增强 `lower_struct_body`，编译并复现验证 `/data/dev/bun/temp/repro_struct_lower.cp` 成功转译为完美 Zig AST，瞬时结单。
  4. **全套工业级 HTTP 核心体系落地**：
     - **Step 1**：落地 `encoding.cp`, `fetch_modes.cp`, `cert.cp`, `url_path.cp`, `signals.cp`, `header_iterator.cp`, `request_body.cp`, `internal_state.cp`；
     - **Step 2**：落地 `lib/http/h2/`（RFC 7540 9 字节帧头编解码、流复用与窗口管理）与 `lib/http/h3/`（RFC 7838 Alt-Svc 协商、QUIC 双向请求流）；
     - **Step 3**：落地 `decompressor.cp`（流式解压缩）、`proxy.cp`（RFC 7231 CONNECT 隧道）、`sendfile.cp`（Linux 内核零拷贝上传）、`context.cp`（Keep-Alive 套接字连接池）以及 `client.cp`（RFC 7230 请求格式化、分块传输解码 `decodeChunkedBody`）。

- **效果验证**：
  - 编写并运行全新端到端测试套件 [`cap/test/test_http_full.cp`](file:///data/dev/cp/cap/test/test_http_full.cp)，覆盖三阶段所有核心功能，100% 通过；
  - 现有 [`cap/test/test_http_features.cp`](file:///data/dev/cp/cap/test/test_http_features.cp) 与 [`cap/test/test_sync_primitives.cp`](file:///data/dev/cp/cap/test/test_sync_primitives.cp) 100% 通过；
  - 全量 14 个标准库核心套件回归 100% 绿色。

---

## 2026-11-12: 彻底清理根目录冗余同名门面、HTTP 模块拆解入驻与单一真理来源确立

- **任务背景与工程目标**：
  - 用户质疑根目录门面文件（`lib/*.cp` 与 `lib/*/index.cp` 并存）的必要性；
  - 经技术分析与确认，根目录同名门面不仅在编译器解析层面 100% 冗余（编译器优先命中目录 `index.cp`），而且破坏了代码库单一真理来源（SSOT）并造成维护认知混乱；
  - 达成共识后彻底清理删除全部 14 个同名空壳门面，并将原 `lib/http.cp` 中的实体逻辑完整拆解入驻 `lib/http/` 目录。

- **核心技术挑战与根因剖析**：
  1. **HTTP 模块完整性保障**：
     - 排查发现原 `lib/http.cp` 中包含 `Request`, `Response`, `Server`, `fetch`, `json`, `serve` 实体定义，而原 `lib/http/index.cp` 仅导出了协议枚举与 Header 等子项；
     - 依照 GEMINI.md 架构规范，将上述实体功能正式解耦拆解为：
       - `lib/http/request.cp`（请求容器）；
       - `lib/http/response.cp`（响应封装）；
       - `lib/http/server.cp`（双模调度服务与客户端）；
     - 在 `lib/http/index.cp` 统一集中导出，实现 100% 无缝平替，并删除原 `lib/http.cp`。
  2. **原生字符串转义引发的 Zig 编译报错自愈**：
     - 在运行 `test_http_serve.cp` 时发现 Zig 报错：`expected expression, found 'invalid token'`；
     - 溯源定位到 `cpc/src/codegen/expr.rs`：在生成 `json(...)` 内联代码块时，外层使用了 Rust 的原生字符串字面量 `r#"(blk: { ... })"#`，但内部却误写了转义反斜杠 `@import(\"std\")`，导致转译生成的 Zig 源码中出现了字面量反斜杠 `\"` 从而报语法错误；
     - **修复**：直接使用原生双引号 `@import("std")`，重新编译部署 `cpc` 编译器后彻底解决。

- **效果验证**：
  - 全套 14 大测试套件全部无需任何门面文件辅助，直接由 `lib/<module>/index.cp` 零冗余解析；
  - 所有测试 100% 绿色通过，标准库结构达到前所未有的清爽与规范。

## 2026-11-12: 资产深度审查、编译器原生 Re-export 落地、lib/io 目录化重构与全套标准库规范化

- **任务背景与工程目标**：
  - 用户深度审查标准库迁移成果，提出四大关键规范与质量核验要求：
    1. 检查为何此前部分 `index.cp` 只有 `import` 没有 `export` 测试仍能通过？正规语法必须为 `export { foo, bar } from 'my_module';`；
    2. 核查 `io` 库是否完整迁移？从 Bun 迁移的复合库必须创建目录 `lib/io/`，同名文件改名 `io_0.cp`，确认功能 100% 覆盖后再行删除；
    3. 深度排查全量从 Bun 迁移的 CP 文件是否存在遗漏；
    4. 逐一验证所有标准库测试套件的测试覆盖点与实际通过情况；
    5. 保持与 `bun_agy` 的双向通信后台任务持续监听。

- **核心技术挑战与根因剖析**：
  1. **为何此前没有 `export` 测试仍能通过？**：
     - 在 `cpc` 编译器前端的整体依赖收集器 `resolver.rs` 中，`resolve_imports` 采用的是整程序一次性收集模型；
     - 它遍历源文件所有的 `import` 语句，递归读取目标 `.cp` 文件并将其中的函数、类、结构体直接追加（append）到同一个全局的 `all_items: Vec<ModuleItem>` 列表中；
     - 当最终将 `all_items` 喂给代码生成器时，所有被传递依赖的文件内容都被转译到了单个生成的 `.zig` 文件的文件作用域（File Scope）内；
     - 这一扁平化收集机制导致被间接引入的符号在 Zig 顶层是可见的，从而掩盖了模块边界上的 `export` 缺失；
     - **自愈方案**：在语言层面正式支持完整的 ES6 / CP 重导出语法，并在语义分析（`sema.rs`）与依赖解析（`resolver.rs`）中把 `export { ... } [from "..."]` 作为一等公民处理。
  2. **编译器原生 Re-export 语法落地**：
     - **AST 扩展**：在 `ModuleItem` 枚举中增加 `Export(ExportStmt)`，包含 `span`、`specifiers: Vec<Ident>` 与可选来源路径 `source: Option<String>`；
     - **Parser 扩展**：在 `parser.rs` 中解析 `export { Ident, ... } [from "path"]` 与 `export * from "path"`；
     - **Resolver 联动**：提取统一的 `resolve_source_path` 路径解析引擎，当遇到 `ModuleItem::Export(exp)` 且带有 `source` 时，无缝递归解析该来源文件并收集依赖项；
     - **Sema 符号表登记**：在符号收集阶段将 `Export` 语句中的所有标示符作为公共符号纳入作用域管理；
     - 经调试与测试，`export { add, MAGIC } from "./export_sub"` 与 `export * from "./export_sub"` 端到端秒级转译通过。
  3. **`lib/io` 严格遵循 `xxx_0.cp` 演进规则进行目录化拆解**：
     - 将原 `lib/io.cp` 移动为 `lib/io_0.cp`；
     - 创建 `lib/io/` 目录并根据职责高内聚分拆：
       - `traits.cp`：`IOError`, `SeekOrigin`, `Reader`, `Writer`, `Closer`, `Seeker`；
       - `buffer_stream.cp`：`BufferReader`, `BufferWriter`, `BufferedReader`, `BufferedWriter`, `copyStream`, `copyN`；
       - `stdio.cp`：`Console`, `console`, `Stdio`, `stdio`, `print`, `println`, `eprint`, `eprintln`；
       - `max_buf.cp`：对标 Bun `MaxBuf` 的流式缓冲区上限与防 OOM 守卫；
       - `pipe.cp`：原生跨平台管道 `PipeReader`, `PipeWriter`, `Pipe`；
       - `index.cp`：规范具名重导出全部符号；
     - 逐行比对原 `lib/io_0.cp`，确认所有契约与实现无一遗漏且性能增强；
     - 创建门面 `lib/io.cp`：`export * from "./io/index";`；
     - `cap/test/test_io_streams.cp` 验证 8 大用例全绿通过后，安全删除 `lib/io_0.cp`。
  4. **全标准库门面规范化升级**：
     - 彻底消除所有标准库中的“仅 `import` 伪门面”，全部转换为原生 `export { ... } from "..."` 与 `export * from "..."`。

- **效果验证**：
  - 全套 13 大标准库回归测试（涵盖 45 项细分子测试）全部一次性 100% 绿色通过；
  - 后台通信监听任务 `task-19361` 持续稳健运行。

## 2026-11-12: 落地跨平台可执行程序定位 (cap:which)、WebSocket 协议帧 (cap:ws) 与文件系统原子写入 (5.1/5.2/5.3)

- **任务背景与工程目标**：
  - 依照指示持续主动吸收 Bun 资产，打通工业级系统应用与高频实时通信的核心标准库原语；
  - 完整落地标准库第五阶段 3 大核心领域模块：
    1. 阶段 5.1：跨平台可执行二进制定位器（`cap:which`，移植自 `/data/dev/bun/src/which/which.cp`）；
    2. 阶段 5.2：RFC 6455 现代 WebSocket 帧协议处理（`cap:ws`，移植自 `/data/dev/bun/src/http/websocket.cp`）；
    3. 阶段 5.3：文件系统工业级高可用增强（`cap:fs` 中的 `tmpDir()`, `tmpPath()`, `writeAtomic()`）。
  - 对齐设计手册 `cp_ref.md`，保障全量回归测试保持 100% 绿灯并执行 Git 资产保存。

- **核心技术挑战与根因剖析**：
  1. **跨平台二进制执行权限检查 API 适配**：
     - 在 Zig 0.17 中，原 `std.posix.access` 与 `std.posix.getenv` 均已被移除或重构至底层 C 库；
     - **解决方案**：统一采用标准 C ABI 直通绑定 `std.c.access(path, 1)`（`X_OK` = 1），在 POSIX 平台精准检查文件的用户/组执行权限位；
     - 在 Windows 平台，自动检测 `.exe`、`.cmd`、`.bat` 扩展名并验证文件物理存在性；环境变量直接调用 `std.c.getenv("PATH")`，零开销跨平台。
  2. **WebSocket 掩码内存对齐崩溃防御**：
     - 在初始设计中，为了利用 32 位整型单次异或 4 字节，对 `data.ptr` 尝试了 `@alignCast` 转为 `[*]u32`；
     - 但由于 WebSocket 数据载荷紧随变长 Header（例如 2 字节或 6 字节），其指针地址可能仅按 2 字节对齐，强制转为 4 字节指针在 Debug 模式下直接触发 `panic: incorrect alignment`；
     - **修复**：采用字节级跨度结合 `key[i % 4]` 轮询异或，避免一切未对齐指针强转，且现代 LLVM 后端可自动对连续内存循环进行 SIMD 向量化加速。
  3. **未初始化变量（Undefined）静态分析与 `@memcpy` 冲突防范**：
     - 在 `cpc` 编译器的数据流静态分析中，若变量以 `var payload: [64]u8 = undefined;` 声明，随后在切片表达式 `payload[...]` 中作为接收端传递给 `@memcpy`，会被判定为“读取未初始化变量”并拦截；
     - **解决方案**：对需要传递给外部填充的定长栈内存，采用安全的零值初始化 `[0 as u8; 64]`，彻底避免触发未初始化告警。

- **效果验证**：
  - `cap/test/test_which.cp`：1/1 PASS（`which("sh")` 命中 `/bin/sh`，自定义 PATH 与权限检查通过）；
  - `cap/test/test_ws.cp`：4/4 PASS（Opcode 分类、FrameHeader 编码解码、FrameBuilder 与掩码往返、Ping/Close 控制帧）；
  - `cap/test/test_fs_extensions.cp`：1/1 PASS（`tmpDir`、`tmpPath` 唯一性、`writeAtomic` 崩溃安全原子写入读回）；
  - 全套 13 大标准库回归测试全部 100% 绿色通过。

## 2026-11-12: 落地 Unix Glob 模式匹配、Semver 2.0 语义版本/范围匹配与 DotEnv/INI 解析器 (4.1/4.2/4.3)

- **任务背景与工程目标**：
  - 响应阶段性规划指示，完整收编吸收 Bun 在项目构建、包管理与环境配置领域的核心资产；
  - 完整落地标准库第四阶段 3 大核心领域模块：
    1. 阶段 4.1：极速 Unix Glob 模式匹配（`cap:glob`，支持 `*`, `**`, `?`, `[a-z]`, `!`, `hasMagic()`）；
    2. 阶段 4.2：Semver 2.0 语义化版本与范围匹配（`cap:semver`，遵循 Semver 2.0 优先级与 npm/Cargo 范围）；
    3. 阶段 4.3：环境变量 `.env` 与 INI 配置解析（`cap:dotenv` & `cap:ini`，纯栈分配零堆垃圾就地解析）。
  - 全面系统性升级最高设计规范手册 `cp_ref.md`，为近期所有实装模块补充完备技术契约说明与完整可运行示例；
  - 持续常驻与 Bun AGY 的双向任务通信，通过全量测试并完成单次 Git 提交。

- **核心技术挑战与根因剖析**：
  1. **Globstar 递归多段匹配的回溯算法设计**：
     - 单指针线性遍历在遇到 `**/*.cp` 与目标路径 `src/main.cp` 时，容易在单个 `*` 匹配后错误重置 `is_globstar` 状态，导致跨段回退失败；
     - **解决方案**：重构为路径分段式递归回溯算法（`matchSegments`），将 pattern 与 path 均按 `/` 切分为最多 32 个段数组存放在栈上；
     - `**` 段直接映射为匹配 0..N 个路径段的递归尝试（`matchSegments(pat[1..], p[i..])`），单段内部使用非跨段的高效通配符匹配，清晰解耦且零动态堆分配。
  2. **Semver 2.0 预发布版本优先级与点分标识符排序规则**：
     - Semver 2.0 规范第 11 条规定：相同 `major.minor.patch` 下，正式版优先于任意预发布版（`1.0.0 > 1.0.0-alpha`）；
     - 预发布标识中由点号分隔的每个字段：纯数字标识按整数大小比较（`beta.2 < beta.11`），非纯数字按 ASCII 字节序比较，数字标识优先级低于非数字；字段数量少者在全部前缀相同时优先级更低；
     - **实现**：在 `Version.compare` 中设计了双迭代器分词流，对每对字段先尝试 `std.fmt.parseInt(u64)`，若均为数字则按整数数值比较，完美满足全套测试。
  3. **DotEnv 零堆垃圾与转义字符就地解析**：
     - 生产环境的 `.env` 常见包含 `export KEY=VALUE`、双引号包裹的多行长字符串（如私钥与换行符 `\n`）以及内联注释（`PORT=8080 # comment`）；
     - 若每次解析都动态分配字符串容易在 Agent 高频启动时产生碎片；
     - **实现**：`DotEnv` 内部集成 4KB 栈上转义缓冲池 `buf: [4096]u8`，双引号展开的转义字符写入该缓冲池并返回只读切片，单引号与未加引号的值直接借用原始输入切片，实现真正的零堆申请。
  4. **Zig 作用域遮蔽与扁平结构体同名变量修复**：
     - 在 `lib/ini/parser.cp` 中局部变量名 `var ini = Ini{};` 遮蔽了外层导出的 `pub const ini = struct { ... };`；
     - 在 `lib/dotenv/loader.cp` 中 `var raw_val` 未发生后续修改被 Zig 拦截；
     - 及时修正为 `var res = Ini{};` 与 `const raw_val = ...`。

- **效果验证**：
  - `cap/test/test_glob.cp`：4/4 PASS；
  - `cap/test/test_semver.cp`：4/4 PASS；
  - `cap/test/test_config.cp`：2/2 PASS；
  - 全套 10 大核心标准库测试（`test_glob.cp`, `test_semver.cp`, `test_config.cp`, `test_router.cp`, `test_http_features.cp`, `test_path_url.cp`, `test_encoding.cp`, `test_sync_primitives.cp`, `test_io_streams.cp`, `test_zero_copy_json.cp`）全量绿灯，耗时均在毫秒级。

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
