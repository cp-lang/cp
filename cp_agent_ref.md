# CP 智能体权威语法契约规范手册 (cp_agent_ref.md)

> **文档性质**：CP 语言面向 AI 智能体（AGY）的唯一最高权威语法与语义契约。  
> **设计责任**：CP 架构团队权威制定并全权负责，为主/子智能体与下游协同智能体编写 CP 代码的唯一规范准绳。  
> **核心原则**：100% 纯正 CP 语法、零遗漏完整契约与权威代码范式，坚决杜绝 Zig / Rust / TypeScript / Python 语法幻觉渗漏。  
> **权威对照**：面向人类开发者的详尽 RFC 演化与技术白皮书全量记载于 `cp_ref.md`。

---

## 一、 模块系统、工程清单与工具链 (`import` / `cap` / `cpc`)

### 1. 唯一合法导入语法 (`import`)
```typescript
// 1. CP 模块具名导入与相对路径导入
import { Router, Route } from "./router";
import { HttpClient } from "../http/client.cp";

// 2. 命名空间全量导入 (import * as)
import * as math from "./utils/math";

// 3. 官方标准库协议导入（cap:* 统一协议通道，共 32 个已实装标准库模块）
import { Mutex, RwLock, Atomic, Shared, Channel } from "cap:sync";
import { Vec, VecDeque, BinaryHeap, BTreeMap, HashMap, HashSet } from "cap:collections";
import { sha256, Aes256Gcm, randomBytes } from "cap:crypto";
import { EventLoop, Timer, TimerHeap } from "cap:event_loop";
import { Reader, Writer, BufferedReader, copyStream, print } from "cap:io";
import { spawn, join } from "cap:parallel";
import { send, reply, receive, self_pid, sender, yield, sleep } from "cap:actor";
import { PID } from "cap:beam"; // 注：@spawn/@receive/@reply/@send/@sender/@self 为内建原语，无需 import
import { test, expect_eq } from "cap:test";

// 4. 第三方包导入（自动触发三层向上攀升查找引擎）
import { Matrix } from "tensor-math";

// 5. 默认导入、混合导入与别名导入
import App from "./app.cp";
import App, { start } from "./app.cp";
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
// 1. 前端 (cpc)：赋予透明穿透 Type::Any，不因未展开 Zig 宏而中断
// 2. 后端 (Zig AOT)：调用 Zig 编译器进行全量强类型检查与单态化

// 标准工程姿态：深层别名透传与边界类型收束
pub type LocalFD = bun.FD;
pub type LocalDirInfo = DirInfo;
let max_conn: u32 = bun.getMaxConnections() as u32;
```
- **防腐层命名规范**：以 `_` 开头的文件（如 `_boringssl.zig`、`_usockets.cp`）专用于私有底层绑定；普通 `.cp` 文件包装为纯 Safe 的面向对象 API 对外导出。

### 3. 模块级导出与 `pub` 推荐规范
> **架构契约**：在模块顶层（`struct` 与 `class` 之外），**`pub` 等价于 `export`，官方推荐统一使用简洁现代的 `pub`**：
```typescript
// 1. 推荐统一使用 pub 声明顶层导出
pub const MAX_BUFFER_SIZE: usize = 65536;
pub let global_status: i32 = 0;
pub fn add(a: i32, b: i32): i32 { return a + b; }
pub @c struct Point { x: i32 = 0; y: i32 = 0; }
pub class Engine { ... }
pub static class MathUtils { ... }
pub enum State { Idle, Running }
pub err Error { OutOfMemory, Timeout }
pub trait Reader { fn read(*_, buf: []u8): usize!; }
pub type SocketFD = i32;
pub class AgentCore { ... }
export { AgentCore as default }; // 当前默认导出规范（暂不支持 pub default class 前缀简写）

// 2. export 向下兼容保留，并用于聚合重导出（index.cp 单一真理出口规范）
export { Reader, Writer, Closer } from "./traits.cp";
export { BufferReader, BufferWriter } from "./buffer.cp";
export { add as sum, PI as TAU };
export * from "./sub_module.cp";
```
- **架构铁律**：复合多文件库统一使用 `lib/<name>/index.cp` 作为唯一对外出口，根目录下严禁创建多余同名门面文件（如严禁创建 `lib/io.cp`）。

### 4. `cap.json` 工程清单与工具链 CLI (`cap` & `cpc`)
```json
{
  "name": "nexus-agent",
  "version": "1.0.0",
  "main": "src/main.cp",
  "mode": "parallel",
  "dependencies": { "tensor-math": "^0.2.1" },
  "build": { "opt": "fast", "out_dir": "bin", "c_flags": ["-O3"], "link_libs": ["c", "m"] }
}
```
| 工具指令 | 核心用途与行为 |
| :--- | :--- |
| `cap <file>` / `cap run <file>` | Debug 模式百毫秒级直接构建并运行 |
| `cap init [template]` / `add [-g]` | 初始化脚手架工程（`app`/`agent`/`lib`）；`add`/`remove`/`update` 管理依赖 |
| `cap build [dir]` (`-r`/`-f`/`-s`/`-B`/`-b`) | 构建产物：`-r` 全平台矩阵、`-f` ReleaseFast、`-s` ReleaseSmall、`-B` 强制全量、`-b` Nest BEAM 动态库 |
| `cap build --lib` / `--obj` | 输出静态库 `.a` 或底层 `.o` 目标文件 |
| `cap test [dir]` / `bench` / `clean` | 运行测试（`bin/test/`）、`bench` 基准测试、`clean` 清理 `.cap/` 与 `bin/` |
| `cpc [compile] <file.cp> -o <out.zig> [--lib]` | 编译为 Zig 源码（`--lib` 省略 main 并保留导出符号开启 DCE） |
| `cpc check [--json] <file.cp>` | 极速静态语法与借用检查（`--json` 输出机器自愈诊断） |
| `cpc emit-types` / `demangle` / `lsp` | 生成 `.d.cp` 类型契约 / 反解崩溃堆栈（亦可用 `cap demangle`）/ 启动 LSP |

---

## 二、 变量绑定、可变性与所有权/借用检查体系 (Bindings, Ownership & Borrowing)

### 1. 变量声明矩阵
CP 彻底废除 `let mut`、`var` 与 `threadlocal var`，全面拥抱清晰确定的可变性语义：
- **`let`**：可变变量声明。局部变量写 `let x = ...;`，模块顶层可变变量写 `pub let g_state = ...;`。允许重新赋值与就地修改内部字段；
- **`const`**：常量与深度不可变声明。阻止一切重新赋值与内部属性修改；
- **`@tls let`（或兼容写法 `threadlocal let`）**：线程局部存储变量声明（严禁 `threadlocal var`）；
- **🚨 绝对禁止 `var` 与 `let mut`**：声明可变变量 100% 直接写 `let`。

### 2. 所有权转移与自动释放 (Ownership & Auto-Drop)
- **单写者排他所有权**：值语义物理拷贝，引用语义转移所有权；
- **显式所有权转移**：使用 **`move val`** 或 **`own val`**；移交后原调用方变量即刻失效并被借用检查器拦截；
- **确定性析构 (Auto-Drop)**：离开作用域时，实现了 `release(*_)` 的类型自动触发析构，零 GC 运行。

### 3. 借用检查器、多字段分离借用与循环迭代锁死 (Loop-Borrow Lock)
- **读写互斥原则**：同一生命周期内允许多个只读借用（`&T`），或仅允许**唯一一个**可变借用（`*T`），读写严格互斥；基础类型可变借用通过 `p.* = val` 就地修改；
- **多字段分离借用 (Disjoint Field Borrowing)**：只要两个借用指向同一对象的不同字段（如 `*data.q_len` 与 `*data.active_timer`），编译器允许同时持有多个可变借用；借用整个结构体（`*data`）依然与子字段互斥；
- **🚨 循环迭代锁死铁律 (`Loop-Borrow Lock`)**：在 `for (item of container)` 遍历受检容器（`Vec`/`Array`/`Map`/`Set`/`Dict`）期间，容器处于迭代锁定态，**严禁在循环体内调用 `push`/`pop`/`set`/`delete`/`clear` 或索引修改容器结构及重赋值容器本身**（编译器静态报错拦截）。

```typescript
class InternalLoopData { q_len: i32; active_timer: i32; }

fn process_loop(*data: InternalLoopData) {
    let q = *data.q_len;
    let t = *data.active_timer; // 多字段分离借用：合法并行持有
    q.* = 42;
    t.* = 100;
}
```

