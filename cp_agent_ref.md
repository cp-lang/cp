# CP 智能体权威语法契约规范手册 (cp_agent_ref.md)

> **文档性质**：CP 语言面向 AI 智能体（AGY）的唯一最高权威精简语法与语义契约。  
> **设计责任**：CP 架构团队权威制定并全权负责，为主/子智能体与下游协同智能体编写 CP 代码的唯一规范准绳。  
> **核心原则**：纯正 CP 语法，100% 形式化规则与正规精炼示例，坚决杜绝 Zig / Rust / TypeScript / Python 语法幻觉渗漏。  
> **权威对照**：面向人类开发者的详尽 RFC 演化、历史论证、宏观哲学与系统白皮书全量记载于 `cp_ref.md`。

---

## 一、 模块系统与跨语言导入导出 (Modules, Package System & Zig Interop)

### 1. 唯一合法导入语法 (`import`)
```typescript
// 1. CP 模块具名导入与相对路径导入
import { Router, Route } from "./router";
import { HttpClient } from "../http/client.cp";

// 2. 命名空间全量导入 (import * as)
import * as math from "./utils/math";

// 3. 官方标准库协议导入（cap:* 统一协议通道）
import { Mutex, RwLock, Channel } from "cap:sync";
import { sha256, Aes256Gcm, randomBytes } from "cap:crypto";
import { EventLoop, Timer, TimerHeap } from "cap:event_loop";
import { Reader, Writer, BufferedReader, copyStream } from "cap:io";
import { test, expect_eq } from "cap:test";
import { Base64 } from "cap:encoding";
import { CSRF, constantTimeEql } from "cap:security";

// 4. 第三方包导入（自动触发向上攀升查找引擎）
import { Matrix } from "tensor-math";

// 5. 默认导入与混合导入
import App from "./app.cp";
import App, { start } from "./app.cp";

// 6. 别名导入 (as)
import { query_user as getUser } from "./db";
```

- **绝对禁手**：严禁 `require(...)` 与 CommonJS 动态加载；严禁 `use xxx;`；严禁在函数、类或局部代码块内部写 `import` / `export`。

### 2. 直接透明引入 Zig 模块与 `@import` 语法
> **工程规范**：直接引入 `.zig` 源代码是 CP 迁移 Bun 资产的核心过渡桥梁（在未完成全量 CP 重写时使用）：
```typescript
// 姿态 1：ES6 模块语法直接引入未迁移的 Zig 模块
import DirInfo from "../resolver/dir_info.zig";
import Fs from "../resolver/fs.zig";

// 姿态 2：原生 Zig 风格 @import 直接导入（完全合法）
const bun = @import("bun");
const std = @import("std");

// 前柔后刚两级类型模型：
// 1. 前端阶段 (cpc)：被导入的 Zig 对象赋予透明穿透的 Type::Any，不因未展开 Zig 宏而中断
// 2. 后端阶段 (Zig AOT)：调用 Zig 编译器进行全量类型检查与单态化，绝对保障强类型安全

// 3 种标准工程姿态：
// 姿态 A (深层类型别名透传):
pub type LocalFD = bun.FD;
pub type LocalDirInfo = DirInfo;

// 姿态 B (边界显式类型收束):
let max_conn: u32 = bun.getMaxConnections() as u32;

// 姿态 C (防腐层包装规范):
// 底层以 _ 开头的文件（如 _boringssl.zig, _usockets.cp）负责私有绑定；
// 对外面向用户的模块（如 socket.cp, fs.cp）包装为纯 Safe 的 CP class 或 struct 导出。
```

### 3. 导出与聚合重导出 (`export` / `pub`)
```typescript
// 1. 声明并直接导出
export const MAX_BUFFER_SIZE: usize = 65536;
export let global_status: i32 = 0;
export fn add(a: i32, b: i32): i32 { return a + b; }
export @c struct Point { pub x: i32 = 0; pub y: i32 = 0; }
export class Engine { ... }
export enum State { Idle, Running }
export trait Reader { fn read(*_, buf: []u8): usize!; }
export default class AgentCore { ... }

// 2. 聚合重导出（index.cp 单一真理来源出口规范）
export { Reader, Writer, Closer } from "./traits.cp";
export { BufferReader, BufferWriter } from "./buffer.cp";
export { add as sum, PI as TAU };
export * from "./sub_module.cp"; // 批量重导出
```

- **架构铁律**：复合多文件库统一使用 `lib/<name>/index.cp` 作为唯一对外出口，根目录下严禁创建多余同名门面文件（如严禁创建 `lib/io.cp`，统一经由 `lib/io/index.cp` 或 `cap:io` 导出）。

---

## 二、 变量绑定、可变性与所有权/借用检查体系 (Bindings, Ownership & Borrowing)

### 1. 变量声明矩阵
CP 彻底废除 `let mut`、`var` 与 `threadlocal var`，全面拥抱清晰确定的可变性语义：
- **`let`**：可变变量声明。局部变量写 `let x = ...;`，模块顶层可变变量写 `export let g_state = ...;`。允许重新赋值与就地修改内部字段；
- **`const`**：常量与深度不可变声明。阻止一切重新赋值与内部属性修改；
- **`@tls let`**：线程局部存储变量声明（对标 Zig `threadlocal var`，严禁书写 `threadlocal var`）；
- **🚨 绝对禁止 `var`**：CP 中不存在 `var` 关键字（`var` 是 Zig 遗留语法），源代码中声明可变变量必须 100% 写作 `let`；
- **🚨 绝对禁止 `let mut`**：CP 声明可变变量直接写 `let`，禁止 `let mut`。

