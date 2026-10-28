# 角色

你是资深编程大师，精通zig、rust、js、es6、Typescript、c、c++、c#等各种编程语言和各种编程库，现在，你与我一起创建一个新的编程语言，语法类似Typescript，上手容易；安全类似Rust，避免内存泄露、空指针和程序崩溃；模块、包生态类似Npm；包管理、编译、测试、运行类似Bun，使用简单。

同时通过-nest参数编译为支持抢占式调度智能体，在Nest虚拟机中动态加载、运行，基于云端智能体平台，开放给用户编写智能体，上传cp代码，云端编译后，动态加载到Nest中运行。

Nest是单机百万级（线程）智能体运行容器，在计算机并行内核之上实现的多线程平台，非操作系统线程，普通服务器支持百万-千万线程，每线程200KB内存，远低于Docker容器和Node.js，专用于智能体运行容器，每个智能体运行在单独的Nest线程中。

# cp(Concurrent Plus) 智能体编程语言

cp(Concurrent Plus) 是一门创新编程语言，它集成了 TypeScript/es6 的简洁 + Rust 的安全 + Zig 的跨平台与速度**

写起来像 TS，
安全媲美 Rust，
运行速度 Zig，
专门为 AI 大模型并行计算而生。

cp 代码通过rust编写的cpc编译器，编译为zig代码，再通过zig编译为全平台机器码，直接运行在操作系统上，不像Typescript/es6运行在Node或Bun的JS Core/V8虚拟机上。

cp对于zig，类似es6对于es5。

## 运行在以下两个场景

### 直接运行于各种操作系统

1. 使用zig编译为机器码，直接运行于各种操作系统。
2. 与zig相比，由于借鉴了Rust的安全特性，比zig安全。
3. 与Rust相比，有类似的安全性，但是语法类似Typescript，上手更容易。
4. 借鉴bun针对js语言的包、包管理、编译、测试、运行等，由cap针对cp语言提供类似的包、包管理、编译、测试、运行等。
5. 类似es6相比es5，提供了强大的语言特性、语法糖、模块管理、异步等，让js语言脱胎换骨；类似，cp在zig基础上，借鉴了es6语法，让zig脱胎换骨。

### 运行于Nest虚拟机

1. 带 `-nest` 编译选项，编译产物作为Actor，运行于Nest智能体容器中。
2. Nest通过zig编译，与bun一起成为Bun-Nest双核心的百万智能体运行平台。
3. cp语言通过cpc编译器插装让权代码，支持Nest的抢占式协程调度，用于多租户百万智能体研发，避免一个智能体死循环影响其他智能体运行。
4. 与Nest及Nest庞大、稳定的调度生态无缝集成，继承Nest强大的资源库，避免重新造轮子。

## **最实用、最适合并行/高性能、又不破坏 TS 美感**的语法特性

- **必加（强烈推荐）**
- **适合并行计算的加分项**
- **底层安全但不破坏 TS 风格**

全部都是能直接融入 `.cp` 代码里的。

---

# 一、必加：Rust + Zig 里最香、最适合你的语法

## 1）模式匹配 `switch`（全面取代 `match` 的一等公民表达式）

TS 的 `switch` 只有语句且容易穿透（fallthrough），CP 将 `switch` 升级为融合 Rust `match` 威力的表达式（无穿透、无需 `break`、支持解构与守卫 `if`）。
AI 大模型里大量状态判断、错误联合体解构、分布式任务状态，用 `switch` 直接起飞。

```cp
// CP 里的 switch 表达式（TS 关键字 + Rust 级模式匹配与穷尽校验）
let status = task.state;

switch status {
  TaskState.Pending => console.log("等待中"),
  TaskState.Running => runModel(),
  TaskState.Failed(err) => console.log("错误:", err),
  * => handleDefault() // 使用 * 作为通配符，避免与代表 this 的 _ 符号冲突
}
```

优点：

- 强制编译期穷尽匹配（不容易漏分支）
- 支持字面量、范围、元组、结构体、枚举、错误联合体（成功值 / `null` / `Err(e)`）解构与模式守卫 `if`
- `*` 代表默认通配，与代表当前实例 `this` 的 `_` 彻底解耦
- 写大模型调度逻辑极清晰

## 2）三态返回 `?T ! E` + `?` 错误冒泡 + `? default` 兜底 + `try {}` 自动冒泡块

AI 大模型里 IO、网络、分布式调用全是易错操作。CP 语言融合 Rust 与 Zig 的精华，提供零 `Ok()` 包装的三态返回签名 `?T ! E`：

```cp
// 1. 尾部 ? 自动向上冒泡错误（及空值）
let file = readFile("model.bin")?;
let data = loadTensor(file)?;

// 2. 尾部 ? default：遇到错误或 null 时直接返回兜底默认值
let cfg = readFile("config.json") ? "{}";

// 3. try { ... } catch err => { ... }：块内自动为带 !E 的调用补齐尾部 ?
try {
  let f = readFile("model.bin");
  let t = loadTensor(f);
} catch err => {
  console.log("加载失败:", err);
}
```

配套：

- 三态返回签名 `?T ! E`（正常 `T`、可选空 `null`、错误集 `E`，成功返回无需 `Ok()`，错误返回使用 `Err(...)`）
- `?` 向上冒泡、`? default` 错误降级兜底、`?? default` 纯 `null` 合并
- `try { ... } catch err => { ... }` 消除满屏 `?` 视觉噪音

## 3）Zig：**`comptime` 编译期执行**

大模型需要大量编译期计算：
算子融合、常量折叠、权重预计算、内核生成。

```cp
// 编译期就算出结果，不占运行时
comptime {
  const MODEL_VERSION = "v7b";
  const MAX_BATCH = 256;
}

fn forward() {
  // 这些都是编译期常量
  let batch = MAX_BATCH;
}
```

优点：

- 性能直接拉满
- 适合大模型里大量固定配置、算子模板

## 4）Zig：**错误集（error set）**

比 TS `any`/`unknown` 更安全，比 Rust 更轻。

```cp
// 定义一组错误
error ModelError {
  LoadFailed,
  OutOfMemory,
  ShapeMismatch
}

// 函数声明
fn loadWeights() !Tensor {
  if (!file) return error.LoadFailed;
  return tensor;
}
```

和 TS 兼容，但**更安全、更适合底层系统编程**。

---

# 二、适合「并行计算 / AI 大模型」的语法

## 5）Rust：** trait（接口增强版，支持 struct 与单继承 class）**

比 TS interface 更强，支持**默认方法实现（Default Methods）**、**内联 `implements`** 与 **retroactive（后向扩展 `impl ... for ...`）**，且全面融合 `_`（等同于 `this`）的所有权与借用检查：

```cp
trait TensorOps {
  fn norm(): i32;                         // 缺省只读借用（省略 _ / this，兼容 &_）
  fn scale(*_, factor: i32);              // 显式可变借用 *_ (亦兼容 mut _)
  fn consume(move _): i32;                // 显式所有权消费 move _ (亦兼容 move this)

  // Trait 默认方法实现：内部可省略 _. 直接互调同 Trait 方法
  fn normalize(*_) {
    let n = norm();
    if (n > 0) {
      scale(1);
    }
  }
}

// 方式 A：TS 风格内联 implements（同时支持 class 单继承 extends）
class Matrix extends BaseTensor implements TensorOps { ... }

// 方式 B：Rust 风格后向扩展（给已有的类型实现 trait）
impl TensorOps for Tensor {
  fn norm(): i32 { return _.data; }
  fn scale(*_, factor: i32) { _.data = _.data * factor; }
  fn consume(move _): i32 { return _.data; }
}
```

写 AI 算子库、插件系统极舒服。

## 6）Zig：**`defer` 延迟执行**

资源安全神器，大模型显存/文件/线程句柄必备。

```cp
fn infer() {
  let mem = allocMemory(1024);
  defer freeMemory(mem); // 函数退出自动执行

  // 中途 return 也会释放
  if (err) return;
}
```

**必加，否则写底层必漏资源。**

## 7）Rust：**`async` / `.await` 但更现代化**

你是并行语言，`async` 必须有，但可以做得比 JS 更强：

```cp
async fn runPipeline() {
  let [a, b] = await Promise.all([
    loadData(),
    compileModel()
  ]);
}
```

可以加 Rust 特色：

- `async fn` 不依赖 Promise 污染
- 支持无 GC 绿色线程
- 天然适配多机分布式并行

## 8）Zig：**多返回值**

比 TS 数组/对象解构更清爽：

```cp
// 直接返回多个值
fn splitShape(t: Tensor) -> (usize, usize, usize) {
  return (t.dim(0), t.dim(1), t.dim(2));
}

let (b, c, h, w) = model.shape();
```

---

# 三、安全 & 底层，但不破坏 TS 风格

## 9）Rust：**所有权简化版（不搞那么复杂）**

不用全套 Rust 生命周期，只保留：

- `let` 不可变
- `let mut` 可变
- 传参 `&` 共享 / `*` 原始指针（底层用）

```cp
const a = Tensor.randn(); // 不可变
let b = a.clone();  // 可变
b.add_(1);
```

保持 TS 手感，同时获得底层安全。

## 10）Zig：**精准整数类型**

AI 大模型必备：`u8` / `i32` / `f16` / `f32`

```cp
let a: u8 = 255;
let b: f16 = 0.0001;
let c: usize = 1024; // 机器字长
```

比 TS `number` 强太多，适合张量、量化、CUDA 编程。

## 11）Rust：**枚举（Enum）带数据**

TS 枚举太弱，Rust enum 是杀手级：

```cp
enum Task {
  Idle,
  Compute{ batch: usize },
  Sync{ node: string }
}

match task {
  Task.Compute{batch} => runBatch(batch)
}
```

大模型任务系统、分布式调度神器。

---

# 四、最精简、最适合你的「最终组合」

给你一套**不折腾、不破坏 TS、但性能/安全拉满**的组合：

### 必加（核心）

- Rust `match` 模式匹配
- Rust `Result<T, E>` + `?` 错误传播
- Zig `defer` 资源释放
- Zig `comptime` 编译期执行
- 精准类型 `u8/i32/f16/f32/usize`

### 推荐（并行/AI 专用）

- Rust `enum` 带字段
- Rust `trait` 扩展方法
- Zig 多返回值
- 轻量所有权与深度内存保护（`const` / `let`）

### 完全保留 TS

- `import/export`
- `class/type`
- `async/await`
- 数组、对象、解构、箭头函数
- 完整的包生态

---

# 五、一句话总结你的 CP 语法气质

如果你愿意，我可以下一步直接给你写一段**完整示例代码**，把上面所有语法融合成一段 `.cp` 的大模型推理代码，让你直接看到最终语言长啥样。

# 代码示例

下面给你写一段**完整、可直观感受风格**的 **.cp 示例代码**。
风格：**主体 TypeScript + Rust 安全语法 + Zig 底层便利**，
场景：AI 大模型里最常见的 **张量加载 + 前向推理 + 错误处理 + 资源安全释放**。

---

# 示例：model-infer.cp

```cp
// 导入内置并行运行时 & 张量库
import { Tensor, Device } from "std:ai";
import { File } from "std:fs";
import { TaskPool } from "std:parallel";

// 像 TS 一样定义类型
type Config = {
  batchSize: usize,
  device: Device,
  maxSeqLen: u32
};

// Rust 风格带数据枚举（调度任务状态）
enum InferTask {
  Idle,
  Loading { path: string },
  Running { batch: Tensor },
  Completed { output: Tensor },
  Failed { reason: string }
}

// Zig 风格错误集
error ModelError {
  WeightsNotFound,
  ShapeMismatch,
  OutOfVRAM,
  DeviceUnsupported
}

// 主函数（支持 async，像 TS）
async fn main() !void {
  // 编译期常量（Zig）
  comptime {
    const MODEL_VERSION = "llama-3.1-7b-cp";
    const QUANT_TYPE = "f16";
  }

  // 不可变变量（默认 immutable，Rust 风格）
  const config: Config = {
    batchSize: 32,
    device: Device.GPU(0),
    maxSeqLen: 2048
  };

  // 默认可变变量（TS风格）
  let taskPool = TaskPool.init(16); // 并行线程数

  console.log(`Starting ${MODEL_VERSION} on ${config.device}`);

  // ==============================================
  // defer：函数退出自动释放资源（Zig 最强特性之一）
  // ==============================================
  let weightsFile = try File.open("./model.weights.bin")?;
  defer weightsFile.close(); // 不管怎么退出，都会执行

  let weights = try weightsFile.readTensor(QUANT_TYPE)?;
  defer weights.freeVRAM();

  // ==============================================
  // match 模式匹配（Rust）
  // ==============================================
  let task = InferTask.Loading { path: "./model.weights.bin" };

  match task {
    InferTask.Loading { path } => {
      console.log(`Loading from ${path}`);
    }
    InferTask.Failed { reason } => {
      return error.ModelError.LoadFailed;
    }
    _ => {}
  }

  // ==============================================
  // 多返回值（Zig）
  // ==============================================
  let (batch, seqLen, hiddenDim) = weights.shape();
  console.log(`Shape: ${batch} x ${seqLen} x ${hiddenDim}`);

  // ==============================================
  // 并行调度（语言原生支持）
  // ==============================================
  let result = await taskPool.spawn(async () => {
    return forward(weights, config.batchSize);
  });

  console.log("Inference done, output shape:", result.shape());
}

// ==============================================
// ! 表示函数返回 Result<T, Error>
// ? 自动向上传播错误（Rust）
// ==============================================
fn forward(weights: Tensor, batchSize: usize) !Tensor {
  if weights.dim(0) != batchSize {
    return error.ModelError.ShapeMismatch;
  }

  // 内置张量运算
  let hidden = weights.matmul(weights.transpose())?;
  let output = hidden.gelu()?.layerNorm()?;

  return output;
}
```

---

# 这段代码体现了你语言的核心亮点

- **像 TypeScript**
  import/export、async/await、对象结构、箭头函数、类型注解

- **加了 Rust 精华**
  - `Result<T, E>` + `?` 错误传播
  - `match` 模式匹配
  - 带数据的 `enum`
  - 默认只读不可变 `const` / 可变 `let`

- **加了 Zig 精华**
  - `defer` 资源自动释放（显存/文件/线程永不泄漏）
  - `comptime` 编译期常量
  - 精准类型 `usize / u32 / f16`
  - 多返回值 `(a, b, c)`
  - 轻量 `error` 错误集

- **专为 AI 并行设计**
  - 原生 `TaskPool` 并行调度
  - 内置 `Tensor` / `Device`
  - 显存安全、批量推理、算子链

---

# 整套体系现在完全闭环了

- 语言名：**Concurrent Plus (CP)**
- 文件后缀：**.cp**
- 主命令：**con**
- 编译器：**cpc**
- 项目配置：**con.json**（支持注释）
- 语法：**TS + Rust + Zig**
- 定位：**AI 大模型并行计算语言**

如果你想，我可以再帮你写：

- 一段更简单的入门 `hello.cp`
- 或者写一个完整的 `con.json` 配置模板
- 或者设计语言的关键字/内置类型清单

# hello.cp

给你写一个最简单、最能体现风格的入门版，干净好懂：

```cp
// Concurrent Plus - Hello World
import { print } from "std:io";

// 主入口函数
fn main() {
  print("Hello, Concurrent Plus!");
  print("专为 AI 大模型并行计算而生 🚀");
}
```

运行命令（就是你之前定的）：

```bash
con run hello.cp
```

输出：

```
Hello, Concurrent Plus!
专为 AI 大模型并行计算而生 🚀
```

---

如果你想要**带一点并行特性**的进阶 hello，我也给你写一个：

# hello_parallel.cp

