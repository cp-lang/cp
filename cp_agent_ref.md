# CP 智能体权威语法契约规范手册 (cp_agent_ref.md)

> **文档性质**：CP 语言面向 AI 智能体（AGY）的唯一最高权威精简语法与语义契约。  
> **设计责任**：CP 架构团队权威制定并全权负责，作为主智能体与协同智能体编写代码的唯一规范准绳。  
> **核心原则**：纯正 CP 语法，100% 形式化契约与正规精炼示例，坚决杜绝 Zig / Rust / TypeScript / Python 语法幻觉渗漏。  
> **人类文档对照**：面向人类开发者的详尽 RFC 演化、历史论证与宏观设计详见 `cp_ref.md`。

---

## 一、 模块系统与导入导出 (Modules & Imports)

### 1. 唯一合法导入语法
```typescript
// 1. CP 模块与相对路径导入（大括号具名导入）
import { Router, Route } from "./router";
import { HttpClient } from "../http/client.cp";

// 2. 官方标准库协议导入（cap:* 统一协议通道）
import { Mutex, Channel, RwLock } from "cap:sync";
import { sha256, Aes256Gcm, randomBytes } from "cap:crypto";
import { EventLoop, Timer, TimerHeap } from "cap:event_loop";
import { test, expect_eq } from "cap:test";
import { Base64 } from "cap:encoding";
import { Reader, Writer, BufferedReader } from "cap:io";

// 3. 第三方包导入（自动向上攀升寻找 cap_modules/）
import { Matrix } from "tensor-math";

// 4. 默认导入与混合导入
import App from "./app.cp";
import App, { start } from "./app.cp";

// 5. 别名导入
import { query_user as getUser } from "./db";
```

- **绝对禁手**：
  - 严禁 `require(...)`；
  - 严禁 `use xxx;`；
  - 严禁 `import * as foo from "..."`；
  - 严禁在函数或局部代码块内部写 `import` / `export`。

### 2. 导出与聚合重导出 (Exports & Re-exports)
```typescript
// 1. 声明并直接导出
export const MAX_BUFFER_SIZE: usize = 65536;
export fn add(a: i32, b: i32): i32 { return a + b; }
export struct Point { pub x: i32 = 0; pub y: i32 = 0; }
export class Engine { ... }
export enum State { Idle, Running }
export trait Reader { fn read(*_, buf: []u8): usize!; }
export default class AgentCore { ... }

// 2. 聚合重导出（多文件标准库 index.cp 唯一出口规范）
export { Reader, Writer, Closer } from "./traits.cp";
export { BufferReader, BufferWriter } from "./buffer.cp";
export * from "./sub_module.cp";
```

- **架构铁律**：复合多文件库统一使用 `lib/<name>/index.cp` 作为唯一对外出口，根目录下严禁创建多余同名门面文件（如严禁创建 `lib/io.cp`，统一经由 `lib/io/index.cp` 或 `cap:io` 导出）。

---

## 二、 变量绑定、可变性与参数借用 (Bindings & Mutability)

### 1. 变量声明矩阵
| 语法 | 语义 | 规则约束 |
| :--- | :--- | :--- |
| `const x: i32 = 42;` | 不可变局部/全局绑定 | 默认首选，值与内部属性均不可变 |
| `let y: i32 = 0;` 或 `var y: i32 = 0;` | 局部可变变量 | 允许重新赋值与字段修改，**严禁 `let mut`** |
| `threadlocal var counter: usize = 0;` | 线程局部变量 | 独占当前线程存储 |

### 2. 函数形参传递模式矩阵
| 参数声明 | 传递语义 | 内部权限 | 调用方状态 |
| :--- | :--- | :--- | :--- |
| `fn foo(item: T)` | 只读借用（默认） | 内部只能读取，不可修改字段 | 保持所有权，不被修改 |
| `fn foo(*item: T)` | 可变借用指针 | 允许就地修改内部字段 | 保持所有权，原数据被修改 |
| `fn foo(mut val: i32)` | 值传递可变本地拷贝 | 允许在函数内修改本地拷贝 `val` | 调用方值不受影响 |
| `fn foo(own item: T)` | 独占所有权转移 | 函数接管生命周期与析构 | 调用方放弃所有权，不可再用 |

### 3. 借用修改限制（Const Mutation Prevention）
- 若需修改变量内部字段，或调用会变异容器的方法（如 `push`、`pop`、`set`），**该变量必须声明为 `let` 或 `var`**；
```typescript
// ❌ 错误：const 变量不可修改属性或调用变异方法
const heap = TimerHeap.init();
heap.push(timer); // 编译器拦截报错！

// ✅ 正确：使用 let 声明
let heap = TimerHeap.init();
heap.push(timer); // PASS
```

---

## 三、 静态强类型系统与纯正后缀修饰符体系 (Types & Suffix Modifiers)

