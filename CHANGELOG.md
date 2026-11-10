# CP 变更日志 (CHANGELOG) - 2026-11-11 (热数据)

> 历史变更日志已完整归档至 [CHANGELOG_H.md](file:///data/dev/cp/CHANGELOG_H.md)。本项目采用冷热分离架构，本文件仅保留当日活跃记录。

## [2026-11-11] - 编译器 Prelude 作用域防遮蔽 (Anti-Shadowing) 与 GEMINI 规范升级

### 优化与修复 (Fixed)
- [`cpc/src/codegen.rs`](file:///data/dev/cp/cpc/src/codegen.rs) / [`cpc_dev/src/codegen.rs`](file:///data/dev/cp/cpc_dev/src/codegen.rs):
  - 解决 `task-prelude-t-shadowing`：将 CodeGen Prelude 中所有泛型辅助函数（`cp_iter_slice`、`is_string_like`、`CpValType`、`cp_string_from`、`cp_opt_is_null`、`cp_opt_unwrap`、`LockGuard`、`ReadGuard`、`WriteGuard`、`CpMutex`、`CpRwLock`、`CpAtomic`、`cp_unpack_elem` 等）中的 `const T` / `comptime T` 全面重命名为私有前缀 `__T`；
  - 将所有自动析构注入（`defer if (!__x_moved) { const __T = @TypeOf(x); ... }`）中的类型标识符重命名为 `__T`；
  - 彻底杜绝底层 Zig 0.17 在用户业务代码声明顶层或局部 `T` 时的作用域遮蔽（`local constant shadows declaration of 'T'`）报错。

### 规则与架构升级 (Changed)
- [`GEMINI.md`](file:///data/dev/cp/GEMINI.md):
  - 全面放开 Bash 终端输出重定向（`>`、`>>`、管道 `|`），允许毫秒级链式自愈验证；
  - 建立四线文档冷热分离机制（当日活跃数据存储于 `SESSION.md`、`CHANGELOG.md`、`DEVLOG.md`，历史冷归档存储于 `*_H.md`）；
  - 建立任务分类驱动机制（Hotfix 极速响应通道 vs Milestone 集中汇总归档）；
  - 制定编译器核心文件（`codegen.rs`、`parser.rs`、`sema.rs`）按功能模块化分拆规范。
- [`cp_ref.md`](file:///data/dev/cp/cp_ref.md):
  - 新增第 11.9.5.2 节《运行时 Prelude 辅助函数命名空间隔离与防遮蔽机制（Anti-Shadowing）》，明确规范设计理念与代码示例。

### 新增测试用例 (Added)
- [`cap/test/test_prelude_t_shadowing.cp`](file:///data/dev/cp/cap/test/test_prelude_t_shadowing.cp):
  - 验证用户在模块顶层定义 `pub const T = enum { Alpha, Beta }` 时，切片迭代与辅助函数共存且 100% 编译运行通过。