```cp
import { print } from "cap:io";
import { spawn, join } from "cap:parallel";

async fn main() {
  // 并发执行两个任务
  let task1 = spawn(async () => {
    return "Hello from Task 1";
  });

  let task2 = spawn(async () => {
    return "Hello from Task 2";
  });

  // 等待结果
  let [msg1, msg2] = await join(task1, task2);

  print(msg1);
  print(msg2);
  print("Hello, Concurrent Plus!");
}
```

# con.json

（支持注释，格式完全对标 package.json，你直接就能用）

```json
{
  // 项目基本信息
  "name": "hello-cp",
  "version": "0.1.0",
  "description": "Hello Concurrent Plus",
  "author": "",
  "license": "MIT",

  // 入口文件（.cp）
  "main": "hello.cp",

  // 命令：con run / con build 等
  "scripts": {
    "start": "con run hello.cp",
    "run": "con run",
    "build": "con build",
    "dev": "con dev",
    "test": "con test"
  },

  // 依赖管理（和 npm 一样用法，con install 自动安装）
  "dependencies": {
    // 示例：
    // "@std/ai": "^0.2.0",
    // "@std/parallel": "^0.1.0"
  },

  // 开发依赖
  "devDependencies": {
    // "@cpc/lint": "^0.1.0"
  },

  // 编译器配置（底层 cpc 编译参数）
  "compilerOptions": {
    "target": "native",
    "optimize": true,
    "outDir": "./dist",
    "sourceMap": false,
    "parallel": true
  },

  // 运行时配置
  "runtime": {
    "workers": 4,
    "memoryLimit": "8GB",
    "device": "auto"
  }
}
```

### 使用方式

- 安装依赖：  
  `cap install`
- 直接运行：  
  `cap run`
- 构建二进制：  
  `cap build`

整个结构和 `package.json` 几乎一样，前端/Node 过来的用户一眼就能上手。

# Concurrent Plus (CP)

## 关键字 & 内置类型 完整清单

风格：**TypeScript 主体 + Rust 安全 + Zig 底层**，简洁、现代、适合 AI 并行计算。

---

# 一、核心关键字（必选）

### 1. 声明类

- `fn`
- `let`
- `mut`
- `const`
- `comptime`
- `class`
- `type`
- `interface`
- `enum`
- `impl`
- `trait`
- `struct`

### 2. 控制流

- `if`
- `else`
- `for`
- `while`
- `loop`
- `break`
- `continue`
- `return`
- `match`
- `defer`

### 3. 错误处理

- `?`（运算符关键字）

### 4. 异步 & 并行

- `async`
- `await`
- `spawn`
- `join`
- `parallel`

### 5. 模块 & 导入

- `import`
- `export`
- `from`
- `as`

### 6. 访问/修饰

- `pub`
- `priv`
- `this`
- `super`

### 7. 空/布尔

- `true`
- `false`
- `null`
- `void`

### 8. comptime

- `!`

---

# 二、内置基础类型（Zig 风格精准类型）

### 整数（有符号）

- `i8`
- `i16`
- `i32`
- `i64`
- `i128`

### 整数（无符号）

- `u8`
- `u16`
- `u32`
- `u64`
- `u128`

### 浮点数（AI 专用）

- `f16`
- `f32`
- `f64`
- `bf16`

### 指针/长度（底层专用）

- `usize`
- `isize`
- `*T`（指针）
- `&T`（引用）

### 通用类型（TS 风格）

- `bool`
- `char`
- `string`

---

# 三、内置容器/泛型类型

- `Array<T>` / `T[]`
- `Map<K, V>`
- `Set<T>`
- `Option<T>`
- `Result<T, E>`
- `Promise<T>`
- `Task<T>`

---

# 四、AI / 并行专用内置类型

- `Tensor`
- `Shape`
- `Device`
- `Worker`
- `Channel<T>`
- `Buffer`
- `Stream`

---

# 五、完整极简版（可直接放文档）

```
fn let const comptime
class type interface enum impl trait struct
if else for while loop break continue return match defer
? !
async await spawn join parallel
import export from as
pub priv this super
true false null void

i8 i16 i32 i64 i128
u8 u16 u32 u64 u128
f16 f32 f64 bf16
usize isize *T &T
bool char string any

Array<T> Map<K,V> Set<T>
Option<T> Result<T,E>
Promise<T> Task<T>

Tensor Shape Device Worker
Channel<T> Buffer Stream
```

需要我再给你做一份 **CP 语言官方语法速查表（一页版）** 吗？可以直接放官网或文档里。

你这个判断**非常准、非常底层**，完全说到关键点上了：

**CP 要编译到 Zig → 就不能走 JS/TS 那种动态路线**
Zig 是**静态强类型、AOT 编译、无GC、无动态原型**，
所以：

下面我给你一套**能完美编译到 Zig、又保留 TS 书写体验**的最终方案，
既不破坏语法美感，又能1:1翻译成合法 Zig。

---

# 一、先明确结论

## 1. `any` → **直接去掉，不要支持**

Zig 没有动态类型，没有原型链，不能随便加属性、不能随便赋值。
如果在 CP 里加 `any`，你编译器会非常痛苦，而且**失去静态安全+性能优势**。

替代方案：

- 用 `unknown`（严格安全的顶层类型）
- 用泛型 `<T>`
- 用 `union` 类型
- 底层编译直接对应 Zig 的 **Tagged Union / 泛型**

## 2. `try/catch` → **不要用，改用 Rust/Zig 风格错误处理**

Zig **没有异常（Exception）**，只有：

- error set
- error union

所以你在 CP 里：

- 保留 **`?` 错误传播**（最爽）
- 保留 **`Result<T, E>`**
- 保留 **`error` 定义**
- **彻底去掉 JS 风格的 `try/catch/throw`**

这样编译到 Zig 是**丝滑无缝**的。

---

# 二、修正后的 CP 核心设计（适配 Zig 静态编译）

## 1）类型系统：只保留**静态可编译**的

- 保留：
  - `bool`
  - `string`
  - 数字类型：`u8 i32 f32 f16 bf16 usize`
  - `struct` / `enum` / `type` / `interface`
  - `union`（编译到 Zig Tagged Union）
  - 泛型 `<T>`
  - `Option<T>` → 编译成 Zig 可选类型
  - `Result<T, E>` → 编译成 Zig error union

这一套**100% 静态、无动态行为、完美下翻到 Zig**。

## 2）错误系统：完全对齐 Zig

### CP 里这样写（你之前的语法）

```cp
error IOError {
  FileNotFound,
  ReadFailed
}

fn readFile(path: string) !string {
  if (!exists(path)) {
    return error.FileNotFound;
  }
  return read(path);
}

// ? 自动向上抛错误
fn load() !void {
  let content = readFile("a.txt")?;
}
```

### 编译到 Zig 就是直接对应：

```zig
const IOError = error{
  FileNotFound,
  ReadFailed
};

fn readFile(path: []const u8) ![]const u8 {
  // ...
}

fn load() !void {
  const content = try readFile("a.txt");
}
```

**完美一一对应，没有任何动态魔法。**

---

# 三、那 TS 的动态体验怎么保留？

你不用真的上动态，只要**语法像 TS**就行：

## 1. 对象字面量 → 编译成 struct

```cp
let obj = {
  name: "cp",
  version: 1
};
```

↓ 编译到 Zig

```zig
struct {
  name: []const u8,
  version: i32
}
```

## 2. 函数自动类型推导 → 看起来像动态，其实是静态

```cp
fn add(a, b) {
  return a + b;
}
```

编译器自动推：

```cp
fn add(a: i32, b: i32) i32 {
  return a + b;
}
```

## 3. 静态结构体与动态字典权责分离

- **静态匿名对象 `{ ... }` 与 `class`/`struct`**：对应底层编译期确定内存布局的静态结构体，追求寄存器级极速与零开销，不支持在静态结构体上动态增删未声明字段；
- **动态字典 `#{ ... }` 与 `Map`**：若业务场景需要类似 JS Object / Python dict 的任意动态增删属性与键值扩展，CP 提供了专属的**动态字典语法糖 `#{ ... }` (Dict)** 与 **`Map`**（详见第十九、二十章），兼顾静态极致性能与动态灵活性。

---

# 四、最终精简：CP 应该支持 / 不支持清单

## ✅ 支持（能编译到 Zig，且好用）

- 静态类型 + 类型推导
- `struct` / `enum` / `union` / 泛型
- `Result<T, E>` / `Option<T>`
- `error` + `?` 错误传播
- `defer`（Zig 神器，必须有）
- `@comptime` + `!` 编译期执行
- 模块 `import/export`
- 模式匹配 `swich`
- 异步/并行 `async/await/spawn`（编译成 Zig 异步/线程）

## ❌ 不支持（和 Zig 冲突，必须砍掉）

- **`any`**
- **JS 风格 `try/throw` 异常**
- 动态对象增删属性
- 原型链
- 运行时 `eval` / 动态代码
- 隐式类型强制转换（尽量少）

---

# 五、一句话总结（最关键）

**CP = TypeScript 书写风格 + Rust 错误模式 + Zig 静态底层**
**不做动态，不做 any，不做异常，只做高性能可AOT编译的并行语言。**

这样你的编译器 **cpc** 才能轻松把 `.cp` → 高质量 Zig → 高效二进制。

要不要我按这个最终版，重新给你整理一版**正式的 CP 关键字+内置类型规范**？可以直接放进语言白皮书。

## 异常处理

> 没有 try catch 和 throw 的话，那我们怎么处理这个错误？怎么抛出错误呢？

非常好的问题！
既然要**编译到 Zig**，而且 Zig 本身**没有异常（try/catch/throw）**，那我们就直接用 **Zig 那套错误机制**，但语法包装成 **TypeScript 风格**，用户写起来舒服，你编译也轻松。

我给你一套**完全可落地、能直接翻译成 Zig**的错误处理方案：

# 核心思路

- 不用：`try {} catch {}`
- 不用：`throw new Error()`
- 改用：**错误集 + ？返回标记 + ? 自动向上抛 + 手动 return error**

这套就是 **Zig + Rust 混合**，但写起来像现代脚本语言。

---

# 1）怎么定义错误？（替代 throw）

直接定义一组 **error**（和 Zig 一模一样）

```cp
// 定义一组错误类型
error ModelError {
  FileNotFound,
  OutOfMemory,
  ShapeMismatch,
  DeviceNotSupported
}
```

要“抛出”时，**直接 return error** 就行：

```cp
fn loadWeights(path: string) ?Tensor {
  if (!fileExists(path)) {
    // 这就是“抛出错误”
    return error.FileNotFound;
  }

  return Tensor.load(path);
}

```

---

# 2）函数怎么标记会抛错？

用 **?** 放在返回值前面

```cp
// 表示：返回 Tensor 或 错误
fn load() ?Tensor { ... }

// 表示：返回 void 或 错误
fn init() ?void { ... }
```

这会直接编译成 Zig 的：

```zig
fn load() !Tensor { ... }
```

---

# 3）调用时怎么处理错误？（替代 try/catch）

## 方式 A：用 ? 自动向上抛（最常用）

```cp
fn main() ?void {
  // 出错直接往上抛，不用写 catch
  let weights = loadWeights("model.bin")?;
  let model = buildModel(weights)?;

  model.run();
}
```

等价于：

> 如果出错 → 直接返回错误，不往下走

## 方式 B：用 match 匹配错误（完整捕获）

```cp
let result = loadWeights("model.bin");

match result {
  Ok(tensor) => {
    print("加载成功");
  }
  Err(error.FileNotFound) => {
    print("模型文件不存在");
  }
  Err(err) => {
    print("其他错误: ", err);
  }
}
```

## 方式 C：用 if 解构（简单判断）

```cp
if let Ok(tensor) = loadWeights("model.bin") {
  // 成功
} else {
  // 失败
}
```

## Option

对应 zig 中的 `orelse`

```cp
// 用于 Option
fn first_char(s: string) ?u8 {
    let c = s.chars().next()?;  // 如果是 None 就返回
    Some(c)
}

// 甚至可以混用（需要转换）
fn get_first_char() ?u8  {
    let s = read_string()?;           // Result -> 传播错误
    let c = s.chars().next().ok_or(io::ErrorKind::UnexpectedEof)?;  // Option -> Result
    Ok(c)
}
```

## comptime

```cp
// 定义：普通样子
fn add(a: i32, b: i32) i32 {
  return a + b
}

// 调用：加 ! 表示编译期执行
let x = add!(1, 2)
```

问题：

- 同一个函数，既可以运行时调用，又可以编译期调用 → 混乱
- 函数行为**由调用方决定**，不是函数本身属性

# 最漂亮的示例（最终版）

```cp
// 普通函数
fn foo() i32 {
  return 10
}

// 可错误函数
fn bar() ?i32 {
  return 20
}

// 调用全部干净自然
fn main() {
  let a = foo!()
  let b = bar?()
}
```

---

2）**调用加 !，定义可以不加吗？**
→ **不推荐，会造成语言混乱、难以静态编译。**

---

# 4）这套机制的好处

- **完全静态类型**，没有运行时开销
- **1:1 编译成 Zig**，你编译器几乎不用做额外工作
- **没有 GC、没有异常栈、没有动态开销**
- 写起来比 try/catch 更干净、更可控
- 适合 AI/并行/底层系统

---

# 5）一句话总结

**CP 语言里：**

- 抛错 → `return error.Xxx`
- 标记抛错 → `fn() ?T`
- 传递错误 → `fn()?`
- Option -> `const value = maybe_null?`
- 捕获错误 → `match` / `if let Ok`

**完全不需要 try/catch/throw。**

---

# 给你一个完整可运行的 hello-error.cp

```cp
import { print } from "std:io";

error DemoError {
  SomethingWrong
}

fn doSomething() ?string {
  return error.SomethingWrong;
}

fn main() ?void {
  // 出错直接往上抛
  let msg = doSomething()?;
  print(msg);
}
```

---

如果你愿意，我可以再给你整理一份：
**CP 错误处理完整规范（可直接写进语言文档）**
包括：错误集、!、?、match、Ok/Err 全部用法。

# Concurrent Plus（CP）语言错误处理完整规范

我结合CP语言**编译至Zig、静态强类型、AI并行计算**的核心定位，参照Zig错误机制+Rust错误处理逻辑，同时贴合TS语法书写习惯，整理了这份完整、可直接落地、适配编译器开发的**CP语言错误处理完整规范**，包含语法定义、使用规则、编译映射、示例全流程。

## 一、规范总则

1. **核心定位**：CP为**静态编译型语言**，底层编译至Zig，不支持动态异常、无运行时异常栈，摒弃JS/TS风格`try/catch/throw`异常机制
2. **设计依据**：完全对齐Zig错误模型，融合Rust Result模式，兼顾底层性能、类型安全与TS语法易用性
3. **适用场景**：AI大模型并行计算、系统级开发、资源调度等高性能、无GC场景
4. **核心原则**：错误可显式感知、静态编译检查、无运行时开销、1:1映射至Zig代码

## 二、核心错误语法定义

### 1. 错误集（Error Set）

- **定义**：用于声明一组相关的具名错误，替代传统异常类，编译直接映射为Zig错误集
- **语法**：

```cp
// 全局/局部错误集定义
error 错误集名 {
  错误类型1,
  错误类型2,
  错误类型3,
  ...
}
```

- **规则**：
  - 错误集名为大驼峰，错误类型采用大驼峰命名
  - 支持空错误集、自定义业务错误集
  - 支持内置通用错误集（如`IOError`、`MemError`）
- **示例**：

