const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    // Custom cache and output directories
    b.cache_root = std.Build.Cache.Directory{
        .path = ".cap/zig-cache",
        .handle = std.fs.cwd().openDir(".cap/zig-cache", .{}) catch blk: {
            std.fs.cwd().makePath(".cap/zig-cache") catch {};
            break :blk std.fs.cwd().openDir(".cap/zig-cache", .{}) catch unreachable;
        },
    };
    b.install_path = "bin";

    const exe = b.addExecutable(.{
        .name = "cap",
        .root_module = b.createModule(.{
            .root_source_file = b.path("src/main.zig"),
            .target = target,
            .optimize = optimize,
        }),
    });

    // Install directly into bin/ (the prefix is already set via install_path)
    const install_artifact = b.addInstallArtifact(exe, .{
        .dest_dir = .{ .override = .{ .custom = "" } }
    });
    b.getInstallStep().dependOn(&install_artifact.step);

    const run_cmd = b.addRunArtifact(exe);
    run_cmd.step.dependOn(b.getInstallStep());

    if (b.args) |args| {
        run_cmd.addArgs(args);
    }

    const run_step = b.step("run", "Run the app");
    run_step.dependOn(&run_cmd.step);

    const unit_tests = b.addTest(.{
        .root_module = b.createModule(.{
            .root_source_file = b.path("test/main_test.zig"),
            .target = target,
            .optimize = optimize,
        }),
    });

    const run_unit_tests = b.addRunArtifact(unit_tests);
    const test_step = b.step("test", "Run unit tests");
    test_step.dependOn(&run_unit_tests.step);
}
