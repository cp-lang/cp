import { expect_eq } from "cap:test";

fn add(a: i32, b: i32): i32 {
    return a + b;
}

fn get_comptime_val(@comptime val: usize): usize {
    return val;
}

fn get_type_size(val: auto): usize {
    return @sizeOf(@typeOf(val));
}

fn main(): ?void {
    @print("--- CP Comptime System Test ---");

    // 1. @{ } 编译期代码执行块
    const computed_val = @{
        let a = 15;
        let b = 25;
        a + b
    };
    expect_eq(computed_val, 40);

    // 2. @fn() 编译期函数调用
    const sum_res = @add(30, 70);
    expect_eq(sum_res, 100);

    // 3. @comptime 参数约束
    const const_len = get_comptime_val(1024);
    expect_eq(const_len, 1024);

    // 4. val: auto 编译期自动推导形参
    let num_i32: i32 = 42;
    let int_size = get_type_size(num_i32);
    expect_eq(int_size, 4);

    let num_f64: f64 = 3.14;
    let float_size = get_type_size(num_f64);
    expect_eq(float_size, 8);

    // 5. @if / @else 编译期分支修剪
    let branch_val = 0;
    @if true {
        branch_val = 1;
    } @else {
        branch_val = 2;
    }
    expect_eq(branch_val, 1);

    // 6. @for 编译期循环展开
    let total = 0;
    @for i of 0..5 {
        total = total + i;
    }
    expect_eq(total, 10);

    @print("All CP Comptime tests passed successfully!");
}