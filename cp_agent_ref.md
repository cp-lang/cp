# CP 智能体权威语法契约规范手册 (cp_agent_ref.md)

> **文档性质**：CP 语言面向 AI 智能体（AGY）的唯一最高权威精简语法与语义契约。  
> **设计责任**：CP 架构团队权威制定并全权负责，为主/子智能体与下游协同智能体编写 CP 代码的唯一规范准绳。  
> **核心原则**：纯正 CP 语法，100% 契约规则与正规精炼示例，坚决杜绝 Zig / Rust / TypeScript / Python 语法幻觉渗漏。  
> **权威对照**：面向人类开发者的详尽 RFC 演化、历史论证、宏观哲学与系统白皮书全量记载于 `cp_ref.md`。

---

## 一、 模块系统与导入导出 (Modules & Package System)

### 1. 导入语法 (`import`)
```typescript
// 1. CP 模块与相对路径具名导入
import { Router, Route } from "./router";
import { HttpClient } from "../http/client.cp";

// 2. 官方标准库协议导入（cap:* 统一协议通道）
import { Mutex, RwLock, Channel } from "cap:sync";
import { sha256, Aes256Gcm, randomBytes } from "cap:crypto";
import { EventLoop, Timer, TimerHeap } from "cap:event_loop";
import { Reader, Writer, BufferedReader, copyStream } from "cap:io";
import { test, expect_eq } from "cap:test";
import { Base64 } from "cap:encoding";
import { CSRF, constantTimeEql } from "cap:security";

// 3. 第三方包导入（自动触发向上攀升查找引擎）
import { Matrix } from "tensor-math";

// 4. 默认导入与混合导入
import App from "./app.cp";
import App, { start } from "./app.cp";

// 5. 别名导入 (as)
import { query_user as getUser } from "./db";
```

- **绝对禁手**：
  - 严禁 `require(...)` 与 CommonJS 动态加载；
  - 严禁 `use xxx;`；
  - 严禁 `import * as foo from "..."`；
  - 严禁在函数、类或局部代码块内部写 `import` / `export`，导入导出只能位于模块顶层。

### 2. 导出与聚合重导出 (`export`)
```typescript
// 1. 声明并直接导出
export const MAX_BUFFER_SIZE: usize = 65536;
export let global_status: i32 = 0;
export fn add(a: i32, b: i32): i32 { return a + b; }
export struct Point { pub x: i32 = 0; pub y: i32 = 0; }
export class Engine { ... }
export enum State { Idle, Running }
export trait Reader { fn read(*_, buf: []u8): usize!; }
export default class AgentCore { ... }

// 2. 聚合重导出（index.cp 单一真理来源出口规范）
export { Reader, Writer, Closer } from "./traits.cp";
export { BufferReader, BufferWriter } from "./buffer.cp";
export * from "./sub_module.cp";
```

- **架构铁律**：复合多文件库统一使用 `lib/<name>/index.cp` 作为唯一对外出口，根目录下严禁创建多余同名门面文件（如严禁创建 `lib/io.cp`，统一经由 `lib/io/index.cp` 或 `cap:io` 导出）。

---

## 二、 变量绑定与可变性哲学 (Bindings & Mutability)

### 1. 变量声明矩阵
CP 彻底废除 `let mut`、`var` 与 `threadlocal var`，全面拥抱清晰确定的可变性语义：
- **`let`**：可变变量声明。局部变量写 `let x = ...;`，模块顶层可变变量写 `export let g_state = ...;`。允许重新赋值与就地修改内部字段；
- **`const`**：常量与深度不可变声明。阻止一切重新赋值与内部属性修改；
- **🚨 绝对禁止 `var`**：CP 中不存在 `var` 关键字（`var` 是 Zig 遗留语法），源代码中声明可变变量必须 100% 写作 `let`；
- **🚨 绝对禁止 `let mut`**：CP 声明可变变量直接写 `let`，禁止 `let mut`；
- **🚨 绝对禁止 `threadlocal var`**：独占所有权移交使用 `move` 或 `own`。

