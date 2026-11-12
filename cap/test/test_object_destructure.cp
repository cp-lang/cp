import { test, expect_eq } from "cap:test";

pub fn testObjDestructure(): i32 {
    const pair = .{ 42, true };
    const [num, flag] = pair;
    return if (flag) num else 0;
}

pub fn testNamedObjDestructure(): i32 {
    const pair = .{ .x = 100, .y = true };
    const [num, flag] = pair;
    return if (flag) num else 0;
}

pub fn testRestDestructure(): i32 {
    const triple = .{ 10, 20, 30 };
    const [first, ...rest] = triple;
    return first + i32(rest.len);
}

fn main(): void {
    test("anonymous struct tuple destructuring (positional and named)", () => {
        const res1 = testObjDestructure();
        expect_eq(res1, 42);

        const res2 = testNamedObjDestructure();
        expect_eq(res2, 100);

        const res3 = testRestDestructure();
        expect_eq(res3, 12);
    });
}