### 4. 栈绑定视图与参数名直观生命周期绑定 (`ref struct` & `: &param`)
- **`ref struct`**：声明栈绑定只读视图结构体，生命周期严格锁定当前栈帧，**物理上严禁跨越 `spawn` / `await` 异步边界逃逸**；
- **参数名直观生命周期绑定（彻底废除 Rust `<'a>`）**：当多参数函数返回借用且只关联特定入参时，直接使用参数名标注绑定：
  - 单参数绑定：`fn selectFirst(x: Tensor, y: Tensor): &x Tensor { return &x; }` / 可变：`: *x Tensor`
  - 联合参数绑定：`fn pick(x: Tensor, y: Tensor, cond: bool): &(x | y) Tensor` / 可变：`: *(x | y) Tensor`
  - 接收者实例绑定：`fn getSlice(_): &_ []u8` / 可变：`: *_ []u8`
  - 可空与错误联合包裹借用：支持 `(&T)?`、`&(T!E)` 等组合标注
  - 亦支持显式生命周期注解 `*(a)T`、`&(a)T`；大多数场景由编译器非词法生命周期（NLL）全自动推导。

---

## 三、 静态强类型系统、四层转换法则与 C-ABI 指针 (Types & Pointers)

### 1. 标量类型系统与任意位宽整型
- **标准标量**：`i8`, `u8`, `i16`, `u16`, `i32`, `u32`, `i64`, `u64`, `i128`, `u128`, `isize`, `usize`, `f32`, `f64`, `bool` (`true` / `false`), `void`；
- **任意位宽原生整型**：支持 `u1`~`u128` 与 `i1`~`i128`（如 `u1`、`u2`、`u4`、`u7`），用于硬件寄存器、位域 `@packed struct` 及紧缩枚举底层标签；
- **函数风格标量构造转换（推荐）**：
  ```typescript
  let fd_num: i32 = i32(raw_fd);
  let len: usize = usize(byte_count);
  let val_u32: u32 = u32(val_i32);
  let is_ok: bool = bool(code); // 非零转 true，0 转 false
  ```

### 2. 类型转换四层黄金法则 (Golden Rules of Type Conversion)
> **核心架构契约**：CP 彻底废除 Zig 繁琐强转 Builtins（`@ptrCast`, `@alignCast`, `@intCast`, `@truncate`），确立清晰四层流转转换体系：

1. **第一层：无损自然流转（Implicit Coercion —— 禁止强加 `as` 噪音）**
   - 字面量自动适配：`let a: u64 = 100;`（绝不写 `100 as u64`）；
   - 数值无损拓宽：`u8 -> u16 -> u32 -> u64 -> usize`、`i8 -> i16 -> i32 -> i64`、`i32 -> f64`、`f32 -> f64` 自动隐式流转；
   - 安全包装与权限收紧：`T -> T?`、返回值自动包装进 `T!E`、`*T -> &T`、子类转基类、`*Concrete -> &dyn Trait`；
   - 字节视图隐式流转：只读上下文中 `[]u8`、`[u8; N]`、`cstring` 自动零拷贝流转为 `string`。
2. **第二层：有损截断与重解释（强制显式 `as`）**
   - 高位截断与符号变更：`let small = big as u8;`、`let unsigned = signed_i32 as u32;`、`let int_pi = 3.14 as i32;`；
   - 枚举与数值双向互转：`kind as i32`（底层 `@intFromEnum`）、`raw_int as SocketKind`（底层 `@enumFromInt`）；
   - 动态类型投影：`raw_msg as string`。
3. **第三层：底层裸指针、地址与物理位重解释（强制 `unsafe { ... }`）**
   - 指针与地址强转：`ptr as *TargetType!`（编译器自动织入 `@ptrCast(@alignCast(...))`）；`ptr as usize`（等价 `@intFromPtr`）、`addr as *i32!`（等价 `@ptrFromInt`）；
   - **`@bitCast` 与 `as` 严格界分**：`as` 用于数值截断、符号变更与指针/地址转换；**`@bitCast(expr)`**（或 `@as(T, @bitCast(expr))`）专用于**零转换物理位模式重解释**（如 Tagged Pointer 指针高低位打标、`f32 <-> u32` IEEE-754 位模式互转、`@Vector` 位掩码转换），不改变底层二进制位。
4. **第四层：高阶语义解析与转换（🚨 严禁对字符串用 `as`！走标准库方法）**
   - **❌ 绝对禁手**：`let n = "123" as i32;`；
   - **✅ 正确规范**：`let port: i32 = i32.parse(env_str)?;`、`let text: string = port.toString();`、`let s = string.from(val);`。

### 3. 定长数组、重复初始化、哨兵切片与双轨对象模型
```typescript
// 1. 定长数组类型（[N]T 与 [T; N] 两种写法完全等价互通）
let buf1: [128]u8 = undefined;           // 栈上未初始化缓冲区，零堆开销
let buf2: [u8; 4] = [1, 2, 3, 4];

// 2. 数组编译期重复初始化语法 [val; count]
let zeros: [u8; 256] = [0; 256];
let masks = [0xFF; 64];

// 3. 动态切片与区间截取
let slice: []u8 = buf1[0..len];
let tail: []u8 = slice[offset..];
let head: []u8 = slice[..end];

// 4. 带零结尾哨兵的切片截取语法 (:0) 与 cstring.ptr
let c_sub: [:0]const u8 = c_str_slice[start..end :0];
let raw_c_ptr: cstr = c_str_slice.ptr;

// 5. 裸指针切片化与切片写入（必须置于 unsafe 块内）
unsafe {
    let view: []u8 = raw_ptr[0..len];
    raw_ptr[0..data.len] = data;
}

// 6. 双轨对象模型：静态匿名结构体 vs 动态哈希字典
const static_cfg = { host: "127.0.0.1", port: 8080 }; // 静态零开销结构体
let dyn_dict = #{ "key": "value", "env": "prod" };    // 动态堆哈希字典 Map
```

### 4. 编译期自动推导占位符 (`auto`) 与内省原语
- **`auto`**：仅用于函数形参触发编译期单态化推导：`pub fn print(v: auto)`、`pub fn wrapper(@comptime cb: auto)`；🚨 严禁用于局部变量或结构体字段（`anytype` 仅作历史兼容）。
- **核心内省与散列指令**：`@sizeOf(T)`, `@bitSizeOf(T)`, `@alignOf(T)`, `@typeOf(expr)`, `@typeName(T)`, `@typeInfo(T)`（判断 `.pointer`, `.@"struct"`, `.@"enum"`, `.@"union"` 等）, `@fieldsOf(T)`, `@field(obj, name)`, `@hasDecl(T, name)`, `@hasField(T, name)`, `@as(T, expr)`, `@bitCast(expr)`, `@byteSwap(v)`, `@hash(v1, v2, ...)`, `@compileError(msg)`, `@compileLog(...)`。
- **SIMD 向量与硬件位计数原语**：`@Vector(N, T)`, `@splat(v)`, `@clz(v)`, `@ctz(v)`, `@popCount(v)`, `@reduce(op, vec)`, `@select(T, pred, a, b)`（广泛用于字符串/Unicode SIMD 高速扫描）。

### 5. 一等公民 C 字符串体系：`cstr` vs `cstring` vs `string`
| 字符串类型 | 底层物理映射 | 是否带长度 | 安全级别 | 典型应用场景 |
| :--- | :--- | :---: | :---: | :--- |
| **`string`** | `[]const u8` (切片) | ✅ 带长度 | Safe | CP 原生不可变字符串切片 |
| **`cstring`** | `[:0]const u8` / `[:0]u8` | ✅ 带长度 `.len` | Safe | 附带 `\0` 哨兵胖切片，支持 `s[a..b :0]` 与 `.ptr` |
| **`cstr`** | `[*:0]const u8` / `[*:0]u8` | ❌ 无长度 (薄指针) | Safe/Borrow | C 原生 0 结尾薄指针，高效系统 FFI 传参 |
| **`cstr?`** | `?[*:0]const u8` / `?[*:0]u8` | ❌ 无长度 (可空) | Safe/Borrow | 可空 C 字符串指针（如 `getenv` 返回值或可选 FFI 入参） |

- **多态字符串转换 `string.from(val)`**：统一将 `cstr`、`enum`、`bool`、数值及实现 `toString()`（兼容 `to_string()`）的对象零摩擦转换为 `string`。

### 6. 纯正纯后缀修饰符体系 (STRICT POSTFIX NOTATION)
> **最高设计铁律**：彻底废除前置 `?T` 与 `!T`，确立“类型主体居首、修饰状态纯后缀向右展开”：

