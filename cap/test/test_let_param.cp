import { print } from "cap:io";

struct Counter {
    val: i32,
}

// 1. 测试值参数可变本地拷贝 (fn foo(let x: i32))
fn add_locally(let x: i32, delta: i32): i32 {
    x += delta;
    x *= 2;
    return x;
}

// 2. 测试结构体值参数可变本地拷贝 (不影响外部调用方)
fn bump_struct_copy(let c: Counter): i32 {
    c.val += 100;
    return c.val;
}

// 3. 测试默认值参 (不可变) 与可变值参组合
fn sum_range(let start: i32, end: i32): i32 {
    let total: i32 = 0;
    while (start <= end) {
        total += start;
        start += 1;
    }
    return total;
}

export fn main() {
    let orig_x: i32 = 5;
    let res_x = add_locally(orig_x, 3);
    if (res_x != 16 || orig_x != 5) {
        @panic("let value param scalar failed");
    }

    let c = Counter{ val: 42 };
    let res_c = bump_struct_copy(c);
    if (res_c != 142 || c.val != 42) {
        @panic("let value param struct failed");
    }

    let total = sum_range(1, 5);
    if (total != 15) {
        @panic("let value param loop failed");
    }

    print("ALL_LET_PARAM_TESTS_PASSED");
}
