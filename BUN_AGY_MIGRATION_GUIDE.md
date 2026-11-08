# Bun AGY 开发者协作备忘：CP 编译器新语法特性与高效迁移指南

> **致 Bun.js 迁移智能体（Bun AGY）**：
> CP 编译器正式版（`/data/dev/cp/cap/bin/cpc`）已完成核心架构升级与新特性发布。
> 为大幅消减在将 Bun.js（Zig 1.3.14 源码库）迁移到 CP 过程中的样板代码负担，提升转译代码的简洁性与内存安全，请在后续迁移开发中优先采用以下杀手级新语法与自愈工具。

---

## 一、 杀手级特性 1：`@c { ... }` 直接内联吸纳 C 头文件（彻底废除手写胶水代码）

### 1. 痛点场景
Bun.js 底层大量重度依赖 C 库（如 BoringSSL, uSockets, picohttpparser, c-ares, SQLite3, zlib 等）。过去迁移时，必须将 Zig 的 `@cImport` 手动翻译为几十行乃至上百行脆弱的 `extern "c" fn ...` 声明，极易因指针级数、C 结构体 padding 误判导致编译失败或内存越界。

### 2. 推荐最佳实践（极简 1 行）
在 CP 模块顶层直接使用 `@c { "xxx.h" }` 引入头文件，编译器底层自动解析并导出至统一的 `c` 全局安全命名空间：

```typescript
// 传统手写：需要手写 30+ 行 extern "c" fn
// 现代 CP：直接 1 行吸纳头文件
@c {
    "stdio.h",
    "sqlite3.h",
    "uv.h"
}

pub fn open_database(path: cstr): *any! {
    var db: *any! = null;
    // 直接以 c.xxx 调用 C 符号，零粘合代码，零 ABI 对齐风险
    let rc = c.sqlite3_open(path, &db);
    if (rc == 0) {
        return db;
    }
    return null;
}
```

> **提示**：同一个 `@c { ... }` 块内支持头文件与自定义 C 声明混合书写，满足定制化垫片函数的需要。

---

## 二、 杀手级特性 2：极简 3 字母 `own` 关键字（无别名单写者所有权转移）

### 1. 痛点场景
在将 Zig 的手动内存分配、Buffer 及跨线程 Task 传递转换到 CP 时，为了表达“交出独占所有权、杜绝并发别名冲突”，过去容易混淆使用 `move` 或被迫写冗长的约定逻辑。

### 2. 现代推荐语法（`own` 关键字）
CP 现已正式确立 **`own`**（3 字母）作为一级所有权关键字，完全替代 `move unique`：

#### A. 函数形参修饰（独占单写者契约）
```typescript
// 参数冠以 own，强制调用方必须排他转移所有权，确保函数内部拥有绝对的单写者安全
fn process_buffer(own buf: Buffer): void {
    buf.transform();
    // 离开函数时，由 buf 接管 RAII 释放
}
```

#### B. 表达式一元移交（与自动析构 Drop Flag 联动）
```typescript
let my_buf = Buffer.alloc(4096);

// 转移所有权给函数或新变量
process_buffer(own my_buf);

// ❌ 编译期静态精准拦截：error[E0203]: use of moved value 'my_buf'
// 编译器自动生成 Drop Flag，防止当前作用域离开时发生二次释放（Double Free）！
```

---

## 三、 杀手级工具：AI-First 机器自愈通道（`cpc check --json`）

### 1. 痛点场景
当转换的代码存在语法微调（如缺少圆括号、const 重新赋值等）时，通过分析冷冰冰的文本终端报错往往需要多轮试错。

### 2. 推荐使用命令
Bun AGY 在检查转换好的 `.cp` 文件时，请优先使用 `--json` 参数：

```bash
/data/dev/cp/cap/bin/cpc check --json path/to/file.cp
```

### 3. Machine-Readable Edits 自动自愈响应
当发生错误时，编译器将直接输出结构化 JSON，内含精确计算好的 `edits` 数组（字符偏移量与插入替换建议）：

```json
{
  "status": "error",
  "diagnostics": [
    {
      "code": "E0101",
      "severity": "error",
      "message": "CP Syntax Error: 'if' condition requires parentheses.",
      "file": "path/to/file.cp",
      "line": 15,
      "col": 8,
      "edits": [
        { "offset": 240, "delete_len": 0, "insert": "(" },
        { "offset": 248, "delete_len": 0, "insert": ")" }
      ]
    }
  ]
}
```
> **自愈建议**：Bun AGY 解析此 JSON 后，可直接根据 `offset` 执行单步文件插入/替换，实现 0 秒毫秒级自愈！检查完全通过时将输出 `{"status":"ok","diagnostics":[]}`。

---

## 四、 核心语法注意点（避免踩坑）

1. **控制流括号铁律**：`if (...)`、`while (...)`、`for (...)` 必须包裹圆括号，杜绝与结构体实例化大括号产生歧义；
2. **解包语法铁律（全面废除 Zig 管道语法）**：
   - **CP 统一采用海象解包**：
     - 常规只读解包：`if (val := opt) { ... }` 或 `while (val := opt) { ... }`
     - 可变指针就地解包：`if (*val := opt) { val = 200; }` 或 `while (*val := opt) { val = 200; }`
     - *（状态机内部已透明支持块级局部变量解引用，可直接就地赋值修改）*；
   - **严禁使用 Zig 风格管道语法**：CP **不支持** `if (opt) |*val|` 或 `if (opt) |val|`。若误写管道语法，`cpc` 编译器将在解析期直接报 `E0101` 语法错误，并提示改用海象解包！
3. **结构体展开更新（Spread）**：
   - 支持 TS/Zig 风格的展开构造：`let next_state = State { ...prev_state, status: 1 };`；
4. **编译测试标准命令**：
   - 始终通过 `cap` 工具执行测试：`./cap/bin/cap cap/test/test_xxx.cp`；正式版二进制位于 `/data/dev/cp/cap/bin/cpc`（单文件转译耗时约 2~5ms）。

---

*祝 Bun.js 核心库迁移极速、安全、顺畅！*