| 类型标注 | 语义说明 | 映射底层 | 典型示例 |
| :--- | :--- | :--- | :--- |
| `T?` | **可选类型**（可能为 null） | `?T` | `let name: string? = null;` |
| `T!` | **全局推导错误联合**（等价于 `T ! AnyErr`） | `anyerror!T` | `fn read(): usize! { ... }` |
| `T ! ErrorSet` | **具名错误联合** | `ErrorSet!T` | `fn open(): File ! FileErr { ... }` |
| `T ! Err` | **业务动态错误** | `CpResult(T, Err)` | `fn query(): User ! Err { ... }` |
| `T?!` | **可空且可能报错** | `!?T` | `fn find(): User?! { ... }` |
| `!` / `?!` | **无返回值仅可能报错**（推荐极简省略 `void`） | `!void` | `fn flush()! { ... }` / `fn ping(): !` |

### 7. 全面 C-ABI 与系统级指针矩阵 (Global Symbol Matrix)
| 语法标注 | 核心语义与底层映射 | 安全级别 | 典型场景 |
| :--- | :--- | :--- | :--- |
| `&T` | **只读安全借用** | 安全 | 函数形参只读视图，零拷贝 |
| `*T` | **安全可变借用** | 安全 | 独占写权限借用，就地修改字段 |
| `move T` / `own T` | **所有权转移** | 安全 | 独占所有权移交，调用方原变量失效 |
| `*T!` / `*T?!` | **单对象强类型裸指针** | unsafe | 解引用 `ptr.*`；**🚨 严禁下标算术 `p[i]`** |
| `*(*T?!)` | **二级可空句柄出参指针**（`*?*T`） | unsafe/FFI | C 库句柄初始化出参（如 `sqlite3_open`） |
| `*!` / `*!?` | **通用弱类型裸指针**（`*anyopaque` / `?*anyopaque`） | unsafe | 对应 C `void*`，**推荐省略 `any`** |
| `[*]T!` / `[*]T?!` | **未定长多项裸指针**（`[*]T` / `?[*]T`） | unsafe | 连续内存缓冲与下标偏移 `p[i]`、切片 `p[0..n]` |
| `[*:0]T!` / `[*:0]T?!` | **0 结尾哨兵裸指针**（`[*:0]T`） | unsafe | C 字符串底层缓冲区操作 |
| `[*]([*:0]u8!)?!` | **字符串指针数组** | unsafe | 命令行 `argv` 指针数组 |
| `[*:null]([*:0]u8?!)?!` | **null 结尾可选字符串指针数组** | unsafe | `posix_spawn` 环境表 `envp` |
| `[*c]T` / `[*c]T?` | **C-ABI 兼容宽松指针（⚠️ 仅限 FFI 边界）** | unsafe/FFI | 对接传统 C 头文件签名，**纯 CP 业务严禁使用** |
| `[]T` / `[]const T` | **动态切片** | 安全 | 可变/只读切片视图 |
| `[N]T` / `[T; N]` | **栈分配定长数组** | 安全 | 固定长度本地缓存 |

### 8. 类型别名与原生运算符矩阵
```typescript
pub type FileDescriptor = i32;
pub type ByteSlice = []const u8;
pub type Callback = @c ((ctx: *!?, status: i32): void)?; // C-ABI 可选函数指针类型
pub type ArrowCb = (*i32, i32) -> bool;                   // 箭头函数签名写法
pub type SimpleCb = fn (x: u8) bool;                     // Zig 兼容简写函数类型
```
- **算术**：`+`, `-`, `*`, `/`, `%`；**环绕运算**：`+%`, `-%`, `*%`；**饱和运算**：`+|`, `-|`
- **位运算**：`&`, `|`, `^`, `~`, `<<`, `>>`；**比较运算**：`==`, `!=`, `<`, `<=`, `>`, `>=`
- **复合赋值**：`=`, `+=`, `-=`, `*=`, `/=`, `%=`, `&=`, `|=`, `^=`, `<<=`, `>>=`
- **空值与链式解包**：`?.`（可选链：如 `user?.profile?.name`）、`.?`（强制解包可空值，空则 panic）、`??`（空值兜底，不捕获错误）、`orelse`（上下文软关键字空值/错误兜底）
- **逻辑与管道**：`&&`, `||`, `!`（智能体统一写 `&&`/`||`；`and`/`or` 仅作为编译器上下文软关键字兼容）、`|>`（管道：`data |> filter() |> process()`）

---

## 四、 裸指针机制、Unsafe 操作、内嵌汇编与逃逸舱 (Raw Pointers, Unsafe & Inline Asm)

### 1. 变量常量性与指针只读性的正交解耦
1. **`const p: *T! = ...;`**：指针自身不可重新指向，但可通过 `p.* = val` 修改目标内存；
2. **`let p: *const T! = ...;`**：指针变量可重新赋值指向新地址，但目标内存绝对只读；
3. **`const p: *const T! = ...;`**：双重不可变。

### 2. 获取变量裸指针与裸指针转换矩阵 (`unsafe` 内)
```typescript
let num: i32 = 100;
let v: i32 = 42;
unsafe {
    // 方式一：前置 * 与尾置 ! 直接取裸指针
    let p: *i32! = *num!;
    p.* = 200;
    let pp: **i32! = **p!;
    pp.*.* = 300;

    // 方式二：从借用显式转换为裸指针
    let p_ro: *const i32! = &v as *const i32!;
    let p_rw: *i32! = *v as *i32!;

    // 完整裸指针转换矩阵
    let p_u8: *u8! = p as *u8!;          // 按位重解释 (自动 @ptrCast(@alignCast))
    let addr: usize = p as usize;         // 指针 -> 物理地址
    let restored: *i32! = addr as *i32!;  // 地址 -> 指针
    let generic: *! = p as *!;            // 转为 *anyopaque

    let buf: [4]u8 = [1, 2, 3, 4];
    let many: [*]u8! = buf as [*]u8!;     // 数组 -> 多项指针
}
```

### 3. `unsafe { ... }` 四大拦截边界 (`E0209`) 与自动解引用
未包裹在 `unsafe {}` 内的底层内存操作将触发编译拦截 `error[E0209]`：
1. 基础标量裸指针解引用：`p.* = 100`、`let v = p.*;`；
2. 底层裸指针类型转换：`p as *T!`、`p as usize`；
3. 裸指针切片化与切片写入：`raw_ptr[0..len]`、`raw_ptr[0..len] = data;`；
4. C-FFI 外部裸函数调用与 **`unsafe fn`**：调用标记了 `unsafe fn` 的危险函数必须在 `unsafe {}` 内；
- **🚨 自动解引用铁律**：通过结构体指针/借用访问成员字段时**全自动解引用**（写 `alloc.is_active = true;`），**严禁写 `alloc.*.is_active`**！

### 4. 原生内联汇编表达式 (`@asm(...)` / `asm volatile (...)`)
```typescript
@asm("nop");
asm volatile ("nop");

let rc = asm volatile (
    "local.get %[ptr]\nlocal.get %[expected]\nmemory.atomic.wait32 0\nlocal.set %[ret]"
    : [ret] "=r" (-> u32),
    : [ptr] "r" (&ptr.raw), [expected] "r" (expect)
);
```

### 5. `zig { ... }` 原生逃逸舱与多级点路径值类型 (`Dotted Path Types`)
- 脱离 CP 借用检查，直通底层 OS 原生能力，支持内嵌于模块顶层、`class`/`struct` 内部及 `enum` 内部；
- **多级命名空间点路径类型与 `zig` 块类型别名直通**：在参数、字段或返回值类型注解中使用多级点路径类型（如 `alloc: std.mem.Allocator`、`slice: jsc.ZigString.Slice`）或在 `zig { ... }` 块中声明的 `const T = ...` 别名时，编译器自动识别其为外部原生值类型并保留完整路径，**不会像普通 `class` 那样附加 `*` 堆指针前缀**；
- 🚨 严禁使用带下划线的 `__zig__`。

---

## 五、 函数系统、形参契约、闭包、静态容器与泛型 (Functions & Generics)