### 2. 所有权转移与自动释放 (Ownership & Auto-Drop)
- **单写者排他所有权**：变量赋给另一个变量时，值语义进行物理拷贝，引用语义转移所有权；
- **显式所有权转移**：使用 **`move val`** 或 **`own val`**；移交后，原调用方变量即刻失效并被借用检查器拦截；
- **确定性析构 (Auto-Drop)**：离开作用域时，实现了 `release(*_)` 的类型自动触发析构，零 GC 运行。

### 3. 借用检查器（Borrow Checker）与多字段分离借用
- **读写互斥原则**：在同一生命周期内，允许存在多个只读借用（`&T`），或仅允许存在**唯一一个**可变借用（`*T`）。可变借用与只读借用严格互斥；
- **多字段分离借用 (Disjoint Field Borrowing)**：
  - 借用检查器将借用追踪升级为 `(root_var, field_path)` 细粒度元组；
  - 只要两个借用指向同一对象的不同字段（如 `*data.q_len` 与 `*data.active_timer`），编译器判定为 Disjoint 独立访问，**允许同时持有多个可变借用**，无需打散结构体；
  - 借用整个结构体（`*data`）依然与任何子字段借用保持排他互斥。

```typescript
class InternalLoopData {
    q_len: i32;
    active_timer: i32;
}

fn process_loop(*data: InternalLoopData) {
    // 多字段分离借用：安全同时持有两个不同字段的可变借用
    let q = *data.q_len;
    let t = *data.active_timer;
    q.* = 42;
    t.* = 100;
}
```

### 4. 栈绑定视图与生命周期逃逸保护 (`ref struct`)
- **`ref struct`**：声明栈绑定只读视图结构体，生命周期严格锁定在当前调用栈帧，**物理上严禁跨越 `spawn` / `await` 异步边界逃逸**；
- **显式生命周期参数**：支持 `*(a)T`、`&(a)T` 标注关联生命周期，多数场景由编译器非词法生命周期（NLL）全自动推导释放。

---

## 三、 静态强类型系统、C-ABI 指针与纯后缀修饰符 (Types & Pointers)

### 1. 标量类型与整型/枚举双向转换
- **整数**：`i8`, `u8`, `i16`, `u16`, `i32`, `u32`, `i64`, `u64`, `i128`, `u128`, `isize`, `usize`；
- **浮点与逻辑**：`f32`, `f64`, `bool` (`true` / `false`), `void`；
- **整型与枚举显式双向智能转换 (`as`)**：
  - `kind as i32`：枚举转整数（发射 `@intFromEnum`）；
  - `raw_int as SocketKind`：整数转枚举（发射 `@enumFromInt`）；
- **类型显式转换**：数值转换使用 `@as(TargetType, expr)` 或 `@intCast(expr)`。整数字面量默认为 `i32`，给 `usize` 赋值显式使用 `@as(usize, 0)` 或依靠类型推导；
- **未初始化内存**：使用 `undefined` 声明未初始化的栈缓冲区（如 `let buf: [64]u8 = undefined;`）。业务中的“空”必须使用可选类型 `T?` 与 `null`，严禁滥用 `undefined`。

### 2. 编译期自动推导形参占位符 (`auto`)
> **核心规范**：官方首选语法为 **`auto`**（对标 Zig 的 `anytype`，`anytype` 仅作历史兼容）：
- 仅用于函数形参以触发编译期单态化推导：`pub fn print(v: auto) { ... }`；
- 结合编译期参数：`pub fn wrapper(@comptime callback: auto) { ... }`；
- 🚨 严禁在局部变量（`let x: auto`）或结构体字段中声明 `auto`。

### 3. 一等公民 C 字符串体系：`cstr` vs `cstring`
| 字符串类型 | 底层物理映射 | 是否带长度 | 安全级别 | 典型应用场景 |
| :--- | :--- | :---: | :---: | :--- |
| **`string`** | `[]const u8` (切片) | ✅ 带长度 | Safe | CP 原生字符串，只读字节切片 |
| **`cstring`** | `[:0]const u8` (const) / `[:0]u8` (let) | ✅ 带长度 (.len) | Safe | 附带 `\0` 哨兵的胖切片，高频安全 C 字符串操作 |
| **`cstr`** | `[*:0]const u8` (const) / `[*:0]u8` (let) | ❌ 无长度 (薄指针) | Safe/Borrow | C 原生以 0 结尾字符串薄指针，用于高效系统 FFI 传参 |

- **多态字符串转换**：使用 `string.from(cstr_val)` 零开销将 `cstr` 转换为 `string`；`string` 可零拷贝直通传给接收只读 C 缓冲区的接口。

### 4. 纯正纯后缀修饰符体系 (STRICT POSTFIX NOTATION)
> **最高设计铁律**：CP 彻底废除前缀 `?T` 与 `!T`，确立“类型主体永远居首、修饰状态纯后缀链式向右展开”：

