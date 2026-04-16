
**CP (Concurrent Plus)** is a modern, high-performance programming language designed from the ground up for the Multi-Agent era. By combining a strict TypeScript-like syntax, an Erlang/BEAM-inspired Actor model, and the raw power of Zig's cross-compilation, CP allows you to build millions of concurrent AI agents without fear of memory leaks or race conditions.

---

### 🏗️ Monorepo Architecture
 
This repository contains the entire CP language ecosystem, elegantly separated by concern:
 
* 🦀 **`cpc/` (Compiler):** The lightning-fast frontend written in Rust. Contains the Lexer, Parser, AST definitions, and CPS state-machine generator.
* ⚡ **`cap/` (Toolchain):** The global command-line interface written in Zig. Orchestrates builds, manages the `.cap/` workspace, and invokes native cross-compilation.
* 📚 **`lib/` (Standard Library):** The official CP standard library, mapping directly to the `cap:` protocol (e.g., `cap:io`, `cap:net`).