```typescript
let count = 1;
count = count + 1; // 合法重新赋值

const MAX_LIMIT: usize = 1024;
// MAX_LIMIT = 2048; // 编译报错：Cannot assign to constant 'MAX_LIMIT'

let p = Point{ x: 10, y: 20 };
p.x = 30; // 合法修改内部属性

const fixed = Point{ x: 1, y: 2 };
// fixed.x = 5; // 编译报错：Cannot modify field of const variable 'fixed'
```

---

## 三、 静态强类型系统与纯正后缀修饰符体系 (Types & Postfix Modifiers)

### 1. 标量类型
- **整数**：`i8`, `u8`, `i16`, `u16`, `i32`, `u32`, `i64`, `u64`, `i128`, `u128`, `isize`, `usize`；
- **浮点与逻辑**：`f32`, `f64`, `bool` (`true` / `false`), `void`；
- **字符串与 C 字符串**：
  - `string`：一等公民只读 UTF-8 字节切片（等价于 `[]const u8`）；
  - `cstr`：以 `\0` 结尾的 C 字符串（映射为 `[*:0]const u8`），一等公民，脱离 `unsafe` 限制；
- **类型转换**：数值转换使用 `@as(TargetType, expr)` 或 `@intCast(expr)`。整数字面量默认为 `i32`，给 `usize` 赋值显式使用 `@as(usize, 0)` 或依靠类型推导；
- **未初始化内存**：使用 `undefined` 声明未初始化的栈缓冲区（如 `let buf: [64]u8 = undefined;`）。业务中的“空”必须使用可选类型 `T?` 与 `null`，严禁滥用 `undefined`。

### 2. 类型别名 (`type`)
```typescript
export type FileDescriptor = i32;
export type ByteSlice = []const u8;
export type Callback = fn(status: i32): void;
```

### 3. 纯正纯后缀修饰符体系 (STRICT POSTFIX NOTATION)
> **最高设计铁律**：CP 彻底废除前缀 `?T` 与 `!T`，确立“类型主体永远居首、修饰状态纯后缀链式向右展开”的全局正交规范：

| 类型标注 | 语义说明 | 映射底层 | 典型示例 |
| :--- | :--- | :--- | :--- |
| `T?` | **可选类型**（可能为 null） | `?T` | `let name: string? = null;` |
| `T!` | **推导错误联合**（可能抛出推导错误） | `!T` | `fn read(): usize! { ... }` |
| `T!ErrorSet` | **具名错误联合**（抛出显式错误集） | `ErrorSet!T` | `fn open(): File!IOError { ... }` |
| `T?!` | **可空且可能报错** | `!?T` | `fn find(): User?! { ... }` |
| `void!` 或 `!` | **无成功返回值但可能报错** | `!void` | `fn flush(): ! { ... }` |

### 4. 指针、借用与切片矩阵 (Global Symbol Matrix)
| 语法标注 | 核心语义 | 安全级别 | 典型场景 |
| :--- | :--- | :--- | :--- |
| `&T` | **只读安全借用** | 安全 | 函数形参传递不可变数据视图，禁止原地修改，零拷贝 |
| `*T` | **安全可变借用** | 安全 | 独占写权限借用，允许就地修改内部字段 |
| `move T` / `own T` | **所有权转移** | 安全 | 独占所有权移交，调用方变量即刻失效 |
| `*T!` | **类型化单项裸指针** | unsafe | 仅在 `unsafe {}` 内合法，底层操作 `ptr.*`（必须带 trailing `!`） |
| `*any!` | **非类型化通用裸指针 (非空)** | unsafe | 对应 `*anyopaque`，用于跨 FFI 传递宿主 Context |
| `*any?!` | **非类型化通用裸指针 (可空)** | unsafe | 对应 `?*anyopaque`，用于用户可空回调上下文 |
| `[*]T` | **未定长多项裸指针** | unsafe | C 数组首地址与指针偏移算术 |
| `[]T` | **动态可变切片** | 安全 | 缓冲区切片视图，元素可就地修改 |
| `[]const T` | **动态只读切片** | 安全 | 常量视图或字符串（`string` 等价于 `[]const u8`） |
| `[N]T` | **栈分配定长数组** | 安全 | 固定长度本地缓存，如 `let buf: [64]u8 = undefined;` |