| 类型标注 | 语义说明 | 映射底层 | 典型示例 |
| :--- | :--- | :--- | :--- |
| `T?` | **可选类型**（可能为 null） | `?T` | `let name: string? = null;` |
| `T!` | **全局推导错误联合**（等价于 `T ! AnyErr`） | `anyerror!T` | `fn read(): usize! { ... }` |
| `T ! ErrorSet` | **具名错误联合**（抛出显式错误集） | `ErrorSet!T` | `fn open(): File ! FileErr { ... }` |
| `T ! Err` | **业务动态错误**（返回 Err 结构体） | `CpResult(T, Err)` | `fn query(): User ! Err { ... }` |
| `T?!` | **可空且可能报错** | `!?T` | `fn find(): User?! { ... }` |
| `void!` 或 `!` | **无成功返回值但可能报错**（推荐极简 `!`） | `!void` | `fn flush(): ! { ... }` |

### 5. 全面 C-ABI 与系统级指针矩阵 (Global Symbol Matrix)
| 语法标注 | 核心语义与底层映射 | 安全级别 | 典型场景 |
| :--- | :--- | :--- | :--- |
| `&T` | **只读安全借用** | 安全 | 函数形参传递不可变数据视图，零拷贝只读 |
| `*T` | **安全可变借用** | 安全 | 独占写权限借用，就地修改内部字段 |
| `move T` / `own T` | **所有权转移** | 安全 | 独占所有权移交，调用方变量即刻失效 |
| `*T!` | **类型化单项裸指针**（解引用 `ptr.*`） | unsafe | 仅在 `unsafe {}` 内合法，底层操作 `ptr.*` |
| `*!` | **非空通用弱类型裸指针**（`*anyopaque`） | unsafe | 对应 C `void*`，推荐省略 `any`，直接写 `*!` |
| `*!?` | **可选通用弱类型裸指针**（`?*anyopaque`） | unsafe | 对应 C 可空 `void*`，推荐省略 `any`，直接写 `*!?` |
| `[*c]T` / `[*c]T?` | **C-ABI 兼容指针**（映射 `[*c]T` / `?[*c]T`） | unsafe/FFI | 专用于 C 头文件对接，与 `&buf` / `*buf` 隐式无缝互通 |
| `[*:0]T!` / `[*:0]T?!` | **0 结尾哨兵裸指针**（映射 `[*:0]T`） | unsafe | C 风格字符串缓冲区底层操作 |
| `[*]([*:0]u8!)?!` | **字符串指针数组** | unsafe | 命令行参数 `argv` 或环境变量字符串指针连续数组 |
| `[*:null]([*:0]u8?!)?!`| **null 结尾的可选字符串指针数组** | unsafe | 系统 `posix_spawn` 环境表指针数组 |
| `[*]T` | **未定长多项裸指针** | unsafe | C 数组首地址与裸指针偏移算术 |
| `[]T` | **动态可变切片** | 安全 | 缓冲区切片视图，元素可就地修改 |
| `[]const T` | **动态只读切片** | 安全 | 常量视图或字符串（`string` 等价于 `[]const u8`） |
| `[N]T` | **栈分配定长数组** | 安全 | 固定长度本地缓存，如 `let buf: [64]u8 = undefined;` |

### 6. 类型别名 (`type`)
```typescript
export type FileDescriptor = i32;
export type ByteSlice = []const u8;
export type Callback = @c fn (ctx: *!?, status: i32): void;
```

### 7. 原生运算符矩阵
- **标准算术**：`+`, `-`, `*`, `/`, `%`
- **环绕运算（截断溢出）**：`+%`, `-%`, `*%`
- **饱和运算（边界钳位）**：`+|`, `-|`
- **位运算符**：`&`, `|`, `^`, `~`, `<<`, `>>`
- **逻辑运算符**：`&&`, `||`, `!`（**🚨 严禁使用 Python/Zig 的 `and` / `or`**）
- **管道操作符**：`|>`（如 `data |> filter() |> process()`）

---

## 四、 裸指针核心机制与 Unsafe 内存操作 (Raw Pointers & Unsafe)

### 1. 变量常量性与指针只读性的正交解耦
在 CP 中，“变量本身的重新赋值约束”与“指针指向的内存数据只读性”是两个完全解耦的维度：
1. **`const p: *T! = ...;`（指针自身不可变，指向的内存可写）**：
   - `const` 约束的是变量 `p` 自身，不能让 `p` 重新指向别的地址（禁止 `p = other;`）；
   - 但指针类型为 `*T!`，开发者依然可以通过 `p.* = val` 修改目标内存中的值。
2. **`let p: *const T! = ...;`（指针自身可变，指向的内存只读）**：
   - 类型为 `*const T!`，编译器在静态语义期**硬性拦截任何修改操作**（写入 `p.* = val` 抛出编译错误：`Cannot mutate through read-only pointer`）；
   - 但因为声明为 `let`，变量 `p` 可以被重新赋值指向另一个目标对象（允许 `p = other;`）。
3. **`const p: *const T! = ...;`（双重不可变）**：
   - 指针不可换，指向的内存数据亦绝对只读。