### 1. 函数声明与五维形参传递契约矩阵
```typescript
// 1. 默认值参 / 只读借用（默认不可变！函数体内禁止重新赋值或修改字段）
fn inspect(data: Point, limit: i32) {
    // limit = 10; // ❌ 编译报错：Cannot mutate read-only variable 'limit'
    print(data.x, limit);
}

// 2. 可变本地值拷贝形参（fn foo(let p: T) —— 官方唯一规范，彻底淘汰 mut）
// 允许在函数局部栈帧内自由重新赋值，对调用方原变量无任何副作用！
fn accumulate(let cursor: usize, steps: usize): usize {
    cursor += steps;
    return cursor;
}

// 3. 可变借用参数（独占可变访问，就地修改调用方数据）
fn modifyPoint(pt: *Point, dx: i32) { pt.x += dx; }
fn modifyPointAlt(*pt: Point, dx: i32) { pt.x += dx; }

// 4. 类型化裸指针形参（unsafe 级别）
fn rawOperation(ptr: *i32!) { unsafe { ptr.* = 100; } }

// 5. 所有权转移参数（move / own，函数接管生命周期，调用方原变量失效）
fn consumeUser(move u: User) { u.saveToDb(); }
fn consumeUserOwn(own u: User) { ... }

// 6. 极简无冒号返回签名（省略 void 与冒号）
fn logMessage(msg: string) { ... }
fn flushSocket()! { ... }                           // 等价于 : void!
fn findToken()?! { ... }                            // 等价于 : void?!
fn writeExact(): void ! std.Io.Writer.Error { ... } // 点路径显式错误集

// 7. 编译期自动推导参数、unsafe 函数声明与内联修饰符 (@inline)
fn handleCallback(@comptime callback: auto) { ... }
pub unsafe fn direct_io_write(ptr: [*]const u8!, len: usize): ! { ... }
@inline pub fn fastAdd(a: i32, b: i32): i32 { return a + b; } // 发射为 pub inline fn
```

### 2. 匿名函数、闭包 (`=>`) 与函数类型
```typescript
// 1. 胖箭头表达式闭包与多行块闭包
let add = (a: i32, b: i32) => a + b;
let logger = (msg: string) => {
    print("[LOG]", msg);
};

// 2. move 捕获外部所有权进入闭包（常用于跨线程 spawn）
let task = spawn.cpu (move payload) => {
    processHeavy(payload);
};

// 3. 标准函数类型标注与别名
fn apply(x: i32, y: i32, op: (i32, i32): i32): i32 {
    return op(x, y);
}
pub type Predicate = fn (x: u8) bool; // Zig 兼容函数类型简写
```

### 3. 泛型、常量泛型、全大写泛型推导与类型函数
```typescript
// 1. 类型泛型与多 Trait 约束 (+)
fn map<T, U>(list: []T, func: (T): U): []U { ... }
fn processStream<T: Reader + Closer>(stream: *T) { ... }

// 2. 全大写多字符泛型占位符自动识别（如 UD, CTX, SOCKET）
// 编译器自动识别全大写标识符为泛型类型，保留值/泛型透传不强加 * 指针前缀
pub class App<UD, CTX> {
    pub data: UD;
}

// 3. 标量与多维数组常量泛型
fn makeBuffer<const N: usize>(): [N]u8 {
    return [0; N];
}
pub class Tensor<T, const shape: [usize; 2]> {
    pub data: [T; shape[0] * shape[1]];
}

// 4. 泛型类型生成函数 (Type Functions，映射 Zig fn(comptime T: type) type) & 类型标识符实参
pub fn MaybeOwned(comptime Allocator: type): type {
    return struct {
        allocator: Allocator,
        is_owned: bool = false,
        pub fn init(alloc: Allocator): Self {
            return Self{ allocator: alloc, is_owned: false };
        }
    };
}
let trimmed = std.mem.trim(u8, raw_str, " \t"); // 支持原生类型标识符 (u8/i32/usize) 直接作为实参
let is_same = std.mem.eql(u8, slice_a, slice_b);
```

### 4. 方法接收者（Receiver）统一规范：`*this` / `this` 与 `*_` / `_`
- **只读实例方法**：首参写 `this` 或 `_`（无二义性时可省略，直接裸写字段名 `return balance;`）；
- **独占可变方法**：首参必须显式声明为 **`*this`** 或 **`*_`**；
- **所有权消耗方法**：首参声明为 `move this` 或 `move _`；
- **静态方法**：冠以 `static` 修饰，不含接收者，通过 `ClassName.method()` 调用；
- **🚨 绝对禁手**：**严禁书写 `self: *Foo` 或 `self: &Foo`！**

### 5. 纯静态命名空间容器 (`static class` 与 `static struct`)
- 内部所有字段、常量与方法**默认全自动静态**，无需逐个写 `static`；
- **严禁实例化**（`new` 或 `{}` 报错）、**严禁声明 `init`/`release`**、**严禁 `extends` 继承**；支持 `pub`/`priv`。

---

## 六、 面向对象、枚举、联合体与 C-FFI 布局 (OOP, Enums, Unions & C-FFI)

### 1. 结构体 (`struct`) 与 类 (`class`)
- **`struct`（值语义，纯栈分配，成员默认 `pub`）**：支持 `priv` 私有字段、方法、`@inline` 内联方法、内嵌常量及 `pub const Info = struct { ... };`；实例化使用 `Point{ x: 10, y: 20 }`。
- **`class`（引用语义，堆分配，成员默认 `priv`）**：外部公开成员必须显式加 `pub`；
  - **私有哈希前缀标识符 (`#identifier`)**：支持 `#ext: *!;` 声明硬隔离私有字段，杜绝符号遮蔽；
  - **零样板默认 `init` 自动合成**：若类包含字段且未手写 `fn init`，编译器自动合成全字段参数构造器 `new Class(f1, f2, ...)`；
  - **流式迭代器契约**：任何实现 `fn next(*_): Item?` 的类直接支持 `for (n of stream)`；
  - **内嵌 `zig { ... }`** 与 **常量**：`class` / `struct` 内部允许直接内嵌 `zig { ... }`。
- **单继承、构造/析构钩子与 `super`**：
  - 使用 `class Child extends Parent`（亦支持内联契约 `class Dog extends Animal implements Printable`）；
  - 子类构造中通过 **`super.init(...)`** 调用父类初始化；通过 `super.method(...)` 调用父类方法；
  - 析构清理钩子声明为 **`pub fn release(*_)`**，离开作用域自动触发确定性释放。

### 2. 枚举全特性与自洽映射体系 (Enums)
```typescript
// 1. 指定底层整数位宽、特殊逃逸标识符、内嵌方法与 zig {}
export enum FetchRequestMode: u2 {
    same_origin = 0,
    @"no-cors" = 1,       // 逃逸含连字符或保留字的标识符
    cors = 2,
    navigate = 3,

    pub fn isCors(_): bool {
        return _ == .cors || _ == .@"no-cors";
    }
}

enum TaskState { Idle = 0, Running = 1, Completed = 2 }

let st = TaskState.Running;
let name = st.name;                   // "Running"（官方内置内省属性，底层 @tagName）
let code = st as i32;                 // 1（首选：直接强转获取底层数值）
let from_int = TaskState(2);          // TaskState.Completed（函数风格反向构造）
let from_get = TaskState.get("Idle"); // TaskState?（首选静态查找，等价兼容 fromName("Idle")）
let has_idle = TaskState.has("Idle"); // bool（编译期静态存在性判断）

// 2. 开放枚举（Open Enum）：末尾使用 `_` 允许承载任意未命名整型值
pub enum ExtFlag: usize { none = 0, _ }
```

### 3. 联合体与带标签联合体 (Unions & Tagged Unions / ADT)
```typescript
// 1. 带载荷强类型联合枚举（Tagged Union / ADT，映射 Zig union(enum)）
pub enum Task {
    Idle,
    Text: string,
    Move: [i32, i32],
    Compute: { batch: i32, priority: i32 },
}
// 上下文点前缀简写构造（变量/传参/返回值类型已知时省略枚举名）：
let t1: Task = .Idle;
let t2: Task = .Text("hello");
let t3: Task = .Move [15, 25];                   // 元组载荷直接接方括号
let t4: Task = .Compute { batch: 200, priority: 7 };

// 2. C-ABI 兼容重叠联合体（映射 Zig extern union）
pub @c union SocketAddr { addr4: struct_in_addr, addr6: struct_in6_addr }

// 3. 紧凑位内存联合体与无标签裸联合体
pub @packed union PackedBits { raw: u32, flag: u8 }
pub union ValueUnion { int_val: i32, byte_val: u8 }
```

