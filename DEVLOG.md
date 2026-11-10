# CP 研发日志 (DEVLOG) - 2026-11-11 (热数据)

> 历史研发日志已完整归档至 [DEVLOG_H.md](file:///data/dev/cp/DEVLOG_H.md)。本项目采用冷热分离架构，本文件仅保留当日活跃记录。

## 2026-11-11: 落地 cpc Panic-Mode 语法容错同步与多错误批量诊断系统

- **背景与痛点**：
  - 传统编译器往往在遇到第一个语法异常时立即抛出 `Err` 退出。下游 Bun AGY 在迁移几百行的复杂模块（如 `json.cp`、`http.cp`）时，反复经历“修改 1 个分号 -> 重新编译 -> 报出下一处符号不匹配”的单轮往返，造成大量时间与 Token 损耗。
  - Bun AGY 建议实现语句级 Panic-Mode 语法容错恢复，单次编译尽可能收集全量语法错误，并保持 Debug 模式秒级热更新。

- **核心架构与算法设计**：
  1. **复合解析错误定义 (`ParseError::Multiple`)**：
     - 在 `parser.rs` 中引入 `ParseError::Multiple(Vec<Diagnostic>)`，并实现 `render_all()` 和 `to_json_report()`，实现终端可读与机器可读的统一聚合。
  2. **语句级同步器 (`recover_stmt`)**：
     - 在 `Parser` 中引入 `push_error(diag)` 方法，限制单模块最多收集 50 处错误并做跨行去重；
     - `recover_stmt` 基于局部括号深度（Brace Depth）遍历，当深度为 0 且遇到语句终止符 `;`、闭合括号 `}` 或新语句前导词（`let`、`if`、`while`、`for`、`return`、`switch`、`break`、`continue`、`defer`、`errdefer`）时平滑完成同步；
     - 遇到不可识别 Token 强制 bump 推进，确保指针永不驻留，彻底消除自愈死循环风险。
  3. **顶层项同步器 (`recover_module_item`)**：
     - 模块顶层发生错误时，同步至下一个顶级定义关键字（`pub`、`export`、`fn`、`class`、`struct`、`let`、`const`、`import`、`@`）或顶层代码块边界。
  4. **两层收集流重构**：
     - 重构 `parse_block_stmt`：块内每条语句解析失败时，收集错误并调用 `recover_stmt()`，继续解析后续语句；
     - 重构 `parse_module`：顶层每项解析失败时，收集错误并调用 `recover_module_item()`，最终若存在累积错误，统一返回 `ParseError::Multiple(self.errors)`。

- **验证与效果**：
  - 创建回归测试用例 [`cap/test/test_error_recovery_fail.cp`](file:///data/dev/cp/cap/test/test_error_recovery_fail.cp)（包含 `let mut`、`if` 缺少圆括号等多处独立语法错误）；
  - `cpc check` 单次输出所有 3 个错误的精准代码切片高亮与自愈建议；
  - `cpc check --json` 输出包含 3 个诊断项与对应 `edits` 的 JSON 对象，格式 100% 结构化；
  - 全量现有测试（`test_prelude_t_shadowing.cp`、`test_const_generics.cp`、`test_struct_let.cp` 等）无损通过。

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