```cp
// AI模型相关错误集
error ModelError {
  WeightsNotFound,    // 权重文件不存在
  ShapeMismatch,      // 张量形状不匹配
  OutOfVRAM,          // 显存不足
  DeviceUnsupported   // 设备不支持
}

// 文件IO错误集
error IOError {
  FileOpenFailed,
  ReadDenied,
  WriteFailed
}
```

### 2. 可抛错函数声明

- **定义**：用`!`标记函数可能返回错误，替代`throws`关键字，明确函数错误风险
- **语法**：

```cp
// 无返回值、可抛错
fn 函数名(参数列表) !void { ... }

// 有返回值、可抛错
fn 函数名(参数列表) !返回类型 { ... }
```

- **规则**：
  - `!`位于返回类型前，代表函数返回**结果或错误**
  - 未标记`!`的函数，禁止返回/抛出任何错误
  - 编译映射为Zig`!T`错误联合类型
- **示例**：

```cp
// 加载张量，返回Tensor或ModelError
fn loadTensor(path: string) !Tensor { ... }

// 初始化并行池，无返回值、可抛错
fn initParallelPool(workers: usize) !void { ... }
```

### 3. 抛出错误

- **定义**：通过`return error.错误类型`显式抛出错误，替代`throw`关键字
- **语法**：

```cp
return error.错误类型;
```

- **规则**：
  - 仅可在标记`!`的可抛错函数内抛出错误
  - 抛出的错误必须属于当前函数兼容的错误集
  - 无隐式抛错，所有抛错行为均为显式编码
- **示例**：

```cp
fn loadTensor(path: string) !Tensor {
  if (!fileExists(path)) {
    // 显式抛出错误
    return error.WeightsNotFound;
  }
  if (getVRAMUsage() >= MAX_VRAM) {
    return error.OutOfVRAM;
  }
  return Tensor.fromFile(path);
}
```

### 4. 错误传播（? 运算符）

- **定义**：通过`?`自动向上传递错误，简化多层函数错误嵌套，替代多层try/catch
- **语法**：

```cp
// 调用可抛错函数，直接传播错误
let 变量 = 可抛错函数调用?;
```

- **规则**：
  - 仅可在标记`!`的函数内使用`?`
  - 函数调用后紧跟`?`，若返回错误，直接向上层返回该错误
  - 无额外运行时开销，编译直接映射为Zig`try`关键字
- **示例**：

```cp
fn buildModel(weightPath: string) !Model {
  // 错误直接向上传播，无需手动捕获
  let tensor = loadTensor(weightPath)?;
  let model = initModel(tensor)?;
  return model;
}
```

### 5. 错误捕获与处理

#### （1）模式匹配捕获（match，推荐）

- 用`match`匹配`Ok/Err`分支，完整处理所有错误类型，编译映射为Zig模式匹配
- **语法&示例**：

```cp
fn main() !void {
  let result = buildModel("./model.weights");
  // 模式匹配处理结果与错误
  match result {
    // 成功分支：获取返回值
    Ok(model) => {
      model.runInfer();
      print("模型推理启动成功");
    },
    // 精准匹配指定错误
    Err(error.WeightsNotFound) => {
      print("错误：模型权重文件不存在");
    },
    Err(error.OutOfVRAM) => {
      print("错误：显存不足，无法加载模型");
    },
    // 匹配其余所有错误
    Err(err) => {
      print("模型构建失败，错误码：", err);
    }
  }
}
```

#### （2）条件解构捕获（if let Ok）

- 简单场景下的快捷捕获，仅处理成功分支，忽略/简处理错误
- **语法&示例**：

```cp
fn main() {
  // 非抛错函数内，只能用此方式处理
  if let Ok(model) = buildModel("./model.weights") {
    model.runInfer();
  } else {
    print("模型启动失败");
  }
}
```

## 三、内置泛型结果类型

### 1<T, E>

- **定义**：内置泛型类型，代表**成功值T**或**错误E**，无需手动声明
- **说明**：可抛错函数`fn() !T`<T, 对应错误集>`
- **使用场景**：显式声明结果变量、函数传参、异步返回

### 2. Option<T>

- **定义**：内置泛型类型，代表**有值T**或**空值None**，用于非错误性空值场景
- **语法&示例**：

```cp
// 返回可选张量，非错误，仅代表无值
fn get<Tensor> {
  if (cacheExists()) {
    return Some(loadCache());
  }
  return None;
}
```

## 四、语法禁用规则

1. 禁止使用JS/TS风格`try {} catch {} finally {}`
2. 禁止使用`throw`关键字抛错
3. 禁止隐式错误捕获、隐式错误传播
4. 禁止在非抛错函数（无`!`标记）内处理/传播错误

## 五、编译映射规则（CP → Zig）

| CP语法                    | 对应Zig语法                  | 说明                    |
| ------------------------- | ---------------------------- | ----------------------- |
| `error 错误集 {}`         | `const 错误集 = error{...}`  | 直接映射Zig错误集       |
| `fn() !T`                 | `fn() !T`                    | 完全对齐Zig错误联合类型 |
| `return error.Xxx`        | `return error.Xxx`           | 显式返回错误            |
| `func()?`                 | `try func()`                 | 错误传播直接转换        |
| `match result { Ok/Err }` | `switch result { .ok/.err }` | 模式匹配映射            |

## 六、最佳实践

1. **按业务拆分错误集**：将AI、IO、并行、内存相关错误分类定义，便于精准处理
2. **优先用?传播错误**：多层调用时减少冗余错误处理代码
3. **顶层必捕获错误**：main函数等入口函数，必须用match处理所有错误
4. **禁止忽略错误**：不允许调用可抛错函数后不做任何处理
5. **资源释放搭配defer**：错误处理前用`defer`保证资源自动释放

## 七、完整实战示例

```cp
// 导入标准库
import { print, fileExists } from "std:io";
import { getVRAMUsage } from "std:device";

// 定义错误集
error ModelError {
  WeightsNotFound,
  OutOfVRAM,
  InitFailed
}

// 可抛错函数：加载权重
fn loadWeights(path: string) !Tensor {
  if (!fileExists(path)) {
    return error.WeightsNotFound;
  }
  if (getVRAMUsage() >= 8192) {
    return error.OutOfVRAM;
  }
  return Tensor.fromFile(path);
}

// 可抛错函数：构建模型
fn buildModel(path: string) !Model {
  // 错误传播
  let weights = loadWeights(path)?;
  let model = Model.create(weights)?;
  return model;
}

// 主函数：错误捕获处理
fn main() {
  // 资源自动释放
  let modelFile = fileOpen("./model.weights");
  defer modelFile.close();

  // 模式匹配捕获所有错误
  match buildModel("./model.weights") {
    Ok(model) => {
      model.startParallelInfer(4);
      print("AI大模型并行推理启动成功");
    },
    Err(error.WeightsNotFound) => {
      print("【错误】模型权重文件不存在，请检查路径");
    },
    Err(error.OutOfVRAM) => {
      print("【错误】显存不足，无法加载模型");
    },
    Err(err) => {
      print("【错误】模型启动失败，未知错误：", err);
    }
  }
}
```

这份规范直接适配你的**cpc编译器**开发，也能直接作为CP语言官方文档，需要我把里面的**核心语法提炼成简易速查表**，方便开发者快速查阅吗？

---

# 八、最新核心架构与语法特性 (2026-10 落地)

近期，CP 编译器在底层完成了由 JS Agent 向 Native Agent 的历史性跨越（基于 Bun-Nest 混合底座），并正式实装了一系列核心语法。

## 1. Move 语义与所有权转移 (Ownership & Move)

CP 引入了类似 Rust 的严格所有权编译期拦截，但语法保持极简。

- **转移语法**：通过 `move` 关键字显式转移变量的所有权（例如跨 Actor 传递大数据集或句柄）。
- **静态保护**：一旦变量被 `move`，若在作用域内继续访问，编译器将直接报错 (`CP Compile Error: Use of moved value`)，从根本上杜绝悬垂指针与数据竞争。

## 2. Await 异步调用与 CPS 状态机切片 (Context Lifting)

CP 不使用传统的调用栈，而是基于 CPS（连续传递风格）实现无栈协程。

- **Context 提升**：遇到 `await` 时，跨越该断点的所有局部变量，会被编译器自动提取并提升至堆分配的 `_Context` 结构体中。
- **原生态让权**：`await` 在底层被翻译为物理 `return`，将执行权（和当前的 Context 序列化快照）交还给 Nest 调度器（或原生的 Epoll Reactor），实现真正的非阻塞抢占。
- **OOP Await**：完美支持面向对象的异步调用，例如 `await obj.method()`。

## 3. 面向对象：init 构造函数与 release 析构函数

`class` 被定位为纯粹的数据容器（内存布局）。为了对接底层资源的获取与销毁：

- **`init` 构造器**：负责分配资源（如打开网络 Socket，拉起 Worker 线程）。
- **`release` 析构器**：负责精准销毁底层句柄（如 `std.posix.close`）。

## 4. 变量自动 defer 与 RAII

为了配合 `release` 析构，CP 编译器引入了基于 Zig `comptime` 的 **Auto-Drop 机制**。

- **自动释放**：局部作用域内声明的类实例对象，当离开作用域时，编译器会自动隐式注入 `defer obj.release()` 和 `errdefer`。
- **零泄漏**：无论是正常返回，还是通过 `?` 向上抛出错误，亦或是 Actor 被强制回收，底层资源（显存、文件描述符）都将 100% 安全释放，全程无 GC 卡顿。

## 5. 多线程 Worker 与跨线程无锁信箱 (cap:worker)

借助 PID（`node_id:local_id`）结构，CP 实现了原生的 Erlang 风格并发。

- **独立 Reactor**：通过 `@worker_spawn` 启动独立 OS 线程，每个线程拥有自己 `threadlocal` 的任务队列与 Epoll 反应器。
- **无锁环形信箱**：Worker 线程间通过原子操作读写专属的环形缓冲区（Ring Buffer）实现极速跨线程通信，无需使用全局 Mutex 锁。

## 6. 高阶标准库生态

- **ES6 模板字符串**：支持 `` `Hello ${name}` `` 语法，底层映射为零开销的 Zig `allocPrint`（由 Actor Arena 内存池极速回收）。
- **TS 字符串方法**：完美对标 ES5/ES6 接口（如 `indexOf`, `slice`, `replace`, `split`），实现 0 开销静态桥接。
- **`cap:http` 与网络抽象**：对标 Bun.js 的原生 `serve({ port, fetch })` API 范式。具备从底层 Socket 到 HTTP 报文序列化的完整非阻塞 I/O 能力。
- **双模运行底座** (集成 CP NATIVE ARCH)：
  - **Bun-Nest 模式**：不允许原生阻塞 IO。网络调用等动作转换为极速二进制 `10-89` 指令派发给 `99990001` 网关处理，Actor 自身 `yield`。
  - **Native 单机模式**：脱机独立运行，由内置的 `epoll/kqueue` Reactor 事件分发驱动高速 I/O。

---

# 九、架构演进与完善建议 (Architecture Master Proposals)

作为编程语言设计者，为了让 CP 语言在目前的坚实基础上更加**完整、完美、优雅、简洁、高效**，我提出以下 5 点深度演进建议：

### 1. 泛型信箱与强类型消息通道 (Typed Mailbox & Channels)

**现状**：目前 Actor 通信依赖全局的 `Message` 联合体进行底层收发。
**建议**：引入泛型约束的通道（类似 Rust 的 `mpsc::Sender<T>` 或 Erlang 增强版），例如 `let pid: PID<TaskEvent> = spawn(...)`。这样在编译期就能验证跨 Actor/线程 的消息类型，消除由于运行期解包匹配（`match`）带来的潜在崩溃风险（毒药消息）。

### 2. 管线操作符 (Pipeline Operator `|>`)

**现状**：在数据密集型或 HTTP 流式处理场景中，深度嵌套的函数调用会破坏可读性。
**建议**：引入函数式编程的流式计算符。例如：`request.body |> sanitize() |> json() |> send(client)`。这不仅符合 TS 现代心智，更与 CP 的 CPS 切片和零拷贝设计完美契合，使得代码数据流极其清爽。

### 3. 降维模式匹配与 Guard 守卫

**现状**：`match` 已经成功用于错误和枚举匹配。
**建议**：进一步增强 `match`，使其支持对象结构的深层解构匹配，并引入守卫表达式（Guard Clauses）。例如：

```cp
match req {
    { method: "GET", url } if url.startsWith("/api") => handleApi(url),
    * => return 404
}
```

这将大幅度精简网络路由框架的样板代码。

### 4. 编译期鸭子类型 (Comptime Duck Typing) 与 Trait

**现状**：为了极致性能废除了 JS 的 `interface` 动态行为，数据和方法解耦。
**建议**：利用底座 Zig 强大的 `@hasDecl` 与 `comptime` 反射机制，实现静态的鸭子类型（`impl Trait for Struct`）。这样即便没有虚函数表（VTable）开销，也能在编译期实现多态。这使得未来写跨节点的 RPC 调用就像调用本地方法一样自然。

### 5. 开放 Comptime 宏沙盒，实现自举

**现状**：许多底层指令（如 `@spawn`, `@json_stringify`）依旧依赖 Rust 编译器前端硬编码处理。
**建议**：下放部分元编程权限，为开发者提供一个 `comptime {}` 的宏执行沙盒。允许资深用户用 CP 语法本身去控制编译期的代码生成（例如自动生成路由表、 ORM 实体映射），使得语言真正具备“自我进化”的能力。

# 十、内存安全模型 (Zero-Friction Memory Safety)

CP 语言通过编译器底层的“所有权、隐式借用、读写互斥与零摩擦生命周期”四大支柱，实现了对标 Rust 的 100% 编译期内存安全，同时彻底摒弃了 Rust 繁琐的 `<'a>` 生命周期泛型传染与冗长的 `&mut` 噪音，完美保留了 TypeScript 般丝滑的开发心智。

## 0. 全局借用、所有权与指针符号统一标准 (Global Symbol Matrix)

在 CP 语言中，**1 个符号永远代表 1 种固定语义（全局零例外）**：
- **`&`（Ampersand）**：永远代表 **只读借用（Read-Only Borrow）**（在默认只读的位置可省略 `&`）。
- **`*`（Star，无尾置 `!`）**：永远代表 **安全可变借用（Mutable Borrow）**（因指针解引用采用后缀 `p.*`，前置 `*` 全局专职代表受编译器生命周期与读写互斥保护的可变借用）。
- **`*... !`（前置 `*` + 尾置 `!`）**：永远代表 **Unsafe 裸指针（Raw Pointer）**（前置 `*` 个数代表指针层级，尾置 `!` 代表非托管裸指针，仅限 `unsafe` 块内使用）。
- **`move`（及直接赋值 `let a = b`）**：永远代表 **所有权转移（Ownership Transfer）**。

| 语法场景 | 只读借用 (Read-Only) | 可变借用 (Mutable) | 所有权转移 (Move) | Unsafe 裸指针 (Raw Ptr) |
| :--- | :--- | :--- | :--- | :--- |
| **1. 函数参数** | `fn f(x: T)` *(默认省略)*<br>*(兼容显式 `fn f(&x: T)`)* | **`fn f(*x: T)`**<br>*(或 `fn f(x: *T)`)* | **`fn f(move x: T)`** | `unsafe fn f(p: *T!)` |
| **2. 变量赋值** | **`let a = &b`** | **`let a = *b`** | **`let a = b`**<br>*(或 `let a = move b`)* | `let a = *b!`<br>`let pp = **a!` |
| **3. 类型声明 / 返回值** | **`: &T`** | **`: *T`** | **`: T`** | **`: *T!`** / **`: **T!`** |
| **4. 对象/结构 `this`/`_`** | `fn get()` *(默认省略 `_`)*<br>*(兼容 `fn get(_)` / `fn get(&_)`) * | **`fn set(*_, v: i32)`**<br>*(兼容 `*this` / `mut _`)* | **`fn close(move _)`**<br>*(兼容 `move this`)* | — |
| **5. `for` 循环迭代** | `for x of list` *(默认省略)*<br>*(兼容 `for &x of list`)* | **`for *x of list`** | **`for move x of list`** | — |