### 1. 标量基元
- **强类型整数**：`i8`, `u8`, `i16`, `u16`, `i32`, `u32`, `i64`, `u64`, `i128`, `u128`, `isize`, `usize`。
- **浮点与布尔**：`f32`, `f64`, `bool` (`true` / `false`), `void`。
- **字符串与 C 字符串**：
  - `string`：一等公民只读 UTF-8 字节切片（等价于 `[]const u8`）；
  - `cstr`：以 `\0` 结尾的 C 字符串裸切片（映射为 `[*:0]const u8`），脱离 `unsafe` 限制。
- **整数推断约束**：整数字面量默认为 `i32`。给 `usize` 变量赋值建议使用显式转换或依靠类型推导：
  ```typescript
  let len: usize = @as(usize, 0); // 或 let len = 0;
  ```

### 2. 纯正纯后缀修饰符体系 (STRICT POSTFIX NOTATION)
> **最高铁律**：CP 彻底废除前缀 `?T` 与 `!T`，确立“类型主体永远居首、修饰状态纯后缀链式向右展开”：

| 类型标注 | 语义说明 | 映射底层 | 典型示例 |
| :--- | :--- | :--- | :--- |
| `T?` | **可选类型**（可能为 null） | `?T` | `let name: string? = null;` |
| `T!` | **推导错误联合**（可能抛出错误） | `!T` | `fn read(): usize! { ... }` |
| `T!ErrorSet` | **具名错误联合**（指定错误集合） | `ErrorSet!T` | `fn open(): File!IOError { ... }` |
| `T?!` | **可空且可能报错** | `!?T` | `fn find(): User?! { ... }` |
| `void!` 或 `!` | **无返回值但可能报错** | `!void` | `fn flush(): ! { ... }` |

### 3. 指针与切片矩阵
| 语法 | 语义 | 安全级别 | 使用场景 |
| :--- | :--- | :--- | :--- |
| `&T` | 只读安全借用 | 安全 | 只读传递不可变数据视图 |
| `*T` | 可变安全借用指针 | 安全 | 修改对象字段或数组元素 |
| `[]T` | 动态可变切片 | 安全 | 可读可写缓冲区，元素可就地修改 |
| `[]const T` | 动态只读切片 | 安全 | 字符串、常量切片视图 |
| `[N]T` | 栈分配定长数组 | 安全 | 固定长度缓存，如 `let buf: [64]u8 = undefined;` |
| `*any!` | 通用非类型化裸指针 (非空) | unsafe | 对应 `*anyopaque`，用于跨 FFI 上下文传递 |
| `*any?!` | 通用非类型化裸指针 (可空) | unsafe | 对应 `?*anyopaque`，用于用户 Context 回调 |
| `*T!` | 类型化单项裸指针 | unsafe | 底层指针解引用操作 `ptr.*` |
| `[*]T` | 未定长多项裸指针 | unsafe | 底层 C 数组首地址与指针偏移算术 |

---

## 四、 复合数据类型：结构体 (`struct`) vs 类 (`class`)

### 1. 结构体 (`struct`) —— 值语义（栈分配，零额外开销）
结构体专用于轻量级数据载荷，无继承，支持默认字段值，采用 TypeScript 风格对象字面量初始化：
```typescript
export struct Point {
    pub x: i32 = 0;
    pub y: i32 = 0;

    // 实例方法（只读接收者完全省略）
    pub fn distanceSq(): i32 {
        return x * x + y * y;
    }

    // 状态修改方法（显式标注 *_ 可变借用）
    pub fn translate(*_, dx: i32, dy: i32) {
        x += dx;
        y += dy;
    }
}

// 结构体实例化语法（严禁 Zig 的 .x = 1 前导点风格！）
let pt = Point{ x: 10, y: 20 };
pt.translate(5, -5);
```

### 2. 类 (`class`) —— 引用语义（堆分配，支持 RC/RAII 自动生命周期）
类用于具备独立生命周期和复杂行为的对象，使用 `new` 实例化，支持构造/析构与方法分发：
```typescript
export class TimerQueue {
    capacity: usize = 64;
    count: usize = 0;

    // 构造函数
    pub fn init(*_, initial_cap: usize) {
        capacity = initial_cap;
        count = 0;
    }

    // 只读方法（可省略接收者，直接访问成员）
    pub fn len(): usize {
        return count;
    }

    // 变异方法（显式标注 *_ 可变接收者）
    pub fn push(*_, timer: *Timer) {
        count += 1;
    }

    // 静态工厂方法（无接收者）
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

### 3. 方法接收者（Receiver）终极规则：`_`
- 在 CP 中，**`_` 是方法接收者（Receiver）的唯一专属符号**（等同于 TS 中的 `this`）；
- **只读实例方法**：形参列表中的接收者**完全省略**，方法体内直接裸写字段名（如 `return count;`）；
- **变异实例方法**：形参列表必须显式声明 **`*_`**（可变借用）；
- **遮蔽消除**：仅当局部变量或形参与成员字段同名时，写 `_.count` 消除二义性；
- **静态方法**：必须添加 `static` 修饰符，无接收者。

### 4. 联合体 (`union(enum)`) 与代数数据类型
```typescript
// 带标签联合体
export union(enum) Message {
    Quit,
    Move: Point,
    Write: string,
}

