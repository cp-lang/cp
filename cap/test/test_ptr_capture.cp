import { print } from "cap:io";

// 测试 CP 指针捕获与海象解包特性: if (*val := opt)
fn main(): void {
    let mut_val: i32? = 100;

    // 1. 测试 if (*val := mut_val) 指针捕获与就地修改
    if (*val := mut_val) {
        val = 200; // 通过可变指针修改原值
    }

    if (v := mut_val) {
        print(v);
    }

    // 2. 测试再次海象指针捕获就地修改
    if (*val := mut_val) {
        val = 300;
    }

    if (v := mut_val) {
        print(v);
    }

    // 3. 测试赋值忽略 _ = ...
    _ = mut_val;
    print("all ptr capture tests passed!");
}