### 5. 原生运算符矩阵
- **标准算术**：`+`, `-`, `*`, `/`, `%`
- **环绕运算（截断溢出）**：`+%`, `-%`, `*%`
- **饱和运算（边界钳位）**：`+|`, `-|`
- **位运算符**：`&`, `|`, `^`, `~`, `<<`, `>>`
- **逻辑运算符**：`&&`, `||`, `!`（**🚨 严禁使用 Python/Zig 的 `and` / `or`**）
- **管道操作符**：`|>`（如 `data |> filter() |> process()`）

---

## 四、 函数系统、形参传递与泛型 (Functions & Generics)

### 1. 函数声明与四种形参传递模式
```typescript
// 1. 只读借用参数（默认行为，零拷贝只读传递，禁止内部修改，等同于 data: &Point）
fn inspect(data: Point) {
    print(data.x);
}
fn inspectExplicit(data: &Point) { ... }

// 2. 可变借用参数（独占可变访问，允许就地修改字段）
fn modifyPoint(pt: *Point, dx: i32, dy: i32) {
    pt.x += dx;
    pt.y += dy;
}
// 亦支持前置星号修饰形参名：fn modifyPointAlt(*pt: Point, ...)

// 3. 类型化裸指针形参（unsafe 级别，必须带 trailing !）
fn rawOperation(ptr: *Point!) {
    unsafe {
        ptr.*.x = 100;
    }
}

// 4. 所有权转移参数（move 或 own，函数接管生命周期，调用方原变量失效）
fn consumeUser(move u: User) {
    u.saveToDb();
} // 作用域结束时 u 自动释放

fn consumeUserOwn(own u: User) { ... }

// 5. 默认形参值
fn connect(host: string, port: u16 = 8080) { ... }

// 6. 返回值类型纯后缀标注
fn parsePort(str: string): u16! {
    if (str.len == 0) return error.EmptyString;
    return 8080;
}
```

### 2. 泛型与编译期约束 (Generics & Bounds)
```typescript
// 1. 类型泛型与 Trait 约束
fn map<T, U>(list: []T, func: fn(T): U): []U { ... }
fn printArea<T: Measurable>(item: &T) {
    print(item.area());
}

// 2. 多 Trait 约束复合 (+)
fn processStream<T: Reader + Closer>(stream: *T) { ... }

// 3. 常量泛型 (Const Generics)
fn makeBuffer<const N: usize>(): [N]u8 {
    let buf: [N]u8 = undefined;
    return buf;
}
```

### 3. 方法接收者（Receiver）终极规则：`_`
- 在结构体与类方法中，**`_` 是接收者（Receiver）的唯一专属符号**（等同于 TS 中的 `this`）；
- **只读实例方法**：形参列表中的接收者**完全省略**，方法体内直接裸写字段名（如 `return x * x;`）；
- **变异实例方法**：形参列表必须显式声明 **`*_`**（独占可变借用）；
- **所有权消耗方法**：形参列表声明 **`move _`** 或 **`own _`**（调用后实例即刻销毁）；
- **遮蔽消除**：仅当形参或局部变量与成员字段同名时，写 `_.x` 消除二义性；
- **静态方法**：必须冠以 `static` 修饰符，无接收者，通过 `ClassName.method()` 调用。

### 4. Lambda 闭包与箭头函数
```typescript
// 箭头表达式
const double = (x: i32): i32 => x * 2;

// 箭头代码块
const logger = (msg: string) => {
    print(`[LOG]: ${msg}`);
};
```

---

## 五、 面向对象与契约解耦体系 (`struct`, `class`, `trait`, `impl`)

