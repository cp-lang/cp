import { print } from "cap:io";

fn modify_buffer(*buf: [4]i32) {
    buf[0] = 42;
    buf[1] = 100;
}

fn test_valid_parentheses_and_borrows(): bool {
    let mut_arr: [4]i32 = [0, 0, 0, 0];
    modify_buffer(&mut_arr);
    if (mut_arr[0] != 42 || mut_arr[1] != 100) {
        return false;
    }

    let i = 0;
    let sum = 0;
    while (i < 2) {
        sum += mut_arr[i];
        i += 1;
    }
    if (sum != 142) {
        return false;
    }

    return true;
}

fn main() {
    print("Testing AI diagnostic and borrowing validations...");
    if (!test_valid_parentheses_and_borrows()) {
        print("TEST_FAILED");
        return;
    }
    print("ALL_AI_DIAGNOSTICS_COMPATIBILITY_TESTS_PASSED");
}
