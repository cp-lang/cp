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
