import { print } from "cap:io";

const MyFunc = fn (x: u8) bool;

fn check_even(x: u8): bool {
    return (x % 2) == 0;
}

pub fn testWhileStep(): i32 {
    var i: usize = 0;
    var count: i32 = 0;
    while (i < 10) : (i += 1) {
        if (i == 5) {
            break;
        }
        count += 1;
    }
    return count;
}

fn main(): void {
    let f: MyFunc = check_even;
    let is_even = f(4);
    if (!is_even) {
        @panic("expected f(4) to be true");
    }

    let steps = testWhileStep();
    if (steps != 5) {
        @panic("expected testWhileStep() to return 5");
    }

    print("zig syntax compat test passed!\n");
}