### 2. 获取变量裸指针的标准方式 (在 `unsafe` 块中)
在 CP 中，获取变量的底层裸指针支持以下两种标准方式（均须在 `unsafe` 上下文中进行）：
- **方式一：直接裸指针字面语法（`*v!` 与 `**v!`）**
  ```typescript
  let num: i32 = 100;
  unsafe {
      // 1. 获取一级裸指针 (*num!)
      let p: *i32! = *num!;
      p.* = 200; // num 变为 200

      // 2. 获取二级裸指针 (**p!)
      let p_copy: *i32! = p;
      let pp: **i32! = **p_copy!;
      pp.*.* = 300; // num 变为 300
  }
  ```
- **方式二：通过安全借用显式转换（`&v as *T!` 与 `*v as *T!`）**
  ```typescript
  let v: i32 = 42;
  unsafe {
      // 1. 从只读借用 &v 转换为只读裸指针 *const i32!
      let p_ro: *const i32! = &v as *const i32!;

      // 2. 从只读借用 &v 转换为可变裸指针 *i32! (底层强制升级，需由开发者负责)
      let p_rw: *i32! = &v as *i32!;

      // 3. 从可变借用 *v 转换为可变裸指针 *i32!
      let p_mut: *i32! = *v as *i32!;
      p_mut.* = 88; // v 变为 88
  }
  ```

### 3. 完整裸指针类型转换矩阵 (Type Casting Matrix in Unsafe)
所有涉及裸指针的类型转换均须在 `unsafe` 块内通过 `as` 完成：
```typescript
unsafe {
    let num: i32 = 1000;
    let p: *i32! = *num!;

    // 1. 裸指针按位重解释（指针类型互转）
    let p_u8: *u8! = p as *u8!;

    // 2. 裸指针转换为物理内存整型地址 (usize)
    let addr: usize = p as usize;

    // 3. 物理内存整型地址还原为强类型裸指针
    let p_restored: *i32! = addr as *i32!;
    print(p_restored.*); // 1000

    // 4. 裸指针转换为通用非类型化指针 (*! 或 *any!，等价 C void*)
    let generic: *! = p as *!;
    let back: *i32! = generic as *i32!;

    // 5. 数组切转为未定长多项裸指针 ([*]T!)
    let buf: [4]u8 = [1, 2, 3, 4];
    let many: [*]u8! = buf as [*]u8!;
    let val = many[0]; // 1
}
```

### 4. `zig { ... }` 原生逃逸舱语义
- **内嵌 Zig 代码完全脱离 CP 借用检查**，等价于最大程度 unsafe，由 Zig 自身的编译期规则管控；
- 专用于调用极端底层 OS 原生能力（如 `clock_gettime`、`epoll_create`）；
- 🚨 严禁使用带下划线的 `__zig__`。

---

## 五、 函数系统、形参契约与泛型 (Functions & Generics)

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

// 5. void 省略哲学（推荐极简签名）
fn logMessage(msg: string) { ... }       // 省略 ': void'
fn flushSocket(): ! { ... }              // 纯报错函数推荐省略 void，写作 ': !'

// 6. 编译期自动推导参数 (@comptime 与 auto)
fn handleCallback(@comptime callback: auto) { ... }
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

---

## 六、 面向对象与系统级内存布局 (@c, @root, @value, @align, @refCount)

### 1. 结构体 (`struct`) —— 值语义（纯栈分配，零额外开销）
```typescript
export struct Point {
    pub x: i32 = 0;
    pub y: i32 = 0;

    pub fn distanceSq(): i32 { return x * x + y * y; }
    pub fn translate(*_, dx: i32, dy: i32) { x += dx; y += dy; }
}

// 泛型结构体
export struct Vector<T> {
    pub items: []T = [];
    pub fn len(): usize { return items.len; }
}

// 实例化语法（严格禁止 Zig 的 .x = 1 前导点风格！）
let pt = Point{ x: 10, y: 20 };
```

### 2. 类 (`class`) —— 引用语义（堆分配，支持 RC/RAII 自动生命周期）
```typescript
export class TimerQueue {
    capacity: usize = 64;
    count: usize = 0;

    pub fn init(*_, initial_cap: usize) { capacity = initial_cap; count = 0; }
    pub fn release(*_) { /* 清理底层非托管资源 */ }

    pub static fn create(cap: usize): *TimerQueue {
        let q = new TimerQueue();
        q.init(cap);
        return q;
    }
}
let tq = new TimerQueue();
```

### 3. 系统级核心装饰器与内存布局体系（Bun 迁移必备）
| 装饰器注解 | 语义与底层发射 | 典型应用场景 |
| :--- | :--- | :--- |
| **`@c struct Bar { ... }`** | 严格遵循 C-ABI 内存物理对齐（对标 Zig `extern struct`，向下兼容 `@extern`） | 与 C 动态库/头文件物理结构体对齐（如 `phr_header`） |
| **`@root struct Name { ... }`** | 映射至 Zig 文件顶层根作用域 `@This()`，带 Prelude 运行时命名隔离 | 将 Zig“文件即结构体”模块无缝迁移至 CP 纯正结构体 |
| **`@value struct Compact { ... }`**| 值语义紧凑 POD 结构体，直接嵌入栈上或父结构体，按值传递无指针间接开销 | 短索引、句柄包装器，免除析构守卫 |
| **`@packed struct Foo { ... }`** | 紧凑位内存布局，无对齐填充，不生成默认 init | 二进制协议头、网络数据包位域 |
| **`@opaque struct/class Baz { ... }`**| 不透明抽象类型（映射为 `pub const Baz = opaque {};`） | 外部 C 返回的系统指针/句柄（`us_loop_t`） |
| **`@align(N)`** | 字段级或结构体级物理内存字节对齐注解（如 `@align(16) buf: [64]u8`） | SIMD 向量化计算、硬件高速缓存行对齐 |
| **`@refCount(cleanup)`** | 类顶层自动侵入式引用计数管理（提供 `.ref()` 与 `.deref()`） | 共享网络套接字、资源池自动归零析构（对标 `bun.ptr.RefCount`） |
| **`@callconv(.c)`** | 强制 C 函数调用约定 | C 回调函数跳板、动态链接库导出 |

