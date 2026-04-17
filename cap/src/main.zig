const std = @import("std");
const process = std.process;
const builtin = @import("builtin");
const manifest = @import("project/manifest.zig");
const semver = @import("project/semver.zig");

const REGISTRY_PATH = "/data/cps/mock_registry";

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    const args = try process.argsAlloc(allocator);
    defer process.argsFree(allocator, args);

    if (args.len < 2) { usage(); return; }

    const cmd = args[1];
    if (std.mem.eql(u8, cmd, "run")) { _ = try handleRun(allocator, args[2..]); }
    else if (std.mem.eql(u8, cmd, "build")) { try buildPackage(allocator, args[2..]); }
    else if (std.mem.eql(u8, cmd, "init")) { try handleInit(allocator); }
    else if (std.mem.eql(u8, cmd, "add")) { try handleAdd(allocator, args[2..]); }
    else if (std.mem.eql(u8, cmd, "install")) { try handleInstall(allocator); }
    else if (std.mem.eql(u8, cmd, "sniff")) { try sniffHardware(); }
    else if (std.mem.eql(u8, cmd, "help")) { usage(); }
    else {
        if (std.mem.endsWith(u8, cmd, ".cp")) { try runFile(allocator, cmd); }
        else {
            const is_script = try handleRun(allocator, args[1..]);
            if (!is_script) {
                std.debug.print("Unknown command or script: {s}\n", .{cmd}); usage();
            }
        }
    }
}

fn usage() void {
    std.debug.print(
        \\CAP (Concurrent Agent Packager) v0.1.0
        \\The unified build tool and runner for CP.
        \\
        \\Usage:
        \\  cap <file.cp>           Run a CP file directly
        \\  cap run <script>        Execute a script from cap.json
        \\  cap <script>            Execute a script directly (e.g. cap start)
        \\  cap init                Initialize a new CP project
        \\  cap build <file.cp>     Build a CP package
        \\  cap add <package>       Add a dependency
        \\  cap install             Install all dependencies
        \\  cap sniff               Detect hardware features
        \\  cap help                Show this help
        \\
    , .{});
}

fn handleRun(allocator: std.mem.Allocator, args: []const []const u8) !bool {
    if (args.len == 0) return false;
    const script_name = args[0];
    const file = std.fs.cwd().openFile("cap.json", .{}) catch {
        return false;
    };
    defer file.close();
    const content = try file.readToEndAlloc(allocator, 1024 * 1024);
    defer allocator.free(content);
    var mf = try manifest.Manifest.parse(allocator, content);
    defer mf.deinit(allocator);

    if (mf.scripts.get(script_name)) |script_cmd| {
        std.debug.print("Executing script: {s}\n", .{script_cmd});
        var count: usize = 0;
        var it_count = std.mem.tokenizeAny(u8, script_cmd, " ");
        while (it_count.next()) |_| count += 1;
        var cmd_list = try allocator.alloc([]const u8, count);
        defer allocator.free(cmd_list);
        var it = std.mem.tokenizeAny(u8, script_cmd, " ");
        var i: usize = 0;
        while (it.next()) |token| { cmd_list[i] = token; i += 1; }
        var child = std.process.Child.init(cmd_list, allocator);
        _ = try child.spawnAndWait();
        return true;
    }
    return false;
}

fn handleAdd(allocator: std.mem.Allocator, args: []const []const u8) !void {
    if (args.len == 0) return;
    const pkg_arg = args[0];
    var pkg_name = pkg_arg;
    var version_range = try allocator.dupe(u8, "^0.1.0"); // Default

    if (std.mem.indexOfScalar(u8, pkg_arg, '@')) |idx| {
        pkg_name = pkg_arg[0..idx];
        version_range = try allocator.dupe(u8, pkg_arg[idx+1..]);
    }

    const file = std.fs.cwd().openFile("cap.json", .{ .mode = .read_write }) catch {
        std.debug.print("Error: cap.json not found.\n", .{});
        return;
    };
    defer file.close();
    const content = try file.readToEndAlloc(allocator, 1024 * 1024);
    defer allocator.free(content);
    var mf = try manifest.Manifest.parse(allocator, content);
    defer mf.deinit(allocator);

    try mf.addDependency(allocator, pkg_name, version_range);
    try mf.save(allocator, "cap.json");

    std.debug.print("Added dependency: {s}@{s}\n", .{pkg_name, version_range});
    try handleInstall(allocator);
}

