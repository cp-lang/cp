import { expect_eq } from "cap:test";
import { print } from "cap:io";
import * as std from "std";

pub fn testEndsWith(name: []const u8): bool {
    return std.mem.endsWith(u8, name, "_reply");
}

pub fn testStartsWith(name: []const u8): bool {
    return std.mem.startsWith(u8, name, "task_");
}

async fn main(): void {
    let name1 = "task_solve_reply";
    let name2 = "other_name";

    expect_eq(testEndsWith(name1), true);
    expect_eq(testEndsWith(name2), false);
    expect_eq(testStartsWith(name1), true);
    expect_eq(testStartsWith(name2), false);

    print("std.mem.endsWith and startsWith verified successfully!\n");
}
