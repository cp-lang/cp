import { print } from "cap:io";
import SubModule from "./mock_sub_module.zig";

pub type DirInfo = SubModule.DirInfo;

// --- 1. 顶层命名空间常量与别名提取（摆脱 zig { ... }） ---
const std = @import("std");

pub struct MockSchema {
    pub const api: i32 = 77;
}

pub struct MockBun {
    pub const Environment: i32 = 1;
    pub const FD: i32 = 2;
    pub const Output: i32 = 42;
    pub const schema = MockSchema;
}

const Environment = MockBun.Environment;
const FD = MockBun.FD;
const Output = MockBun.Output;
const api = MockBun.schema.api;

pub type PathString = string;

// 顶层全局变量与常量
pub const APP_TAG: string = "BUN_COMPAT";
var global_counter: i32 = 100;

// --- 2. 上下文软关键字测试：验证 and/or 作为方法名时不被误伤 ---
pub struct LogicCalculator {
    pub value: i32,
    pub fn and(self: *LogicCalculator, mask: i32): i32 {
        return self.value & mask;
    }
    pub fn or(self: *LogicCalculator, mask: i32): i32 {
        return self.value | mask;
    }
}

fn main(): void {
    // 验证透明导入外部 .zig 模块及顶层属性别名
    let dir_info: DirInfo = SubModule.get_mock_dir();
    if (dir_info.id != 88 || dir_info.name != "mock_dir") {
        print("FAIL: default import from .zig failed\n");
        return;
    }

    // 验证顶层别名与深层级联常量读取
    if (Environment != 1 || FD != 2 || Output != 42 || api != 77) {
        print("FAIL: top-level constants mismatch\n");
        return;
    }

    // 验证顶层可变变量读取与修改
    global_counter += 5;
    if (global_counter != 105) {
        print("FAIL: top-level var failed\n");
        return;
    }

    // 验证 soft keywords：and 与 or 作为二元逻辑表达式
    let cond1 = true;
    let cond2 = true;
    let cond3 = false;
    if (!(cond1 and cond2)) {
        print("FAIL: soft keyword 'and' failed\n");
        return;
    }
    if (!(cond3 or cond1)) {
        print("FAIL: soft keyword 'or' failed\n");
        return;
    }

    // 验证 orelse 软关键字空值兜底
    let opt_val: i32? = null;
    let fallback_val = opt_val orelse 999;
    if (fallback_val != 999) {
        print("FAIL: soft keyword 'orelse' failed\n");
        return;
    }

    // 验证 and / or 作为成员方法调用（未被词法硬替换破坏）
    let calc = LogicCalculator { value: 0x0F };
    let and_res = calc.and(0x07);
    let or_res = calc.or(0x30);
    if (and_res != 0x07 || or_res != 0x3F) {
        print("FAIL: method 'and'/'or' conflict failed\n");
        return;
    }

    // 验证带哨兵的切片（Sentinel Slice: buf[0..3 :0]）
    let buf: [8]u8 = [65, 66, 67, 0, 0, 0, 0, 0];
    let slice_sentinel = buf[0..3 :0];
    if (slice_sentinel.len != 3 || slice_sentinel[0] != 65) {
        print("FAIL: sentinel slice failed\n");
        return;
    }

    print("TEST PASSED: Bun top-level imports, sentinel slices and soft keywords fully functional!\n");
}