### 4. C-FFI 零样板头文件吸纳、导出修饰符与系统布局装饰器
```typescript
// 1. @c { ... } 零样板头文件吸纳（自动注入 c.* 全局安全命名空间）与自定义原型混写
@c {
    "stdio.h",
    "sqlite3.h",
    fn custom_native_helper(fd: i32, flags: u32): i32;
}
_ = c.puts("Hello via C-FFI");

// 2. @c("alias") 符号重命名导入
@c("phr_parse_request")
pub fn parse_http_request(buf: cstr, len: usize): i32;

// 3. 单行外部变量/函数声明与导出 C-ABI 函数
@c fn getpid(): i32;                             // 极简无 extern 单行外部 C 函数声明
pub extern fn mimalloc_free(ptr: *!): void;      // 显式 extern 外部 C 函数声明
extern const c_global_version: u32;              // 外部 C 符号常量/变量 (extern const / extern let)
@c pub fn on_open_trampoline(s: *!): void { ... } // 生成 callconv(.c)（亦支持 @c export fn 导出动态库符号）
```

| 装饰器注解 | 语义与底层发射 | 典型应用场景 |
| :--- | :--- | :--- |
| **`@c struct Bar { ... }`** | 严格遵循 C-ABI 内存对齐（映射 `extern struct`，兼容 `@extern`） | 物理结构与 C 头文件对齐 |
| **`@c union Bar { ... }`** | 严格遵循 C-ABI 内存重叠联合体（映射 `extern union`） | `SocketAddr` 网络地址复用 |
| **`@root struct Name { ... }`** | 映射至 Zig 文件顶层根作用域 `@This()`，带防遮蔽隔离 | 迁移 Zig“文件即结构体”模块 |
| **`@value struct Compact { ... }`**| 紧凑 POD 值结构体，按值传递无指针与析构守卫开销 | 短索引、句柄包装器 |
| **`@packed struct/union`** | 紧凑位域内存布局（支持 `u1`~`u7` 位宽），无对齐填充 | 网络协议头、硬件寄存器 |
| **`@opaque struct/class Baz`** | 不透明类型句柄（映射 `pub const Baz = opaque {};`） | `us_loop_t`、`App` 等 C 内核句柄 |
| **`@align(N)`** | 字段或类型级物理内存对齐（如 `@align(16) buf: [64]u8`） | SIMD 与缓存行物理对齐 |
| **`@refCount(cleanup)`** | 自动侵入式引用计数（提供 `.ref()` 与 `.deref()` 归零清理） | 共享套接字与资源池 |
| **`@c fn` / `@c pub fn`** | 强制 C 调用约定（`callconv(.c)`），**彻底替代 `@callconv`** | C 回调跳板与动态库导出 |

### 5. Trait 契约体系、默认方法、内置 Trait 与动态分发
```typescript
pub trait Printable {
    fn format(_): string;
    // Trait 支持直接编写带函数体的默认实现方法
    fn display(_) { print(_.format()); }
}

// 支持内联 implements 与独立 impl 两种实现语法
pub class Dog implements Printable {
    name: string;
    pub fn format(_): string { return _.name; }
}
impl Printable for i32 {
    fn format(_): string { return _.toString(); }
}

// 异构胖指针容器 (Box.new) 与运行时 O(1) downcast<T>() / downcast_mut<T>()
let handlers: Vec<Box<dyn EventHandler>> = Vec.new();
handlers.push(Box.new(NetworkHandler.init()));
if (net := handler.downcast<NetworkHandler>()) { net.flush_socket(); }
if (net_mut := handler.downcast_mut<NetworkHandler>()) { net_mut.reset(); }
```
- **5 大内置 Trait**：`Comparable`, `ToString`, `Clone`, `Hash`, `Debug`（标量天然实现，类可通过自定义方法或 `@[...]` 派生宏满足）。

---

## 七、 字面量、解构赋值与全景 `switch` 模式匹配 (Destructuring & Switch)

### 1. 纯正 CP 字面量与现代解构赋值
```typescript
// 1. 纯正字面量（严禁 Zig .{} 与 &.{}）
const hex = 0xFF; const bin = 0b1010; const oct = 0o755; const ch: u8 = '\n';
const pair = [10, 20];
const obj = { name: "cp", port: 8080 };
let empty_slice: []u8 = [];
let dict = #{ "key": "value" };

// 2. 数组多模式解构（支持尾展开、首尾双端提取与通配符 * 忽略）
const [a, b] = [100, 200];
const [head, ...tail] = [1, 2, 3, 4];          // tail 为切片视图
const [startVal, ..., endVal] = arr;           // 首尾双端提取，忽略中间元素
const [first, *] = [10, 20];                   // 忽略元素用 *，严禁用 _

// 3. 对象解构（字段提取、字段重命名别名、剩余属性零堆开销强类型打包）
const { x, y } = pt;
const { id, name: userName } = user;           // 重命名映射为局部变量 userName
const { id, ...extraProps } = user;            // 剩余字段打包为强类型匿名结构体 extraProps
const { id, ... } = user;                      // 忽略其余字段
```

### 2. 全景 `switch` 模式匹配体系
```typescript
switch (expr) {
    // 1. 枚举变体、载荷解构与剩余字段打包
    .Move [x, y] => { updatePos(x, y); },
    .Compute { batch, priority } => { processBatch(batch, priority); },
    .Compute { ...fields } => { print(fields.batch); },
    .Text text => { print(text); },

    // 2. 分支原位可变借用修改 (|*ptr|)
    .Active => |*u| { u.score += 10; },

    // 3. Walrus 子模式绑定、动态类型分支与编译期字段匹配
    s := 90..=100 => s * 2,
    w: AIWorker => w.work(100),
    @field(Tag, "foo") => 100,

    // 4. 多模式联合、区间与 if 守卫
    400 | 401 | 403 => "Client Error", // 首选竖线 | 联合（编译器亦兼容逗号 400, 401, 403 =>）
    1..=10 => "Small Range",
    1...ARES_ENOSERVER | -ARES_ENOSERVER...-1 => "Ares Range", // 支持常量端点与负数范围
    { age, role } if age >= 18 && role == "Admin" => "Superuser",

    // 5. 原生三态 & 错联合匹配（含动态 Err 状态码解构、载荷错误与 downcast）
    ok_val => handleOk(ok_val),
    err(404, msg) => handleNotFound(msg),
    err(code, msg) => handleHttpCode(code, msg),
    err(.Move [x, y]) => handleMoveErr(x, y),
    err(e: TimeoutError) => retry(e.retry_after),
    err(e := SqlError) => logSql(e.sql_state),
    err.InvalidPadding => handleInvalidPadding(),
    err(e) => handleGenericErr(e),
    null => handleNull(),

    // 6. 通配兜底
    else => handleFallback(), // 亦支持 * =>
}
```

### 3. 字符串模板插值（必须反引号）、内置 `RegExp` 字面量 (`/pat/flags`) 与管道 (`|>`)
- **`RegExp` 为语言内建类，无需任何 `import`**：同时支持 TypeScript 风格正则字面量 **`/pattern/flags`**（支持 `\/` 转义与 `i` 忽略大小写、`m` 多行等标志）及 `new RegExp(pattern, flags)`，二者均受确定性 RAII 自动释放。
```typescript
let summary = `Agent ${agent_name} executed ${task_count} reductions.`;
let email_regex = /^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$/i; // 无需 import
let re_alt = new RegExp("^hello [a-z]+$", "i");                          // 二者等价
let ok = email_regex.test("dev@cp-lang.org");

let r1 = x |> mul(2);                          // 默认传入首参 -> mul(x, 2)
let r2 = r1 |> add(5, *);                      // 通配符 * 指定插槽 -> add(5, r1)
let trimmed = "  cp  " |> .trim();             // 直接调用方法
move user_obj |> modify_user(*);               // 管道所有权转移 (move)
let cfg = "config.txt" |> load_config(*)?;     // 管道尾部直接附加 ? 冒泡或 ?? 兜底
```

---

## 八、 控制流、守卫三剑客与流式类型收窄 (Control Flow & Guard Trinity)

### 1. 条件语句、海象只读与可变指针就地解包 (`*val :=`)
> **强制圆括号铁律**：`if`、`while`、`for` 头部必须加小括号 `(...)`。
```typescript
// 1. 只读海象解包 (val :=)
if (timer := heap.pop()) { timer.callback(timer.ctx); }
while (line := reader.nextLine()) { processLine(line); }

// 2. 可变指针就地解包 (*val :=) —— 零拷贝修改 Option 内部值
if (*val := state) { val = 200; }
while (*item := buffer.next()) { item.processed = true; }
```