## 1. 所有权与自动释放 (Ownership & Auto-Drop)

- **局部变量去冗余**：废除局部变量的 `let mut`，统一使用 `let`（可变绑定）与 `const`（深度不可变）。
- **严禁同作用域重复声明变量（No Same-Scope Redeclaration）**：对标 TypeScript 与 Zig 的工程安全直觉，在同一个 `{}` 块作用域内严禁重复声明同名变量（如连续写 `let v = 10; let v = [10, 20];` 或在函数顶层块重声明同名参数），编译器将直接报错拦截：`Compile Error: Cannot redeclare block-scoped variable 'v'`（通配丢弃符 `_` 除外，且允许在不同/嵌套的子块 `{ ... }` 中声明同名局部变量）。
- **创建即拥有**：`let tensor = new Tensor();` 创建对象时，`tensor` 拥有该堆内存的唯一所有权。
- **作用域释放 (RAII)**：当所有者变量离开作用域且未被转移所有权时，编译器自动注入 `defer tensor.release()`（无需 GC，零停顿）。
- **赋值即 Move**：`let a = b;`（或显式 `let a = move b;`）会将 `b` 的所有权转移给 `a`；而 `let a = &b;` 创建只读借用，`let a = *b;` 创建可变借用。所有权转移后继续访问 `b` 将触发编译期错误：`CP Compile Error: Use of moved value`。

## 2. 隐式借用与读写推导 (Implicit Borrow Deduction)

函数传参默认**不转移所有权**，而是由编译器自动生成**隐式借用**。借用权限由函数签名中的**参数修饰符**决定：

- **只读借用 (默认或 `&`)**：`fn view(data: Tensor)`（或 `fn view(&data: Tensor)`）-> 调用 `view(a)` 时，自动传入只读引用 `&T`，函数内禁止修改。
- **可变借用 (`*`)**：`fn modify(*data: Tensor)`（或 `fn modify(data: *Tensor)`，兼容 `mut data: Tensor`）-> 调用 `modify(a)` 时，自动传入可变引用 `*T`，允许原地修改。
- **转移所有权 (`move`)**：`fn consume(move data: Tensor)` -> 调用 `consume(a)` 时，转移所有权，调用后外层 `a` 立即失效。

## 3. 借用检查器规则与可变借用操作规范 (Borrow Checker & Mutable Borrow Operations)

- **读写互斥**：在同一有效生命周期内，允许存在多个只读借用（`&T`），或仅允许存在**唯一一个**可变借用（`*T`）。可变借用与任何其他借用严格互斥。
- **借用期间锁定所有者**：当变量 `a` 存在尚未结束的活跃借用 `r`（即 `r` 在后续代码中还会被读取）时，严禁将 `a` 的所有权 Move 转移或对其进行破坏性修改。
- **可变借用变量的三类核心操作（自动解引用与显式 `.*`）**：
  1. **基础值类型（`i32`, `f32`, `bool` 等）**：通过后缀解引用 `p.* = val` 原地修改所借用变量的值；
  2. **结构体 / 对象字面量（`{a: 10, b: 20}` / `struct` / `class`）**：支持自动解引用，直接通过 `p.a = val` 原地修改字段；
  3. **容器类型（`Array` / `Vec` / `T[]`, `Map`, `Set`, `Dict`）**：支持自动解引用，直接通过 `p.push(...)`、`p.pop()`、`p.set(...)`、`p.add(...)`、`p.delete(...)`、`p.clear()`、`p[idx] = val` 原地修改容器内容。
  ```cp
  // 1. 基础类型可变借用：后缀 .* 解引用赋值
  {
      let v = 10;
      let p = *v;
      p.* = 20; // v 同步变为 20
  }

  // 2. 对象/结构体可变借用：自动解引用修改字段
  {
      let v = {a: 10, b: 20};
      let p = *v;
      p.a = 100; // v.a 同步变为 100
  }

  // 3. 数组/容器可变借用：自动解引用调用可变方法与索引赋值
  {
      let v = [10, 20];
      let p = *v;
      p.push(30); // v 同步扩容为 [10, 20, 30]
      p[1] = 25;
  }
  ```
- **容器改变操作全面纳入可变借用检查 (`Array` / `Vec`, `Map`, `Set`, `Dict`)**：
  - **受检容器改变操作清单**：
    - `Array<T>` / `Vec<T>` / `T[]`：`push(v)`、`pop()`、`clear()`、索引写入 `arr[i] = v`、`for *x of arr` 原地可变遍历；
    - `Map<K, V>`：`set(k, v)`、`delete(k)`、`clear()`、索引写入 `map[k] = v`；
    - `Set<T>`：`add(v)`、`delete(v)`、`clear()`；
    - `Dict<K, V>` (`#{}`)：`set(k, v)`、`put_all(other)`、`delete(k)`、`clear()`、索引写入 `dict[k] = v`。
  - **四重编译期拦截保障**：
    1. **禁止通过只读借用 (`&v`) 或只读参数改变容器**：如 `let r = &v; r.push(30);` 直接报错 `Cannot perform mutable operation (container method 'push') through read-only borrow 'r'`；
    2. **读写互斥与活跃借用冲突拦截**：当容器存在活跃的只读借用 `let r = &m;` 或另一个可变借用时，对原容器或冲突借用执行 `set`/`push`/`delete`/`clear`/`[] =` 立即报错 `Borrow conflict on 'm': cannot borrow as mutable while 'r' is still active`；
    3. **防御迭代器失效（Loop-Borrow Lock）**：在 `for x of container` 或 `for *x of container` 遍历期间，严禁在循环体内对该容器（或其借用别名）调用 `push`、`pop`、`set`、`add`、`delete`、`clear` 或重赋值；
    4. **Use-After-Move 拦截**：容器所有权 `move` 转移后，任何读写或方法调用立即报错拦截。

## 4. 零摩擦生命周期 (Zero-Friction Lifetimes)

CP 彻底废除了 Rust 晦涩且具有传染性的 `<'a, 'b>` 生命周期泛型符号，采用 **“局部 NLL 全自动推导 + 函数返回三级梯队 + 结构体免传染”** 架构：

### （1）局部作用域：非词法生命周期 (NLL) 与防悬垂推导（100% 零标注）

- **作用域深度防悬垂 (Scope Depth Check)**：外层变量深度小（活得长），内层块 `{}` 变量深度大（死得早）。**严禁将内层短命变量的借用赋值给外层长命变量**，否则编译报错：`Borrowed value does not live long enough`。
- **非词法生命周期 (Non-Lexical Lifetimes - NLL)**：借用变量 `let r = &a;`（只读）或 `let m = *a;`（可变）的生命周期**并非持续到花括号结束，而是精准终止于该借用变量的最后一次使用（Last Use）处**。在最后一次使用之后，原变量 `a` 立即解除锁定，可安全进行 `move` 或可变操作。

### （2）函数返回借用：三级推导梯队（99% 零标注，1% 参数名直观绑定）

首先，**严禁函数返回内部创建的局部变量的借用**（悬垂返回直接编译报错）。当函数返回一个借用（只读 `: &Tensor` 或可变 `: *Tensor`）时，按以下三级梯队解析来源：

1. **第 1 级（自动省略 Elision）**：若只有一个引用类参数，或为类方法（含 `this` / `_`），返回值的生命周期自动绑定到该唯一参数或 `this` / `_`。
2. **第 2 级（默认交集 Default Union —— 无需标注）**：若有多个对象参数且未写标注（如 `fn pick(a: Tensor, b: Tensor): &Tensor`），编译器**自动将返回值生命周期绑定为所有对象参数生命周期的交集 `min(a, b)`**。只要调用处使用返回值时 `a` 和 `b` 均存活，即完全合法！
3. **第 3 级（极简参数名绑定 `&param` / `*param` —— 告别 `<'a>`）**：仅当存在多个对象参数，但返回值只借用自特定参数（且希望其他参数提前释放）时，直接在 `&` 或 `*` 后指认参数名：
   ```cp
   // 明确声明：返回的只读借用 Tensor 仅借用自参数 x，与 y 的死活无关！
   fn selectFirst(x: Tensor, y: Tensor): &x Tensor {
       return &x;
   }
   ```
   若可能借用自多个指定参数，支持 TS 风格的联合指认：`: &(x | y) Tensor` 或 `: *(x | y) Tensor`。

### （3）结构体与并发逃逸法则：Shared<T> 与 栈绑定视图 (Stack-Bound View / ref struct)

为了从根源上消灭 Rust 结构体 `struct S<'a>` 带来的全局 `<'a>` 病毒式传染，同时不损失哪怕 1% 的底层极致性能，CP 将类/结构体严格划分为两类：

1. **普通结构体 / 类（可自由逃逸、跨协程传递）**：
   - 字段只允许存储**拥有所有权的数据（Move 传入）**或**共享所有权句柄 `Shared<T>`**。
   - **`Shared<T>`（对标 Rust `Arc<T>` / Nest `Refc Binary`）**：基于原子引用计数与全局共享堆，支持零拷贝跨函数逃逸、跨 Actor 传递。为杜绝数据竞争与循环引用，**`Shared<T>` 默认只读不可变**。
2. **栈绑定借用视图（借鉴 C# `ref struct`，零开销局部借用，干掉最后 1% 缺陷）**：
   - 支持显式声明 `ref class` / `ref struct`，或者当一个类/结构体的字段中包含裸借用（如只读借用 `data: &Tensor` 或可变借用 `data: *Tensor`）时，编译器自动将其推导为 **栈绑定视图（Stack-Bound View）**。
   - **零开销优势**：无需原子引用计数，直接持有裸指针/切片，支持原地可变借用视图（`*T`），性能与手写 C/Rust 100% 一致，且**依然无需编写 `<'a>` 泛型**！
   - **逃逸清零铁律**：编译器对栈绑定视图执行严格的逃逸拦截：
     - ❌ 严禁将其 `return` 出所借用变量的存活作用域（或返回出当前函数，除非绑定到输入参数生命周期）；
     - ❌ 严禁将其赋值给外层更长生命周期的变量；
     - ❌ 严禁将其装入堆容器或通过 `spawn` / 消息发送给其他 Actor 协程。

## 5. `trait`、单继承 `class` 与自身实例（`_` / `this`）的所有权、借用与生命周期

CP 语言将普通函数的“默认/`&` 只读借用、`*` 可变借用、`move` 所有权转移”法则通过首参 `_`（等同于 `this`）完美统一到了面向对象与 Trait 契约体系中：

### （1）`_` 等同于 `this` 与成员互调省略

- **`_` 作为 `this` 的一等极简别名**：在 `class`、`struct`、`trait` 与 `impl` 成员函数中，`_` 与 `this` 完全等价（推荐写 `_`，代码更精简）。
- **成员互调与字段访问省略前缀**：在同一类或 Trait 内部，成员函数互相调用可直接省略 `_.` / `this.`（如直接写 `helper(x)`），不与局部变量冲突时也可直接访问字段。
- **模式匹配通配符消歧**：为避免与代表自身实例的 `_` 冲突，`match` / `switch` 模式匹配与解构中的通配符统一使用 `*`（如 `* => handleDefault()`）。

### （2）首参 `_` 的三维借用与所有权标记

| 实例权限模式                 | 方法声明语法（推荐 `_`，兼容 `this`）                                            | 编译期安全检查规则                                                                                                                                  |
| :--------------------------- | :------------------------------------------------------------------------------- | :-------------------------------------------------------------------------------------------------------------------------------------------------- |
| **1. 只读借用 (默认或 `&_`)** | `fn getVal(): i32`<br>_(缺省省略 `_`，等价于 `fn getVal(_): i32` 或 `fn getVal(&_): i32`)_ | • 内部禁止修改 `_.field`<br>• 内部禁止调用 `*_` 或 `move _` 方法<br>• 调用 `obj.getVal()` 产生只读借用 `&obj`                                       |
| **2. 可变借用 (`*_`)**       | **`fn setVal(*_, v: i32)`**<br>_(兼容 `*this` / `mut _`)_                        | • 允许修改自身字段及调用其他 `*_` 方法<br>• 调用 `obj.setVal(1)` 时，若 `obj` 存在未结束的只读/可变借用，立即报错拦截                               |
| **3. 所有权消费 (`move _`)** | `fn close(move _)`<br>`fn intoVal(move _): i32`                                  | • 调用 `obj.close()` 后，外层 `obj` **所有权立即失效**（后续访问报 `Use of moved value`）<br>• 自动取消外层 `defer obj.release()`，杜绝 Double Free |
| **4. 静态方法 (`static`)**   | `static fn create(): Tensor`                                                     | • 无实例接收者 `_`，通过 `ClassName.create()` 调用                                                                                                  |

### （3）实例方法的生命周期联动推导

- **第 1 级自动绑定 `_`**：类/Trait 方法返回借用（如 `fn getSlice(): &Tensor` 或 `fn getMutSlice(*_): *Tensor`）时，无需任何标注，返回值的生命周期自动绑定到当前实例 `_` (`this`)。在外层借用变量存活期间（NLL），原实例对象自动锁定，禁止调用冲突的 `*_` 或 `move _` 方法。
- **第 3 级显式参数绑定 `&_` / `*_`**：当方法含多个引用参数且需明确绑定来源时，支持 `: &_ Tensor` 或 `: *_ Tensor`（以及 `: &otherParam Tensor`）。

### （4）单继承 `class`（`extends`）与 Rust 级 `trait`

- **单继承 `class`**：支持 `class Child extends Parent`，子类自动继承父类的字段与方法，支持方法重写与 `super.method()` 调用；重写方法时，首参 `_` / `*_` / `move _` 的权限修饰必须与父类契约严格一致。
- **强悍 `trait` 系统**：
  - 同时支持 `struct` 与单继承 `class`；
  - 支持携带 `_` / `*_` / `move _` 权限契约的方法签名及**默认方法实现（Default Methods）**（默认方法内可直接省略 `_.` 互调同 Trait 的抽象方法）；
  - 支持 TS 风格内联 `class A implements MyTrait` 与 Rust 风格后向扩展 `impl MyTrait for A`；
  - 当函数参数类型为 `Trait`（如 `fn run(item: MyTrait)` vs `fn runMut(*item: MyTrait)`）时，编译器静态校验实参类型是否实现了该 Trait，并严格拦截在只读 Trait 引用上调用 `*_` / `move _` 方法的行为。

---

# 十一、函数三态返回、错误冒泡捕获、可选链、`Box` 胖指针与 `switch` 模式匹配

CP 语言吸收了 TypeScript 的极简表达、Zig 的错误集与 `catch`、以及 Rust 的编译期穷尽模式匹配与 `Box<dyn Trait>` 胖指针，构建了一套零 `Ok()` / `Some()` 包装冗余、同时 100% 编译期类型安全的返回与错误处理体系。

## 1. 函数返回类型定义（正常 `T`、空 `?T`、错误 `!E` 三态合一）

1. **省略 `: void`**：函数不声明 `: ReturnType` 时，默认返回 `void`。
2. **四类返回签名**：
   - `fn f(): T` —— 正常值返回（非空、无错）。
   - `fn f(): ?T` —— 可选空返回（成功返回 `T`，空值返回 `null`）。
   - `fn f(): T ! E` —— 错误联合体（成功直接写 `return val` 无需 `Ok(val)`，失败必须写 `return Err(e)`）。
   - `fn f(): ?T ! E` —— 三态联合体（成功直接 `return val`，空返回 `return null`，错误返回 `return Err(e)`）。
