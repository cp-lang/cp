import { test, expect_eq } from "cap:test";

fn calculate_offset(base: i32, multiplier: i32): i32 {
    let result = base * multiplier;
    return result + 10;
}

fn compute_nested(step: i32): i32 {
    let intermediate = calculate_offset(step, 3);
    return intermediate * 2;
}

fn main() {
    test("source map line resolution and nested calls", () => {
        let val = compute_nested(5);
        expect_eq(val, 50); // (5 * 3 + 10) * 2 = 50

        let val2 = calculate_offset(10, 2);
        expect_eq(val2, 30); // 10 * 2 + 10 = 30
    });
}