// 现代枚举（支持绑定静态工具方法）
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

## 五、 字面量、解构赋值与模式匹配 (Literals & Destructuring)

### 1. 纯正 CP 字面量标准
- **数组/元组字面量**：`const pair = [10, 20];`（方括号，严禁 Zig 的 `.{ 10, 20 }`）；
- **对象字面量**：`const obj = { name: "cp", port: 8080 };`（花括号，严禁 Zig 的 `.{ .name = ... }`）；
- **空切片**：`let slice: []u8 = [];`（方括号，严禁 Zig 的 `&.{}`）；
- **哈希字典字面量**：`let dict = #{ "key": "value" };`。

### 2. 解构赋值语法 (TypeScript 风格)
```typescript
// 1. 元组与数组解构
const [a, b] = [100, 200];
const [head, ...tail] = [1, 2, 3, 4]; // tail 展开为切片

// 2. 结构体与对象解构
const pt = Point{ x: 10, y: 20 };
const { x, y } = pt;

// 3. 忽略丢弃元素（使用通配符 *，严禁使用 _ 丢弃，因为 _ 是接收者保留字）
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
```

---

## 六、 控制流与现代迭代 (Control Flow)

### 1. 条件语句与海象操作符 (`:=`)
> **强制规则**：`if`, `while`, `for` 条件必须加小括号 `(...)`。
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

// 3. 集合可变遍历（就地修改元素）
for (*item of items) {
    item.active = true;
}

// 4. while 循环
while (loop.hasActiveHandles() && !loop.stopped) {
    _ = loop.tick(null);
}
```

---

## 七、 现代错误处理与 RAII 确定性清理 (Error Handling & Defer)

### 1. 废除 `try/catch`，统一使用 `?`、`catch` 与 `??`
> **绝对禁手**：
> - 严禁 `try { ... } catch (e) { ... }` 代码块；
> - 严禁 Zig 关键字前缀 `try expr;`，统一使用后缀冒泡操作符 `expr?`；
> - 严禁 Zig 管道捕获 `catch |_| { ... }`。

```typescript
// 1. 后缀错误冒泡符 '?'（要求调用方函数签名返回类型带 !）
const file = openFile("config.json")?;

// 2. 错误捕获降级 (catch fallback)
const port = parsePort(port_str) catch 8080;

// 3. 错误捕获并早退
const data = parseData() catch return false;

// 4. 断言绝对无错误 (catch unreachable)
const buf = allocator.alloc(u8, 1024) catch unreachable;

// 5. 空值合并操作符 '??'（仅针对可选类型 T? 的 null，不捕获错误）
const host = opt_host ?? "127.0.0.1";
```

### 2. 确定性资源释放 (`defer` 与 `errdefer`)
```typescript
fn processFile(path: string): void! {
    let file = openFile(path)?;
    defer file.close(); // 作用域退出时无论正常还是报错必执行

    let conn = connectServer()?;
    errdefer conn.abort(); // 仅当后续代码发生错误提前退出时触发

    file.pipeTo(conn)?;
}
```

---

## 八、 底层互操作与 C-FFI (FFI & Escape)

```typescript
// 1. 内联 C 声明块
@c {
    #include <unistd.h>
    #include <sys/socket.h>
}

// 2. 外部符号绑定
extern "c" fn getpid(): i32;
extern "c" var errno: i32;

