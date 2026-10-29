**CP (Concurrent Plus)** is a modern, high-performance programming language designed from the ground up for the Multi-Agent era. By combining a strict TypeScript-like syntax, an Erlang/BEAM-inspired Actor model, and the raw power of Zig's cross-compilation, CP allows you to build millions of concurrent AI agents without fear of memory leaks or race conditions.

---

### 🏗️ Monorepo Architecture

This repository contains the entire CP language ecosystem, elegantly separated by concern:

- 🦀 **`cpc/` (Compiler):** The lightning-fast frontend written in Rust. Contains the Lexer, Parser, AST definitions, and CPS state-machine generator.
- ⚡ **`cap/` (Toolchain):** The global command-line interface written in Zig. Orchestrates builds, manages the `.cap/` workspace, and invokes native cross-compilation.
- 📚 **`lib/` (Standard Library):** The official CP standard library, mapping directly to the `cap:` protocol (e.g., `cap:io`, `cap:net`).

## 安装

```bash
mkdir -p ~/.cap/bin
ln -sf /data/dev/cp/cap/bin/cap ~/.cap/bin/cap
ln -sf /data/dev/cp/cap/bin/cpc ~/.cap/bin/cpc

# 将 ~/.cap/bin 加入 PATH
echo 'export PATH="$HOME/.cap/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

│ 验证全局生效：
│ 终端输入 cap help 或 cpc --help，若打印出带有 ^ 箭头的 CAP : Concurrent Application Plus 图标徽标，即表示配置成功。

──────

## 编译/运行

cap 已经内置了类似 bun 的全套工作流（即时运行、原生编译、单测、基准测试）：

执行 cap build（或日常调试 cap <file>），控制台会自动度量并分段展示：

1. cpc 前端转译耗时（毫秒级，通常只需 2~15ms）；
2. zig 原生优化与链接耗时（秒/毫秒级）；
3. 构建全生命周期总耗时。

│ 输出如下：
Building: cap/test/hello.cp (Mode: ReleaseSafe)...
→ [cpc] Transpiled in 2ms
→ Compiling for Native (ReleaseSafe)...
✓ Built Native in 16.15s
Build completed in 16.16s! Artifacts in bin/
──────

### 1. 即时运行（日常极速调试模式，百毫秒级）

对标 bun run，底层使用 Zig 的 Debug 模式直接转译并秒级启动执行：

```bash
cap cap/test/hello.cp # 或显式使用 run 子命令：
cap run cap/test/hello.cp

# 输出效果：
Compiling: cap/test/hello.cp (Mode: Debug)...
Linking: .cap/zig/cap/test/hello.zig...
Executing: bin/cap/test/hello...
Hello, Concurrent Plus!
专为 AI 大模型并行计算而生 🚀
```

### 2. 原生生产级构建（默认 ReleaseSafe）

编译生成带内存安全检查和栈保护的原生独立二进制（Native Executable），产物输出到 bin/ 对应子目录下：

```bash
cap build cap/test/hello.cp

#构建成功后直接运行产物：
./bin/cap/test/hello
```

### 3. 极限性能（ReleaseFast）与最小体积（ReleaseSmall）构建

```bash
# 极限性能优化（开启全套 LLVM 向量化与循环展开）：
cap build -f cap/test/hello.cp

#极限体积压缩（去除调试信息，极致小体积）：
cap build -s cap/test/hello.cp
```

### 4. 自定义产物输出目录

通过 -o 参数自定义可执行文件的输出路径：

```bash
cap build cap/test/hello.cp -o dist/
./dist/hello
```

### 5. 运行测试套件（内置 Test Runner）

对标 bun test，自动执行断言并生成统计报表：

`cap test cap/test/hello.cp`

### 6. 运行微秒级基准评测（内置 Benchmark Suite）

自动以 ReleaseFast 模式编译预热，进行微秒级吞吐耗时评测：

`cap bench cap/test/hello.cp`

### 7. 生成的 hello.zig 中间代码

生成的中间态 Zig 源码位于当前工程根目录的隐藏机房特区：
**hello.zig**
其路径结构与 .cp 源代码保持 100% 相对镜像映射：

• src/main.cp → .cap/zig/src/main.zig
• cap/test/hello.cp → .cap/zig/cap/test/hello.zig
• test/test_actor.cp → .cap/zig/test/test_actor.zig
──────

### 8. Zig 编译产生的中间产物

Zig 在将 .zig 编译为原生独立二进制时，产生的中间产物包括：

1. 内容哈希编址的增量编译单元与构建元数据（Comptime AST 缓存、符号依赖图、类型指纹）；
2. 底层机器码目标对象（.o / .obj）；
3. 平台链接胶水与管道文件。
   存储位置：

• 项目级本地缓存机房：zig-cache/（cap 显式通过 --cache-dir .cap/zig-cache 隔离存放，不污染全局）；
• 用户级系统全局缓存：~/.cache/zig/（Zig 自带的全局对象共享池，包含 o/ 目标文件缓存与 z/ 元数据缓存）。
──────

### 9. 代码未更改时跳过中间 zig 与 native 编译的机制

cpc 与 cap 双端实现了完整的三级增量阻断机制：

1. 第一道防线：产物与源文件 mtime 判定（Level 1）
   • 当执行 cap build 时，cap 自动比对 bin/ 下的目标可执行文件与源 .cp 文件的物理修改时间戳（mtime）；
   • 若 mtime(bin) >= mtime(.cp)，判定为全系统未修改，直接输出 [Up-to-date] 并 0ms 退出，完全不启动子进程。
2. 第二道防线：中间态 Zig 源码时间戳判定（Level 2）
   • 若二进制缺失但 .cap/zig/xxx.zig 的修改时间仍晚于 .cp 文件，cap 自动跳过 cpc compile 转译步骤，直接复用已有 Zig 源码。
3. 第三道防线：转译内容防抖（Level 3）
   • 我们在 cpc/src/main.rs 中增加了哈希一致性防护：若转译出的 Zig
   代码内容与磁盘已有文件完全相同，直接跳过写盘操作，严格保护磁盘文件的原始修改时间（mtime），防止意外破坏下游 Zig 的构建缓存。
4. 强制全量编译参数：
   • 若您修改了外部依赖或希望无条件重新生成，随时可传入 -B 或 --force：
   cap build -B cap/test/hello.cp

──────

### 10. 如何清理生成的中间代码与编译产物？

#### cap clean 指令

```bash
cap clean

#输出效果：

Cleaning build caches and intermediate artifacts...
  ✓ Removed intermediate Zig source (.cap/zig/)
  ✓ Removed Zig compiler cache (.cap/zig-cache/)
  ✓ Removed object files (.cap/obj/)
  ✓ Removed build output directory (bin/)
Clean completed successfully!
```

│ 清理之后再次运行 cap build，相当于没有任何缓存的 100% 全量纯净编译。

#### 手动清理

如果您希望手动通过 shell 彻底擦除，只需执行：

```bash
rm -rf .cap/zig .cap/zig-cache bin/
```

（注：cap 与 cpc 独立工具链二进制位于 cap/bin/ 中，受 cap clean 保护，绝对不会被误删。）