### 4. 行为契约 (`trait`)、关联静态契约与实现 (`impl`)
```typescript
export trait Factory<T> {
    fn create(): T; // Trait 静态构造契约
}

export trait Measurable {
    fn area(_): f64;
}

export struct Circle { pub radius: f64 = 0.0; }

impl Measurable for Circle {
    fn area(_): f64 { return 3.1415926535 * _.radius * _.radius; }
}
```

### 5. 动态分发胖指针与运行时向下转型 (`dyn` & `downcast`)
```typescript
// 1. 异构容器持有胖指针 Box<dyn Trait>
let handlers: [Box<dyn EventHandler>] = [];
handlers.push(Box.new(NetworkHandler.init()));

// 2. 运行时安全向下转型 (Downcast)
for (handler of handlers) {
    if (net := handler.downcast(NetworkHandler)) {
        net.flush_socket();
    }
}
```

---

## 七、 字面量、解构赋值与全景 `switch` 模式匹配 (Destructuring & Switch)

### 1. 纯正 CP 字面量标准
- **数组/元组字面量**：`const pair = [10, 20];`（方括号，严禁 Zig 的 `.{ 10, 20 }`）；
- **匿名对象字面量**：`const obj = { name: "cp", port: 8080 };`（花括号，严禁 Zig 的 `.{ .name = ... }`）；
- **空切片**：`let slice: []u8 = [];`（方括号，严禁 Zig 的 `&.{}`）；
- **动态哈希字典字面量**：`let dict = #{ "key": "value" };`。

### 2. 现代解构赋值
```typescript
// 1. 元组与数组解构
const [a, b] = [100, 200];
const [head, ...tail] = [1, 2, 3, 4]; // tail 展开为切片视图

// 2. 结构体与对象解构
const pt = Point{ x: 10, y: 20 };
const { x, y } = pt;
const { id, ... } = user; // 忽略对象其他属性

// 3. 忽略丢弃单个元素（必须使用通配符 *，严禁使用 _，因为 _ 是方法接收者保留字）
const [first, *] = [10, 20];
```

### 3. 全景 `switch` 模式匹配体系（全解构形式）
```typescript
switch (expr) {
    // 1. 枚举变体与载荷元组解构
    .Move [x, y] => { updatePos(x, y); },

    // 2. 结构体载荷精确解构
    .Compute { batch, priority } => { processBatch(batch, priority); },

    // 3. 结构体剩余字段整包捕获（只读结构体变量 fields，零堆开销）
    .Compute { ...fields } => { print(fields.batch); },

    // 4. 单载荷变量捕获
    .Text text => { print(text); },

    // 5. 分支原位可变借用修改 (|*ptr|)
    .Active => |*u| { u.score += 10; },

    // 6. Walrus 子模式绑定 (ident := subpattern)
    s := 90..=100 => s * 2,

    // 7. 动态类型分支窄化匹配
    w: AIWorker => w.work(100),

    // 8. 多模式联合与区间匹配
    400 | 401 | 403 => "Client Error",
    1..=10 => "Small Range",

    // 9. if 守卫条件表达式
    { age, role } if age >= 18 && role == "Admin" => "Superuser",

    // 10. 编译期动态字段匹配
    @field(Tag, "foo") => 100,

    // 11. 原生三态解包匹配（成功值、错误、空值）
    ok_val => handleOk(ok_val),
    err(e: TimeoutError) => retry(e.retry_after), // 胖指针具体错误 downcast
    err(e := SqlError) => logSql(e.sql_state),     // Walrus 风格 downcast
    err.InvalidPadding => handleInvalidPadding(), // 匹配 AnyErr 即席静态标签
    err(e) => handleGenericErr(e),
    null => handleNull(),

    // 12. 通配兜底分支
    else => handleFallback(), // 或 * => 通配兜底
}
```

---

## 八、 控制流与流式类型智能收窄 (Control Flow & Type Narrowing)

### 1. 条件语句与海象解包 (`:=`)
> **强制规则**：`if`, `while`, `for` 条件头部必须加小括号 `(...)`。
```typescript
// 1. 标准条件分支（逻辑运算符必须用 && 和 ||，严禁 and / or）
if (count > 0 && is_active) { process(); } else { reset(); }

// 2. 海象非空解包赋值 (Walrus Unwrap)
if (timer := heap.pop()) {
    timer.callback(timer.ctx);
}
```