fn handleInstall(allocator: std.mem.Allocator) !void {
    const file = std.fs.cwd().openFile("cap.json", .{}) catch {
        std.debug.print("Error: cap.json not found.\n", .{});
        return;
    };
    defer file.close();
    const content = try file.readToEndAlloc(allocator, 1024 * 1024);
    defer allocator.free(content);
    var mf = try manifest.Manifest.parse(allocator, content);
    defer mf.deinit(allocator);

    var iter = mf.dependencies.iterator();
    while (iter.next()) |entry| {
        const name = entry.key_ptr.*;
        const range_text = entry.value_ptr.*;
        const range = try semver.VersionRange.parse(range_text);
        
        const best_ver = try resolveBestVersion(allocator, name, range);
        try fetchAndLink(allocator, name, best_ver);
    }
}

fn resolveBestVersion(_: std.mem.Allocator, name: []const u8, range: semver.VersionRange) !semver.Version {
    var reg_dir = try std.fs.openDirAbsolute(REGISTRY_PATH, .{ .iterate = true });
    defer reg_dir.close();
    var pkg_dir = try reg_dir.openDir(name, .{ .iterate = true });
    defer pkg_dir.close();

    var best: ?semver.Version = null;
    var it = pkg_dir.iterate();
    while (try it.next()) |entry| {
        if (entry.kind == .directory) {
            const v = semver.Version.parse(entry.name) catch continue;
            if (range.satisfies(v)) {
                if (best == null or v.compare(best.?) > 0) {
                    best = v;
                }
            }
        }
    }

    if (best) |v| return v;
    return error.PackageNotFound;
}

fn fetchAndLink(allocator: std.mem.Allocator, name: []const u8, version: semver.Version) !void {
    const v_str = try version.format(allocator);
    defer allocator.free(v_str);

    // 1. Mock Global Cache (~/.cap/cache)
    const home = process.getEnvVarOwned(allocator, "HOME") catch try allocator.dupe(u8, "/root");
    defer allocator.free(home);
    const cache_root = try std.fs.path.join(allocator, &[_][]const u8{ home, ".cap", "cache" });
    defer allocator.free(cache_root);
    try std.fs.cwd().makePath(cache_root);

    const cache_pkg_path = try std.fs.path.join(allocator, &[_][]const u8{ cache_root, name, v_str });
    defer allocator.free(cache_pkg_path);

    // Make sure parent directory exists (~/.cap/cache/<name>)
    const cache_pkg_parent = std.fs.path.dirname(cache_pkg_path).?;
    {
        var root_dir = try std.fs.openDirAbsolute("/", .{});
        defer root_dir.close();
        try root_dir.makePath(cache_pkg_parent[1..]);
    }

    var cache_exists = true;
    var d = std.fs.openDirAbsolute(cache_pkg_path, .{}) catch blk: {
        cache_exists = false;
        break :blk std.fs.cwd();
    };
    if (cache_exists) {
        d.close();
    } else {
        std.debug.print("Fetching {s}@{s} to cache...\n", .{name, v_str});
        const reg_pkg_path = try std.fs.path.join(allocator, &[_][]const u8{ REGISTRY_PATH, name, v_str });
        defer allocator.free(reg_pkg_path);
        
        try copyDir(reg_pkg_path, cache_pkg_path);
    }

    // 2. Link to cap_modules/
    try std.fs.cwd().makePath("cap_modules");
    const link_path = try std.fs.path.join(allocator, &[_][]const u8{ "cap_modules", name });
    defer allocator.free(link_path);

    // Remove existing link/file if any
    std.fs.cwd().deleteFile(link_path) catch {};
    std.fs.cwd().deleteTree(link_path) catch {};

    std.debug.print("Linking {s}@{s} -> cap_modules/{s}\n", .{name, v_str, name});
    if (@import("builtin").os.tag == .windows) {
        try copyDir(cache_pkg_path, link_path);
    } else {
        try std.posix.symlink(cache_pkg_path, link_path);
    }
}

