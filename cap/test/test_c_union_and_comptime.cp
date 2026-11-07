import { print } from "cap:io";

// 1. 特性 A: @c union 与普通 union
@c union UnionAddr {
    addr4: u32,
    addr6: [u8; 16],
}

union ValueUnion {
    int_val: i32,
    byte_val: u8,
}

// 2. 特性 B: switch 范围端点放宽 (支持常量标识符、负数标识符、...)
const ARES_SUCCESS = 0;
const ARES_ENOSERVER = 10;

fn classify_ares_code(rc: i32): i32 {
    return switch (rc) {
        0 => 0,
        1...ARES_ENOSERVER => 100,
        -ARES_ENOSERVER...-1 => -100,
        * => 999,
    };
}

// 3. 特性 C: @if 条件类型别名与深层命名空间类型
pub type ares_socket_t = @if (true) i32 else i16;
pub type hostent_int = @if (false) i16 else i32;
pub type struct_sockaddr = std.posix.sockaddr;

// 4. 特性 D: 块级带值跳出表达式 (Labeled Block Expressions)
fn compute_with_labeled_block(): i32 {
    let host_ptr: i32 = brk: {
        let a = 10;
        let b = 20;
        break :brk a + b;
    };
    return host_ptr;
}

fn compute_with_expr_block(): i32 {
    let result: i32 = ({
        let base = 50;
        break base * 2;
    });
    return result;
}

fn main() {
    print("=== Testing CP New Features ===");

    // Test union memory size / field presence
    let u: UnionAddr = undefined;
    print("UnionAddr created successfully");

    // Test switch range patterns
    let code_zero = classify_ares_code(0);
    let code_pos = classify_ares_code(5);
    let code_neg = classify_ares_code(-3);
    let code_out = classify_ares_code(99);

    if (code_zero != 0) {
        print("FAIL: code_zero expected 0");
        return;
    }
    if (code_pos != 100) {
        print("FAIL: code_pos expected 100");
        return;
    }
    if (code_neg != -100) {
        print("FAIL: code_neg expected -100");
        return;
    }
    if (code_out != 999) {
        print("FAIL: code_out expected 999");
        return;
    }
    print("Switch range matches with identifiers and negative values passed!");

    // Test conditional type alias and deep namespace type
    let s: ares_socket_t = 42;
    let h: hostent_int = 1024;
    let addr: struct_sockaddr = undefined;
    let p_addr: &std.posix.sockaddr = &addr;
    _ = &p_addr;
    if (s != 42 || h != 1024) {
        print("FAIL: conditional type variables");
        return;
    }
    print("Conditional type alias and deep namespace type passed!");

    // Test block expressions
    let block_res1 = compute_with_labeled_block();
    let block_res2 = compute_with_expr_block();
    if (block_res1 != 30) {
        print("FAIL: block_res1 expected 30");
        return;
    }
    if (block_res2 != 100) {
        print("FAIL: block_res2 expected 100");
        return;
    }
    print("Labeled block and expression block returned values correctly!");

    print("ALL TESTS PASSED SUCCESSFULLY!");
}
