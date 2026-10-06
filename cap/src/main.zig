const std = @import("std");

pub var global_io: std.Io = undefined;
pub var global_env: *std.process.Environ.Map = undefined;

pub const fs_shim = struct {
    pub fn cwd() CwdShim {
        return CwdShim{};
    }
};

pub const CwdShim = struct {
    pub fn statFile(self: CwdShim, sub_path: []const u8) !std.Io.Dir.Stat {
        _ = self;
        return std.Io.Dir.statFile(.cwd(), global_io, sub_path, .{});
    }
    pub fn openDir(self: CwdShim, sub_path: []const u8, options: std.Io.Dir.OpenOptions) !std.Io.Dir {
        _ = self;
        return std.Io.Dir.openDir(.cwd(), global_io, sub_path, options);
    }
    pub fn openFile(self: CwdShim, sub_path: []const u8, options: std.Io.Dir.OpenFileOptions) !std.Io.File {
        _ = self;
        return std.Io.Dir.openFile(.cwd(), global_io, sub_path, options);
    }
    pub fn createDirPath(self: CwdShim, io: std.Io, sub_path: []const u8) !void {
        _ = io;
        _ = self;
        return std.Io.Dir.createDirPath(.cwd(), global_io, sub_path);
    }
    pub fn deleteFile(self: CwdShim, sub_path: []const u8) !void {
        _ = self;
        return std.Io.Dir.deleteFile(.cwd(), global_io, sub_path);
    }
    pub fn deleteTree(self: CwdShim, sub_path: []const u8) !void {
        _ = self;
        return std.Io.Dir.deleteTree(.cwd(), global_io, sub_path);
    }
    pub fn realpathAlloc(self: CwdShim, allocator: std.mem.Allocator, sub_path: []const u8) ![]u8 {
        _ = self;
        return std.Io.Dir.realPathFileAlloc(.cwd(), global_io, sub_path, allocator);
    }
    pub fn readFileAlloc(self: CwdShim, allocator: std.mem.Allocator, sub_path: []const u8, max_bytes: usize) ![]u8 {
        _ = self;
        return std.Io.Dir.readFileAlloc(.cwd(), global_io, sub_path, allocator, @enumFromInt(max_bytes));
    }

    pub fn createFile(self: CwdShim, sub_path: []const u8, options: std.Io.Dir.CreateFileOptions) !std.Io.File {
        _ = self;
        return std.Io.Dir.createFile(.cwd(), global_io, sub_path, options);
    }
};
const process = std.process;
const builtin = @import("builtin");
const manifest = @import("project/manifest.zig");
const semver = @import("project/semver.zig");

const REGISTRY_PATH = "/data/dev/cp/mock_registry";

pub fn main(init: std.process.Init) !void {
    global_io = init.io;
    global_env = init.environ_map;
    const allocator = init.arena.allocator();

    var args_iter = std.process.Args.iterate(init.minimal.args);
    var args_list: std.ArrayList([]const u8) = .empty;
    while (args_iter.next()) |arg| {
        try args_list.append(allocator, arg);
    }
    const args = args_list.items;

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
        if (std.mem.endsWith(u8, cmd, ".cp")) {
            try runFile(allocator, cmd);
        } else {
            const stat = fs_shim.cwd().statFile(cmd) catch null;
            if (stat != null and stat.?.kind == .directory) {
                if (try resolveDirectoryMain(allocator, cmd)) |main_file| {
                    defer allocator.free(main_file);
                    try runFile(allocator, main_file);
                } else {
                    std.debug.print("Error: Could not resolve entry point in directory '{s}'\n", .{cmd});
                }
            } else {
                const is_script = try handleRun(allocator, args[1..]);
                if (!is_script) {
                    std.debug.print("Unknown command, script, or directory: {s}\n", .{cmd}); usage();
                }
            }
        }
    }
}