### 1. 结构体 (`struct`) —— 值语义（纯栈分配，零额外开销）
结构体专用于定义纯内存布局容器，无虚表指针，采用 TypeScript 风格对象字面量初始化：
```typescript
export struct Point {
    pub x: i32 = 0;
    pub y: i32 = 0;

    // 只读实例方法（省略接收者，直接访问字段）
    pub fn distanceSq(): i32 {
        return x * x + y * y;
    }

    // 变异实例方法（显式标注 *_ 可变借用接收者）
    pub fn translate(*_, dx: i32, dy: i32) {
        x += dx;
        y += dy;
    }
}

// 泛型结构体
export struct Vector<T> {
    pub items: []T = [];
    pub fn len(): usize { return items.len; }
}

// 结构体实例化语法（严格禁止 Zig 的 .x = 1 前导点风格！）
let pt = Point{ x: 10, y: 20 };
pt.translate(5, -5);
```

### 2. 类 (`class`) —— 引用语义（堆分配，支持 RC/RAII 自动生命周期）
类用于具备独立生命周期和复杂行为的对象，使用 `new` 实例化，支持构造与生命周期钩子：
```typescript
export class TimerQueue {
    capacity: usize = 64;
    count: usize = 0;

    // 构造函数
    pub fn init(*_, initial_cap: usize) {
        capacity = initial_cap;
        count = 0;
    }

    // 只读方法
    pub fn len(): usize {
        return count;
    }

    // 变异方法
    pub fn push(*_, timer: *Timer) {
        count += 1;
    }

    // 析构清理钩子（离开作用域自动执行）
    pub fn release(*_) {
        // 清理底层非托管资源
    }

    // 静态工厂函数
    pub static fn create(cap: usize): *TimerQueue {
        let q = new TimerQueue();
        q.init(cap);
        return q;
    }
}

// 实例化类
let tq = new TimerQueue();
let tq2 = TimerQueue.create(128);
```

### 3. 系统级内存布局注解
- `@packed struct Foo { ... }`：紧凑位内存布局，无对齐填充；
- `@extern struct Bar { ... }`：严格对齐 C ABI 内存布局；
- `@opaque class Baz { ... }`：不透明抽象类型（映射为 `pub const Baz = opaque {};`），专用于 C/C++ 动态库返回的句柄，编译器自动豁免堆分配构造器；
- `@value class Qux { ... }`：值语义类，作为连续内存直接嵌入栈上或父结构体内部。

### 4. 行为契约 (`trait`) 与后向解耦实现 (`impl`)
CP 彻底废除 `interface`，使用 `trait` + `impl` 实现数据与行为彻底解耦：
```typescript
export trait Measurable {
    fn area(_): f64;
}

export struct Circle {
    pub radius: f64 = 0.0;
}

// 为已有类型独立实现契约
impl Measurable for Circle {
    fn area(_): f64 {
        return 3.1415926535 * _.radius * _.radius;
    }
}
```

### 5. 联合体 (`union(enum)`) 与枚举 (`enum`)
```typescript
// 1. 带标签联合体（Tagged Union）
export union(enum) Message {
    Quit: void,
    Move: Point,
    Write: string,
}

// 2. 现代枚举（支持绑定静态工具方法）
export enum HttpStatus {
    Ok = 200,
    NotFound = 404,
    InternalError = 500,

    pub static fn isSuccess(code: u16): bool {
        return code >= 200 && code < 300;
    }
}
```

---

## 六、 字面量、解构赋值与模式匹配 (Literals, Destructuring & Switch)

### 1. 纯正 CP 字面量标准
- **数组/元组字面量**：`const pair = [10, 20];`（纯方括号，严禁 Zig 的 `.{ 10, 20 }`）；
- **匿名对象字面量**：`const obj = { name: "cp", port: 8080 };`（纯花括号，严禁 Zig 的 `.{ .name = ... }`）；
- **空切片**：`let slice: []u8 = [];`（纯方括号，严禁 Zig 的 `&.{}`）；
- **动态哈希字典字面量**：`let dict = #{ "key": "value" };`。

