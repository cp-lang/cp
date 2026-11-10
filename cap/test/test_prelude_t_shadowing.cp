import { test, expect_eq } from "cap:test";

pub const T = enum {
    Alpha,
    Beta,
};

pub fn iterate_slice(items: []const u8): usize {
    let count: usize = 0;
    for (items) |c| {
        if (c > 0) {
            count = count + 1;
        }
    }
    return count;
}

pub fn main(): void {
    test("Prelude helper T shadowing resistance", () => {
        let val: T = T.Alpha;
        expect_eq(val, T.Alpha);
        let data: []const u8 = "hello";
        let cnt = iterate_slice(data);
        expect_eq(cnt, 5);
    });
}