fn resolveDirectoryMain(allocator: std.mem.Allocator, dir_path: []const u8) !?[]const u8 {
    var dir = fs_shim.cwd().openDir(dir_path, .{}) catch return null;
    defer dir.close(global_io);

    // 1. Check main.cp
    if (dir.statFile(global_io, "main.cp", .{})) |_| {
        return try std.fmt.allocPrint(allocator, "{s}/main.cp", .{dir_path});
    } else |_| {}

    // 2. Check index.cp
    if (dir.statFile(global_io, "index.cp", .{})) |_| {
        return try std.fmt.allocPrint(allocator, "{s}/index.cp", .{dir_path});
    } else |_| {}

    // 3. Check cap.json main
    if (std.Io.Dir.readFileAlloc(dir, global_io, "cap.json", allocator, @enumFromInt(1024 * 1024))) |content| {
        
        
        defer allocator.free(content);
        var mf = manifest.Manifest.parse(allocator, content) catch return null;
        defer mf.deinit(allocator);
        if (mf.main) |main_file| {
            return try std.fmt.allocPrint(allocator, "{s}/{s}", .{dir_path, main_file});
        }
    } else |_| {}

    return null;
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
        \\  cap build --beam       Build for BEAM VM (shared library)
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
    const content = fs_shim.cwd().readFileAlloc(allocator, "cap.json", 1024 * 1024) catch {
        return false;
    };
    
    
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
        var child = try std.process.spawn(global_io, .{ .argv = cmd_list });
        _ = try child.wait(global_io);
        return true;
    }
    return false;
}

fn handleAdd(allocator: std.mem.Allocator, args: []const []const u8) !void {
    if (args.len == 0) return;
    var pkg_arg: []const u8 = undefined;
    var is_global = false;
    for (args) |arg| {
        if (std.mem.eql(u8, arg, "-g") or std.mem.eql(u8, arg, "--global")) {
            is_global = true;
        } else {
            pkg_arg = arg;
        }
    }
    
    var pkg_name = pkg_arg;
    var version_range = try allocator.dupe(u8, "^0.1.0"); // Default

    if (std.mem.indexOfScalar(u8, pkg_arg, '@')) |idx| {
        pkg_name = pkg_arg[0..idx];
        version_range = try allocator.dupe(u8, pkg_arg[idx+1..]);
    }

    if (is_global) {
        const best_ver = try resolveBestVersion(allocator, pkg_name, try semver.VersionRange.parse(version_range));
        try fetchAndLinkGlobal(allocator, pkg_name, best_ver);
        std.debug.print("Globally installed: {s}@{s}\n", .{pkg_name, version_range});
        return;
    }

    const content = fs_shim.cwd().readFileAlloc(allocator, "cap.json", 1024 * 1024) catch {
        std.debug.print("Error: cap.json not found.\n", .{});
        return;
    };
    
    
    defer allocator.free(content);
    var mf = manifest.Manifest.parse(allocator, content) catch return;
    defer mf.deinit(allocator);

    try mf.addDependency(allocator, pkg_name, version_range);
    try mf.save(allocator, "cap.json");

    std.debug.print("Added dependency: {s}@{s}\n", .{pkg_name, version_range});
    try handleInstall(allocator);
}