### 2. 现代解构赋值
```typescript
// 1. 元组与数组解构
const [a, b] = [100, 200];
const [head, ...tail] = [1, 2, 3, 4]; // tail 展开为切片视图

// 2. 结构体与对象解构
const pt = Point{ x: 10, y: 20 };
const { x, y } = pt;

// 3. 忽略丢弃元素（必须使用通配符 *，严禁使用 _，因为 _ 是方法接收者保留字）
const [first, *] = [10, 20];
```

### 3. `switch` 原生模式匹配
```typescript
// 1. 枚举标签匹配
switch (status) {
    .Ok => { handleSuccess(); },
    .NotFound => { handleNotFound(); },
    else => { handleFallback(); }, // 或 * => 通配兜底
}

// 2. 联合体载荷解包匹配
switch (msg) {
    .Quit => { exit(); },
    .Move => |pt| { updatePos(pt.x, pt.y); },
    .Write => |str| { print(str); },
}

// 3. 分支原位可变借用修改 (|*ptr|)
switch (user) {
    .Active => |*u| {
        u.score += 10; // 原地可变借用
    },
    * => {},
}
```

---

## 七、 控制流与现代迭代 (Control Flow)

### 1. 条件语句与海象解包 (`:=`)
> **强制规则**：`if`, `while`, `for` 条件头部必须加小括号 `(...)`。
```typescript
// 1. 标准条件分支（逻辑运算符必须用 && 和 ||，严禁 and / or）
if (count > 0 && is_active) {
    process();
} else if (count == 0 || is_idle) {
    idle();
} else {
    reset();
}

// 2. 海象非空解包赋值 (Walrus Unwrap)
if (timer := heap.pop()) {
    timer.callback(timer.ctx);
} else {
    // timer 为 null 时的分支
}
```

### 2. 循环遍历 (`for ... of`)
> **绝对禁手**：严禁 Zig 风格的 `for (collection) |item|` 或 `for (0..n) |i|`！
```typescript
// 1. 范围遍历（半开区间 0..n 或闭区间 0..=n）
for (i of 0..10) {
    print(i);
}

// 2. 集合只读遍历
for (item of items) {
    process(item);
}

// 3. 集合原地可变遍历
for (*item of items) {
    item.active = true;
}

// 4. 集合元素所有权转移遍历
for (move item of items) {
    consume(item);
}

// 5. while 循环
while (loop.hasActiveHandles() && !loop.stopped) {
    _ = loop.tick(0);
}
```

---

## 八、 现代错误处理与 RAII 确定性清理 (Error Handling & Defer)

### 1. 废除 `try/catch`，统一使用 `?`、`catch` 与 `??`
> **绝对禁手**：
> - 严禁 `try { ... } catch (e) { ... }` 异常块；
> - 严禁 Zig 关键字前缀 `try expr;`，统一使用后缀冒泡操作符 `expr?`；
> - 严禁 Zig 管道捕获 `catch |_| { ... }`。

```typescript
// 1. 后缀错误冒泡符 '?'（要求调用方函数签名返回类型带 !）
const file = openFile("config.json")?;

// 2. 错误捕获降级 (catch fallback)
const port = parsePort(port_str) catch 8080;

// 3. 错误捕获并早退
const data = parseData() catch return false;

// 4. 确定性断言绝对无错误 (catch unreachable)
const buf = allocator.alloc(u8, 1024) catch unreachable;

// 5. 空值合并操作符 '??'（仅针对可选类型 T? 的 null，不捕获错误）
const host = opt_host ?? "127.0.0.1";

// 6. 可选链操作符 '?.'
const zip = user?.address?.zip;

// 7. 强制解空操作符 '.?'
const val = opt_val.?;
```

### 2. 确定性资源释放 (`defer` 与 `errdefer`)
```typescript
fn processFile(path: string): void! {
    let file = openFile(path)?;
    defer file.close(); // 作用域退出时无论正常还是报错必执行清理

    let conn = connectServer()?;
    errdefer conn.abort(); // 仅当后续代码发生错误提前退出时触发清理

    file.pipeTo(conn)?;
}
```

---

## 九、 特殊代码块与底层逃逸舱 (Blocks & FFI)

