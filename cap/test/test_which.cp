// cap/test/test_which.cp
const std = @import("std");
import { test, expect_eq } from "cap:test";
import { Which, which, isExecutable } from "cap:which";

fn testWhichBasics(): !void {
    var out_buf: [512]u8 = undefined;

    // 1. Locate standard system binary 'sh'
    const sh_path = which("sh", .{}, &out_buf);
    expect_eq(sh_path != null, true);
    if (sh_path) |p| {
        expect_eq(std.mem.endsWith(u8, p, "/sh"), true);
        expect_eq(isExecutable(p), true);
    }

    // 2. Locate absolute path
    const ls_path = which("/bin/sh", .{}, &out_buf);
    expect_eq(ls_path != null, true);
    if (ls_path) |p| {
        expect_eq(std.mem.eql(u8, p, "/bin/sh"), true);
    }

    // 3. Non-existent binary returns null
    const nonexistent = which("non_existent_binary_xyz_12345", .{}, &out_buf);
    expect_eq(nonexistent == null, true);

    // 4. Custom PATH search
    var custom_buf: [512]u8 = undefined;
    const custom_found = which("sh", .{ .path = "/bin:/usr/bin" }, &custom_buf);
    expect_eq(custom_found != null, true);
}

fn main(): void {
    test("Which binary locator and PATH resolution", () => {
        try testWhichBasics();
    });
}
