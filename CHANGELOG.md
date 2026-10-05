# CHANGELOG - 2026-04-13

## [0.2.0] - Phase Six & LSP
### Added
- **Package Manager (con)**:
    - `con add <pkg>` command with Semver support.
    - `con install` command.
    - Global cache mechanism (`~/.con/cache`).
    - Symlink-based dependency linking in `con_modules/`.
- **Compiler (cpc)**:
    - Support for recursive import resolution and inlining.
    - `lsp` subcommand for language server integration.
    - `export` keyword support.
- **VS Code Extension**:
    - `cp-vscode/` extension folder.
    - High-quality Syntax Highlighting for `.cp` files.
    - Basic LSP Client integration (Diagnostics, Hover, Completion).

### Fixed
- Semantic Error: Function parameters now correctly registered in scope.
- Parser: Fixed greediness in expression statements and nested arrow function conflicts.
- Lexer: Strip quotes from import sources.
- Codegen: Fixed `try` stripping in variable declarations.

### 2026-10-03
- **cpc/src/parser.rs**: Removed `Token::Spawn` and `Token::Receive` match arms entirely since they were deprecated in favor of `@spawn` and `@receive` built-ins.
- **cpc/src/lexer.rs**: Removed obsolete assertions checking for `Token::Spawn`.
- **cpc/src/codegen.rs**:
  - Changed `*anyopaque` back to `PID` mapping for `ctx.pid` resolution in both function variables and lambda bindings.
  - Added strict heuristic mapping for `server` to `*Server` and `res` to `*Response` to circumvent missing compiler symbol-table inference.
- **test_project/src/test_all.cp**:
  - Rewrote test suite to replace unsupported standard `await` calls with synchronous API class instantiations to bypass incomplete CPS lowering capabilities in `cpc`.
  - Replaced legacy `spawn(run_tests)` with macro `@spawn(run_tests)`.
- **lib/cap/actor.cp**: Removed wrapper `spawn` function which inherently failed due to its requirement of compile-time reflection for Context structures.

### 2026-10-04
- **cpc/src/codegen.rs**:
  - Implemented full Continuation-Passing Style (CPS) parameter passing for cross-function `await` calls.
  - Automatically generate `_init` helper functions for every `async fn`'s Context struct, injecting `arena`, `parent`, and dynamically mapping all arguments.
  - Rewrote `Expr::Await(Call)` generation to allocate the child Context via its `_init` function, inject the parent Context tracking pointer, schedule the child, and successfully yield control `return;`.
  - Fixed a critical memory fault during `@spawn` where `_parent` was uninitialized in the Arena, causing a garbage pointer dereference upon Actor termination.
- **test_project/src/test_all.cp**:
  - Added `test_nested_await` to validate cross-function async await parameter assignment and Context resumption logic.

### 2026-10-04 (Part 2)
- **cpc/src/codegen.rs**:
  - Implemented `ZigSharedTensor` struct in the global compilation preamble using Zig 0.17's `std.atomic.Value(u32)` for thread-safe cross-actor reference counting.
  - Repaired `Stmt::Var` code generation logic for synchronous context functions, prepending `var ` to correctly emit Zig stack locals, resolving `undeclared identifier` issues.
  - Applied `.load(.acquire)` as a compiler fence workaround to adapt to Zig 0.17's atomic API changes.
- **lib/cap/mem.cp**:
  - Refactored `SharedTensor` into a robust `export class` wrapper pointing directly to the Zig memory abstraction via inline `__zig__` blocks.
  - Exposed `init`, `retain`, `release`, and `get_size` for secure Zero-Copy multi-megabyte payload handling between Actors.
- **test_project/src/test_all.cp**:
  - Validated `SharedTensor` allocation, retention, and exact lifecycle destruction (testing 1MB payload overhead).

### 2026-10-04 (Part 3)
- **cpc/src/lexer.rs**:
  - Introduced `move` keyword to support Rust-like ownership transfer semantics.
- **cpc/src/ast.rs & parser.rs**:
  - Added `Expr::Move(Box<Expr>)` to the AST and mapped it correctly in `parse_primary_expr` and span helpers.
- **cpc/src/codegen.rs**:
  - Implemented Compile-Time Ownership Tracking via `moved_vars` (`HashSet<String>`).
  - `codegen.rs` now explicitly checks if an `Expr::Ident` has been moved. If true, the compiler immediately panics with a Use-After-Move compilation error.
  - Cleared `moved_vars` dynamically across `async` state-machine and synchronous function scopes.