fn handleInstall(allocator: std.mem.Allocator) !void {
    const content = fs_shim.cwd().readFileAlloc(allocator, "cap.json", 1024 * 1024) catch {
        std.debug.print("Error: cap.json not found.\n", .{});
        return;
    };
    
    
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
    var reg_dir = try std.Io.Dir.openDirAbsolute(global_io, REGISTRY_PATH, .{ .iterate = true });
    defer reg_dir.close(global_io);
    var pkg_dir = try reg_dir.openDir(global_io, name, .{ .iterate = true });
    defer pkg_dir.close(global_io);

    var best: ?semver.Version = null;
    var it = pkg_dir.iterate();
    while (try it.next(global_io)) |entry| {
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

fn fetchAndLinkGlobal(allocator: std.mem.Allocator, name: []const u8, version: semver.Version) !void {
    const v_str = try version.format(allocator);
    defer allocator.free(v_str);

    const home = global_env.get("HOME") orelse "/root";
    defer allocator.free(home);
    const global_root = try std.Io.Dir.path.join(allocator, &[_][]const u8{ home, ".cap", "global", "cap_modules" });
    defer allocator.free(global_root);
    try fs_shim.cwd().createDirPath(global_io, global_root);
    
    const global_pkg_path = try std.Io.Dir.path.join(allocator, &[_][]const u8{ global_root, name });
    defer allocator.free(global_pkg_path);
    
    fs_shim.cwd().deleteFile(global_pkg_path) catch {};
    fs_shim.cwd().deleteTree(global_pkg_path) catch {};

    const reg_pkg_path = try std.Io.Dir.path.join(allocator, &[_][]const u8{ REGISTRY_PATH, name, v_str });
    defer allocator.free(reg_pkg_path);
        
    try copyDir(reg_pkg_path, global_pkg_path);
}

fn fetchAndLink(allocator: std.mem.Allocator, name: []const u8, version: semver.Version) !void {
    const v_str = try version.format(allocator);
    defer allocator.free(v_str);

    // 1. Mock Global Cache (~/.cap/cache)
    const home = global_env.get("HOME") orelse "/root";
    defer allocator.free(home);
    const cache_root = try std.Io.Dir.path.join(allocator, &[_][]const u8{ home, ".cap", "cache" });
    defer allocator.free(cache_root);
    try fs_shim.cwd().createDirPath(global_io, cache_root);

    const cache_pkg_path = try std.Io.Dir.path.join(allocator, &[_][]const u8{ cache_root, name, v_str });
    defer allocator.free(cache_pkg_path);

    // Make sure parent directory exists (~/.cap/cache/<name>)
    const cache_pkg_parent = std.Io.Dir.path.dirname(cache_pkg_path).?;
    {
        var root_dir = try std.Io.Dir.openDirAbsolute(global_io, "/", .{});
        defer root_dir.close(global_io);
        try root_dir.createDirPath(global_io, cache_pkg_parent[1..]);
    }

    var cache_exists = true;
    var d = std.Io.Dir.openDirAbsolute(global_io, cache_pkg_path, .{}) catch blk: {
        cache_exists = false;
        break :blk std.Io.Dir.cwd();
    };
    if (cache_exists) {
        d.close(global_io);
    } else {
        std.debug.print("Fetching {s}@{s} to cache...\n", .{name, v_str});
        const reg_pkg_path = try std.Io.Dir.path.join(allocator, &[_][]const u8{ REGISTRY_PATH, name, v_str });
        defer allocator.free(reg_pkg_path);
        
        try copyDir(reg_pkg_path, cache_pkg_path);
    }

    // 2. Link to cap_modules/
    try fs_shim.cwd().createDirPath(global_io, "cap_modules");
    const link_path = try std.Io.Dir.path.join(allocator, &[_][]const u8{ "cap_modules", name });
    defer allocator.free(link_path);

    // Remove existing link/file if any
    fs_shim.cwd().deleteFile(link_path) catch {};
    fs_shim.cwd().deleteTree(link_path) catch {};

    std.debug.print("Linking {s}@{s} -> cap_modules/{s}\n", .{name, v_str, name});
    if (@import("builtin").os.tag == .windows) {
        try copyDir(cache_pkg_path, link_path);
    } else {
        try std.Io.Dir.symLinkAbsolute(global_io, cache_pkg_path, link_path, .{});
    }
}

fn copyDir(src: []const u8, dest: []const u8) !void {
    const argv = &[_][]const u8{ "cp", "-r", src, dest };
    var child = try std.process.spawn(global_io, .{ .argv = argv });
    _ = try child.wait(global_io);
}

fn runFile(allocator: std.mem.Allocator, filename: []const u8) !void {
    std.debug.print("Compiling: {s}...\n", .{filename});

    const full_base_name = filename[0 .. filename.len - 3];
    const zig_file = try std.fmt.allocPrint(allocator, ".cap/zig/{s}.zig", .{full_base_name});
    defer allocator.free(zig_file);
    
    if (std.Io.Dir.path.dirname(zig_file)) |dirname| {
        fs_shim.cwd().createDirPath(global_io, dirname) catch {};
        if (false) {
            copyDir("/data/dev/cp/sdk/beam.zig", dirname) catch {};
            copyDir("/data/dev/cp/sdk/erl_nif.zig", dirname) catch {};
        }
    }

    var cpc_args_buf: [15][]const u8 = undefined;
    var cpc_argc: usize = 0;
    cpc_args_buf[cpc_argc] = "/data/dev/cp/cpc/target/release/cpc"; cpc_argc += 1;
    cpc_args_buf[cpc_argc] = "compile"; cpc_argc += 1;
    cpc_args_buf[cpc_argc] = filename; cpc_argc += 1;
    cpc_args_buf[cpc_argc] = "--output"; cpc_argc += 1;
    cpc_args_buf[cpc_argc] = zig_file; cpc_argc += 1;
    cpc_args_buf[cpc_argc] = "-I"; cpc_argc += 1;
    cpc_args_buf[cpc_argc] = "/data/dev/cp/cap/cap_modules"; cpc_argc += 1;
    cpc_args_buf[cpc_argc] = "-I"; cpc_argc += 1;
    cpc_args_buf[cpc_argc] = "/data/dev/cp/lib"; cpc_argc += 1;
    if (false) {
        cpc_args_buf[cpc_argc] = "-b"; cpc_argc += 1;
    }
    
    var compile_child = try std.process.spawn(global_io, .{ .argv = cpc_args_buf[0..cpc_argc] });
    const compile_res = try compile_child.wait(global_io);
    switch (compile_res) {
        .exited => |code| if (code != 0) return error.CompileFailed,
        else => return error.CompileFailed,
    }

    var bin_subpath: []const u8 = full_base_name;
    if (std.mem.startsWith(u8, full_base_name, "src/")) {
        bin_subpath = full_base_name[4..];
    }
    // Emit Type Definitions (.d.cp)
    {
        const type_out_path = try std.fmt.allocPrint(allocator, "bin/{s}.d.cp", .{bin_subpath});
        defer allocator.free(type_out_path);
        const emit_types_args = &[_][]const u8{ "/data/dev/cp/cpc/target/release/cpc", "emit-types", filename, "--output", type_out_path };
        var emit_types_child = try std.process.spawn(global_io, .{ .argv = emit_types_args });
        _ = try emit_types_child.wait(global_io);
    }

    std.debug.print("Linking: {s}...\n", .{zig_file});
    
    
    
    const run_path = try std.fmt.allocPrint(allocator, "bin/{s}", .{bin_subpath});
    defer allocator.free(run_path);
    
    if (std.Io.Dir.path.dirname(run_path)) |dirname| {
        fs_shim.cwd().createDirPath(global_io, dirname) catch {};
    }

    const emit_bin_arg = try std.fmt.allocPrint(allocator, "-femit-bin={s}", .{run_path});
    defer allocator.free(emit_bin_arg);
    fs_shim.cwd().createDirPath(global_io, ".cap/zig-cache") catch {};
    const build_args = &[_][]const u8{ "zig", "build-exe", zig_file, emit_bin_arg, "--cache-dir", ".cap/zig-cache", "-lc" };
    var build_child = try std.process.spawn(global_io, .{ .argv = build_args });
    const build_res = try build_child.wait(global_io);
    if (build_res.exited != 0) return error.BuildFailed;

    std.debug.print("Executing: {s}...\n", .{run_path});
    var run_child = try std.process.spawn(global_io, .{ .argv = &[_][]const u8{run_path} });
    _ = try run_child.wait(global_io);
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
    var is_beam = false;
    var optimize_mode: []const u8 = "ReleaseSafe";
    var build_type: []const u8 = "build-exe";
    var zig_args: [100][]const u8 = undefined;
    var zig_args_len: usize = 0;

    var has_target = false;

    for (args) |arg| {
        if (std.mem.eql(u8, arg, "-r") or std.mem.eql(u8, arg, "--release") or std.mem.eql(u8, arg, "release")) {
            is_release = true;
        } else if (std.mem.eql(u8, arg, "-b") or std.mem.eql(u8, arg, "--beam")) {
            is_beam = true;
            build_type = "build-lib";
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
        if (fs_shim.cwd().statFile("src")) |_| {
            if (targets_len < 100) { targets_to_build[targets_len] = "src"; targets_len += 1; }
        } else |_| {}
        if (fs_shim.cwd().statFile("test")) |_| {
            if (targets_len < 100) { targets_to_build[targets_len] = "test"; targets_len += 1; }
        } else |_| {}
    }

    for (targets_to_build[0..targets_len]) |target_path| {
        try buildTarget(allocator, target_path, is_release, is_beam, optimize_mode, build_type, zig_args[0..zig_args_len]);
    }
    std.debug.print("Build completed! Artifacts in bin/\n", .{});
}

fn buildTarget(allocator: std.mem.Allocator, target_path: []const u8, is_release: bool, is_beam: bool, optimize_mode: []const u8, build_type: []const u8, zig_args: []const []const u8) !void {
    const stat = fs_shim.cwd().statFile(target_path) catch return;
    if (stat.kind == .directory) {
        var dir = try fs_shim.cwd().openDir(target_path, .{ .iterate = true });
        defer dir.close(global_io);
        var it = dir.iterate();
        while (try it.next(global_io)) |entry| {
            const child_path = try std.Io.Dir.path.join(allocator, &[_][]const u8{ target_path, entry.name });
            defer allocator.free(child_path);
            try buildTarget(allocator, child_path, is_release, is_beam, optimize_mode, build_type, zig_args);
        }
    } else if (std.mem.endsWith(u8, target_path, ".cp")) {
        try compileSingleFile(allocator, target_path, is_release, is_beam, optimize_mode, build_type, zig_args);
    }
}

fn compileSingleFile(allocator: std.mem.Allocator, filename: []const u8, is_release: bool, is_beam: bool, optimize_mode: []const u8, build_type: []const u8, zig_args: []const []const u8) !void {
    std.debug.print("Building: {s}...\n", .{filename});

    const base_name = filename[0 .. filename.len - 3];
    const zig_file = try std.fmt.allocPrint(allocator, ".cap/zig/{s}.zig", .{base_name});
    defer allocator.free(zig_file);
    
    if (std.Io.Dir.path.dirname(zig_file)) |dirname| {
        fs_shim.cwd().createDirPath(global_io, dirname) catch {};
        if (is_beam) {
            copyDir("/data/dev/cp/sdk/beam.zig", dirname) catch {};
            copyDir("/data/dev/cp/sdk/erl_nif.zig", dirname) catch {};
        }
    }

    var cpc_args_buf: [15][]const u8 = undefined;
    var cpc_argc: usize = 0;
    cpc_args_buf[cpc_argc] = "/data/dev/cp/cpc/target/release/cpc"; cpc_argc += 1;
    cpc_args_buf[cpc_argc] = "compile"; cpc_argc += 1;
    cpc_args_buf[cpc_argc] = filename; cpc_argc += 1;
    cpc_args_buf[cpc_argc] = "--output"; cpc_argc += 1;
    cpc_args_buf[cpc_argc] = zig_file; cpc_argc += 1;
    cpc_args_buf[cpc_argc] = "-I"; cpc_argc += 1;
    cpc_args_buf[cpc_argc] = "/data/dev/cp/cap/cap_modules"; cpc_argc += 1;
    cpc_args_buf[cpc_argc] = "-I"; cpc_argc += 1;
    cpc_args_buf[cpc_argc] = "/data/dev/cp/lib"; cpc_argc += 1;
    if (is_beam) {
        cpc_args_buf[cpc_argc] = "-b"; cpc_argc += 1;
    }
    
    var compile_child = try std.process.spawn(global_io, .{ .argv = cpc_args_buf[0..cpc_argc] });
    const compile_res = try compile_child.wait(global_io);
    switch (compile_res) {
        .exited => |code| if (code != 0) return error.CompileFailed,
        else => return error.CompileFailed,
    }

    var bin_subpath: []const u8 = base_name;
    if (std.mem.startsWith(u8, base_name, "src/")) {
        bin_subpath = base_name[4..];
    }
    // Emit Type Definitions (.d.cp)
    {
        const type_out_path = try std.fmt.allocPrint(allocator, "bin/{s}.d.cp", .{bin_subpath});
        defer allocator.free(type_out_path);
        const emit_types_args = &[_][]const u8{ "/data/dev/cp/cpc/target/release/cpc", "emit-types", filename, "--output", type_out_path };
        var emit_types_child = try std.process.spawn(global_io, .{ .argv = emit_types_args });
        _ = try emit_types_child.wait(global_io);
    }

    fs_shim.cwd().createDirPath(global_io, ".cap/zig-cache") catch {};

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

    
    if (std.mem.startsWith(u8, base_name, "src/")) {
        bin_subpath = base_name[4..];
    }
    
    const base_ext = if (std.mem.eql(u8, build_type, "build-obj")) ".o" else if (is_beam) ".so" else if (std.mem.eql(u8, build_type, "build-lib")) ".a" else "";

    for (targets[0..target_count]) |t| {
        var dir_path: ?[]const u8 = null;
        defer if (dir_path) |d| allocator.free(d);
        var out_dir: []const u8 = "bin";
        if (!std.mem.eql(u8, t.name, "Native")) {
            dir_path = try std.fmt.allocPrint(allocator, "bin/{s}", .{t.name});
            out_dir = dir_path.?;
            fs_shim.cwd().createDirPath(global_io, out_dir) catch {};
        }

        const out_file_path = try std.fmt.allocPrint(allocator, "{s}/{s}{s}{s}", .{out_dir, bin_subpath, base_ext, t.ext});
        defer allocator.free(out_file_path);

        if (std.Io.Dir.path.dirname(out_file_path)) |dirname| {
            fs_shim.cwd().createDirPath(global_io, dirname) catch {};
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
        if (is_beam) {
            child_args[argc] = "-dynamic"; argc += 1;
        }
        child_args[argc] = zig_file; argc += 1;
        child_args[argc] = emit_arg; argc += 1;
        child_args[argc] = "--cache-dir"; argc += 1;
        child_args[argc] = ".cap/zig-cache"; argc += 1;
        child_args[argc] = "-O"; argc += 1;
        child_args[argc] = optimize_mode; argc += 1;
        child_args[argc] = "-lc"; argc += 1;
        
        if (!std.mem.eql(u8, t.name, "Native")) {
            child_args[argc] = "-target"; argc += 1;
            child_args[argc] = t.triple; argc += 1;
        }
        
        for (zig_args) |za| {
            child_args[argc] = za; argc += 1;
        }

        var build_child = try std.process.spawn(global_io, .{ .argv = child_args[0..argc] });
        const build_res = try build_child.wait(global_io);
        if (build_res.exited != 0) {
            std.debug.print("Failed to build for {s}\n", .{t.name});
        }
    }
}

fn handleInit(allocator: std.mem.Allocator) !void {
    std.debug.print("Welcome to CAP (Concurrent Agent Packager)!\n\n", .{});
    
    // Create src directory
    fs_shim.cwd().createDirPath(global_io, "src") catch {};

    // Create cap.json if it doesn't exist
    if (fs_shim.cwd().statFile("cap.json")) |_| {
        std.debug.print("cap.json already exists.\n", .{});
    } else |_| {
        const cwd_path = try fs_shim.cwd().realpathAlloc(allocator, ".");
        defer allocator.free(cwd_path);
        const basename = std.Io.Dir.path.basename(cwd_path);
        
        var file = try fs_shim.cwd().createFile("cap.json", .{});
        
        const json_content = try std.fmt.allocPrint(allocator, "{{\n  \"name\": \"{s}\",\n  \"version\": \"0.1.0\",\n  \"main\": \"src/main.cp\",\n  \"scripts\": {{\n    \"start\": \"cap src/main.cp\"\n  }},\n  \"dependencies\": {{}}\n}}\n", .{basename});
        defer allocator.free(json_content);
        try file.writeStreamingAll(global_io, json_content);
        std.debug.print("Created cap.json\n", .{});
    }

    // Create src/main.cp if it doesn't exist
    if (fs_shim.cwd().statFile("src/main.cp")) |_| {
        std.debug.print("src/main.cp already exists.\n", .{});
    } else |_| {
        var file = try fs_shim.cwd().createFile("src/main.cp", .{});
        
        try file.writeStreamingAll(global_io, "import { print } from \"cap:io\";\n\nfn main(): ?void {\n    @print(\"Hello from CP! 🚀\");\n}\n");
        std.debug.print("Created src/main.cp\n", .{});
    }

    // Create .gitignore if it doesn't exist
    if (fs_shim.cwd().statFile(".gitignore")) |_| {
        std.debug.print(".gitignore already exists.\n", .{});
    } else |_| {
        var file = try fs_shim.cwd().createFile(".gitignore", .{});
        
        try file.writeStreamingAll(global_io, ".cap/\nbin/\ncap_modules/\n*.cp.zig\n");
        std.debug.print("Created .gitignore\n", .{});
    }

    std.debug.print("\nDone! Run `cap start` to run your code.\n", .{});
}