### 2. 流式类型智能收窄 (Flow-Sensitive Type Narrowing)
编译器内建控制流感知类型收窄，无需多余手工解包：
- **正向收窄**：`if (u != null)` 分支内，`u: User?` 自动收窄为 `User`；
- **短路收窄**：`if (u != null && u.age >= 18)`，`&&` 右侧自动享受左侧收窄；
- **早期返回卫语句收窄**：`if (ptr == null) return;` 之后代码中，`ptr` 自动为非空；
- **裸指针收窄**：`ptr: *T!?` 在判空 `ptr != null` 后，自动收窄为非空可解引用裸指针 `*T!`。

### 3. 循环遍历 (`for ... of`) 与标签控制
> **绝对禁手**：严禁 Zig 风格的 `for (collection) |item|` 或 `for (0..n) |i|`！
```typescript
// 1. 范围遍历（半开区间 0..n 或闭区间 0..=n）
for (i of 0..10) { print(i); }

// 2. 集合只读遍历 / 原地可变遍历 / 所有权转移遍历
for (item of items) { process(item); }
for (*item of items) { item.active = true; }
for (move item of items) { consume(item); }

// 3. while 循环与步进表达式 (: step)
let i: usize = 0;
while (i < 10) : (i += 1) { ... }

// 4. 具名循环标签与跨层跳出
outer: for (x of rows) {
    for (y of cols) {
        if (found(x, y)) break :outer;
    }
}

// 5. 显式丢弃表达式结果
_ = flushSocket();
```

---

## 九、 统一原生错误体系：AnyErr、动态业务 Err 与具名联合 (Unified Error Architecture)

### 1. 错误返回类型全景分类矩阵
在 CP 中，所有可产生错误的函数，其返回值类型在底层对应清晰的分层错误体系：

| 错误类型形态 | 函数返回值签名 | 抛出语法 | 底层物理映射 | 规则与约束说明 |
| :--- | :--- | :--- | :--- | :--- |
| **全局即席错误** | `T!` 或 `T ! AnyErr` | `return err.NotFound;` | Zig `anyerror!T` | `T!` 是 `T ! AnyErr` 的官方语法糖省略。**只能返回静态标签，严禁动态错误装箱** |
| **业务动态错误** | `T ! Err` 或 `T ! err` | `return err("msg");`<br>`return err(404, "msg");`| `CpResult(T, Err)` | 内置轻量结构体 `Err`（`code: i32, msg: string, cause: ?*anyopaque`），**签名必须显式标注 `T ! Err`** |
| **具名错误集合** | `T ! FileErr` | `return FileErr.NotFound;`<br>或 `return .NotFound;` | Zig `FileErr!T` | 使用 `err FileErr { NotFound, AccessDenied }` 声明。**函数内直接返回变体，无需 `err()` 包裹** |
| **多错误集合并** | `T ! FileErr \|\| NetErr` | `return FileErr.NotFound;`<br>`return NetErr.Timeout;` | Zig `(FileErr \|\| NetErr)!T` | 零成本原生联合，支持具名联合与匿名联合（如 `FileErr \|\| err{A, B}`） |
| **混合动态升格** | `T ! FileErr \|\| Err` | `return FileErr.NotFound;`<br>`return err(500, "fail");`| `CpResult(T, Err)` | 具名错误与动态错误混合联合，具名错误在必要时自动升格包装为 `Err` |
| **普通枚举错误** | `T ! HttpError` | `return err(HttpError.BadRequest);`<br>或 `return err(.BadRequest);` | 错误态枚举包装 | 普通枚举作为错误码。**必须使用 `err()` 包裹以标明进入错误态**（类似 Rust `Err()`） |

### 2. 错误声明与处理代码示例
```typescript
// 1. 具名错误集合声明（模块级导出 pub err Error 直接对标 Zig 的 pub const Error = error{ ... };）
pub err FileErr {
    NotFound,
    AccessDenied,
}

pub err NetErr {
    Timeout,
    ConnectionReset,
}

// 2. 具名错误函数：直接返回变体，无需 err() 包裹
fn openFile(path: string): string ! FileErr {
    if (path == "missing.txt") {
        return FileErr.NotFound; // 或直接 return .NotFound;
    }
    return "file content";
}

// 3. 全局错误 AnyErr / T!：函数内只能返回 err.TagName 即席静态标签
fn calc(val: i32): i32! { // 展开等价于 i32 ! AnyErr
    if (val < 0) {
        return err.NegativeNotAllowed; // 静态即席标签
    }
    // return err("bad"); // ❌ 编译拦截：禁止在 T! / T ! AnyErr 中返回动态装箱错误
    return val * 2;
}

// 4. 业务动态错误 Err：签名必须显式标注 T ! Err，支持动态字符串与状态码
fn queryDb(sql: string): string ! Err {
    if (sql.len == 0) {
        return err(404, "record not found"); // 动态构建 Err 实例
    }
    return "result";
}

// 5. 多错误集合并声明 (||)
fn fetchResource(id: i32): string ! FileErr || NetErr {
    if (id == 0) return FileErr.NotFound;
    if (id < 0) return NetErr.Timeout;
    return "data";
}
```