### 1. 四大原生代码块矩阵
| 代码块语法 | 核心语义与职责 | 安全边界 | 典型场景 |
| :--- | :--- | :--- | :--- |
| `unsafe { ... }` | 安全逃逸块：关闭借用检查，允许解引用与操作裸指针（`*T!`） | unsafe | 底层系统操作、硬件寄存器、裸指针解引用 |
| `zig { ... }` | Native Zig 原生逃逸舱（**严禁带下划线的 `__zig__`**） | 底层交互 | 复杂 OS 系统调用（如 `clock_gettime`）、编译期查找表 |
| `arena { ... }` | 作用域区域分配器：离开作用域时整块内存 $O(1)$ 瞬时清空释放 | 安全 | 高吞吐微服务请求处理、临时对象群销毁、彻底零内存碎片 |
| `@c { ... }` | 内联 C 头文件声明块：引入系统底层 C 符号与库函数原型 | FFI | 引入 POSIX 系统头文件、底层套接字常数 |

### 2. 代码块使用示例
```typescript
// 1. arena 作用域分配块（离开作用域瞬间整体 O(1) 释放）
arena {
    let temp_buf = allocator.alloc(u8, 4096) catch unreachable;
    processBuffer(temp_buf);
} // 内部所有动态分配在此瞬间清空，零碎片！

// 2. zig 原生逃逸块（仅用于底层系统直接调用）
fn getWallTimeMs(): u64 {
    zig {
        var ts: std.os.linux.timespec = undefined;
        _ = std.os.linux.clock_gettime(std.os.linux.CLOCK.REALTIME, &ts);
        const sec: u64 = @as(u64, @intCast(ts.sec));
        const nsec: u64 = @as(u64, @intCast(ts.nsec));
        return sec * 1000 + (nsec / 1000000);
    }
}

// 3. unsafe 裸指针解引用块
unsafe {
    let raw: *Point! = getRawPointer();
    raw.*.x = 100; // 裸指针解引用
}

// 4. @c 声明与外部 C 符号绑定
@c {
    #include <unistd.h>
}
extern "c" fn getpid(): i32;
```

---

## 十、 异步协程与并发模型 (Async, Await & Concurrency)

```typescript
// 1. 异步函数声明与调用
async fn fetchBody(url: string): string! {
    let client = HttpClient.init();
    defer client.destroy();

    const response = await client.get(url)?;
    return response.readText()?;
}

// 2. 协程并发派生
let task = spawn asyncWork();
let result = await task;

// 3. CPU 密集型 Worker 物理线程池派生
let cpu_task = spawn.cpu heavyMatrixMultiply(matrixA, matrixB);
let computed = await cpu_task;

// 4. Actor 消息通信
send(worker_pid, Message.Data{ payload: chunk });
receive {
    Message.Result => |res| handleResult(res),
    Message.Error => |err| handleError(err),
    timeout(5000) => handleTimeout(),
}
```

---

## 十一、 官方标准库协议矩阵 (`cap:*`)

| 标准库协议 | 核心导出符号 | 职责说明 |
| :--- | :--- | :--- |
| `cap:test` | `test`, `expect_eq` | 原生单元测试与断言框架 |
| `cap:sync` | `Mutex`, `RwLock`, `Condition`, `Channel`, `MpmcQueue`, `Futex` | 工业级无锁与线程同步原语 |
| `cap:crypto` | `sha256`, `sha512`, `randomBytes`, `fillRandom`, `randomUUID`, `Aes256Gcm`, `Hmac`, `hmacSha256Raw` | 密码学安全随机数、散列与 AEAD 加密 |
| `cap:event_loop` | `EventLoop`, `Timer`, `TimerHeap`, `ConcurrentQueue`, `PollEngine`, `PollEvents` | 统一高性能事件循环驱动 |
| `cap:http` | `HttpClient`, `HttpServer`, `Request`, `Response`, `Headers`, `H2ClientSession`, `H3ClientSession` | 现代 HTTP/1.1、HTTP/2、HTTP/3 全协议通信栈 |
| `cap:encoding` | `Base64` | 高性能 Base64 / Base64URL 编解码 |
| `cap:router` | `Router`, `Route` | 极速 Radix-Tree 路由匹配引擎 |
| `cap:io` | `Reader`, `Writer`, `BufferedReader`, `BufferedWriter`, `BufferReader`, `copyStream` | 统一流契约与零拷贝管道 |
| `cap:fs` | `readFile`, `writeFile`, `exists`, `stat`, `tmpDir`, `tmpPath`, `writeAtomic` | 现代文件系统高阶抽象 |
| `cap:json` | `parse`, `stringify`, `Value` | 零拷贝流式 JSON 解析框架 |
| `cap:security` | `CSRF`, `TokenFormat`, `constantTimeEql` | 安全防护与防跨站请求伪造子系统 |