fn copyDir(src: []const u8, dest: []const u8) !void {
    const argv = &[_][]const u8{ "cp", "-r", src, dest };
    var child = std.process.Child.init(argv, std.heap.page_allocator);
    _ = try child.spawnAndWait();
}

fn runFile(allocator: std.mem.Allocator, filename: []const u8) !void {
    std.debug.print("Compiling: {s}...\n", .{filename});

    const full_base_name = filename[0 .. filename.len - 3];
    const zig_file = try std.fmt.allocPrint(allocator, ".cap/zig/{s}.zig", .{full_base_name});
    defer allocator.free(zig_file);
    
    if (std.fs.path.dirname(zig_file)) |dirname| {
        std.fs.cwd().makePath(dirname) catch {};
    }

    const compile_args = &[_][]const u8{ "/data/cps/cpc/target/release/cpc", "compile", filename, "--output", zig_file, "-I", "/data/cps/cap/cap_modules", "-I", "/data/cps/lib" };
    var compile_child = std.process.Child.init(compile_args, allocator);
    const compile_res = try compile_child.spawnAndWait();
    if (compile_res.Exited != 0) return error.CompileFailed;

    std.debug.print("Linking: {s}...\n", .{zig_file});
    
    var bin_subpath: []const u8 = full_base_name;
    if (std.mem.startsWith(u8, full_base_name, "src/")) {
        bin_subpath = full_base_name[4..];
    }
    
    const run_path = try std.fmt.allocPrint(allocator, "bin/{s}", .{bin_subpath});
    defer allocator.free(run_path);
    
    if (std.fs.path.dirname(run_path)) |dirname| {
        std.fs.cwd().makePath(dirname) catch {};
    }

    const emit_bin_arg = try std.fmt.allocPrint(allocator, "-femit-bin={s}", .{run_path});
    defer allocator.free(emit_bin_arg);
    std.fs.cwd().makePath(".cap/zig-cache") catch {};
    const build_args = &[_][]const u8{ "zig", "build-exe", zig_file, emit_bin_arg, "--cache-dir", ".cap/zig-cache" };
    var build_child = std.process.Child.init(build_args, allocator);
    const build_res = try build_child.spawnAndWait();
    if (build_res.Exited != 0) return error.BuildFailed;

    std.debug.print("Executing: {s}...\n", .{run_path});
    var run_child = std.process.Child.init(&[_][]const u8{run_path}, allocator);
    _ = try run_child.spawnAndWait();
}

fn sniffHardware() !void {
    std.debug.print("--- CAP Hardware Sniffing ---\n", .{});
    std.debug.print("CPU: {s}\n", .{@tagName(builtin.cpu.arch)});
    if (builtin.cpu.arch == .x86_64) {
        const avx2 = std.Target.x86.featureSetHas(builtin.cpu.features, .avx2);
        std.debug.print("AVX2 Support: {}\n", .{avx2});
    }
    std.debug.print("OS: {s}\n", .{@tagName(builtin.os.tag)});
}

fn buildPackage(allocator: std.mem.Allocator, args: []const []const u8) !void {
    var targets_to_build: [100][]const u8 = undefined;
    var targets_len: usize = 0;

    var is_release = false;
    var optimize_mode: []const u8 = "ReleaseSafe";
    var build_type: []const u8 = "build-exe";
    var zig_args: [100][]const u8 = undefined;
    var zig_args_len: usize = 0;

    var has_target = false;

    for (args) |arg| {
        if (std.mem.eql(u8, arg, "-r") or std.mem.eql(u8, arg, "--release") or std.mem.eql(u8, arg, "release")) {
            is_release = true;
        } else if (std.mem.eql(u8, arg, "-f")) {
            optimize_mode = "ReleaseFast";
        } else if (std.mem.eql(u8, arg, "-s")) {
            optimize_mode = "ReleaseSmall";
        } else if (std.mem.eql(u8, arg, "--exe")) {
            build_type = "build-exe";
        } else if (std.mem.eql(u8, arg, "--obj")) {
            build_type = "build-obj";
        } else if (std.mem.eql(u8, arg, "--lib")) {
            build_type = "build-lib";
        } else if (std.mem.startsWith(u8, arg, "-")) {
            if (zig_args_len < 100) { zig_args[zig_args_len] = arg; zig_args_len += 1; }
        } else {
            has_target = true;
            if (targets_len < 100) { targets_to_build[targets_len] = arg; targets_len += 1; }
        }
    }

    if (!has_target) {
        if (std.fs.cwd().statFile("src")) |_| {
            if (targets_len < 100) { targets_to_build[targets_len] = "src"; targets_len += 1; }
        } else |_| {}
        if (std.fs.cwd().statFile("test")) |_| {
            if (targets_len < 100) { targets_to_build[targets_len] = "test"; targets_len += 1; }
        } else |_| {}
    }

    for (targets_to_build[0..targets_len]) |target_path| {
        try buildTarget(allocator, target_path, is_release, optimize_mode, build_type, zig_args[0..zig_args_len]);
    }
    std.debug.print("Build completed! Artifacts in bin/\n", .{});
}