3. **`Err(...)` 编译器内置错误域构造器**：
   - `Err` 是专门用于构造 `T ! E` / `?T ! E` 错误域值的内置关键字构造器，在编译期零开销映射为底层错误判别式，并在 `switch` 模式匹配中精确区分正常分支与错误分支。
   - 对于匿名错误集或已知错误枚举，支持 `Err(.Variant)` 上下文简写或 `Err(EnumName.Variant)` 完整写法。

## 2. 错误集合（Error Set）、转换与内置 `Error` 对象

1. **具名错误枚举 (`enum`)**：
   ```cp
   enum FileErr { FileNotFound, PermDenied, ReadEOF }
   ```
2. **错误集并集 (`E1 | E2`)**：
   ```cp
   fn sync(): void ! FileErr | NetErr { ... }
   ```
3. **匿名内联错误集 (`!{ err1, err2 }`)**：
   ```cp
   fn parsePort(s: string): i32 ! { InvalidFormat, OutOfRange } {
       if (s == "") return Err(.InvalidFormat);
       return 8080;
   }
   let p: i32 ! { InvalidFormat, OutOfRange } = parsePort("8080"); // 类型标注可省略，支持自动推导
   ```
4. **错误集收敛转换（拒绝自动膨胀，消灭繁琐 `impl From`）**：
   - **并集子集自动提升**：若外层返回 `FileErr | NetErr`，内部调用返回 `FileErr` 的函数并用 `?` 冒泡时，编译器自动提升。
   - **`enum ... from` 声明式转换**：在上层错误枚举中通过 `from` 绑定下层错误集，编译器自动生成 `?` 冒泡转换代码：
     ```cp
     enum AppErr {
         Io(FileErr) from FileErr,
         AuthFailed
     }
     ```
   - **内联 `catch` 转换**：`readFile(p) catch e => return Err(AppErr.Custom);`
5. **业务层内置统一 `Error` 对象**：
   - 内置结构体 `Error { code: i32, msg: string }`（当函数声明返回 `T ! Error` 时，内部任意静态错误集通过 `?` 冒泡均可自动转换为 `Error` 对象，也可直接 `return Err(Error { code: 500, msg: "fail" })`）。

## 3. 错误冒泡 `?`、错误兜底 `? default`、`catch` 与 `try {}` 自动冒泡块

| 语法形式                               | 适用目标                  | 核心语义                                                                                                                                                             | 示例                                           |
| :------------------------------------- | :------------------------ | :------------------------------------------------------------------------------------------------------------------------------------------------------------------- | :--------------------------------------------- |
| **`expr?`** _(后无表达式)_             | `T ! E` / `?T ! E` / `?T` | **向上冒泡**：若为 `Err(e)`（或外层同为 `?U` 时的 `null`），立即中断当前函数并向上返回该错误/空值；成功则解包出 `T`。                                                | `let txt = readFile("a.txt")?;`                |
| **`expr ? default`** _(后跟表达式)_    | `T ! E` / `?T ! E`        | **错误/空值降级兜底**：当调用返回 `Err` 或 `null` 时，不向上冒泡，直接以 `default` 作为表达式结果（或 `expr ? return val`）。                                        | `let txt = readFile("a.txt") ? "fallback";`    |
| **`expr ?? default`**                  | 仅限 `?T`                 | **纯空值 (`null`) 合并**：仅当 `?T` 为 `null` 时返回 `default`（与错误处理 `? default` 严格正交分离）。                                                              | `let city = user?.city ?? "BJ";`               |
| **`expr catch err => ...`**            | `T ! E` / `?T ! E`        | **就地捕获错误**（无需前置 `try`）：当 `expr` 返回 `Err(err)` 时进入处理块或返回替代值。                                                                             | `let n = load() catch err => { return 0; };`   |
| **`try { ... } catch err => { ... }`** | 块内所有 `!E` 调用        | **省略尾部 `?` 区域标记**：编译期自动为 `try` 块内所有返回错误联合体的函数调用补齐隐式 `?`，一旦出错立即跳转至 `catch` 块（若省略 `catch` 则直接向当前函数外冒泡）。 | `try { step1(); step2(); } catch e => { ... }` |

## 4. 可选类型 `?T` 与可选链 `?.`

1. **`null` 强类型约束**：字面量 `null` 只能赋值给可选类型 `?T`（或 `?T ! E`），严禁赋值给非空类型 `T`。
2. **可选借用**：支持 `?&T`（可选只读借用）与 `?&mut T`（可选可变借用），底层享受零空间开销的空指针优化。
3. **可选链短路求值 (`expr?.member` / `expr?.method()`)**：当 `expr` 为 `null` 时立即短路返回 `null`，后续链式访问不执行；支持类型全自动推导（如 `let city = user?.addr?.city;` 自动推导为 `?string`）。

## 5. 内置 `Box<dyn Trait>` 胖指针与运行时 `downcast<T>()`

> **架构铁律**：底层库核心路径优先使用零开销静态错误集（`enum` / `!{}`）与静态多态；仅在动态加载插件、第三方用户智能体模块或需要携带异构动态错误实例时使用 `Box<dyn Trait>`。

1. **编译期自动生成 VTable**：当声明 `class T implements Trait` 或 `impl Trait for T` 时，`cpc` 编译器自动为该类型生成包含方法函数指针、64 位编译期唯一类型哈希 `__type_id` 以及析构函数指针 `__drop` 的静态虚函数表（彻底告别 Zig 手写 VTable 的繁琐与不安全）。
2. **`Box<dyn Trait>` 内存布局与构造**：通过 `Box.new(instance)`（或自动装箱）生成包含 `{ data: *anyopaque, vtable: *const Trait_VTable }` 的胖指针。
3. **`downcast<T>()` 安全向下转换**：内置方法 `box_val.downcast<ConcreteType>()` 基于 `__type_id` 进行 $O(1)$ 运行时校验，匹配成功返回可选借用 `?&ConcreteType`（生命周期自动绑定至 `box_val`），失败返回 `null`。
4. **所有权与借用安全**：
   - `Box<dyn Trait>` 独占堆实例所有权，离开作用域且未被 Move 时自动通过 `vtable.__drop` 释放底层实例与堆内存；
   - 严格受 CP 读写借用互斥与 NLL 生命周期检查约束；
   - **严禁栈上裸 `dyn Trait`**：任何未包裹在 `Box<...>` 中的裸 `dyn Trait` 声明在编译期直接报错拦截。

## 6. `switch` 表达式与编译期穷尽模式匹配

1. **`switch` 取代 `match`**：`switch` 是一等公民表达式，无分支穿透（无需 `break`），支持模式守卫 `if`（如 `x if x > 0 => ...`）。
2. **三态返回解构规则**：
   - **不带 `Err` 且非 `null` 的模式**（如 `content => ...`）：匹配成功域 `T` 并直接绑定变量；
   - **`null => ...`**：匹配空值域；
   - **`Err(pat) => ...`**：匹配错误域 `E`。
3. **通配符 `*` 与编译期穷尽检查**：
   - 使用 `* => ...` 作为默认通配分支（避免与代表 `this` 的 `_` 冲突）；
   - 对枚举、`?T`、`T ! E`、`?T ! E` 默认开启 **编译期穷尽性检查（Exhaustiveness Check）**：若遗漏任何枚举变体、`null` 分支或成功分支，且未提供 `*` 通配分支，编译期直接报错提示遗漏项。

---

# 第十二章：MetaAnnotation 极简元注解系统规范

> **定位**：元注解（MetaAnnotation）是附加在程序元素（Class、Struct、Method、Field、Param 等）上的**纯静态元数据标记**。它只存储结构化数据键值，**不执行任何可执行逻辑，不修改类型结构，不注入隐式方法**。
> **核心原则**：声明极简（仅 4 字母关键字 `meta`）、使用极简（少敲 50% 代码）、零运行时开销（默认编译期擦除，仅在 `@reflect` 时按需保留）。

## 1. 声明规范（`meta` 关键字）

1. **统一声明关键字**：采用 4 字母的 `meta` 关键字声明元注解，废除冗长沉重的 `annotation`：
   ```cp
   meta Route(path: string);
   meta Column(name: string, isPrimary: bool = false);
   ```
2. **无参免括号**：若元注解无需参数，声明端直接免写空括号 `()`：
   ```cp
   meta JsonIgnore;
   ```
3. **参数默认值与类型**：参数类型支持基础类型、静态匿名对象、字符串与数组，支持指定默认值。

## 2. 目标约束（`@target`）与全通默认法则

1. **默认全通法则（零代码心智负担）**：
   若定义元注解时未标注 `@target(...)`，**默认允许挂载在任意目标元素上**（Class、Struct、Method、Field、Param 通通合法），普通标记无需写任何繁琐声明。
2. **防呆目标约束**：
   当需要严格限制应用位置时，使用极简的小写 `@target(...)` 标注，支持单个或逗号罗列多个目标：
   ```cp
   @target(class)
   meta Controller(prefix: string);

   @target(method)
   meta Get(path: string = "");

   @target(field)
   meta Column(name: string);

   @target(class, struct)
   meta Entity;
   ```
3. **编译期严格拦截**：若将受限元注解错置于非允许目标（如将 `@Get` 标注于类或字段上），编译器立即报错拦截：
   `Compile Error: Meta '@Get' can only be applied to method, but was applied to class`。

## 3. 多值自动聚合机制（消灭 `@Repeatable`）与 `@once`

1. **天生聚合数组（彻底告别覆盖 BUG）**：
   在 CP 语言中，若同一个元素上多次标注同名元注解（如同一个方法绑定多个路径），编译器**天生且自动将其收集为元数据数组**，彻底废除 Java 风格的 `@Repeatable` 声明标记。
   ```cp
   @Get("/list")
   @Get("/all")
   fn list(): Array<string> { ... }
   ```
2. **唯一性反向约束（`@once`）**：
   若某类元注解在逻辑上绝对禁止重复书写，只需在其定义处标注 `@once`。违规多写时编译期精准报错：
   ```cp
   @target(class) @once
   meta Singleton;
   ```

## 4. 运行时反射控制（`@reflect` 零成本裁剪）

1. **默认零开销擦除（Zero-Cost Abstraction）**：
   默认情况下，所有类与结构体的元数据仅在编译期生效，最终 Native 机器码中**彻底擦除元数据表**，二进制体积零膨胀。
2. **显式开启反射（`@reflect`）**：
   仅对标记了 `@reflect` 的类或结构体，编译器才在只读数据段生成轻量静态 `TypeInfo` 描述表供运行时反射 API 读取。

## 5. 使用端“三免法则”

1. **无参免括号**：使用无参元注解时免写 `()`，直接写 `@JsonIgnore`、`@reflect`。
2. **单参免键名**：首参或单参直接按位置传参，写 `@Route("/users")` 无需繁琐的 `@Route(path: "/users")`（若参数较多，仍支持具名传参增强可读性）。
3. **免声明即用（双轨制）**：
   - 大型工程/公共库：先声明 `meta Name(...)` 享受强类型检查与 `@target` 校验；
   - 快速脚本/小型原型：**免预先声明直接使用**（如 `@AdhocTag("fast")`），编译器自动作为动态字面量收集。

## 6. 编译期常量安全校验

元注解传入的参数必须是**编译期已知常量**（字符串、整数、浮点数、布尔值、静态匿名对象字面量 `{ ... }`、数组字面量或目标标识符）。传入任何运行时变量或动态计算表达式时，编译器将在类型检查阶段直接报错拦截。

---

# 第十三章：Dual-State Reflection 双态反射模型规范

> **定位**：CP 语言提供兼具**编译期内省**与**运行时零成本裁剪**的双态反射系统。
> **核心原则**：
>
> 1. 入口极简：仅 4 字母 `reflect.type<T>()` 与 2 字母 `reflect.of(val)`。
> 2. 扁平模型：扁平化 `TypeInfo`、`FieldInfo`、`MethodInfo`，杜绝深层递归结构。
> 3. 元数据无缝复用原生动态字典：`target.meta` 直接复用 `Dict<string, any>`（`#{}`、`[]`、`.has()`、`.keys()`、`??`），消灭中间包装类型。
> 4. 零成本硬屏障：未标注 `@reflect` 的类若在代码中尝试反射，编译期直接报错拦截，机器码中反射数据零膨胀。

## 1. 反射入口点（Reflection Entry Points）

CP 语言内置两个极简反射入口（支持全局直接调用，亦可从 `"cap:reflect"` 显式导入）：

1. **类型反射入口**：
   ```cp
   let info = reflect.type<UserController>();
   ```
2. **实例反射入口**：
   ```cp
   let ctrl = new UserController();
   let info = reflect.of(ctrl);
   ```

## 2. 扁平反射模型定义

### 2.1 TypeInfo（类/结构体元信息）

- `name: string`：类型标识符名称（如 `"UserController"`）。
- `isRef: bool`：是否为引用结构体（`ref class`）。
- `fields: Array<FieldInfo>`：字段反射列表。
- `methods: Array<MethodInfo>`：普通成员方法列表（已过滤内部合成方法与析构）。
- `meta: Dict<string, any>`：挂载在类/结构体上的元数据字典。
- `getField(name: string): ?FieldInfo`：按名称安全查找字段。
- `getMethod(name: string): ?MethodInfo`：按名称安全查找方法。

### 2.2 FieldInfo（字段元信息）

- `name: string`：字段名称（如 `"secret"`）。
- `typeName: string`：字段类型名称字符串（如 `"string"`、`"i32"`）。
- `meta: Dict<string, any>`：挂载在字段上的元数据字典。

### 2.3 MethodInfo（方法元信息）

- `name: string`：方法名称（如 `"list"`）。
- `returnTypeName: string`：返回值类型名称（如 `"string"`、`"void"`）。
- `paramNames: Array<string>`：非 receiver 参数名列表。
- `isAsync: bool`：是否为异步协程方法。
- `isStatic: bool`：是否为静态方法。
- `meta: Dict<string, any>`：挂载在方法上的元数据字典。

## 3. 元数据原生字典与多值聚合读取

`target.meta` 原生支持所有动态字典语法：

- 读取键值：`info.meta["Controller"]`
- 判断存在：`info.meta.has("Singleton")`
- 缺省保底：`let route = info.meta["Route"] ?? "/default"`
- 遍历键名：`let keys = info.meta.keys()`
- 多个同名注解自动聚合数组：
  ```cp
  @Get("/list")
  @Get("/all")
  fn list(): string { ... }

  let routes = m.meta["Get"]; // ["/list", "/all"]
  let r0 = m.meta["Get"][0];   // "/list"
  ```

## 4. 零成本编译期安全屏障（Zero-Cost Reflection Guard）

若在代码中调用 `reflect.type<T>()` 或 `reflect.of(val)`，而目标类未标记 `@reflect`：
编译器将在语义分析阶段立即终止编译并输出错误诊断：

```
Compile Error: Type '{TypeName}' cannot be reflected at runtime. Add '@reflect' to the class definition to enable runtime reflection.
```

未启用 `@reflect` 的类型在生成 Native 机器码时完全不生成任何反射表，保证体积与运行性能极致纯粹。

---

# 第十四章：Decorators 极简装饰器与 AOP 静态合成规范

## 1. 核心定位与极简设计原则

CP 语言将**元注解（`meta`）**与**装饰器（`dec`）**从语法与执行模型上彻底解耦：

- **`meta`**：纯静态数据标记，编译期零开销，运行时进静态反射表（不执行任何逻辑代码）；
- **`dec`**：可执行切面逻辑（AOP），在目标方法调用前后进行包装、拦截、重试或增强。

