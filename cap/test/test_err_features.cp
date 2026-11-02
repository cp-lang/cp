err CalcError {
    InvalidPadding,
    Overflow
}

fn test_err_keyword(val: i32): !i32 {
    if (val < 0) {
        return err.InvalidPadding;
    }
    return val * 2;
}

fn test_128_bit(): u128 {
    let a: u128 = 1000;
    let b: i128 = -500;
    return a + 42;
}

fn test_else_switch(val: i32): string {
    return switch (val) {
        1 => "one",
        2 => "two",
        else => "other",
    };
}

fn main() {
    let res = test_err_keyword(10) catch 0;
    let err_res = test_err_keyword(-1) catch 999;
    let big = test_128_bit();
    let msg = test_else_switch(3);

    if (res == 20 && err_res == 999 && big == 1042 && msg == "other") {
        @print("All err and 128-bit features passed successfully!");
    } else {
        @print("Test failed: res={d}, err_res={d}, big={d}, msg={s}", res, err_res, big, msg);
    }
}