fn buildTarget(allocator: std.mem.Allocator, target_path: []const u8, is_release: bool, optimize_mode: []const u8, build_type: []const u8, zig_args: []const []const u8) !void {
    const stat = std.fs.cwd().statFile(target_path) catch return;
    if (stat.kind == .directory) {
        var dir = try std.fs.cwd().openDir(target_path, .{ .iterate = true });
        defer dir.close();
        var it = dir.iterate();
        while (try it.next()) |entry| {
            const child_path = try std.fs.path.join(allocator, &[_][]const u8{ target_path, entry.name });
            defer allocator.free(child_path);
            try buildTarget(allocator, child_path, is_release, optimize_mode, build_type, zig_args);
        }
    } else if (std.mem.endsWith(u8, target_path, ".cp")) {
        try compileSingleFile(allocator, target_path, is_release, optimize_mode, build_type, zig_args);
    }
}

fn compileSingleFile(allocator: std.mem.Allocator, filename: []const u8, is_release: bool, optimize_mode: []const u8, build_type: []const u8, zig_args: []const []const u8) !void {
    std.debug.print("Building: {s}...\n", .{filename});

    const base_name = filename[0 .. filename.len - 3];
    const zig_file = try std.fmt.allocPrint(allocator, ".cap/zig/{s}.zig", .{base_name});
    defer allocator.free(zig_file);
    
    if (std.fs.path.dirname(zig_file)) |dirname| {
        std.fs.cwd().makePath(dirname) catch {};
    }

    const compile_args = &[_][]const u8{ "/data/cps/cpc/target/release/cpc", "compile", filename, "--output", zig_file, "-I", "/data/cps/cap/cap_modules", "-I", "/data/cps/lib" };
    var compile_child = std.process.Child.init(compile_args, allocator);
    const compile_res = try compile_child.spawnAndWait();
    if (compile_res.Exited != 0) return error.CompileFailed;

    std.fs.cwd().makePath(".cap/zig-cache") catch {};

    const Target = struct { name: []const u8, triple: []const u8, ext: []const u8 };
    var target_count: usize = 1;
    if (is_release) target_count = 5;

    const targets = &[_]Target{
        .{ .name = "Native", .triple = "native", .ext = "" },
        .{ .name = "cap-macos-aarch64", .triple = "aarch64-macos", .ext = "" },
        .{ .name = "cap-macos-x86_64", .triple = "x86_64-macos", .ext = "" },
        .{ .name = "cap-linux-x86_64", .triple = "x86_64-linux", .ext = "" },
        .{ .name = "cap-windows-x86_64", .triple = "x86_64-windows", .ext = ".exe" },
    };

    var bin_subpath: []const u8 = base_name;
    if (std.mem.startsWith(u8, base_name, "src/")) {
        bin_subpath = base_name[4..];
    }
    
    const base_ext = if (std.mem.eql(u8, build_type, "build-obj")) ".o" else if (std.mem.eql(u8, build_type, "build-lib")) ".a" else "";

    for (targets[0..target_count]) |t| {
        var dir_path: ?[]const u8 = null;
        defer if (dir_path) |d| allocator.free(d);
        var out_dir: []const u8 = "bin";
        if (!std.mem.eql(u8, t.name, "Native")) {
            dir_path = try std.fmt.allocPrint(allocator, "bin/{s}", .{t.name});
            out_dir = dir_path.?;
            std.fs.cwd().makePath(out_dir) catch {};
        }

        const out_file_path = try std.fmt.allocPrint(allocator, "{s}/{s}{s}{s}", .{out_dir, bin_subpath, base_ext, t.ext});
        defer allocator.free(out_file_path);

        if (std.fs.path.dirname(out_file_path)) |dirname| {
            std.fs.cwd().makePath(dirname) catch {};
        }
        
        var emit_flag: []const u8 = "-femit-bin=";
        if (std.mem.eql(u8, build_type, "build-obj")) emit_flag = "-femit-obj=";
        
        const emit_arg = try std.fmt.allocPrint(allocator, "{s}{s}", .{emit_flag, out_file_path});
        defer allocator.free(emit_arg);

        std.debug.print("Compiling for {s}...\n", .{t.name});

        var child_args: [100][]const u8 = undefined;
        var argc: usize = 0;
        child_args[argc] = "zig"; argc += 1;
        child_args[argc] = build_type; argc += 1;
        child_args[argc] = zig_file; argc += 1;
        child_args[argc] = emit_arg; argc += 1;
        child_args[argc] = "--cache-dir"; argc += 1;
        child_args[argc] = ".cap/zig-cache"; argc += 1;
        child_args[argc] = "-O"; argc += 1;
        child_args[argc] = optimize_mode; argc += 1;
        
        if (!std.mem.eql(u8, t.name, "Native")) {
            child_args[argc] = "-target"; argc += 1;
            child_args[argc] = t.triple; argc += 1;
        }
        
        for (zig_args) |za| {
            child_args[argc] = za; argc += 1;
        }

        var build_child = std.process.Child.init(child_args[0..argc], allocator);
        const build_res = try build_child.spawnAndWait();
        if (build_res.Exited != 0) {
            std.debug.print("Failed to build for {s}\n", .{t.name});
        }
    }
}