### 四大极简铁律：

1. **极短关键字 `dec`**：统一采用 3 字母的 `dec` 声明装饰器，干脆利落；
2. **消灭闭包样板的洋葱切面模型**：无参切面入参直写 `next`，自动捕获并透明透传原函数参数与返回值；
3. **编译期静态合成 (Static Synthesis AOP)**：彻底消灭动态代理（Dynamic Proxy）、动态猴子补丁与运行时闭包堆分配，编译器就地展开静态包装函数（`_raw_xxx` 与 `_wrap_xxx`），直达裸机速度；
4. **调用端三免法则**：无参免括号（`@timer`）、单参免键名（`@retry(3)`）、多装饰器洋葱式嵌套堆叠。

---

## 2. 装饰器定义与切面语法

### 2.1 无参直接切面（Around Aspect）

```cp
dec timer(next) {
    let t0 = clock.now();
    let res = next(); // 自动无损透传原参数，捕获返回值
    print(`[Timer] 耗时: ${clock.now() - t0}ms`);
    return res;
}

dec doubleResult(next) {
    let res = next();
    return res * 2;
}
```

### 2.2 带参装饰器（双轨风格支持）

开发者可以自由选择以下两种风格之一：

- **风格 A：直接参数风格（Direct Style）**

  ```cp
  dec tag(next, prefix: string) {
      let res = next();
      return "[" + prefix + "] " + res;
  }
  ```

- **风格 B：高阶工厂风格（Higher-Order Closure Factory）**
  ```cp
  dec retry(times: i32 = 3) {
      return fn(next) {
          for i in 0..times {
              let res = next();
              if (res != null) { return res; }
          }
          return null;
      };
  }
  ```

---

## 3. 洋葱模型（Onion Execution Model）

当一个方法堆叠了多个装饰器时，其执行顺序严格遵循工业级洋葱模型（从外层进入，内层返回）：

```cp
@outerWrap
@innerWrap
fn compute(x: i32): i32 {
    return x * 2;
}
```

- **调用链展开**：
  `compute(x)` 入口 -> 执行 `outerWrap` 前置 -> 调用 `next()` -> 执行 `innerWrap` 前置 -> 调用 `next()` -> 执行 `_raw_compute` -> 返回并执行 `innerWrap` 后置 -> 返回并执行 `outerWrap` 后置。

---

## 4. 与元注解（`meta`）及反射（`reflect`）的共存

装饰器与元数据天然共存：

```cp
@reflect
class UserService {
    @timer
    @tag("AUTH")
    @Get("/users")
    fn listUsers(): Array<User> {
        return ...;
    }
}
```

- `@timer` 与 `@tag` 由编译器识别为切面逻辑并合成为包装函数；
- `@Get("/users")` 保持为元数据进入反射字典；
- `@reflect` 激活该类的静态反射单例表，反射内省仅暴露公共接口 `listUsers`，内部合成的 `_raw_listUsers` 自动安全隐藏。

---

# 第十五章：类型感知宏与循环安全系统 (Type-Directed Macro & Anti-Cycle System)

CP 元编程第四阶段引入了**类型感知宏（Type-Directed Macro）**与**防死锁循环安全系统**。宏在类型符号扫描完成后的 Phase 2 阶段执行，能够内省类型的符号元信息（Tier 1 TypeDeclInfo）并派生代码（方法与 impl），同时在编译管线中设立铁律防范循环依赖死锁。

## 1. 宏定义语法 (`macro`)

```cp
// 自定义宏定义
macro Tag(target) {
    // 宏展开逻辑
}

macro Table(target, name: string = "") {
    // ORM 表名映射宏
}
```

- **目标约束**：类型感知宏 `@Macro` 只能挂载到 `class` 或 `struct`（非法附着到方法、函数或字段将在语义检查阶段被拦截）。
- **零运行时开销**：`macro` 在编译期完成展开，生成的 Zig 代码中不保留任何宏定义的骨架。

---

## 2. 内建派生宏

### 2.1 `@Json` 强类型序列化派生

读取类的字段声明，自动合成 `toJson(): string` 方法：

```cp
@Json
class User {
    id: i32;
    name: string;
}

let u = new User(1, "Alice");
let json = u.toJson(); // 输出: {"id":1,"name":"Alice"}
```

### 2.2 `@Table` ORM 映射派生

自动合成 `getTableName(): string`：

```cp
@Table("t_users")
class User { ... }
// u.getTableName() -> "t_users"

@Table
class Order { ... }
// o.getTableName() -> "order" (默认取类名全小写)
```

### 2.3 `@ToString` 打印派生

自动合成 `toString(): string`，输出 `ClassName(field1=val, field2=val)`。

---

## 3. 防死锁与循环安全四大铁律

为规避 Rust 过程宏的妥协与类型内省引发的死锁问题，CP 确立了四项不可违背的底层铁律：

1. **类型创建红线（No Top-Level Types）**：
   Phase 2 宏严禁创建新的顶层类型（`class` / `struct` / `trait` / `enum`），仅允许合成方法或 `impl` 块。由此在数学逻辑上彻底阻断了宏自繁殖导致的类型图膨胀。
2. **宏产物卫生铁律（Macro Hygiene）**：
   宏代码展开生成的 AST 节点上，严禁携带任何宏注解（仅手写源码的 AST 节点允许挂载宏），确保自繁殖爆炸概率为绝对的零。
3. **分级内省模型（Two-Tier TypeInfo）**：
   - **Tier 1 (TypeDeclInfo)**：符号声明级内省（类名、字段名、声明类型名），在第一遍符号扫描后完全确立，天然永不死锁。即便 `User` 引用 `Order`、`Order` 引用 `User`，互引类型的宏内省依然可安全完成；
   - **Tier 2 (ConcreteTypeInfo)**：内存物理布局级（字节对齐与偏移），在宏展开及所有拓扑解除后确定。
4. **循环展开调用栈与深度熔断**：
   编译器维护 `macro_call_stack`，同名宏对同类型的递归探测触发 `MacroCycleDetected`，最大展开深度上限设为 128 层，防范死循环与 OOM。

---

# 第十六章：管道操作符与主题占位符 (Pipeline Operator `|>` & Topic Placeholder `.`)

在智能体（AI Agent）系统中，计算本质上是**多阶段、流式的认知与数据处理流水线（Pipeline Dataflow）**：从 Prompt 清洗、向量检索（RAG）、多模态上下文注入，到 LLM 推理、工具调用（Function Calling）、后置校验与输出格式化。

传统嵌套函数调用（`f(g(h(x)))`）存在严重的**括号金字塔（Pyramid of Doom）与逆序阅读心智负担**。CP 语言借鉴 Elixir 与 Hack 的管道设计哲学，引入了符合人体工程学的 `|>` 操作符与轻量主题占位符 `.`。

```
// 传统嵌套：逆序阅读，括号深陷
let reply = simulate_agent(build_context("PROMPT", retrieve_rag("kb", sanitize_prompt(raw), 5)));

// CP 管道流：自顶向下，线性流转
let reply = raw
    |> sanitize_prompt
    |> retrieve_rag("kb", ., 5)
    |> build_context("PROMPT", .)
    |> simulate_agent;
```

---

## 1. 词法、语法与操作符优先级

- **操作符标记**：`|>`
- **结合性**：左结合（Left-Associative）
- **操作符优先级**：优先级定为 **3**（介于比较运算 `2` 与加减运算 `4` 之间）：
  - 赋值运算符 (`=`): `1`
  - 比较运算符 (`==`, `<`, `>`): `2`
  - **管道运算符 (`|>`)**: **`3`**
  - 加减运算 (`+`, `-`): `4`
  - 乘除运算 (`*`, `/`): `5`
  - **设计保证**：`x + 1 |> double` 解析为 `(x + 1) |> double`；`data |> len > 0` 解析为 `(data |> len) > 0`。

---

## 2. 右侧表达式的三级降糖策略 (3-Tier Desugaring)

当数据流经 `left |> right` 时，编译器前端根据 `right` 的语法结构智能降糖：

### 规则一：裸函数标识符 (Bare Identifier)

当右侧为单纯函数名 `f` 时，上游结果作为唯一入参传入：

```cp
a |> f
// 降糖为: f(a)
```

### 规则二：未显式携带占位符的调用 (Default First Argument)

当右侧为函数调用且实参列表中不含 `.` 时，上游结果默认作为**第一个参数**注入（契合 Unix 管道哲学与标准库设计）：

```cp
a |> f(b, c)
// 降糖为: f(a, b, c)
```

### 规则三：显式主题占位符 (`.`)

当右侧显式使用独立的 `.` 时，上游结果被注入到 `.` 所在的确切位置：

```cp
// 注入到任意参数槽位
a |> f(b, .)          // 降糖为: f(b, a)
a |> f(., b)          // 降糖为: f(a, b)

// 成员方法调用
tw |> .prefix("CP")   // 降糖为: tw.prefix("CP")

// 字段解构与算术表达式
point |> .x * .y      // 降糖为: point.x * point.y
```

---

## 3. 严格单次求值与零运行时开销 (No Double Evaluation)

若 `.` 在右侧表达式中被多次引用（例如 `seed |> combine(., .)`），或者上游为一个昂贵的有副作用函数调用（例如 `fetchApi() |> process(., .)`），CP 编译器强制实行 **SSA 单赋值局部块转换**：

```zig
// 编译生成的 Zig 代码
(blk_p_0: {
    const __pipe_0 = fetchApi(); // 严格只计算一次！
    break :blk_p_0 process(__pipe_0, __pipe_0);
})
```

- **零内存分配**：所有管道临时量均在栈上以常量传递，不产生任何堆分配；
- **副作用安全**：`fetchApi()` 保证只被调用一次。

---

## 4. 严谨的作用域语法边界 (Hygiene Boundary)

- **情境激活**：裸 `.` 仅在 `|>` 操作符的**右侧表达式有效求值范围内**被编译器识别为主题占位符；
- **普通作用域隔离**：一旦离开管道右侧上下文，裸 `.` 恢复其作为成员访问符与枚举简写的本义。任何在管道外将 `.` 作为表达式使用的行为（如 `let x = .;` 或 `f(., 5)`）会在语义分析与语法解析阶段被立即拦截：
  ```text
  Error: Pipeline placeholder '.' can only be used on the right-hand side of a pipeline ('|>') operator
  ```

---

# 第十七章 迭代系统 (Iteration System)

CP 语言深度融合了 **ES6 / TypeScript 的极简优雅语法** 与 **Rust 的零悬挂指针所有权借用安全体系**，提供一等公民级别的迭代系统。

---

## 1. 核心三维迭代语义 (Iteration Modes)

在 CP 中，针对被遍历的集合，支持三种清晰、精炼、零样板代码的借用与所有权修饰符：

| 语法模式             | 借用与权限语义                        | 底层机制与安全保证                                                                                                                                                                                                                      | 常见应用场景                                     |
| :------------------- | :------------------------------------ | :-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | :----------------------------------------------- |
| `for x of list`<br>_(兼容 `for &x of list`)_ | **只读借用 (Default Borrow)**         | 集合在循环体内被自动施加只读借用锁（`loop_borrowed_vars`）。循环体内严禁对集合进行任何结构性修改（如 `push`、`clear`、`delete`、重赋值）。杜绝迭代器失效与悬挂指针。循环结束后集合完全完好。                                            | 数据读取、汇总、只读遍历、多轮遍历               |
| **`for *x of list`** | **原地可变借用 (In-Place MutBorrow)** | `x` 获得对集合各元素的就地可变引用（映射为 Zig `for (&list) \|*x\|`）。循环内对 `x = new_val` 或 `x.field = val` 的写操作直接更新底层物理内存。同时禁止改变集合长度，保证绝对内存安全。                                                 | 原地数据加工、状态批量翻转、向量矩阵归一化       |
| `for move x of list` | **所有权转移 (Ownership Transfer)**   | 消费并转移集合内部元素的所有权（类似 Rust `into_iter` / Drain）。在遇到 `for move` 时，原集合立即被标记为已移交（`moved_vars`），后续任何读取均报 `Use of moved value` 编译错误。循环体每次迭代完全拥有 `x`，并在每轮结束自动析构释放。 | 资源管道转移、消息队列抽空处理、批量生命周期接管 |

---

## 2. 语法包容性与 TS 兼容 (Parentheses & Variable Binding)

为了让前端开发者拥有最舒适的直觉，CP 对小括号 `()` 和声明关键字 `let` 提供了完全对称的语法宽容度：

```cp
// 1. 最简极客风格 (无括号，无需 let)
for x of list { ... }       // 默认只读借用（亦可写 for &x of list）
for *x of list { ... }      // 原地可变借用
for move x of list { ... }  // 所有权转移

// 2. TypeScript / ES6 兼容风格 (可选带小括号)
for (x of list) { ... }
for (*x of list) { ... }
for (move x of list) { ... }

// 3. 变量声明兼容器 (可选带 let / const)
for let x of list { ... }
for (let x of list) { ... }
```

---

## 3. 区间范围循环 (Range Iteration)

CP 原生支持类似 Rust / Zig 的极速区间循环，天然拥有零开销与寄存器级展开优势：

```cp
// 1. 半开区间 0..N (包含 0，不包含 5，共迭代 5 次: 0, 1, 2, 3, 4)
let mut_sum = 0;
for i of 0..5 {
    mut_sum = mut_sum + i;
}

// 2. 闭区间 1..=N (包含 1 和 5，共迭代 5 次: 1, 2, 3, 4, 5)
for i of 1..=5 {
    @print("Step: {d}", i);
}

// 3. 动态变量作为区间边界
let start = 10;
let end = 20;
for i of start..end {
    ...
}
```

---

## 4. 模式解构迭代 (Pattern Destructuring)

与 CP 强大的元组与对象解构完全统一，`for` 循环支持就地模式解构：

### （1）元组与键值对解构 (`[k, v]`)

```cp
// 遍历 Map 或二维元组数组
let map = new Map();
map.set("title", "CP");
map.set("ver", 2);

for [k, v] of map.entries() {
    @print("Key={s}, Val={any}", k, v);
}
```

### （2）对象字段解构 (`{ field1, field2 }`)

```cp
class Player {
    id: i32;
    score: i32;
    fn init(id: i32, score: i32) {
        this.id = id;
        this.score = score;
    }
}

let players = [new Player(1, 100), new Player(2, 200)];

for { id, score } of players {
    @print("Player {d}: score = {d}", id, score);
}
```

---

## 5. 编译期安全铁律与错误拦截

1. **借用期结构修改拦截 (MutateWhileBorrowedInFor)**：
   ```cp
   let list: i32[] = [1, 2, 3];
   for x of list {
       list.push(4);
       // ❌ Compile Error: Cannot mutate 'list' while it is borrowed in 'for ... of' loop
   }
   ```
2. **Move 后使用拦截 (UseOfMovedVar)**：
   ```cp
   let list: i32[] = [1, 2, 3];
   for move x of list {
       process(x);
   }
   let len = list.len;
   // ❌ Compile Error: Use of moved value 'list'
   ```

---

# 第十八章：Unsafe 与裸指针系统规范 (Unsafe & Raw Pointer System)

## 1. 架构定位与设计哲学

CP 语言遵循 **“默认安全（Safe by Default），按需下沉（Opt-in Unsafe）”** 的设计哲学：

- 在 99% 的应用业务、智能体调度、数据流管道开发中，CP 依靠所有权模型（Ownership）、非词法生命周期（NLL）、只读借用 `&` 与可变借用 `*` 提供零垃圾回收（No GC）的物理级内存安全与无悬挂指针保证。
- 在 1% 的极限场景（如对接底层 C/Zig/NIF、高性能内存映射 I/O、GPU 显存物理地址直接操控、SIMD 向量寄存器交互）下，开发者可通过 `unsafe` 沙盒放宽编译期保护，直接操纵物理裸指针（Raw Pointer）。