### 3. 错误自动转换机制 (Auto-Conversion) 与 `from` 规则
1. **安全拓扑自动转换**：
   - 具名错误集 `FileErr`、枚举错误 `HttpError`、全局错误 `AnyErr` 在通过 `?` 冒泡到返回 `T ! Err` 的函数时，编译器自动织入 `cp_convert_err`，将其自动转换为统一的动态错误实例 `Err`，并将原始错误无损记录至 `cause`；
2. **声明式 `from` 自动桥接**：
   - 若目标错误类型定义了 `pub fn from(e: SourceErr): Self`，在通过 `?` 传播时，编译器优先调用用户声明的 `from` 进行确定性语义转换。

### 4. 错误传播与降级控制
```typescript
// 1. 后缀错误冒泡符 '?'（外层签名需带 !）
const file = openFile("config.json")?;

// 2. 错误捕获降级 (catch fallback)
const port = parsePort(port_str) catch 8080;

// 3. 错误捕获并早退
const data = parseData() catch return false;

// 4. 确定性断言绝对无错误
const buf = allocator.alloc(u8, 1024) catch unreachable;

// 5. 空值合并操作符 '??'（仅针对可选类型 T? 的 null，不捕获错误）
const host = opt_host ?? "127.0.0.1";

// 6. 可选链操作符 '?.' 与强解空 '.?'
const zip = user?.address?.zip;
const val = opt_val.?;
```

### 5. 确定性资源释放 (`defer` 与 `errdefer`)
```typescript
fn processFile(path: string): ! {
    let file = openFile(path)?;
    defer file.close(); // 作用域退出时无论正常还是报错必执行清理

    let conn = connectServer()?;
    errdefer conn.abort(); // 仅当后续代码报错提前退出时触发清理

    file.pipeTo(conn)?;
}
```

---

## 十、 特殊代码块、编译期函数、宏与元编程 (Comptime & Metaprogramming)

### 1. 四大原生代码块矩阵
| 代码块语法 | 核心语义与职责 | 安全边界 | 典型场景 |
| :--- | :--- | :--- | :--- |
| `unsafe { ... }` | 关闭借用检查，允许解引用与操作裸指针（`ptr.*`） | unsafe | 底层硬件寄存器、系统调用、裸指针算术 |
| `zig { ... }` | Native Zig 原生逃逸舱（**严禁带下划线的 `__zig__`**） | 底层交互 | 极端底层 OS 调用（`clock_gettime`），完全脱离 CP 借用检查 |
| `arena { ... }` | 作用域区域分配器：离开时整块内存 $O(1)$ 瞬时清空释放 | 安全 | 高吞吐网络请求处理、临时对象群销毁、彻底零内存碎片 |
| `@c { ... }` | 内联 C 头文件声明块：引入系统 C 符号与库函数原型 | FFI | 引入 POSIX 系统头文件 `<unistd.h>`、底层常数 |

### 2. 编译期元编程系统 (Comptime Metaprogramming)
- **编译期代码块**：`@{ ... }` 或 `comptime { ... }`；
- **编译期控制流**：`@if` / `@else` 与 `@for (item of items)` 或 `inline for`；
- **编译期内省原语 (Intrinsics)**：
  - `@typeOf(x)`：获取表达式类型；
  - `@fieldsOf(T)`：获取结构体编译期字段数组；
  - `@field(obj, "name")`：编译期字段动态反射访问；
  - `@typeName(T)`：获取类型名称字符串；
  - `@sizeOf(T)` / `@alignOf(T)`：物理内存尺寸与字节对齐；
  - `@hasDecl(T, "method")` / `@hasField(T, "field")`：声明与字段探测。

### 3. 宏系统 (Macro Metaprogramming)
```typescript
// 1. AST 模板插值宏（`${field}` 语法生成）
macro Accessor(target, field, field_type) {
    pub fn get_${field}(_): field_type {
        return _.${field};
    }
    pub fn set_${field}(*_, val: field_type) {
        _.${field} = val;
    }
}

// 2. Comptime 过程宏（内嵌 @{ } 遍历 @fieldsOf）
macro AutoToString(target) {
    pub fn toString(_): string {
        @{
            // 编译期内省生成序列化逻辑
        }
    }
}
```

### 4. 自定义元注解与切面装饰器 (Decorators & Meta)
```typescript
// 1. 装饰器定义
dec log(target, options) {
    // 拦截切面逻辑
}

// 2. 标注装饰器与元数据
@log
@meta(author = "Antigravity", version = 2)
@[sealed, audited]
export class PaymentService { ... }

// 3. 编译期与运行时反射
@reflect
export class User { pub id: i32; pub name: string; }
const has_id = reflect.hasField(User, "id"); // true
```

---

## 十一、 异步协程、多线程并发容器与 Actor 模型 (Concurrency & Containers)

### 1. 异步函数与双轨派生模型
```typescript
// 1. 异步函数声明与等待
async fn fetchBody(url: string): string! {
    let client = HttpClient.init();
    defer client.destroy();
    const response = await client.get(url)?;
    return response.readText()?;
}

// 2. 双轨派生：协程池 vs CPU 密集型物理线程池
let task = spawn asyncWork();                          // 协程轻量调度 (Reductions)
let cpu_task = spawn.cpu heavyMatrixMultiply(mA, mB); // Worker 物理线程池
let res = await cpu_task;
```

