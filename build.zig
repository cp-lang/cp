const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    // =========================================================================
    // 1. uSockets + picohttpparser 静态库 (libusockets.a)
    // =========================================================================
    const usockets_mod = b.createModule(.{
        .target = target,
        .optimize = optimize,
        .link_libc = true,
    });

    // 头文件包含路径 (注意：Linux 文件系统大小写敏感，目录为 vendor/usockets)
    usockets_mod.addIncludePath(b.path("vendor/usockets/src"));
    usockets_mod.addIncludePath(b.path("vendor/picohttpparser"));

    // 编译 C 源码
    usockets_mod.addCSourceFiles(.{
        .files = &.{
            "vendor/usockets/src/bsd.c",
            "vendor/usockets/src/context.c",
            "vendor/usockets/src/loop.c",
            "vendor/usockets/src/socket.c",
            "vendor/usockets/src/eventing/epoll_kqueue.c",
            "vendor/usockets/src/cp_stubs.c",
            "vendor/picohttpparser/picohttpparser.c",
        },
        .flags = &.{
            "-DLIBUS_NO_SSL",
            "-O3",
            "-fno-exceptions",
        },
    });

    const lib = b.addLibrary(.{
        .linkage = .static,
        .name = "usockets",
        .root_module = usockets_mod,
    });

    // 安装头文件到 zig-out/include
    lib.installHeader(b.path("vendor/usockets/src/libusockets.h"), "libusockets.h");
    lib.installHeader(b.path("vendor/picohttpparser/picohttpparser.h"), "picohttpparser.h");

    // 安装静态库到 zig-out/lib/libusockets.a
    b.installArtifact(lib);

    // =========================================================================
    // 2. 一体化可执行程序构建 (类似 Bun 构建管线，将 CP 转译的 Zig 源码与 C 库内联)
    // =========================================================================
    const cp_entry_opt = b.option([]const u8, "cp-entry", "Path to CP-transpiled Zig main file (default: .cap/zig/src/main.zig)");
    const cp_entry = cp_entry_opt orelse ".cap/zig/src/main.zig";

    // 检查入口文件是否存在，若存在则构建可执行目标
    var entry_exists = false;
    if (std.Io.Dir.cwd().access(b.graph.io, cp_entry, .{})) |_| {
        entry_exists = true;
    } else |_| {}

    if (entry_exists) {
        const exe_mod = b.createModule(.{
            .root_source_file = b.path(cp_entry),
            .target = target,
            .optimize = optimize,
            .link_libc = true,
        });

        exe_mod.addIncludePath(b.path("vendor/usockets/src"));
        exe_mod.addIncludePath(b.path("vendor/picohttpparser"));
        exe_mod.linkLibrary(lib);

        const exe = b.addExecutable(.{
            .name = "server",
            .root_module = exe_mod,
        });

        b.installArtifact(exe);

        const run_cmd = b.addRunArtifact(exe);
        run_cmd.step.dependOn(b.getInstallStep());

        const run_step = b.step("run", "Run the server application");
        run_step.dependOn(&run_cmd.step);
    }
}