### 2. 守卫控制流三剑客（The Guard Trinity: `||` / `??` / `?` + 发散语句）
```typescript
while (i < max_len) {
    let cur = items[i];
    i += 1;

    // 1. || 布尔条件守卫：false 时触发右侧发散语句 (continue / break / return)
    is_valid(cur) || continue;
    (cur < 100) || break;
    check_auth(cur) || return 403;

    // 2. ?? 空值守卫解包：null 时触发发散，非空直接解包赋值
    let entry = fetch_next_opt() ?? continue;
    let user = find_user(id) ?? return null;

    // 3. ? 错误联合守卫解包：error 时触发发散，成功直接解包赋值
    let packet = read_socket() ? break;
    let res = execute_step() ? return err(500, "step failed");
}
```

### 3. 流式类型智能收窄 (Flow-Sensitive Type Narrowing)
- `if (u != null)` 块内自动收窄为 `User`；`if (u != null && u.age >= 18)` 右侧享收窄；
- `if (ptr == null) return;` 之后代码 `ptr: *T!?` 自动收窄为非空裸指针 `*T!`。

### 4. 循环遍历 (`for ... of`)、跨层标签与具名代码块带值跳出
```typescript
for (i of 0..10) { print(i); }          // 半开区间（闭区间用 0..=10）
for (item of items) { process(item); }  // 只读遍历（遍历期间禁止修改 items）
for (*item of items) { item.active = true; }
for (move item of items) { consume(item); }

// 步进表达式 while 与编译期 inline while
let i: usize = 0;
while (i < 10) : (i += 1) { ... }
inline while (n > 0) { ... }

// 具名标签跨层跳出与继续 (break :label / continue :label)
outer: for (x of rows) {
    for (y of cols) {
        if (skip(x, y)) continue :outer;
        if (found(x, y)) break :outer;
    }
}

// 具名标签块与匿名表达式块带值跳出 ({ ... break expr; }) + 可选链 (?.) 与强制解包 (.?)
let nickname = user?.profile?.nickname;
let computed = init_blk: {
    if (cached != null) break :init_blk cached.?;
    break :init_blk computeExpensive();
};
let direct_val: i32 = ({ let base = 50; break base * 2; });
_ = computeExpensive();       // 忽略未使用返回值；同一作用域多次忽略/避免遮蔽 `_` 亦支持 `let * = expr;`
const [h, *, t, ...] = items; // 解构忽略元素用 `*`，方法接收者用 `_`
```
- **分号可选与 ASI**：语句末尾分号 `;` 可选，推荐使用分号保证工程清晰度；历史关键字 `match` 兼容等同于 `switch`，统一首选 `switch`。

---

## 九、 统一原生错误体系：AnyErr、动态业务 Err 与具名联合 (Unified Error Architecture)

### 1. 错误返回类型全景分类矩阵
| 错误类型形态 | 函数返回值签名 | 抛出语法 | 规则与约束说明 |
| :--- | :--- | :--- | :--- |
| **全局即席错误** | `T!` 或 `T ! AnyErr` | `return err.NotFound;` | 只能返回静态标签，禁止动态装箱（`error` / `Error` 为 `err` / `Err` 别名，首选 `err`） |
| **业务动态错误** | `T ! Err` 或 `T ! err` | `return err("msg");`<br>`return err(404, "msg");` | `Err`（`code: i32, msg: string, cause: ?*anyopaque`），签名显式 `T ! Err` |
| **具名错误集合** | `T ! FileErr` | `return FileErr.NotFound;`<br>或 `return .NotFound;` | `err FileErr { NotFound }` 声明，直接返回变体 |
| **多错误集合并** | `T ! FileErr \|\| NetErr`<br>`T ! FileErr \|\| err{Timeout, Canceled}` | 直接返回子错误变体或 `.Timeout` | 零成本原生错误集联合（支持具名错误集与匿名即席 `err{...}` 合并） |
| **混合动态升格** | `T ! FileErr \|\| Err` | 返回具名错误或 `err(...)` | 具名错误自动升格包装为 `Err` |
| **结构体/类错误** | `T ! MyStructErr` / `MyClassErr` | `return err(MyStructErr{...});` | Struct 错误栈零开销；Class 错误堆分配支持 `cause` |
| **枚举错误（含载荷）** | `T ! TaskErr` | `return err(TaskErr.Move(1, 2));` | 支持简单状态码枚举与带载荷枚举变体 |

### 2. 错误自动提升、`from` 声明式桥接与降级传播
```typescript
// 1. 枚举声明式 from 自动错误提升（在 ? 冒泡时自动转换子错误）
pub enum AppErr {
    Io(FileErr) from FileErr,
    Net(NetErr) from NetErr,
}
// 亦支持在目标错误类型内定义 pub fn from(e: SourceErr): Self

// 2. 错误冒泡、降级 (? fallback) 与带参 catch
const file = openFile("config.json")?;
const host = ping("bad.host") ? "127.0.0.1";          // 单问号后接默认值直接降级
const cfg = load_config("a.json") catch e => #{ port: 8080 };
const port = parsePort(s) catch 8080;
process() catch err => { print("Failed:", err); };
const buf = allocator.alloc(u8, 1024) catch unreachable; // unreachable 断言绝不失败，否则崩溃

// 3. 动态错误挂载底层原因链 (.with_cause) 并在 switch 中 downcast 还原
let wrapped = Err.init(500, "failed").with_cause(move box_err); // 挂载 Box<dyn Trait> 胖指针原因链
```

### 3. 确定性资源释放 (`defer` 与 `errdefer`)
```typescript
fn processFile(path: string): ! {
    let file = openFile(path)?;
    defer file.close();       // 离开作用域必执行清理
    let conn = connect()?;
    errdefer conn.abort();    // 仅当后续发生错误早退时执行清理
    file.pipeTo(conn)?;
}
```

---

## 十、 特殊代码块、编译期元编程、内置派生宏与 AOP 装饰器 (Comptime, Macros & Reflection)

### 1. 四大原生代码块矩阵
| 代码块语法 | 核心语义与职责 | 安全边界 |
| :--- | :--- | :--- |
| `unsafe { ... }` | 允许裸指针解引用、强转与裸切片写入 | unsafe |
| `zig { ... }` | Native Zig 原生逃逸舱（严禁 `__zig__`） | 底层交互 |
| `arena { ... }` | 作用域区域分配器：支持语句/表达式返回值，退出时 $O(1)$ 销毁（如 `let v: i32 = arena { let p = @alloc(64); ... };`） | 安全 |
| `@c { ... }` | 内联 C 头文件吸纳块（`"stdio.h"`）与 C 原型混合声明 | FFI |

### 2. 编译期元编程、静态计算块与 `@map` 完美哈希表
```typescript
// 1. 编译期求值 const VAL = @{ ... };、三元 @if 与块级 @if / @else
const COMPUTED_SIZE = @{ let a = 15; a + 25 };
const ARCH_TAG = @if (target_os == .linux) "epoll" else "kqueue";
@if (is_debug) {
    let trace = true;
} @else {
    let trace = false; // 死分支编译期彻底修剪
}

// 2. 编译期纯计算函数（显式调用必须加 @ 前缀）与静态循环展开
comptime fn getPageSize(n: usize): usize { return n * 4096; }
const PAGE_4 = @getPageSize(4); // 编译期折叠为 16384
comptime { assert(@sizeOf(SocketAddr) == 128); }
inline for (field of fields) { processField(field); }
@for (i of 0..4) { ... }

// 3. 类型内省与底层 SIMD/位计数/原子原语（用于 ptr/collections/unicode/string/sync 高性能库）
// 反射内省：@This(), @typeInfo(T) (.pointer / .@"struct" / .@"enum" / .@"union"), @is_beam, @panic(msg)
// 位重解释与地址：@bitCast(expr), @intFromPtr, @ptrFromInt, @intFromEnum, @enumFromInt, @intFromBool, @bitSizeOf, @byteSwap
// SIMD 向量与位计数：@Vector(N, T), @splat, @clz, @ctz, @popCount, @reduce, @select, @atomicLoad, @atomicStore

// 4. 编译期静态完美哈希查找表 @map（O(1) 零堆分配查找）
const ACL_MAP = @map(ACL, {
    "private": .Private,
    "public-read": .PublicRead,
});
let acl = ACL_MAP.get("public-read"); // 返回 ACL?
let ok = ACL_MAP.has("private");      // 返回 bool
```

