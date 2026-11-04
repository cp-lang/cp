err MyError {
    BadValue,
}

@value struct NativeBridge {
    id: i32,

    zig {
        pub const Tag = u32;
        pub fn helper() u32 {
            return 42;
        }
    }

    fn getId(): i32 {
        return self.id;
    }
}

fn test_postfix_err(val: i32): usize! {
    if (val < 0) {
        return err.BadValue;
    }
    return 100;
}

fn test_empty_struct(): bool {
    let empty = {};
    let anon = { count: 123 };
    return anon.count == 123;
}

fn main() {
    let bridge = NativeBridge { id: 7 };
    let v = bridge.getId();

    let res_ok = test_postfix_err(10) catch 0;
    let res_err = test_postfix_err(-1) catch 999;

    let empty_ok = test_empty_struct();

    if (v == 7 && res_ok == 100 && res_err == 999 && empty_ok) {
        @print("All extra zig, dotted types, postfix T!, and error member tests passed!");
    } else {
        @print("Test failed: v={d}, res_ok={d}, res_err={d}", v, res_ok, res_err);
    }
}