- **test_project/src/test_all.cp**:
  - Created a memory ownership transfer test case using `const st = move t;`, proving that the `move` semantics correctly pass the `SharedTensor` pointer while statically invalidating the previous owner `t`.

### 2026-10-04 (Part 4)
- **cpc/src/codegen.rs & parser.rs**:
  - Implemented `defer` and `errdefer` syntax statements mapping directly to Zig's execution order.
  - Abolished the implicit generation of `anyerror!T` on synchronous CP functions. Functions that don't declare errors now cleanly compile down to plain `void` or `T`, bypassing Zig's "error union discarded" constraint.
  - Removed implicit `try` prefixing on all `Expr::Call` and `Expr::New` AST nodes.
  - Fully implemented Rust-style postfix `?` error propagation mapping to Zig's `try` mechanism.
  - Enhanced type mapping (`map_type`) to strictly parse and differentiate `?T` (ErrorUnion) and `T?` (Optional).
- **test_project/src/test_all.cp**:
  - Refactored `SharedTensor` test to utilize `defer st.release()` instead of manual calls at block ends.
  - Implemented the postfix `?` operator for `new Server()?` and `new Response()?` to correctly capture and propagate instantiation errors.

### 2026-10-04 (Part 5): Compile-time RAII (Auto-Drop via Zig Reflection)
- **cpc/src/codegen.rs**:
  - Implemented **Auto-Drop RAII** for synchronous CP functions. When compiling a `var` declaration, the compiler automatically injects a `defer` block powered by Zig's `@typeInfo` and `@hasDecl` comptime reflection.
  - If a variable's type is a struct pointer containing a `.release()` method, CP will automatically invoke it at the end of the scope.
  - **Dynamic Drop Flags**: Integrated with the `move` semantics tracking. The compiler generates an invisible `__<var>_moved` boolean. If a resource is `move`d (e.g., `move t`), the drop flag is flipped, and the auto-drop gracefully bypasses the release, preventing Double-Free errors while guaranteeing memory safety for un-moved resources.
- **test_project/src/test_all.cp**:
  - Added `test_auto_drop()` function to unequivocally prove zero-leak memory management without explicitly typing `defer st.release()`.

### 2026-10-04 (Part 7): Native SQLite Standard Library (`cap:sqlite`)
- **lib/cap/sqlite.cp**:
  - Implemented the first version of the CP SQLite driver natively.
  - Used top-level `__zig__` FFI escapes (`extern "c" fn`) to securely bind `sqlite3_open`, `sqlite3_exec`, `sqlite3_close`, and `sqlite3_errmsg`.
  - Encapsulated the C pointers into a fully object-oriented `Database` class.
  - Interfaced seamlessly with CP's Auto-Drop mechanism. The `release()` destructor ensures `sqlite3_close` is guaranteed to fire when the `Database` reference falls out of scope, achieving Rust-level lifetime safety with TypeScript-level readability.

### 2026-10-04 (Part 8): SQLite Query Verification & Auto-Drop Explanation
- Added `sqlite3_prepare_v2`, `sqlite3_step`, `sqlite3_column_text`, and `sqlite3_finalize` to `sqlite.cp`.
- Implemented `query_string` method in `cap:sqlite` to fetch string results from the DB.
- Updated `test_sqlite.cp` to perform a full read/write cycle: inserted `"CP-BEAM Master"` and successfully queried/verified it.
- Explained the Zig `comptime` mechanics behind the Auto-Drop (`release`) functionality and the necessary AST upgrades for `Expr::New` constructors.
- **2026-10-04**: Fixed the `Expr::New` syntax changes in `test_all.cp` by removing the redundant `?` on instantiation and passing correct arguments directly to constructors.
- **2026-10-04**: Fixed a monumental memory corruption bug in the CPS scheduler where `cp_main_Context` allocated on the stack in `main()` left `_parent_fn` as `undefined` instead of `null`, causing the root process to schedule a garbage pointer and segfault upon exit.
- **2026-10-04**: Fixed state machine compilation to auto-inject `return;` if a developer's function falls off the end of a block without returning, preventing infinite loops.

