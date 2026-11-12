# CP 智能体高密度语法契约速查手册 (cp_agent_ref.md)

> **目标受众**：AI Agent / LLM 专用语法规范（主智能体与 Bun AGY 的唯一最高精简契约）。  
> **核心原则**：零冗余设计哲学、100% 形式化语法、高密度信息、杜绝注意力稀释与幻觉。  
> **权威完整文档**：面向人类开发者的详尽 RFC、历史与设计论证详见 `cp_ref.md`。

---

## 一、 模块与导入导出 (Modules)

### 1. 唯一合法导入语法
```typescript
import std from "std";             // 导入标准底层 Zig/C 运行时（受控只读）
import bun from "bun";             // 导入宿主 Bun 原生生态（仅限 bun 迁移工程）
import { foo, Bar } from "./utils"; // 相对路径模块导入
import { Mutex, Channel } from "cap:sync"; // 标准库协议通道导入
import { EventLoop } from "cap:event_loop"; // 统一事件循环体系
```
- **禁手**：严禁 `require(...)`、`import * as ...`、`const x = @import(...)`（顶层直接 `import x from ...`）。

### 2. 规范导出语法 (Single Source of Truth)
```typescript
// 1. 声明即导出
export const MAX_SIZE: usize = 4096;
export fn add(a: i32, b: i32): i32 { return a + b; }
export struct Point { x: i32, y: i32 }
export class Engine { ... }
export enum State { Idle, Running }

// 2. 聚合重导出（index.cp 规范）
export { Reader, Writer } from "./traits";
export { BufferReader } from "./buffer";
```
- **架构铁律**：复合库统一使用目录 `lib/<name>/index.cp` 作为唯一规范真理来源。**严禁在根目录创建冗余同名门面文件**（如 `lib/io.cp`）。

---

## 二、 变量绑定、可变性与借用 (Bindings & Mutability)

| 语法 | 语义 | 备注 |
| :--- | :--- | :--- |
| `const x: i32 = 42;` | 不可变局部/全局绑定 | 默认首选，值不可重新赋值 |
| `let y: i32 = 0;` 或 `var y: i32 = 0;` | 局部可变变量 | **严禁 `let mut`**（Rust 语法在 CP 中非法） |
| `fn foo(param: T)` | 只读不可变借用参数 | 内部不能修改 `param` 字段 |
| `fn foo(*param: T)` | 可变借用指针参数 | 允许就地修改 `param` 内部字段 |
| `fn foo(mut val: i32)` | 值传递的可变拷贝参数 | 允许在函数体内部重新赋值 `val` 本地变量 |

---

## 三、 静态类型系统与指针矩阵 (Types & Pointers)

### 1. 标量基元
- **强类型整数**：`i8`, `u8`, `i16`, `u16`, `i32`, `u32`, `i64`, `u64`, `i128`, `u128`, `isize`, `usize`。
- **浮点与逻辑**：`f32`, `f64`, `bool` (`true` / `false`), `void`。
- **字符串与切片**：`string`（只读 UTF-8）、`[]u8`（可变字节切片）、`[]const u8`（只读字节切片）、`[*]u8`（多项裸切片指针）。

### 2. 原始裸指针与内存安全体系
```typescript
// 1. 未类型化原始裸指针 (Un-typed Raw Pointers，底层 FFI / Context 传递)
type CtxPtr = *!;    // 非空裸指针，映射到 Zig 的 *anyopaque
type OptCtx = *!?;   // 可空裸指针，映射到 Zig 的 ?*anyopaque

// 2. 类型化借用指针与可选类型
type MutNode = *Node;  // 可变指针
type RefNode = &Node;  // 只读借用
type OptNode = ?*Node; // 可空指针（同 ?Node，class 自带指针语义）
```

---

## 四、 复合数据类型：结构体 (struct) vs 类 (class)

### 1. 结构体 (`struct`) —— 值语义（栈分配，无指针头）
```typescript
export struct FilePollRegistration {
    pub fd: i32 = -1;
    pub events: u32 = 0;
    pub callback: PollCallback = undefined;
    pub ctx: ?*anyopaque = null;
}

// 结构体实例化语法（TS 风格无前导点，严禁 Zig 的 .fd = ...）
var reg = FilePollRegistration{
    fd: 3,
    events: 1,
    callback: my_cb,
    ctx: null,
};
// 直接字段修改
reg.events = 2;
```

### 2. 类 (`class`) —— 引用语义（堆分配，带析构与方法分发）
```typescript
export class TimerHeap {
    items: []*Timer = &.{};
    len: usize = 0;

    // 静态工厂函数：构造并返回堆分配指针
    pub static fn init(): *TimerHeap {
        let heap = std.heap.page_allocator.create(TimerHeap) catch unreachable;
        heap.len = 0;
        return heap;
    }

    // 实例方法：接收者显式标注 *self 或 *this
    pub fn count(self: *TimerHeap): usize {
        return self.len;
    }

    pub fn push(self: *TimerHeap, timer: *Timer) {
        self.len += 1;
    }
}
```

