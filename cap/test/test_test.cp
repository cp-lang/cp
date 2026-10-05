import { test, expect_eq } from "cap:test";

fn main(): void {
    test("Math Addition", () => {
        expect_eq(1 + 1, 2);
    });

    test("Math Subtraction", () => {
        expect_eq(10 - 5, 5);
    });
}