---

## 2. 裸指针类型系统（星号层级 + 尾置感叹号属性）

为避免与模式匹配通配符 `*` 以及安全可变借用 `*T` 混淆，CP 采用直观优雅的 **“前置 `*` 标识指针深度，尾置 `!` 标识非托管裸属性”** 语法：

| 裸指针层级     | CP 语法声明 | Zig 目标映射 | 语义说明                            |
| :------------- | :---------- | :----------- | :---------------------------------- |
| **一级裸指针** | `*T!`       | `*T`         | 指向 `T` 内存地址的非托管单层裸指针 |
| **二级裸指针** | `**T!`      | `**T`        | 指向裸指针的非托管双层裸指针        |
| **多级裸指针** | `***T!`     | `***T`       | 多层深度寻址裸指针                  |

### 与三态错误返回 `!` 彻底解耦，绝无语法歧义

在 CP 中，函数可能同时返回裸指针与错误联合（Error Union）：

```cp
// 语法完全正交清晰：*User! 是返回类型，! AllocError 是错误集联合
unsafe fn getFalliblePtr(fail: bool): *User! ! AllocError {
    if (fail) {
        return Err(AllocError.OutOfMemory);
    }
    let u = User{age: 88};
    return &u as *User!;
}
```

编译器在词法与解析阶段具备双向 Lookahead：在类型后遇到 `!` 时，若紧跟标识符/错误类型（如 `! AllocError`），则解析为错误联合；若在基类型末尾收口（如 `*User!`），则解析为裸指针属性标记。

---

## 3. 裸指针获取 (`*b!` / `**b!`) 与显式类型转换 (`expr as TargetType`)

在 `unsafe` 作用域内，既可以使用极简的 **`*b!` / `**b!` 后缀感叹号表达式**直接获取裸指针，也可以使用 `as *T!` 进行显式类型转换：

```cp
let mut user = User{age: 20};

unsafe {
    // 1. 直接使用 *b! 获取一级裸指针（或写 &user as *User!）
    let p: *User! = *user!;

    // 2. 直接使用 **b! 获取二级裸指针（或写 &p_copy as **User!）
    let mut p_copy: *User! = p;
    let pp: **User! = **p_copy!;

    // 3. 标量裸指针获取
    let mut num: i32 = 100;
    let pnum: *i32! = *num!;
}
```

---

## 4. 解引用与字段访问规范

CP 裸指针兼顾了 TypeScript 的极简体验与 Zig 的底层精准控制：

### （1）结构体成员自动解引用（Auto-Deref Member Access）

对于结构体裸指针，访问或修改字段无需繁琐的 `(*p).field`，编译器支持自动成员解引用：

```cp
unsafe fn setAge(p: *User!, val: i32) {
    p.age = val; // ✅ 自动解引用并写入 pointee 内存
}
```

### （2）标量与深度解引用语法 (`.*`)

对于标量或多级裸指针，使用 `.*` 进行显式解引用：

```cp
// 标量解引用写入
unsafe fn setScalar(p: *i32!, val: i32) {
    p.* = val; // ✅ 将 val 写入 p 指向的内存
}

// 多级指针解引用链
unsafe fn modifyViaDoublePtr(pp: **User!, new_val: i32) {
    pp.*.age = new_val; // ✅ 解引用一级后修改目标结构体字段
}
```

---

## 5. 函数与代码块安全拦截机制

### （1）`unsafe fn` 与 `unsafe { ... }`

- **`unsafe fn name(...)`**：声明不安全函数。函数内部直接处于 `unsafe` 上下文中；外部调用该函数必须置于 `unsafe` 块内。
- **`unsafe { ... }`**：创建不安全作用域（支持语句块与表达式块），放宽指针解引用、unsafe 函数调用与类型强转限制。

### （2）编译期安全拦截铁律（Sema Compile Error）

当任何不安全操作逃逸出 `unsafe` 作用域时，Sema 将直接终止编译并给出高可读性错误报告：

1. **未授权调用 Unsafe 函数**：
   ```cp
   setAge(p, 30);
   // ❌ Semantic Error: Compile Error: Call to unsafe function 'setAge' requires an unsafe block or unsafe function
   ```
2. **未授权强转裸指针**：
   ```cp
   let p: *i32! = &num as *i32!;
   // ❌ Semantic Error: Compile Error: Cast to raw pointer requires an unsafe block or unsafe function
   ```
3. **未授权解引用裸指针**：
   ```cp
   let v = p.*;
   // ❌ Semantic Error: Compile Error: Dereferencing raw pointer requires an unsafe block or unsafe function
   ```

---

## 6. 完整生产级运行示例

```cp
import { print } from "cap:io";

struct User {
    age: i32
}

error AllocError {
    OutOfMemory
}

// 1. 接受裸指针的 unsafe 函数
unsafe fn setAge(p: *User!, val: i32) {
    p.age = val;
}

// 2. 返回错误联合与裸指针
unsafe fn getFalliblePtr(fail: bool): *User! ! AllocError {
    if (fail) {
        return Err(AllocError.OutOfMemory);
    }
    let u = User{age: 88};
    return &u as *User!;
}

// 3. 二级裸指针操作
unsafe fn modifyViaDoublePtr(pp: **User!, new_val: i32) {
    pp.*.age = new_val;
}

// 4. 标量指针操作
unsafe fn setScalar(p: *i32!, val: i32) {
    p.* = val;
}

fn main() {
    let mut user = User{age: 20};

    unsafe {
        // 一级指针修改
        let p: *User! = &user as *User!;
        setAge(p, 30);
        @print("User age after setAge: {d}", user.age); // 输出: 30

        // 二级指针修改
        let mut p_copy: *User! = p;
        let pp: **User! = &p_copy as **User!;
        modifyViaDoublePtr(pp, 45);
        @print("User age after modifyViaDoublePtr: {d}", user.age); // 输出: 45

        // 标量指针修改
        let mut num: i32 = 100;
        let pnum: *i32! = &num as *i32!;
        setScalar(pnum, 200);
        @print("Scalar num after setScalar: {d}", num); // 输出: 200

        // 三态返回与模式匹配
        let res = getFalliblePtr(false);
        let out = switch res {
            raw_p => "ok",
            Err(AllocError.OutOfMemory) => "oom",
            Err(*) => "err"
        };
        @print("Fallible ptr: {s}", out); // 输出: ok
    }
}
```

---

# 第十九章：静态匿名对象与动态字典系统 (Static Anonymous Objects & Dynamic Dict `#{ ... }`)

## 1. 架构定位：双轨制对象模型 (Dual-Track Object Model)

在传统 JavaScript/TypeScript 中，对象既承担着“编译期紧凑结构体”的职责，又承担着“运行时动态哈希表”的职责。这种双重身份迫使 V8/JIT 引擎在底层维护极其复杂的隐藏类（Hidden Classes / Shapes）与动态内联缓存，无法做到确定性的内存布局与极致的零垃圾回收。

CP 语言在架构层面彻底进行了**静态与动态的权责解耦**，构建了清晰的双轨制对象模型：

| 特性维度               | 静态匿名对象 (`{ ... }`)                 | 动态字典 (`#{ ... }` / `Dict`)             |
| :--------------------- | :--------------------------------------- | :----------------------------------------- |
| **语法标识**           | `{ key: value, ... }`                    | `#{ key: value, ... }`                     |
| **底层映射**           | 编译期 Zig 匿名结构体 (`struct { ... }`) | 堆分配的高性能动态哈希字典                 |
| **内存位置**           | 栈内存或寄存器直接内联（零堆分配）       | Actor 本地 Arena 内存池                    |
| **字段特性**           | 字段固定、编译期强类型推导、零寻址开销   | 自由增删键值、运行时动态属性扩展           |
| **展开运算符 (`...`)** | 编译期静态合并字段与静态覆盖             | 运行时动态合并字典键值                     |
| **最佳适用场景**       | 接口参数包、坐标点、配置载荷、元组替代物 | 动态 JSON 载荷、环境变量映射、动态配置中心 |

---

## 2. 静态匿名对象规范 (`{ ... }`)

### （1）声明、初始化与字段访问

静态匿名对象的字段类型由编译器在编译期自动推导为不可变的静态结构体类型，字段访问具有零运行时开销：

```cp
// 静态匿名对象声明
let user = { id: 101, name: "Alice", active: true };

// 属性读取（零哈希查找，编译期直接偏移寻址）
let uid: i32 = user.id;
let uname: string = user.name;
let uactive: bool = user.active;
```

### （2）对象解构赋值 (Destructuring Assignment)

全面对标现代 TypeScript 体验，支持就地解构提取变量：

```cp
let { id, name } = user;
// id == 101, name == "Alice"
```

### （3）对象展开运算符 (`...`) 与静态覆盖

静态匿名对象支持使用 `...` 展开运算符将一个或多个对象的字段就地展开并生成新的静态对象。后声明的字段具有覆盖前值的语义：

```cp
// 1. 展开原对象，覆盖 active 属性并追加 role 属性
let updated = { ...user, active: false, role: "admin" };
// updated.id == 101
// updated.name == "Alice"
// updated.active == false (已覆盖)
// updated.role == "admin"

// 2. 多对象级联展开与跨对象字段覆盖
let extra = { score: 99, level: 5 };
let combined = { ...user, ...extra, id: 202 };
// combined.id == 202 (已覆盖 user.id)
// combined.name == "Alice"
// combined.score == 99
// combined.level == 5
```

> [!NOTE]
> 静态对象的展开在编译期完成类型计算与结构体合成，完全不会引发运行时的堆内存复制。

---

## 3. 动态字典系统规范 (`#{ ... }`)

### （1）字典初始化语法糖 (`#{ ... }`)

当需要动态增删属性、处理不确定结构的键值对时，使用前置 `#` 的动态字典语法：

```cp
// 动态字典声明
let config = #{ host: "localhost", port: 8080 };

// 动态索引读取（返回类型为动态 Message）
let host = config["host"]; // "localhost"
let port = config["port"]; // 8080
```

### （2）动态属性增删改查

动态字典允许在运行时任意写入未在初始化时声明的全新属性：

```cp
// 运行时动态添加全新属性
config["ssl"] = true;
config["max_connections"] = 1000;

// 修改已有属性
config["port"] = 8443;

// 动态删除属性
config.delete("max_connections");
```

### （3）动态字典展开与覆盖 (`...`)

动态字典同样支持 `...` 展开运算符，在运行时将基础字典的键值克隆合并到新字典中：

```cp
let prod_config = #{ ...config, port: 443, env: "production" };

// prod_config["host"] == "localhost"
// prod_config["port"] == 443 (动态覆盖)
// prod_config["ssl"] == true
// prod_config["env"] == "production"
```

### （4）空值合并运算符 (`??`) 与动态字典

读取不存在的字典键时不会发生崩溃，结合 `??` 操作符可优雅实现参数默认兜底：

```cp
// 如果 "timeout" 键不存在，优雅返回缺省默认值 5000
let timeout = config["timeout"] ?? 5000;
```

---

# 第二十章：高级集合数据结构规范 (Map & Set Collections)

## 1. ES6 标准 `Map` 集合规范

CP 原生内建了对标 ECMAScript 标准的高性能 `Map` 键值集合类，基于底层高性能哈希映射实现：

### （1）基本 API 清单

```cp
let map = new Map();

// 1. 插入与更新
map.set("title", "CP Language");
map.set("version", 2);

// 2. 获取大小
let size: usize = map.size; // 2

// 3. 读取值（返回可选类型 ?Message）
let title = map.get("title"); // 可通过 title.to_string() 转换

// 4. 键存在性判断
let hasVer: bool = map.has("version"); // true
let hasMissing: bool = map.has("nonexistent"); // false

// 5. 删除指定键
map.delete("version");

// 6. 清空集合
map.clear();
```

### （2）下标索引语法糖 (`map[key]`)

`Map` 原生支持方括号语法进行读取与更新，心智模型与现代脚本语言高度对齐：

```cp
let map = new Map();
map["name"] = "AgentCore";   // 等价于 map.set("name", "AgentCore")
let n = map["name"];          // 等价于 map.get("name")
```

### （3）迭代集合视图与解构循环

`Map` 提供获取集合切片的方法，可无缝对接 `for ... of` 迭代系统：

```cp
let ks = map.keys();     // 获取全部键数组
let vs = map.values();   // 获取全部值数组
let es = map.entries();  // 获取全部 [key, value] 二元组数组

// 模式解构迭代
for [k, v] of map.entries() {
    @print("Key: {s}, Value: {any}", k, v);
}
```

---

## 2. ES6 标准 `Set` 集合规范

CP 原生内建了去重集合 `Set`：

### （1）基本 API 清单

```cp
let set = new Set();

// 1. 添加元素（重复值自动被去重过滤）
set.add(100);
set.add(200);
set.add(100); // 重复添加，集合大小依然为 2

// 2. 获取集合大小
let count: usize = set.size; // 2

// 3. 存在性判断
let exists: bool = set.has(100);  // true
let notFound: bool = set.has(300); // false

// 4. 获取元素列表切片
let vals = set.values(); // [100, 200]

// 5. 删除元素
set.delete(100);

// 6. 清空集合
set.clear();
```

---

## 3. 底层内存架构与无 GC 保证

- **零锁动态哈希**：`Map` 与 `Set` 底层基于 Zig 的 `std.HashMap` 实现，具有高负载因子与抗哈希碰撞性能；
- **Actor Arena 池化释放**：`Map` 与 `Set` 的哈希槽内存均由当前 Actor/协程的 `ctx.arena` 提供支持，生命周期与当前会话绑定，在 Actor 完成调度时整块瞬时回收，无需传统垃圾回收器的停顿（No Stop-The-World）。

---

# 第二十一章：文本与数学核心内建对象 (Strings, Template Literals, RegExp & Math)

## 1. 字符串完整方法套件 (String Methods)

CP 语言中的 `string` 底层为不可变的高性能字节切片（Zig `[]const u8`），但在语法层拥有全面的 ES5/ES6 扩展方法：

| 方法分类       | 签名                                                                                            | 说明                                    |
| :------------- | :---------------------------------------------------------------------------------------------- | :-------------------------------------- |
| **属性**       | `s.length: usize`                                                                               | 字符串字节/字符长度                     |
| **查找定位**   | `s.indexOf(sub: string): i32`<br>`s.lastIndexOf(sub: string): i32`                              | 首次/末次出现索引，未找到返回 `-1`      |
| **前缀后缀**   | `s.startsWith(sub: string): bool`<br>`s.endsWith(sub: string): bool`                            | 判断是否以指定前缀/后缀起始或收尾       |
| **包含判断**   | `s.includes(sub: string): bool`                                                                 | 判断是否包含子串                        |
| **截取分片**   | `s.slice(start: usize, end?: usize): string`<br>`s.substring(start: usize, end: usize): string` | 提取指定区间切片                        |
| **大小写转换** | `s.toLowerCase(): string`<br>`s.toUpperCase(): string`                                          | 转换字母大小写（由 Arena 分配新切片）   |
| **修剪与空白** | `s.trim(): string`                                                                              | 移除首尾空白字符                        |
| **替换与分割** | `s.replace(old: string, new_str: string): string`<br>`s.split(sep: string): string[]`           | 子串替换；按分隔符拆分为字符串数组      |
| **字符与编码** | `s.charAt(index: usize): string`<br>`s.charCodeAt(index: usize): i32`                           | 提取指定字符或其 ASCII/Unicode 整数码点 |

---

## 2. 模板字符串 (Template Literals)