---

## 五、 枚举 (enum) 与 模式匹配 (switch)

### 1. 现代枚举与静态方法
```typescript
export enum HashAlgorithm {
    Sha256,
    Sha512,
    Sha1,
    Md5,

    // 枚举支持绑定静态辅助方法
    pub static fn fromString(str: []const u8): ?HashAlgorithm {
        if (std.mem.eql(u8, str, "sha256")) return HashAlgorithm.Sha256;
        return null;
    }
}

// 调用枚举静态方法：
const algo = HashAlgorithm.fromString("sha256"); // 正确函数调用，严禁生成 struct 初始化
```

### 2. 模式匹配语法
```typescript
switch (status) {
    .Idle => { print("idle"); },
    .Running => { print("running"); },
    else => { print("other"); },
}

// 带载荷变体解包
switch (result) {
    .Ok => |value| { handleValue(value); },
    .Err => |err| { handleError(err); },
}
```

---

## 六、 解构赋值 (Destructuring)

全面继承 TypeScript 现代解构体验：
```typescript
// 1. 元组解构与 Rest
const pair = .{ 42, true };
const [num, flag] = pair;

const list = .{ 10, 20, 30, 40 };
const [first, ...rest] = list; // rest 类型为切片

// 2. 对象 / 结构体解构
const point = .{ .x = 100, .y = 200 };
const { x, y } = point;
```

---

## 七、 控制流与现代错误处理 (Control Flow & Safety)

### 1. 海象解包赋值
```typescript
if (timer := heap.pop()) {
    timer.callback(timer.ctx);
} else {
    // timer 为 null 时的分支
}
```

### 2. 废除 `try/catch`，使用 `?` 与 `catch` 语义
```typescript
// 1. 遇到失败直接向调用方传播（函数返回类型需为 !T 或 ?T）
const file = std.fs.cwd().openFile("data.txt", .{})?;

// 2. 失败降级回退
const port = std.fmt.parseInt(u16, port_str, 10) catch 80;

// 3. 确定性断言无失败
const buf = std.heap.page_allocator.alloc(u8, 64) catch unreachable;
```

### 3. 确定性资源释放
```typescript
const file = std.fs.cwd().openFile("data.txt", .{})?;
defer file.close(); // 退出作用域必执行

const conn = connectSocket()?;
errdefer conn.close(); // 发生错误返回时兜底释放
```

---

## 八、 智能体绝对禁手清单 (Top 8 Antipatterns)

| 禁手序号 | 错误幻觉写法 (WRONG) | CP 唯一合法写法 (CORRECT) | 根因说明 |
| :---: | :--- | :--- | :--- |
| **#1** | `let mut x = 1;` | `let x = 1;` 或 `var x = 1;` | CP 无 `mut` 关键字修饰局部变量绑定 |
| **#2** | `Point{ .x = 1, .y = 2 }` | `Point{ x: 1, y: 2 }` 或 `p.x = 1;` | 结构体初始化采用 TS 风格（无前导点，冒号分隔） |
| **#3** | `catch \|_\| { return false; }` | `catch return false;` | 废除 Zig 的 catch 捕获语法，直接接表达式或语句 |
| **#4** | `try { ... } catch (e) { ... }` | `const val = expr catch fallback;` | CP 彻底废除 Java/JS 风格的 try-catch 块 |
| **#5** | `let x: usize = 0;`（报类型不匹配） | `let x: usize = @as(usize, 0);` 或 `let x = 0;` | CP 是强类型系统，整型字面量 `0` 为 `i32`，不发生隐式符号扩展 |
| **#6** | `pub fn defer(...)` | `pub fn deferTask(...)` | 方法名严禁使用 CP 核心关键字（如 `defer`, `return`, `fn`） |
| **#7** | `export * from "./module";` (遗漏符号) | `export { Foo, bar } from "./module";` | 显式具名重导出保证符号索引与 LSP 确定性 |
| **#8** | 根目录创建同名门面（如 `lib/xyz.cp`） | 仅在 `lib/xyz/index.cp` 导出单一真理来源 | 根目录门面已全面废弃，编译器原生优先目录寻路 |
| **#9** | `const p = create(); p.x = 1;` | `let p = create(); p.x = 1;` | CP 借用检查严格拦截对 `const` 变量的属性赋值与变异方法调用 |
| **#10** | `1_000_000`（数字带下划线分隔） | `1000000`（无下划线数字） | CP 词法分析器暂不支持下划线分隔数字字面量，会分词为标识符导致 `E0206` |
