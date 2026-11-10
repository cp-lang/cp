# CP 研发日志 (DEVLOG) - 2026-11-11 (热数据)

> 历史研发日志已完整归档至 [DEVLOG_H.md](file:///data/dev/cp/DEVLOG_H.md)。本项目采用冷热分离架构，本文件仅保留当日活跃记录。

## 2026-11-11: 解决 Zig 0.17 顶层命名遮蔽 (Task `task-prelude-t-shadowing`) 与开发效能架构升级

- **问题分析与根因排查**：
  - 下游 Bun AGY 在迁移过程中提交了测试用例 `/data/dev/bun/temp/repro_prelude_t.cp`，代码声明了顶层枚举 `pub const T = enum { a, b };` 并在函数内迭代切片 `for (slice) |c|`。
  - 编译器在发射 Zig 代码时，自动引入了 `cp_iter_slice` 等 Prelude 辅助函数，其内部包含了 `const T = @TypeOf(val);`。
  - 在 Zig 0.17 严苛的作用域安全规则中，顶层声明属于外层作用域，辅助函数内部局部的 `const T` 被判定为遮蔽顶层符号：`local constant shadows declaration of 'T'`。
- **解决方案与实现**：
  1. 全面检索并梳理 `cpc/src/codegen.rs` 中所有生成的 Prelude 辅助函数及 `defer drop` 语法块；
  2. 将包含 `T` 的所有局部常量、泛型参数与类型提取全部系统化升级为私有前缀标识符 `__T`（如 `comptime __T: type`、`const __T = @TypeOf(v);`）；
  3. 执行 `bash scripts/sync_cpc.sh` 原子编译并部署正式版 `cpc`；
  4. Write-Through 同步更新沙盒开发分支 `cpc_dev/src/codegen.rs`；
  5. 新增端到端正式回归用例 [`cap/test/test_prelude_t_shadowing.cp`](file:///data/dev/cp/cap/test/test_prelude_t_shadowing.cp)，经 `cap` 测试 **100% PASS**；
  6. 结单 `task-prelude-t-shadowing`，并确立官方规范于 [`cp_ref.md`](file:///data/dev/cp/cp_ref.md) 第 11.9.5.2 节。

- **开发效能体系重大升级**：
  1. **解除终端重定向限制**：允许通过终端管道和重定向执行单步链式验证（如 `cpc ... > temp/*.zig && zig ast-check`），消除中间文件操作与数万 Token 往返；
  2. **四线文档冷热分离**：正式实施热数据（`SESSION.md`、`CHANGELOG.md`、`DEVLOG.md` 仅记当日）与冷数据（`*_H.md` 持久备份）分离，消除文档膨胀；
  3. **任务分类驱动**：明确 Hotfix 快速通道与 Milestone 汇总归档，兼顾敏捷交付与工程严谨性；
  4. **编译器大文件分拆架构**：确立将万行大文件按职责拆解为子模块的路线图，单文件规模控制在 2,000 行以内。