// 3. 原生 Zig 代码块逃逸（仅限复杂 OS 底层系统调用）
fn getMonotonicMs(): i64 {
    __zig__ {
        var ts: std.os.linux.timespec = undefined;
        _ = std.os.linux.clock_gettime(std.os.linux.CLOCK.MONOTONIC, &ts);
        const sec: i64 = @as(i64, @intCast(ts.sec));
        const nsec: i64 = @as(i64, @intCast(ts.nsec));
        return sec * 1000 + @divTrunc(nsec, 1000000);
    }
}
```

---

## 九、 官方标准库协议矩阵 (`cap:*`)

| 标准库协议 | 核心导出符号 | 职责说明 |
| :--- | :--- | :--- |
| `cap:test` | `test`, `expect_eq` | 原生单元测试与断言框架 |
| `cap:sync` | `Mutex`, `RwLock`, `Condition`, `Channel`, `MpmcQueue`, `Futex` | 工业级无锁与线程同步原语 |
| `cap:crypto` | `sha256`, `sha512`, `randomBytes`, `randomUUID`, `Aes256Gcm`, `Hmac` | 密码学安全随机数、散列与 AEAD 加密 |
| `cap:event_loop` | `EventLoop`, `Timer`, `TimerHeap`, `ConcurrentQueue`, `PollEngine`, `PollEvents` | 统一高性能事件循环驱动 |
| `cap:http` | `HttpClient`, `HttpServer`, `Request`, `Response`, `Headers`, `H2ClientSession`, `H3ClientSession` | 现代 HTTP/1.1、HTTP/2、HTTP/3 全协议通信栈 |
| `cap:encoding` | `Base64` | 高性能 Base64 / Base64URL 编解码 |
| `cap:router` | `Router`, `Route` | 极速 Radix-Tree 路由匹配引擎 |
| `cap:io` | `Reader`, `Writer`, `BufferedReader`, `BufferedWriter`, `BufferReader`, `copyStream` | 统一流契约与零拷贝管道 |
| `cap:fs` | `readFile`, `writeFile`, `exists`, `stat` | 现代文件系统高阶抽象 |
| `cap:json` | `parse`, `stringify`, `Value` | 零拷贝流式 JSON 解析框架 |
| `cap:security` | `CSRF`, `TokenFormat`, `constantTimeEql` | 安全防护与防跨站请求伪造子系统 |

---

## 十、 智能体 Top 15 绝对禁手清单 (Top 15 Antipatterns)

| 禁手序号 | 错误幻觉写法 (WRONG) | CP 唯一合法写法 (CORRECT) | 根因与规则说明 |
| :---: | :--- | :--- | :--- |
| **#1** | `try func();` (前缀 try) | `func()?;` (后缀问号) | 彻底废除 Zig 前缀 `try`，统一使用后缀 `?` 进行错误冒泡传播 |
| **#2** | `for (0..n) \|i\|` 或 `for (list) \|x\|` | `for (i of 0..n)` / `for (x of list)` | 循环一律使用 `for (... of ...)`，严禁 Zig 的竖线管道捕获 `\|x\|` |
| **#3** | `cond1 and cond2` / `cond1 or cond2` | `cond1 && cond2` / `cond1 \|\| cond2` | 逻辑与/或使用纯正 C/TS 符号 `&&`、`\|\|`，词法解析器无 `and`/`or` |
| **#4** | `fn f(): !usize` 或 `fn f(): ?User` | `fn f(): usize!` / `fn f(): User?` | 废除前缀 `!T`/`?T`，确立类型主体居首的纯正后缀修饰符体系 |
| **#5** | `Point{ .x = 1, .y = 2 }` | `Point{ x: 1, y: 2 }` | 结构体初始化采用 TS 风格（无前导点），严禁 Zig 的 `.{ .x = ... }` |
| **#6** | `const p = .{ 10, 20 };` | `const p = [10, 20];` | 元组/数组字面量使用方括号 `[...]`，严禁 Zig 匿名元组 `.{ ... }` |
| **#7** | `const o = .{ .x = 1 };` | `const o = { x: 1 };` | 匿名对象字面量使用花括号 `{ x: 1 }`，严禁 Zig 的 `.{ .x = 1 }` |
| **#8** | `items: []*Timer = &.{};` | `items: []*Timer = [];` | 切片字面量使用方括号 `[]`，严禁 Zig 的 `&.{}` |
| **#9** | `const p = alloc(); p.x = 1;` | `let p = alloc(); p.x = 1;` | CP 严格拦截对 `const` 变量的属性变异与变异方法调用，必须为 `let` |
| **#10** | `let mut x = 1;` | `let x = 1;` 或 `var x = 1;` | CP 无 `mut` 局部绑定修饰词，直接用 `let` 或 `var` 声明局部可变 |
| **#11** | `try { ... } catch (e) { ... }` | `const val = expr catch fallback;` | CP 彻底废除 Java/JS 风格的 try-catch 代码块 |
| **#12** | `catch \|_\| { return false; }` | `catch return false;` | 废除 Zig 风格的 catch 错误捕获管道，直接接降级表达式或语句 |
| **#13** | `const [first, _] = pair;` | `const [first, *] = pair;` | CP 的模式匹配通配符是 `*`（而非 `_`），`_` 是方法接收者保留字 |
| **#14** | `1_000_000`（数字带下划线） | `1000000`（无下划线数字） | 词法分析器暂不支持下划线分隔数字，会解析为非法标识符导致 E0206 |
| **#15** | 根目录创建同名门面（如 `lib/io.cp`） | 仅在 `lib/io/index.cp` 导出单一真理来源 | 根目录同名门面已全面废弃，编译器原生优先目录寻路，避免冲突 |