支持使用反引号 `` `...` `` 声明模板字符串，内部通过 `${expr}` 插值内嵌任意变量与表达式：

```cp
let name: string = "World";
let version: i32 = 2;

// 1. 变量直接插值
let t1: string = `Hello, ${name}!`;

// 2. 多类型混合插值
let t2: string = `CP v${version} engine initialized for ${name}`;

// 3. 表达式计算插值
let a = 10;
let b = 20;
let t3: string = `Result is ${a + b * 2}`; // "Result is 50"

// 4. 转义语法（\$ 输出字面量 $，\` 输出字面量反引号）
let t4: string = `Price is \$100 for \`CP Lang\``; // "Price is $100 for `CP Lang`"

// 5. 结构化 JSON 与字面量花括号（零转义负担，原生支持）
let uid = 88;
let t5: string = `{"status": 200, "user_id": ${uid}, "name": "${name}"}`;

// 6. 字符串方法链式调用
let t6: string = `agent ${name}`.toUpperCase(); // "AGENT WORLD"

// 7. 编译期常量折叠（若全为常量，编译器前端自动折叠为纯字面量切片，零运行时开销）
let t7: string = `Status: ${"OK"}, Code: ${200}`;
```

> [!TIP]
> **AI Agent 与 JSON 友好设计**：在 `${...}` 外部出现的普通花括号 `{}` 会被原样直出为字符文本，绝不会触发语法报错或要求手动双写 `{{` / `}}` 转义。插值表达式内部若含有包含花括号的字符串（如 `${"{" + "}"}`），解析器亦能智能识别词法边界，绝无提前截断风险。

---

## 3. 正则表达式引擎 (`RegExp`)

CP 原生封装了 POSIX 高性能正则表达式引擎，无需笨重的解释器中间层：

```cp
// 1. 构造正则表达式对象 (pattern, flags)
let re = new RegExp("^hello [a-z]+$", "");

// 2. 模式测试 test() -> bool
let ok: bool = re.test("hello world"); // true
let fail: bool = re.test("hello 123");  // false

// 3. 字符串捕获 match() -> string
let s = "hello world";
let matched: string = s.match(re); // "hello world"
```

---

## 4. 全局数学计算对象 (`Math`)

全面对接 ECMAScript 标准 `Math` 对象，以零运行时开销直通 CPU 浮点指令：

```cp
// 1. 数学物理常量
let pi: f32 = Math.PI; // 3.14159265...
let e: f32 = Math.E;   // 2.71828182...

// 2. 取整与绝对值
let abs_val = Math.abs(-5.5); // 5.5
let c = Math.ceil(1.2);       // 2.0
let f = Math.floor(1.8);      // 1.0
let r = Math.round(1.5);      // 2.0
let t = Math.trunc(1.9);      // 1.0

// 3. 极值运算
let max_num = Math.max(10.0, 20.0); // 20.0
let min_num = Math.min(10.0, 20.0); // 10.0

// 4. 幂与对数
let sq = Math.sqrt(9.0);     // 3.0
let pw = Math.pow(2.0, 3.0); // 8.0
let lg = Math.log(Math.E);   // 1.0
let lg10 = Math.log10(100.0); // 2.0
let lg2 = Math.log2(8.0);    // 3.0

// 5. 三角函数
let s = Math.sin(0.0);
let cos_val = Math.cos(0.0);
let tan_val = Math.tan(0.0);

// 6. 随机数生成 [0.0, 1.0)
let rand_val = Math.random();
```

---

# 第二十二章：高并发服务、多线程与网络系统 (Bun HTTP, Actor, Worker & Zero-Copy Tensor)

## 1. 对标 Bun.js 的 `serve` 高层 HTTP 微服务系统 (`cap:http`)

CP 借鉴 Bun.js 的极致工程哲学，提供了无需配置复杂中间件的高性能原生 HTTP 服务原语：

### （1）极简服务定义与启动

```cp
import { serve, Response, Request, json } from "cap:http";
import { print } from "cap:io";

// 启动原生异步微服务
let server = serve({
    port: 8080,
    fetch: (req: Request) => {
        // 路由分发
        if (req.url == "/") {
            return new Response("Home Page from CP!");
        }
        if (req.url == "/api/status") {
            // json() 便捷构造器自动处理 application/json 请求头与序列化
            return json({ status: "healthy", version: "1.0.0", framework: "CP" });
        }

        // 404 自定义响应
        let not_found = new Response("404 Route Not Found");
        not_found.status = 404;
        return not_found;
    }
});

@print("HTTP Server listening on port: {d}", server.port);
```

### （2）核心构件定义

- **`Request` 类**：
  - `req.method: string`（如 `"GET"`, `"POST"`）
  - `req.url: string`（请求路径）
  - `req.body: string`（原始请求体）
  - `req.text(): string`（获取文本体）
  - `req.json(): T`（零拷贝调用编译期反序列化器提取结构体）
- **`Response` 类**：
  - `res.status: i32`（HTTP 状态码，缺省 200）
  - `res.contentType: string`（内容类型，缺省 `"text/plain"`）
  - `new Response(body: string)`（构造文本响应）
  - `json(data: any): Response`（将匿名对象或实体类序列化并附带 JSON 标头）
- **`Server` 实例**：
  - `server.port`（监听端口）
  - `server.fd`（底层监听套接字文件描述符）
  - `server.stop()`（优雅关闭服务器）

### （3）反应堆非阻塞架构

`serve` 会自动将底层 Linux 套接字注册至 CP 原生 `poll_fds` 反应堆，采用非阻塞事件通知模型，无需为每个请求启动繁重的操作系统线程。

---

## 2. Actor 抢占式协程模型 (`cap:actor`)

专为百万级 AI Agent 并发设计的轻量级执行单元：

```cp
import { spawn, receive, reply, self } from "cap:actor";

// 1. 派生独立的 Actor 协程
let worker = @spawn((msg) => {
    switch msg {
        .ping |sender| => {
            @print("Worker: 收到来自 {any} 的 Ping，正在回复 Pong...", sender);
            @reply(sender, { pong: @self() });
        },
        * => {
            @print("Worker: 未知消息");
        }
    }
});

// 2. 主协程投递消息
worker.send({ ping: @self() });

// 3. 抢占式挂起接收回复（CPS 状态机切片出让 CPU）
let res = @receive();
```

---

## 3. 多线程 Worker 与无锁环形信箱 (`cap:worker`)

为了充分压榨多核 CPU 与计算集群，CP 支持原生操作系统多线程：

- **`@worker_spawn`**：在独立 OS 线程中初始化完整的事件循环与 I/O 反应堆；
- **Thread-local 隔离**：每个 Worker 线程拥有完全隔离的任务队列与反应堆变量，达成真正的零锁（Lock-Free）多核并发；
- **跨线程环形信箱通道 (Cross-Thread RingBuffer Channel)**：基于原子自旋锁与无锁环形缓冲区投递跨线程消息，实现确定性线程间通信。

---

## 4. 零拷贝全局张量通信 (`SharedTensor`)

在 AI 大模型运算中，针对数百兆甚至数十吉字节的权重张量与嵌入向量（Embeddings），传统的序列化/反序列化（IPC）会造成灾难性的 CPU 与内存拷贝损耗。

CP 通过 `SharedTensor` 机制实现跨 Agent 零拷贝交互：

- 内存分配在全局无 GC 共享堆上；
- 采用硬件级原子引用计数（`std.atomic`）追踪生命周期；
- Agent 之间传递仅传递内存物理指针与元数据，实现 **0 物理内存拷贝** 与 **0 序列化开销**；
- 当最后一个持有的 Agent 释放时，内存自动归还操作系统。

---

## 5. 异步任务编排与多核并行运行时 (`cap:parallel`)

借鉴 Rust Tokio 的双轨并发架构，CP 提供**按需开启的多核 Worker 线程池 (`@parallel`)** 与 **专用计算线程派发 (`spawn.cpu`)**：

### 5.1 运行时模式控制 (`@parallel` / `@single_thread` / `cap.json`)
- **默认单线程 Reactor 模式**：未加标记时，程序默认以零线程切换开销的单线程 Reactor 模式运行，适合轻量 CLI 与纯异步 I/O 脚本。
- **`@parallel` / `@parallel(N)` 多核并行模式**：在 `async fn main()` 上标注 `@parallel`（或指定线程数 `@parallel(4)`），或在项目 `cap.json` 中配置 `"mode": "parallel"`，程序启动时自动根据 CPU 核心数拉起多核共享任务队列（`GlobalTaskQueue`）与 Worker 线程池，使所有 `spawn` 的异步协程自动在多核 CPU 上真正并行执行。
- **`@single_thread` 强制单线程覆盖**：即使 `cap.json` 开启了 `"mode": "parallel"`，也可在特定入口函数上用 `@single_thread` 强制锁定单线程模式。

### 5.2 核心原语与重度计算卸载 (`spawn` vs `spawn.cpu`)
- **`spawn(async () => { ... })`**：派生轻量级无栈异步协程任务，投入多核 Worker 队列，适合高并发 I/O 与常规并发任务，返回 `Task` 句柄。
- **`spawn.cpu(() => { ... })`**：将重度 CPU 密集型计算（如加密、压缩、矩阵运算）或同步阻塞调用卸载至独立的 OS 专用计算线程执行，绝不阻塞或饿死异步 Worker 线程池，同样返回可 `await` / `join` 的 `Task` 句柄。
- **`join(task1, task2, ...)`**：基于原子自旋锁保护的并发屏障（`JoinBarrier`）并行等待多个任务（支持混合等待 `spawn` 与 `spawn.cpu` 任务），待全部完成后按顺序返回结果元组。

```cp
import { print } from "cap:io";
import { spawn, join } from "cap:parallel";

@parallel(4)
async fn main() {
  let shared_prefix = "Prefix:";
  let secret_token = "OwnedSecret-42";

  // 1. 多核 Worker 线程池并发执行异步任务（支持只读借用捕获）
  let task1 = spawn(async () => {
    print(`${shared_prefix} Running Task 1`);
    return "Hello from Task 1";
  });

  // 2. 使用 move 将变量所有权安全转移至并发任务
  let task2 = spawn(async () => {
    let owned = move secret_token;
    print(`Task 2 received ${owned}`);
    return "Hello from Task 2";
  });

  // 3. 使用 spawn.cpu 将重度 CPU 计算卸载至独立计算线程
  let cpu_task = spawn.cpu(() => {
    let mut sum = 0;
    let mut i = 0;
    while (i < 1000) { sum = sum + i; i = i + 1; }
    return "CPU Compute Done";
  });

  // 等待并行结果汇总并按序解构
  let [msg1, msg2, msg3] = await join(task1, task2, cpu_task);

  print(msg1);
  print(msg2);
  print(msg3);
  print("Hello, Concurrent Plus!");
}
```

### 5.3 跨并行边界的内存安全法则 (Read-Only Borrow, Mutability & Ownership Check)
为了根除多线程并行下的数据竞争（Data Race）与悬垂指针（Dangling Pointer），CP 编译器对 `spawn` 与 `spawn.cpu` 捕获的外部变量执行严格的编译期安全检查：
1. **禁止捕获局部临时借用 (`&` / `&mut`)**：
   `CP Compile Error: Temporary borrow 'x' cannot be captured across parallel task spawn boundary (use move or Shared<T>)`
2. **禁止捕获栈绑定 `ref struct`**：
   `CP Compile Error: Stack-bound ref struct 'x' cannot be captured across parallel task spawn boundary`
3. **默认只读借用捕获与禁止跨任务直接修改**：
   未被 `move` 的外部变量以只读方式捕获（同时校验外层是否存在活跃的可变借用冲突），在子任务闭包内严禁直接修改该外部变量：
   `CP Compile Error: Cannot mutate read-only variable or captured outer variable 'x'`
4. **所有权转移 (`move`) 与 Use-After-Move 拦截**：
   在任务闭包内通过 `move x` 转移所有权时，编译器首先校验 `x` 无活跃借用，随后将所有权移交子任务并在外层标记失效；外层后续若再次访问 `x`，立即报错拦截：
   `CP Compile Error: Use of moved value 'x'`

---

## 6. 事件驱动与协程同步子系统 (`cap:events`)

为高层业务与 Agent 协作提供对标 Node.js / TypeScript 优雅习惯的发布订阅模型与协程同步原语：

### 6.1 `EventEmitter` 事件发布订阅
- **`on(event, handler)`**：注册监听器，并返回 Unsubscribe 退订函数（可直接配合 `defer unsub()` 杜绝监听泄漏）；
- **`off(event, handler)`**：注销特定监听器；
- **`emit(event, ...args)`**：同步分发事件；
- **`once(event)`**：单次监听，返回可 `await` 的 `Task` 句柄；
- **`listenerCount(event)`** 与 **`removeAllListeners(event)`**：监听器统计与批量清理。

### 6.2 `Event` 协程同步屏障 (Signal)
- **`isSet()`**：读取事件是否处于触发状态；
- **`set()`**：触发事件，一键唤醒所有挂起等待的任务；
- **`reset()`**：重置事件为未就绪态；
- **`wait()`**：返回异步 `Task`，当事件就绪时立即唤醒等待协程。

```cp
import { EventEmitter, Event } from "cap:events";
import { print } from "cap:io";

async fn main() {
    let mut ee = new EventEmitter();
    let unsub = ee.on("data", (msg) => {
        print(msg);
    });
    ee.emit("data");

    let mut evt = new Event();
    print(evt.isSet()); // false
    evt.set();
    print(evt.isSet()); // true
}
```

---

# 第二十三章：系统标准库生态索引 (Standard Library Reference)

CP 语言通过 `import { ... } from "cap:xxx"` 协议提供全面的极速标准库：

| 标准库命名空间   | 关键导出符号                                     | 职责与能力                                          |
| :--------------- | :----------------------------------------------- | :-------------------------------------------------- |
| **`cap:io`**     | `print`, `@print`                                | 高性能格式化终端与标准 I/O 输出                     |
| **`cap:parallel`** | `Task`, `spawn`, `join`                        | 并发任务编排、无锁屏障与无栈协程调度               |
| **`cap:events`** | `EventEmitter`, `Event`                          | Node.js 风格发布订阅与协程同步事件原语              |
| **`cap:http`**   | `serve`, `Request`, `Response`, `Server`, `json` | 对标 Bun 的超轻量极速 HTTP 微服务开发组件           |
| **`cap:actor`**  | `spawn`, `receive`, `reply`, `self`              | 轻量级协程 Actor 调度与消息投递                     |
| **`cap:worker`** | `Worker`, `@worker_spawn`                        | 操作系统原生多线程与多调度器并发支持                |
| **`cap:fs`**     | `open`, `read`, `write`, `readFile`, `writeFile` | 异步文件系统操作与流式传输                          |
| **`cap:net`**    | `Server`, `Socket`                               | 异步 TCP 网络套接字编程支持                         |
| **`cap:crypto`** | `hash`, `randomUUID`                             | 密码学 SHA-256 哈希与快速 UUID 生成器               |
| **`cap:sqlite`** | `Database`, `Statement`                          | 原生 C SQLite3 极速驱动封装与 RAII 自动生命周期释放 |
| **`cap:json`**   | `@json_parse`, `@json_stringify`                 | 编译期与运行时零拷贝 JSON 解析与序列化              |
| **`cap:mem`**    | `SharedTensor`, `alloc`, `free`                  | 跨 Agent 零拷贝大张量共享内存管理                   |
| **`cap:test`**   | `test`, `expect_eq`, `@expect_eq`                | 零运行时负担的自动化测试断言套件                    |
| **`cap:nest`**   | `Nest`, `NIF`                                    | 与 Nest 智能体容器的动态链接桥接协议                |