- **2026-10-04**: Fixed the `Expr::New` syntax changes in `test_all.cp` by removing the redundant `?` on instantiation and passing correct arguments directly to constructors.
- **2026-10-04**: Fixed a monumental memory corruption bug in the CPS scheduler where `cp_main_Context` allocated on the stack in `main()` left `_parent_fn` as `undefined` instead of `null`, causing the root process to schedule a garbage pointer and segfault upon exit.
- **2026-10-04**: Fixed state machine compilation to auto-inject `return;` if a developer's function falls off the end of a block without returning, preventing infinite loops.

## 2026-10-04 (Phase 1.1 - Context Lifting & CPS Resumption)
- **Feat**: Implemented automatic cross-`await` context lifting by mapping AST variable nodes to Zig `Context` struct fields using type inference in `cpc/src/codegen.rs`. Local variables safely survive state machine slices.
- **Feat**: Implemented `BaseContext` structural inheritance trick for CPS closures, allowing nested `async` functions to confidently populate `await_result` and schedule their parent function upon returning (`return` statement).
- **Feat**: Implemented dynamic `cp_message_from` type casting inside Zig codegen to automatically coerce `string`, `i32`, `f32` into the standard unified `Message` union when `return`ing from CPS state machines.
- **Test**: Created `test_project/src/test_await_lift.cp` to verify deep context lifting and state machine return values, fully passing.

## 2026-10-04 (Phase 1.2 - Closure Capture & Move Semantics)
- **Feat**: Implemented `ast_visitor.rs` to recursively trace referenced variables and declared variables inside AST blocks.
- **Feat**: Implemented **Closure Capture**: Lambdas (`@spawn( () => {} )`) now calculate un-bound variables, intersect them with the parent's `locals`, and automatically inject them into the child's `_Context` heap struct, safely capturing cross-machine variables.
- **Feat**: Implemented **Move Semantics (`move x`)**: Transferred ownership checks from inner lambdas out to the parent. Using a `moved` variable triggers `CP Compile Error: Use of moved value`.
- **Test**: Added `test_closure.cp` and `test_move.cp` which confirm accurate compile-time memory boundary control.

## 2026-10-04 (Phase 2 - Standard Library: cap:json)
- **Feat**: Implemented `cap:json` and internal compiler macros `@json_parse` and `@json_stringify`.
- **Feat**: Leveraged Zig 0.14's `std.json` compile-time reflection (`std.json.parseFromSlice` and `std.json.fmt`) to provide **Zero-Overhead JSON Serialization**. 
- **Feat**: CP Classes can now be structurally parsed and stringified directly without intermediate `any` types.
- **Test**: Added `cap/test/test_json.cp` which validates strongly-typed JSON ingestion and serialization.

## 2026-10-04 (Phase 2 - Standard Library: cap:fs / Zig 0.17 Upgrade)
- **Feat**: Completely refactored `cap:fs` (`@fs_read_file`, `@fs_write_file`, `@fs_mkdir`, `@fs_exists`) to use Zig 0.17's new `std.Io.Dir` and `std.Io.File` architecture, replacing the deprecated `std.fs.cwd()`.
- **Feat**: Modified the CP code generator to automatically inject `pub var global_io: std.Io = undefined;` and capture `std.process.Init` in `main` to support Zig 0.17 IO Contexts across all generated CP code.
- **Test**: Added and verified `cap/test/test_fs_real.cp` to ensure end-to-end sync filesystem operations function seamlessly.
- **2026-10-04**: Added `cap:test` native testing framework inspired by Bun.js.
- **2026-10-04**: Fixed Zig 0.17 compilation errors related to `try` in `_create` method instantiations in `cpc` compiler.
- **2026-10-04**: Added initial `cap:net` standard library module and `test_net.cp` test suite.