### 2. 多线程安全并发容器 (`Shared<T>` 与内部可变性)
> **最高并发铁律**：严禁将栈借用 `&T` / `*T` 跨越 `spawn` / `spawn.cpu` 传递！
- **`Shared<T>`**：T 必须为只读不可变，基于原子引用计数与全局堆，支持跨 Actor / 线程零拷贝传递；
- **`Shared(Mutex(T))` / `Shared(RwLock(T))`**：
  - 内部可变性容器，TS 风格极简构造；
  - 数据与锁强绑定，通过守卫访问，离开作用域自动 RAII 解锁；
  - 天然支持多线程并发派发。

```typescript
// 构造多线程共享互斥容器
let state = Shared(Mutex(AppState{ count: 0 }));

// 跨线程派发
spawn.cpu {
    let guard = state.lock();
    guard.count += 1;
} // guard 离开作用域自动解锁
```

### 3. Actor 消息模型
```typescript
send(worker_pid, Message.Data{ payload: chunk });
receive {
    Message.Result => |res| handleResult(res),
    Message.Error => |err| handleError(err),
    timeout(5000) => handleTimeout(),
}
```

---

## 十二、 官方标准库协议矩阵 (`cap:*`)

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
| `cap:reflect` | `hasField`, `hasMethod`, `fields_of`, `type_of` | 编译期与运行时反射内省标准库 |
| `cap:collections`| `ArrayList`, `HashMap`, `RingBuffer`, `PriorityQueue` | 完备标准容器矩阵 |

---

## 十三、 智能体 Top 18 绝对禁手清单 (Top 18 Antipatterns)

| 禁手序号 | 错误幻觉写法 (WRONG) | CP 唯一合法写法 (CORRECT) | 根因与规则说明 |
| :---: | :--- | :--- | :--- |
| **#1** | `var x: i32 = 0;` | `let x: i32 = 0;` | CP 语法中不存在 `var` 关键字（`var` 是 Zig 遗留），可变变量 100% 写作 `let` |
| **#2** | `let mut x = 1;` | `let x = 1;` | CP 彻底废除 `let mut`，声明可变变量直接写 `let` |
| **#3** | `threadlocal var x: i32 = 0;` | `@tls let x: i32 = 0;` | 彻底废除 `threadlocal var`，线程局部存储使用 `@tls let` |
| **#4** | `try func();` (前缀 try) | `func()?;` (后缀问号) | 彻底废除 Zig 前缀 `try`，统一使用后缀 `?` 进行错误冒泡传播 |
| **#5** | `for (0..n) \|i\|` 或 `for (list) \|x\|` | `for (i of 0..n)` / `for (x of list)` | 循环一律使用 `for (... of ...)`，严禁 Zig 的竖线管道捕获 `\|x\|` |
| **#6** | `cond1 and cond2` / `cond1 or cond2` | `cond1 && cond2` / `cond1 \|\| cond2` | 逻辑与/或使用纯正 C/TS 符号 `&&`、`\|\|`，词法解析器无 `and`/`or` 关键字 |
| **#7** | `fn f(): !usize` 或 `fn f(): ?User` | `fn f(): usize!` / `fn f(): User?` | 废除前缀 `!T`/`?T`，确立类型主体居首的纯正纯后缀修饰符体系 |
| **#8** | `@extern struct Bar { ... }` | `@c struct Bar { ... }` | 声明严格对齐 C-ABI 内存布局结构体统一使用 `@c struct` |
| **#9** | `Point{ .x = 1, .y = 2 }` | `Point{ x: 1, y: 2 }` | 结构体初始化采用 TS 风格（无前导点），严禁 Zig 的 `.{ .x = ... }` |
| **#10** | `const p = .{ 10, 20 };` | `const p = [10, 20];` | 元组/数组字面量使用方括号 `[...]`，严禁 Zig 匿名元组 `.{ ... }` |
| **#11** | `const o = .{ .x = 1 };` | `const o = { x: 1 };` | 匿名对象字面量使用花括号 `{ x: 1 }`，严禁 Zig 的 `.{ .x = 1 }` |
| **#12** | `items: []*Timer = &.{};` | `items: []*Timer = [];` | 切片字面量使用方括号 `[]`，严禁 Zig 的 `&.{}` |
| **#13** | `__zig__ { ... }` | `zig { ... }` | 内嵌 Zig 原生块必须使用 `zig {}`，严禁带下划线的 `__zig__` |
| **#14** | `try { ... } catch (e) { ... }` | `const val = expr catch fallback;` | CP 彻底废除 Java/JS 风格的 try-catch 异常块 |
| **#15** | `catch \|_\| { return false; }` | `catch return false;` | 废除 Zig 风格的 catch 错误捕获管道，直接接降级表达式或语句 |
| **#16** | `const [first, _] = pair;` | `const [first, *] = pair;` | CP 的模式匹配通配符是 `*`（而非 `_`），`_` 是方法接收者专属保留字 |
| **#17** | `let p: *any! = ...;` (冗余 any) | `let p: *! = ...;` / `let p: *!? = ...;` | 通用弱类型裸指针推荐省略 `any`，直接使用极简 `*!` 与 `*!?` |
| **#18** | 根目录创建同名门面（如 `lib/io.cp`） | 仅在 `lib/io/index.cp` 导出单一真理来源 | 根目录同名门面已全面废弃，编译器原生优先目录寻路，避免冲突 |