fn handleInit(allocator: std.mem.Allocator) !void {
    std.debug.print("Welcome to CAP (Concurrent Agent Packager)!\n\n", .{});
    
    // Create src directory
    std.fs.cwd().makePath("src") catch {};

    // Create cap.json if it doesn't exist
    if (std.fs.cwd().statFile("cap.json")) |_| {
        std.debug.print("cap.json already exists.\n", .{});
    } else |_| {
        const cwd_path = try std.fs.cwd().realpathAlloc(allocator, ".");
        defer allocator.free(cwd_path);
        const basename = std.fs.path.basename(cwd_path);
        
        var file = try std.fs.cwd().createFile("cap.json", .{});
        defer file.close();
        const json_content = try std.fmt.allocPrint(allocator, "{{\n  \"name\": \"{s}\",\n  \"version\": \"0.1.0\",\n  \"main\": \"src/{s}.cp\",\n  \"scripts\": {{\n    \"start\": \"cap src/{s}.cp\"\n  }},\n  \"dependencies\": {{}}\n}}\n", .{basename, basename, basename});
        defer allocator.free(json_content);
        try file.writeAll(json_content);
        std.debug.print("Created cap.json\n", .{});
    }

    // Create src/{basename}.cp if it doesn't exist
    const cp_path = try std.fmt.allocPrint(allocator, "src/{s}.cp", .{std.fs.path.basename(try std.fs.cwd().realpathAlloc(allocator, "."))});
    defer allocator.free(cp_path);
    if (std.fs.cwd().statFile(cp_path)) |_| {
        std.debug.print("{s} already exists.\n", .{cp_path});
    } else |_| {
        var file = try std.fs.cwd().createFile(cp_path, .{});
        defer file.close();
        try file.writeAll("import { print } from \"cap:io\";\n\nfn main(): ?void {\n    @print(\"Hello from CP! 🚀\");\n}\n");
        std.debug.print("Created {s}\n", .{cp_path});
    }

    // Create .gitignore if it doesn't exist
    if (std.fs.cwd().statFile(".gitignore")) |_| {
        std.debug.print(".gitignore already exists.\n", .{});
    } else |_| {
        var file = try std.fs.cwd().createFile(".gitignore", .{});
        defer file.close();
        try file.writeAll(".cap/\nbin/\ncap_modules/\n*.cp.zig\n");
        std.debug.print("Created .gitignore\n", .{});
    }

    std.debug.print("\nDone! Run `cap start` to run your code.\n", .{});
}
