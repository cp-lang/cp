# DEVLOG - 2026-04-13

## LSP Extension Design
- **Architecture**: Decided on a hybrid Rust/TypeScript model. The compiler (`cpc`) acts as the source of truth for the language, while the VS Code extension acts as a thin client.
- **Why?**: This ensures that diagnostics in the editor are 100% consistent with the compiler's behavior. Reuse of `Parser` and `Sema` logic reduces maintenance.
- **Protocol**: Using `tower-lsp` for the Rust side provides an async, robust implementation of the LSP protocol.
- **Syntax Highlighting**: TextMate grammar was chosen for compatibility with VS Code's core engine. It covers all RFC-002 keywords, including `actor`, `trait`, and `@` built-ins.
- **Performance**: The LSP server runs `cpc` in-memory on every change. Given the single-pass nature of CP, this is extremely fast.

## Package Manager (con)
- **Bun Inspiration**: Switched to `con_modules` and symlink-based linking to match Bun's speed.
- **Global Cache**: Implemented a global cache at `~/.con/cache` to avoid redundant downloads.
- **Semver**: Added a custom `semver.zig` to handle `^` and `~` ranges, essential for a modern DX.
