// Regression test for Bun agy C-ABI pointers & AI-First features
import { print } from "cap:io";

fn assert(ok: bool, msg: string) {
    if (!ok) {
        print(msg);
        @panic(msg);
    }
}

fn test_cptr_basics() {
    let mut val: i32 = 42;
    let ptr: [*c]i32 = &mut val;
    assert(ptr.* == 42, "cptr dereference failed");

    // Optional cptr
    let opt_ptr: [*c]i32? = null;
    assert(opt_ptr == null, "opt_cptr should be null");

    // Coalesce with cptr
    let resolved = opt_ptr ?? ptr;
    assert(resolved.* == 42, "coalesce with cptr failed");
}

fn test_const_var_protection() {
    const IMMUTABLE_VAL: i32 = 100;
    assert(IMMUTABLE_VAL == 100, "const value check failed");
}

fn test_control_flow() {
    let mut count = 0;
    while (count < 5) {
        count = count + 1;
        if (count == 3) {
            continue;
        }
    }
    assert(count == 5, "while with parens failed");
}

export fn main() {
    test_cptr_basics();
    test_const_var_protection();
    test_control_flow();
    print("ALL_CPTR_AND_AI_MERGED_TESTS_PASSED");
}
