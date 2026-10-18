import { print } from "cap:io";

// ============================================================================
// 1. 领域错误定义与声明式转换 (from)
// ============================================================================

enum FileErr {
    NotFound,
    AccessDenied,
    Corrupted
}

enum NetErr {
    Timeout,
    ConnReset
}

// 统一上层应用错误：通过 `from` 声明，底层抛出的 FileErr 和 NetErr 会自动提升转换
enum AppErr {
    Io(FileErr) from FileErr,
    Net(NetErr) from NetErr,
    Unknown
}

// ============================================================================
// 2. 三态函数返回类型 (?T ! E) 与直接返回值 / null / Err(...)
// ============================================================================

// 返回值三态：
// 1. 成功返回 string（直接返回，不用 Ok(...) 包装）
// 2. 找到文件但内容为空时返回 null
// 3. 失败返回 FileErr（使用 Err(...) 构造）
fn readConfigFile(filename: string): ?string ! FileErr {
    if (filename == "") {
        return Err(FileErr.NotFound);
    }
    if (filename == "forbidden.json") {
        // 支持上下文简写 .AccessDenied
        return Err(.AccessDenied);
    }
    if (filename == "empty.json") {
        return null; // 返回空值
    }
    return "{model: cp-agent-v1, temp: 0.7}"; // 成功直接返回
}

fn fetchRemoteModel(url: string): string ! NetErr {
    if (url == "bad_url") {
        return Err(NetErr.Timeout);
    }
    return "weights_downloaded_ok";
}

// 匿名错误集 !{ ... }
fn validatePrompt(prompt: string): i32 ! { EmptyPrompt, TooLong } {
    if (prompt == "") {
        return Err(.EmptyPrompt);
    }
    if (prompt.indexOf("long") >= 0) {
        return Err(.TooLong);
    }
    return 100; // 校验通过打分 100
}

// ============================================================================
// 3. 可选链 (?.) 与对象模型
// ============================================================================

class HardwareSpec {
    gpu_id: i32;
    fn init(id: i32) {
        _.gpu_id = id;
    }
    fn getGpuId(): i32 {
        return _.gpu_id;
    }
}

class RuntimeEnv {
    spec: ?HardwareSpec;
    fn init(move s: ?HardwareSpec) {
        _.spec = s;
    }
}

class AgentProfile {
    name: string;
    runtime: ?RuntimeEnv;
    fn init(n: string, move r: ?RuntimeEnv) {
        _.name = n;
        _.runtime = r;
    }
}

// ============================================================================
// 4. Box<dyn Trait> 胖指针与运行时向下转换 (downcast)
//    - 只有动态插件/三方扩展才选用 Box<dyn Trait>
//    - 编译器自动生成 vtable 虚表，离开作用域自动执行 release() 回收堆内存
// ============================================================================

trait AgentPlugin {
    fn name(): string;
    fn priority(): i32;
}

class SafetyFilter implements AgentPlugin {
    rules_count: i32;
    fn init(count: i32) {
        _.rules_count = count;
    }
    fn name(): string {
        return "SafetyFilter";
    }
    fn priority(): i32 {
        return 1;
    }
    fn release() {
        @print(">> SafetyFilter 堆内存由 Box 自动释放");
    }
}

class SpeedOptimizer implements AgentPlugin {
    threads: i32;
    fn init(t: i32) {
        _.threads = t;
    }
    fn name(): string {
        return "SpeedOptimizer";
    }
    fn priority(): i32 {
        return 2;
    }
    fn release() {
        @print(">> SpeedOptimizer 堆内存由 Box 自动释放");
    }
}

fn loadDynamicPlugin(type_id: i32): Box<dyn AgentPlugin> {
    if (type_id == 1) {
        let f = new SafetyFilter(42);
        return Box.new(f); // 自动装箱为胖指针
    }
    let o = new SpeedOptimizer(8);
    return Box.new(o);
}

// ============================================================================
// 5. 错误冒泡、兜底、自动冒泡域 (try/catch)
// ============================================================================

// 尾部 ? default: 遇到 Err 或 null 时直接作为表达式缺省值，省略冗余的 return
fn safeGetConfig(filename: string): string {
    let cfg = readConfigFile(filename) ? "DEFAULT_CONFIG";
    return "GOT:" + cfg;
}

// 声明式 from: 多错误类型向统一应用错误 AppErr 自动提升转换
fn assembleService(cfg_path: string, remote_url: string): string ! AppErr {
    // readFile 抛出的 FileErr 自动提升转换为 AppErr.Io(FileErr)
    let cfg = readConfigFile(cfg_path)? ?? "DEFAULT_CONFIG";
    // fetchRemoteModel 抛出的 NetErr 自动提升转换为 AppErr.Net(NetErr)
    let weights = fetchRemoteModel(remote_url)?;
    return cfg + " + " + weights;
}

