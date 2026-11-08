import { print } from "cap:io";

// 测试 Bun AGY 指针捕获特性: if (*val := opt) 与 while (expr) |*val|
fn main(): void {
    let mut_val: i32? = 100;

    // 1. 测试 if (*val := mut_val) 指针捕获与就地修改
    if (*val := mut_val) {
        val = 200; // 通过可变指针修改原值
    }

    if (v := mut_val) {
        print(v);
    }

    // 2. 测试 if 管道风格指针捕获: if (mut_val) |*val|
    if (mut_val) |*val| {
        val = 300;
    }

    if (v := mut_val) {
        print(v);
    }

    // 3. 测试赋值忽略 _ = ...
    _ = mut_val;
    print("all ptr capture tests passed!");
}