---

## 十二、 智能体 Top 15 绝对禁手清单 (Top 15 Antipatterns)

| 禁手序号 | 错误幻觉写法 (WRONG) | CP 唯一合法写法 (CORRECT) | 根因与规则说明 |
| :---: | :--- | :--- | :--- |
| **#1** | `var x: i32 = 0;` | `let x: i32 = 0;` | CP 语法中不存在 `var` 关键字（`var` 是 Zig 遗留），可变变量 100% 写作 `let` |
| **#2** | `let mut x = 1;` | `let x = 1;` | CP 彻底废除 `let mut`，声明可变变量直接写 `let` |
| **#3** | `try func();` (前缀 try) | `func()?;` (后缀问号) | 彻底废除 Zig 前缀 `try`，统一使用后缀 `?` 进行错误冒泡传播 |
| **#4** | `for (0..n) \|i\|` 或 `for (list) \|x\|` | `for (i of 0..n)` / `for (x of list)` | 循环一律使用 `for (... of ...)`，严禁 Zig 的竖线管道捕获 `\|x\|` |
| **#5** | `cond1 and cond2` / `cond1 or cond2` | `cond1 && cond2` / `cond1 \|\| cond2` | 逻辑与/或使用纯正 C/TS 符号 `&&`、`\|\|`，词法解析器无 `and`/`or` 关键字 |
| **#6** | `fn f(): !usize` 或 `fn f(): ?User` | `fn f(): usize!` / `fn f(): User?` | 废除前缀 `!T`/`?T`，确立类型主体居首的纯正纯后缀修饰符体系 |
| **#7** | `Point{ .x = 1, .y = 2 }` | `Point{ x: 1, y: 2 }` | 结构体初始化采用 TS 风格（无前导点），严禁 Zig 的 `.{ .x = ... }` |
| **#8** | `const p = .{ 10, 20 };` | `const p = [10, 20];` | 元组/数组字面量使用方括号 `[...]`，严禁 Zig 匿名元组 `.{ ... }` |
| **#9** | `const o = .{ .x = 1 };` | `const o = { x: 1 };` | 匿名对象字面量使用花括号 `{ x: 1 }`，严禁 Zig 的 `.{ .x = 1 }` |
| **#10** | `items: []*Timer = &.{};` | `items: []*Timer = [];` | 切片字面量使用方括号 `[]`，严禁 Zig 的 `&.{}` |
| **#11** | `__zig__ { ... }` | `zig { ... }` | 内嵌 Zig 原生块必须使用 `zig {}`，严禁带下划线的 `__zig__` |
| **#12** | `try { ... } catch (e) { ... }` | `const val = expr catch fallback;` | CP 彻底废除 Java/JS 风格的 try-catch 异常块 |
| **#13** | `catch \|_\| { return false; }` | `catch return false;` | 废除 Zig 风格的 catch 错误捕获管道，直接接降级表达式或语句 |
| **#14** | `const [first, _] = pair;` | `const [first, *] = pair;` | CP 的模式匹配通配符是 `*`（而非 `_`），`_` 是方法接收者专属保留字 |
| **#15** | 根目录创建同名门面（如 `lib/io.cp`） | 仅在 `lib/io/index.cp` 导出单一真理来源 | 根目录同名门面已全面废弃，编译器原生优先目录寻路，避免冲突 |
