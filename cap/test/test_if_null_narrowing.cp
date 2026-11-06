import { test, expect_eq } from "cap:test";
import { print } from "cap:io";

class User {
    pub name: string;
    pub age: i32;

    pub fn init(name: string, age: i32) {
        this.name = name;
        this.age = age;
    }

    pub fn get_age(): i32 {
        return this.age;
    }

    pub fn have_birthday(mut _) {
        this.age += 1;
    }
}

fn takes_i32(val: i32): i32 {
    return val * 2;
}

fn main(): void {
    test("object flow-sensitive narrowing in if (u != null)", () => {
        let u: User? = new User("Alice", 25);
        if (u != null) {
            // 1. Direct field access without unwrap
            expect_eq(u.name, "Alice");
            // 2. Direct method calls without unwrap
            expect_eq(u.get_age(), 25);
            u.have_birthday();
            expect_eq(u.age, 26);
            // 3. Member assignment
            u.age = 30;
            expect_eq(u.age, 30);
            // 4. Manual unwrap compatibility
            let unwrapped = u.?;
            expect_eq(unwrapped.get_age(), 30);
        } else {
            expect_eq(1, 0); // Should not reach
        }
    });

    test("primitive flow-sensitive narrowing in if (x != null)", () => {
        let mut count: i32? = 15;
        if (count != null) {
            // 1. Direct arithmetic
            let sum = count + 5;
            expect_eq(sum, 20);
            // 2. Passing to non-optional param
            let doubled = takes_i32(count);
            expect_eq(doubled, 30);
            // 3. Reassignment
            count = 100;
            expect_eq(count, 100);
        } else {
            expect_eq(1, 0); // Should not reach
        }
    });

    test("short-circuit && narrowing and inverted null != u", () => {
        let u: User? = new User("Bob", 22);
        let mut reached_and = false;
        if (u != null && u.age > 20) {
            expect_eq(u.name, "Bob");
            reached_and = true;
        }
        expect_eq(reached_and, true);

        let mut reached_inv = false;
        if (null != u) {
            expect_eq(u.age, 22);
            reached_inv = true;
        }
        expect_eq(reached_inv, true);
    });

    test("else-branch narrowing in if (u == null) ... else", () => {
        let u: User? = new User("Charlie", 40);
        let mut reached_else = false;
        if (u == null) {
            expect_eq(1, 0);
        } else {
            u.have_birthday();
            expect_eq(u.age, 41);
            reached_else = true;
        }
        expect_eq(reached_else, true);
    });

    test("nested if narrowing", () => {
        let u: User? = new User("Dave", 18);
        let mut age_val = 0;
        if (u != null) {
            if (u.age >= 18) {
                age_val = u.age;
            }
        }
        expect_eq(age_val, 18);
    });

    print("ALL IF-NULL NARROWING TESTS PASSED!");
}