// 自动冒泡区域: try 块内部省略尾部 ?，任何抛错自动路由至 catch
fn runAgentTask(cfg_path: string): string {
    let result = "init";
    try {
        // try 内部可直接调用带错误返回的函数，自动捕获抛错并跳转 catch
        let cfg_raw = readConfigFile(cfg_path);
        let cfg = cfg_raw ?? "EMPTY_CONTENT";
        result = "Agent Running with config: " + cfg;
    } catch err => {
        result = switch err {
            FileErr.NotFound => "ERROR: 配置文件不存在",
            FileErr.AccessDenied => "ERROR: 权限不足",
            FileErr.Corrupted => "ERROR: 文件损坏"
        };
    }
    return result;
}

fn testPluginLifecycle() {
    let plugin = loadDynamicPlugin(1);
    @print("7. 虚表分发: plugin_name = {s}, priority = {d}", plugin.name(), plugin.priority());

    // 运行时 O(1) 类型校验 downcast
    let filter_ref = plugin.downcast<SafetyFilter>();
    let rules = filter_ref?.rules_count ?? 0;
    @print("8. downcast<SafetyFilter>: rules_count = {d}", rules);

    let wrong_ref = plugin.downcast<SpeedOptimizer>();
    let wrong_threads = wrong_ref?.threads ?? -999;
    @print("9. 错误类型 downcast<SpeedOptimizer>: {d} (返回 null 预期)", wrong_threads);
    // 超出 testPluginLifecycle 作用域时，Box 自动调用 release() 释放
}

// ============================================================================
// 6. 主程序与全场景运行验证
// ============================================================================

fn main() {
    @print("=== CP 语言三态返回、错误处理、Box 胖指针与 switch 示例 ===");

    // A. 三态返回值与 switch 模式匹配
    let res1 = readConfigFile("config.json");
    let desc1 = switch res1 {
        content => "读取成功: " + content,
        null => "文件内容为空",
        Err(FileErr.NotFound) => "错误: 找不到文件",
        Err(FileErr.AccessDenied) => "错误: 访问受限",
        Err(*) => "其他错误"
    };
    @print("1. switch 模式匹配: {s}", desc1);

    // B. 后缀 `? default` (遇到 Err 或 null 均兜底)
    let fallback_val = readConfigFile("forbidden.json") ? "BACKUP_CONFIG";
    @print("2. '? default' 兜底: {s}", fallback_val);

    // C. 后缀 `? default` 赋予缺省值 (省略 return)
    let ret_val = safeGetConfig("");
    @print("3. '? default' 赋予缺省值: {s}", ret_val);

    // D. 内联 catch 与纯 null 合并 (??)
    let inline_res = (readConfigFile("forbidden.json") catch err => "FALLBACK_FROM_ERR") ?? "FALLBACK_FROM_NULL";
    @print("4. catch + ?? 链式处理: {s}", inline_res);

    // E. 匿名错误集与 switch 解构
    let val_res = validatePrompt("test prompt");
    let val_score = switch val_res {
        score => score,
        Err(.EmptyPrompt) => -1,
        Err(.TooLong) => -2
    };
    @print("5. 匿名错误集: score = {d}", val_score);

    // F. 可选链 (?.) 深度访问
    let spec = new HardwareSpec(0);
    let env = new RuntimeEnv(spec);
    let agent1: ?AgentProfile = new AgentProfile("Agent-Alpha", env);
    let agent2: ?AgentProfile = new AgentProfile("Agent-Beta", null);
    let null_agent: ?AgentProfile = null;

    let gpu_id1 = agent1?.runtime?.spec?.getGpuId() ?? -1;
    let gpu_id2 = agent2?.runtime?.spec?.getGpuId() ?? -1;
    let gpu_id3 = null_agent?.runtime?.spec?.getGpuId() ?? -1;
    @print("6. 可选链 (?.): gpu1={d}, gpu2={d}, gpu3={d}", gpu_id1, gpu_id2, gpu_id3);

    // G. Box<dyn Trait> 胖指针与运行时向下转换 (downcast)
    testPluginLifecycle();

    // H. 声明式 from 多错误集提升转换
    let service_res = assembleService("config.json", "http://model-host");
    let service_desc = switch service_res {
        val => "组装服务成功: " + val,
        Err(AppErr.Io(e)) => "服务 IO 错误",
        Err(AppErr.Net(e)) => "服务网络错误",
        Err(AppErr.Unknown) => "未知服务错误"
    };
    @print("10. 声明式 from: {s}", service_desc);

    // I. try/catch 自动冒泡区域
    let task_ok = runAgentTask("config.json");
    let task_err = runAgentTask("forbidden.json");
    @print("11. try/catch 结果:\n    成功 => {s}\n    捕获 => {s}", task_ok, task_err);

    // J. switch 高级特性: 范围 (..=)、守卫 (if) 与 或模式 (|)
    let latency = 85;
    let status = switch latency {
        0..=50 => "极致极速",
        51..=100 if latency < 90 => "良好响应 (延迟 < 90ms)",
        51..=100 => "及格响应",
        404 | 500 => "网络故障",
        * => "超时降级"
    };
    @print("12. switch 范围与守卫: latency={d} => {s}", latency, status);
}