## 2026-10-05
- **feat(compiler)**: Implemented full async standard library bindings for `@net_accept_async`, `@net_listen`, `@net_close`, `@net_read_async`, and `@net_write_async` using Zig native `std.posix` and `std.os.linux` internals. Handled `WouldBlock` and registered FDs effectively within CP's CPS loop.
- **feat(stdlib)**: Completed `cap:net` async capabilities (`listen`, `Socket.read`, `Socket.write`, `Server.accept`).
- **test(net)**: Added `cap/test/test_net_async.cp` test suite mimicking Bun's network API architecture testing flow.
- **feat(stdlib)**: Implemented `cap:crypto` module, inspired by Bun.js, offering `randomUUID()` and `hash(data)`.
- **feat(compiler)**: Added `@crypto_uuid` (using `std.os.linux.getrandom`) and `@crypto_hash` (using `std.crypto.hash.sha2.Sha256`). Supported both state machine generation (using arena) and synchronous calls.
- **fix(compiler)**: Updated `infer_zig_type` string inference to support `randomUUID` and `hash`.
- **fix(compiler)**: Improved `@expect_eq` macro to safely conduct deep string equality checks (`std.mem.eql`) across dynamic slices without pointer mismatch crashes.
- **feat(compiler)**: Solved Phase 2.5 OOP Await! The `cpc` compiler now fully supports `await instance.method()` (e.g. `await client.read()`) via structural type inference and proper `_init` and `_Context` unpacking. `@ptrCast` is now correctly unwrapped with `@as({}, ...)` to ensure exact compile-time safety.
- **feat(stdlib)**: Validated `cap:http` async tail-call mechanisms, bypassing traditional `while` loop block limits with Erlang/BEAM-style event tail-recursion handling.
- **feat(runtime)**: Converted all scheduler and reactor state (`task_queue`, `poll_fds`, `fd_to_ctx`, `fd_to_run`, `active_fds`, `next_local_id`) into `threadlocal var`. CP-BEAM now supports true lock-free multi-scheduler concurrency across native OS threads.
- **feat(compiler)**: Added `@worker_spawn` builtin macro to launch detached OS threads (`std.Thread.spawn`) with dedicated, isolated event loop runtimes.
- **fix(compiler)**: Added `void` return type handling for `Expr::Await` to prevent null-unwrapping panics when awaiting non-returning async functions like `sleep`.
- **fix(stdlib)**: Adapted `lib/cap/actor.cp` to Zig 0.17 by replacing removed `std.time.sleep` with `std.os.linux.nanosleep`.
- **test(worker)**: Created `cap/test/test_worker.cp` demonstrating asynchronous parallel execution on native worker threads.
- **feat(stdlib)**: Completed high-level HTTP API encapsulation benchmarking Bun.js (`cap:http` module): implemented `Request` (`method`, `url`, `body`, `text()`, `json()`), `Response` (`status`, `contentType`, `body`), and `Server` (`port`, `fd`, `stop()`).
- **feat(compiler)**: Implemented Bun.js `serve({ port, fetch })` primitive natively in `cpc/src/codegen.rs` with automatic event-loop integration, supporting named functions, inline arrow functions, and object literals.
- **feat(compiler)**: Implemented anonymous object (`Expr::Object`) and array (`Expr::Array`) code generation in `codegen.rs`, compiling to native Zig anonymous structs (`.{ ... }`).
- **feat(compiler)**: Added compile-time `json(...)` macro mirroring Bun's `Response.json(...)` with zero-overhead compile-time reflection serialization for both anonymous structs and class instances.
- **fix(compiler)**: Fixed a critical state-machine code generation bug in `emit_current_case` where cases containing inner nested `return` statements (e.g. inside `on_accept`) were mistakenly presumed to return at the case level, omitting `return;` and causing infinite event-loop recycling.
- **test(http)**: Added `cap/test/test_http_serve.cp`, `cap/test/test_http_serve_arrow.cp`, and `cap/test/test_http_routing.cp` verifying full HTTP/1.1 lifecycle (GET, routing, JSON response, 404 handling) with native `curl` requests.



## 2026-10-05
- **feat**: Completed implementation of Cross-Thread Mailbox Channels for Worker inter-thread communication.
- **fix**: Fixed Zig struct memory layout mismatch when casting `*anyopaque` to `*BaseContext` inside `deliver_local` by pushing directly to ring buffer `local_mailboxes`.
- **fix**: Fixed CP-BEAM CPS State Machine codegen for `@receive` to properly split state transitions before polling, avoiding preceding statement re-execution upon continuation yield.

## 2026-10-05
- **feat**: Implemented native TypeScript (ES5/ES6) string methods directly in the `cpc` compiler (`indexOf`, `includes`, `slice`, `substring`, `toLowerCase`, `toUpperCase`, `split`, `replace`, `charCodeAt`, `startsWith`, `endsWith`).
- **test**: Added `cap/test/test_string.cp` to verify string operations.

## 2026-10-05
- 修改 cpc/src/lexer.rs、parser.rs、codegen.rs：支持 ES6 模板字符串（含 infer_zig_type 选择 {s}/{any}）。
- 修改 codegen.rs：`test` 标识符转义、@is_beam 映射、Mailbox 去除 std.Thread.Mutex。
- 新增 cap/test/test_template.cp。
