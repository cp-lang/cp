// cap/test/test_fs_extensions.cp
const std = @import("std");
import { test, expect_eq } from "cap:test";
import { tmpDir, tmpPath, writeAtomic, readFileSync, remove, exists } from "cap:fs";

fn testTmpAndAtomicWrite(): !void {
    // 1. tmpDir resolution
    const tmp = tmpDir();
    expect_eq(tmp.len > 0, true);

    // 2. tmpPath generation
    var buf: [512]u8 = undefined;
    const path = tmpPath("agent_test", ".log", &buf);
    expect_eq(path.len > 0, true);
    expect_eq(std.mem.startsWith(u8, path, tmp), true);
    expect_eq(std.mem.endsWith(u8, path, ".log"), true);

    // 3. writeAtomic and read
    var dest_buf: [512]u8 = undefined;
    const dest_path = tmpPath("atomic_test", ".txt", &dest_buf);
    const content = "Atomic Content Payload in CP Standard Library";

    writeAtomic(dest_path, content);
    expect_eq(exists(dest_path), true);

    const read_back = readFileSync(dest_path);
    expect_eq(std.mem.eql(u8, read_back, content), true);

    // Clean up
    remove(dest_path);
    expect_eq(exists(dest_path), false);
}

fn main(): void {
    test("File system tmp directory and atomic write operations", () => {
        try testTmpAndAtomicWrite();
    });
}