### 3. 7 大官方内置派生宏 (`@[...]`) 与自定义宏
> **最高规范**：CP 彻底废除 `@derive(...)`，统一使用原生批量注解 **`@[...]`**。
```typescript
// 一键静态织入 7 大工业级能力（支持单写 @Json 或批量 @[...]）：
@[Clone, Comparable, Hash, ToString, Debug, Json, Table("t_orders")]
pub class OrderItem {
    pub sku: string;
    pub price: f64;
}
// 自动合成：clone()、cmp()/eq()、hash()、toString()、fmt_debug()、toJson()、getTableName()

// AST 模板宏与 Comptime 过程宏（macro 定义 + ${field} 插值 + @fieldsOf/@for/@if）
macro AutoReset(target) {
    pub fn reset(*_): void {
        @for (f of @fieldsOf(target)) {
            @if (f.type == i32 || f.type == f64) { _.${f.name} = 0; }
            @else @if (f.type == string) { _.${f.name} = ""; }
        }
    }
}
@AutoReset class Metrics { pub count: i32; pub label: string; }
```

### 4. AOP 装饰器洋葱模型 (`dec`)、强类型元注解 (`meta`) 与反射 (`reflect`)
```typescript
// 1. 强类型元注解契约定义
@target(method)
meta Get(path: string = "");
meta Auth(role: string);

// 2. AOP 装饰器定义（使用 next() 占位符实现零开销编译期洋葱模型）
dec timer(next) {
    let res = next();
    return res;
}
dec tag(next, prefix: string) {
    let required = meta.get("Auth"); // 读取当前目标元数据
    return "[" + prefix + "] " + next();
}
dec retry(times: i32 = 3) {
    return fn(next) {
        for (i of 0..times) {
            let res = next();
            if (res != null) return res;
        }
        return null;
    };
}

// 3. @reflect 激活运行时静态单例表 + @[...] 批量收拢 + @meta(k: v) 冒号键值标注
@[reflect, meta(author: "Antigravity", timeout: 3000)]
pub class UserService {
    @[timer, tag("API"), Get("/users")]
    pub fn listUsers(): string { return "users"; }
}

// 4. 完整运行时与编译期反射 API（需加 @reflect 才生成运行时 TypeInfo）
let info = reflect.type<UserService>();
let info_dyn = reflect.of(service_inst);
let m = info.getMethod("listUsers");   // 访问 info.fields, info.methods, m.meta["Get"]
let has_f = reflect.has(UserService, "listUsers");
```

---

## 十一、 异步协程、同步原语与 Actor 模型 (Concurrency & Actor)

### 1. 入口并发配置注解、双轨派生与任务协调 (`cap:parallel`)
```typescript
import { spawn, join } from "cap:parallel";

@parallel(8) // 入口注解：@parallel(N) 开启工作窃取多线程池；@single_thread 锁死单线程循环
export async fn main() {
    let t1 = spawn fetchBody("https://cp-lang.org");      // 协程轻量调度 (Reductions)
    let t2 = spawn.cpu heavyMatrixMultiply(mA, mB);       // Worker 物理线程池

    let single = await t2;
    await join(t1);                                       // 协程/任务同步汇聚（规划中的 Task.all/race 见后续版本）
}
```

### 2. 多线程共享、闭包自动 RAII 锁与硬件原子原语 (`cap:sync`)
> **跨线程铁律**：严禁将栈借用 `&T`/`*T` 跨越 `spawn`/`spawn.cpu` 传递！跨线程传递 `Shared<T>` 前必须显式调用 **`.clone()`** 增加原子引用计数。
```typescript
import { Mutex, RwLock, Atomic, Shared } from "cap:sync";

// 1. 推荐闭包模式（withLock / withRead / withWrite）：零死锁自动 defer 释放
let metrics = Shared(Mutex(AppState{ count: 0 }));
let m1 = metrics.clone();
let t = spawn.cpu () => {
    m1.withLock((s) => { s.count += 1; });
};

let cache = RwLock(CacheData{ version: 1 });
cache.withRead((c) => { print(c.version); });
cache.withWrite((c) => { c.version += 1; });

// 2. 守卫模式、非阻塞尝试与裸锁
if (g := metrics.tryLock()) { g.count += 1; } // tryLock / tryRead / tryWrite 返回 Guard?
let rawMu = Mutex(); rawMu.lock(); rawMu.unlock();

// 3. 硬件级原子操作原语 Atomic<T>
let counter = Atomic(0);
counter.inc(); counter.add(10);
let swapped = counter.cas(11, 20); // load, store, swap, cas, compareAndSwap, add, sub, inc, dec, testAndSet, clear
```

### 3. Actor 模型与零拷贝通信 (内建原语 + `cap:actor` / `cap:beam` / `cap:mem`)
> **核心契约**：`@spawn`、`@receive`、`@reply`、`@send`、`@self`、`@sender` 是编译器内建原语，**禁止**写 `import { @spawn }`；常规包装函数与类型从 `"cap:actor"` / `"cap:beam"` 导入。
```typescript
import { send, reply, receive, self_pid, sender } from "cap:actor";
import { PID, Agent, SystemAction } from "cap:beam";
import { SharedTensor, Arena } from "cap:mem";

let worker_pid = @spawn((msg) => {
    switch (msg) {
        .Data d => @reply(@sender(), { Result: processChunk(d.payload) }),
        else => print("Unknown message"),
    }
});
@send(worker_pid, { Data: { payload: chunk } });
let msg = @receive();
switch (msg) {
    .Result res => handleResult(res),
    .Error e => handleError(e),
    else => {},
}
```

---

## 十二、 官方 32 大标准库协议矩阵 (`cap:*`)

| 标准库协议通道 | 核心导出符号（精确 API 命名） | 核心职责 |
| :--- | :--- | :--- |
| `cap:collections` | **`Vec<T>`, `VecDeque<T>`, `BinaryHeap<T>`, `BTreeMap<K, V>`, `BTreeSet<T>`, `HashMap<K, V>`, `HashSet<T>`** | 完备标准容器（严禁写成 Zig `ArrayList`） |
| `cap:io` | `Reader`, `Writer`, `Closer`, `Seeker`, `BufferReader`, `BufferWriter`, `BufferedReader`, `BufferedWriter`, `copyStream`, `PipeReader`, `PipeWriter`, `MaxBuf`, `Pipe`, `print` | 统一流契约、零拷贝管道与标准 I/O |
| `cap:stream` | `Pipe`, `PipeReader`, `PipeWriter`, `MaxBuf`, 统一零拷贝流接口重导出 | 工业级零拷贝流式 I/O 管道 |
| `cap:sync` | `Mutex`, `RwLock`, `Atomic`, `Shared`, `Futex`, `Condition`, `WaitGroup`, `Channel`, `MpmcQueue`, `UnboundedQueue`, `ConcurrentMap`, `ArcSlice`, `Wyhash` | 锁、无锁队列、共享智能指针与并发原语 |
| `cap:parallel` | `spawn`, `join`, `Task`（注：`Task.all` / `Task.race` 为规划接口，当前优先用 `join`） | 多核 Worker 线程池与异步任务汇聚 |
| `cap:actor` | `yield`, `receive`, `sleep`, `join`, `self_pid`, `sender`, `send`, `reply`（内建 `@spawn`/`@receive`/`@reply`/`@send`/`@self`/`@sender`） | BEAM 级轻量级 Actor 消息并发模型 |
| `cap:beam` | `PID`, `Message`, `Agent`, `SystemAction` | BEAM 节点信箱协议、系统调度动作码 |
| `cap:worker` | `Worker` 与 `PID` 联动封装 | 独立隔离工作线程管理 |
| `cap:mem` | `Arena`, `SharedTensor`, 全局共享堆 | 区域内存分配与跨 Actor 零拷贝大张量 |
| `cap:json` | `JsonParser`, `JsonValue`, `JsonBufferPool`, `parse`, `stringify` | 零拷贝流式 JSON 解析框架 |
| `cap:serde` | `@json_parse`, `@json_stringify`, 零拷贝序列化契约 | 高性能编解码与结构化映射 |
| `cap:regex` | **内置 `RegExp`**（`/pattern/flags` 字面量与 `new RegExp(pat, flags)`，无需 `import`） | ES6 风格正则表达式引擎 |
| `cap:sqlite` | `Database`（`init`, `exec`, `query_string`） | 原生轻量级 SQLite 嵌入式数据库驱动 |
| `cap:http` | `Method`, `Headers`, `ETag`, `MimeType`, `Request`, `Response`, `Server`, `HttpClient`, `H2ClientSession`, `H3ClientSession`, `fetch`, `serve` | 现代 HTTP/1.1、HTTP/2、HTTP/3 全协议栈 |
| `cap:ws` | `Opcode`, `CloseCode`, `FrameHeader`, `FrameBuilder`, `Mask` | RFC 6455 WebSocket 协议帧编解码 |
| `cap:router` | `Router<T>`, `RadixNode<T>`, `ParamMap`, `RouteMatch`, `Route` | 极速 Radix-Tree 路由匹配引擎 |
| `cap:net` | TCP 套接字、UDP 广播、`TcpStream`, `TcpServer`, `SafeSocket` | 底层网络反应堆抽象 |
| `cap:event_loop` | `EventLoop`, `Timer`, `TimerHeap`, `ConcurrentTask`, `ConcurrentQueue`, `PollEngine` | 统一事件循环驱动与最小堆定时器 |
| `cap:fs` | `readFile`, `writeFile`, `exists`, `stat`, `tmpDir`, `tmpPath`, `writeAtomic` | 同步/异步现代文件系统高阶抽象 |
| `cap:path` | `path`, `Path`, `PosixPath`, `WindowsPath`（`join`, `resolve`, `normalize`, `dirname`, `basename`, `extname`） | 跨平台路径计算引擎 |
| `cap:url` | `URL`, `URLSearchParams`, `QueryParam` | WHATWG 标准 URL 解析与查询参数管理 |
| `cap:glob` | `Glob`（`match`, `hasMagic`, 支持 `**`, `*`, `?`, `[a-z]`, `!`） | 极速 Unix Glob 通配符匹配 |
| `cap:semver` | `Semver`, `Version`（`parse`, `satisfiesStr`） | Semver 2.0 语义化版本解析与范围匹配 |
| `cap:dotenv` | `DotEnv`（`parse`, `get`, `getOr`） | 环境变量 `.env` 极速加载器 |
| `cap:ini` | `Ini`（`parse`, `get`, `getGlobal`） | 工业级多 Section INI 配置文件解析 |
| `cap:which` | `which`, `isExecutable`, `WhichOptions` | 跨平台 `PATH` 可执行程序定位器 |
| `cap:encoding` | `Base64`（Standard, URL-Safe, 无填充） | 高性能数据编解码库 |
| `cap:crypto` | `sha256`, `sha512`, `randomBytes`, `fillRandom`, `randomUUID`, `Aes256Gcm`, `Hmac`, `hmacSha256Raw`, `pbkdf2` | 密码学安全随机数、散列与 AEAD 加密 |
| `cap:security` | `CSRF`, `TokenFormat`, `constantTimeEql` | 防 CSRF 签名与恒定时间安全比对 |
| `cap:reflect` | `reflect.type<T>()`, `reflect.of()`, `reflect.has()`, `reflect.hasField()`, `reflect.hasMethod()` | 编译期与运行时统一反射内省库 |
| `cap:events` | `EventEmitter` | 类型安全事件总线 |
| `cap:test` | `test("desc", () => { expect_eq(a, b); })` | 原生单元测试与断言框架 |

---

## 十三、 智能体 Top 28 绝对禁手清单 (Top 28 Antipatterns)

| 禁手序号 | 错误幻觉写法 (WRONG) | CP 唯一合法写法 (CORRECT) | 根因与规则说明 |
| :---: | :--- | :--- | :--- |
| **#1** | `var x: i32 = 0;` | `let x: i32 = 0;` | CP 不存在 `var` 关键字，可变变量 100% 写作 `let` |
| **#2** | `let mut x = 1;` | `let x = 1;` | CP 彻底废除 `let mut` |
| **#3** | `fn foo(mut x: i32)` | `fn foo(let x: i32)` | 可变本地拷贝形参使用 `let`，彻底淘汰 `mut` 关键字 |
| **#4** | `threadlocal var x: i32 = 0;` | `@tls let x: i32 = 0;` | 线程局部存储使用 `@tls let`（或 `threadlocal let`） |
| **#5** | `try func();` / `try { ... } catch` | `func()?;` / `expr catch e => ...` | 废除前缀 `try` 与 Java 式 `try/catch` 块，使用后缀 `?` 与表达式 `catch` |
| **#6** | `for (0..n) \|i\|` 或 `for (list) \|x\|` | `for (i of 0..n)` / `for (x of list)` | 循环一律使用 `for (... of ...)`，严禁 Zig 竖线管道 `\|x\|` |
| **#7** | `for (x of vec) { vec.push(1); }` | 先收集或使用索引，严禁遍历中增删 | `Loop-Borrow Lock`：遍历期间容器被借用锁死，禁止内部结构修改 |
| **#8** | `cond1 and cond2` / `cond1 or cond2` | `cond1 && cond2` / `cond1 \|\| cond2` | 智能体编码规范强制用 `&&`/`||`（编译器保留 `and`/`or`/`orelse` 作上下文软关键字） |
| **#9** | `fn f(): !usize` 或 `fn f(): ?User` | `fn f(): usize!` / `fn f(): User?` | 彻底废除前缀 `!T`/`?T`，确立类型主体居首的纯后缀体系 |
| **#10** | `let n = "123" as i32;` | `let n = i32.parse("123")?;` | 禁止对字符串用 `as` 转换数值，必须走标准受检 `parse` 方法 |
| **#11** | `let big: usize = (small as usize);` | `let big: usize = small;` | 数值小转大无损拓宽全自动流转，禁止写冗余 `as` 噪音 |
| **#12** | `@extern struct Bar { ... }` | `@c struct Bar { ... }` | C-ABI 内存对齐结构体统一使用 `@c struct` |
| **#13** | `@callconv("c") pub fn f()` | `@c pub fn f()` | C 调用约定函数统一使用 `@c`，彻底废除 `@callconv` |
| **#14** | `Point{ .x = 1, .y = 2 }` | `Point{ x: 1, y: 2 }` | 结构体初始化无前导点 |
| **#15** | `const p = .{ 10, 20 };` / `.{ .x = 1 }` | `const p = [10, 20];` / `{ x: 1 }` | 元组用方括号 `[...]`，匿名对象用花括号 `{ ... }` |
| **#16** | `items: []*Timer = &.{};` | `items: []*Timer = [];` | 切片字面量用方括号 `[]`，严禁 `&.{}` |
| **#17** | `__zig__ { ... }` | `zig { ... }` | 内嵌 Zig 块严禁下划线前缀 `__zig__` |
| **#18** | `catch \|_\| { return false; }` | `catch return false;` | 废除 Zig 管道捕获，直接接 `catch e =>` 或降级语句 |
| **#19** | `const [first, _] = pair;` | `const [first, *] = pair;` | 模式匹配忽略符是 `*`，`_` 是方法接收者保留字 |
| **#20** | `let p: *any! = ...;` | `let p: *! = ...;` / `let p: *!? = ...;` | 通用裸指针推荐省略 `any`，直接写 `*!` 与 `*!?` |
| **#21** | 纯 CP 代码中滥用 `[*c]T` | `*T!` / `[*]T!` / `[]T` | `[*c]T` 仅限用于 C-FFI 头文件边界，业务代码中严禁使用 |
| **#22** | 根目录创建 `lib/io.cp` 门面 | 仅经 `lib/io/index.cp` 对外导出 | 复合目录库单一真理出口规范 |
| **#23** | `pub fn count(self: *Foo)` | `pub fn count(*this)` / `count(*_)` | 接收者必须用 `*this`/`this` 或 `*_`/`_`，严禁自定 `self` 形参 |
| **#24** | 在 `unsafe {}` 外部 `ptr.*` 或强转 | `unsafe { const p = ptr as *T!; p.* = 1; }` | 裸指针操作必须在 `unsafe {}` 内，否则触发 `E0209` |
| **#25** | `ptr.*.field = val` | `ptr.field = val` | 结构体指针字段访问自动解引用，严禁 `.*.` |
| **#26** | 手写 `@ptrCast(@alignCast(p))` / `@intCast(x)` | `p as *T!` / `u32(x)`（位模式重解释用 `@bitCast`） | 数值与指针强转走 `as`/`T(x)`；物理位重解释保留 `@bitCast` |
| **#27** | `@derive(Clone, Json)` 或 `@meta(k = 1)` | `@[Clone, Json]` 与 `@meta(k: 1)` | 派生宏用 `@[...]`，元数据键值对用冒号 `:` |
| **#28** | 使用 `ArrayList` / `RingBuffer` | `Vec<T>` / `VecDeque<T>` | 标准容器使用 `cap:collections` 的 `Vec`/`VecDeque`/`HashMap` |
