err TestError {
    Failed,
}

fn find_value(val: i32): string? {
    if (val > 0) {
        return "positive";
    }
    return null;
}

struct SimpleIter {
    current: i32,
    max: i32,

    fn next(*_): i32? {
        if (_.current < _.max) {
            let res = _.current;
            _.current = _.current + 1;
            return res;
        }
        return null;
    }
}

fn test_shorthand_bang(fail: bool): i32! {
    if (fail) {
        return err.Failed;
    }
    return 100;
}

fn main() {
    // 1. Test walrus := in if statement
    let mut if_ok = false;
    if (found := find_value(42)) {
        if (found == "positive") {
            if_ok = true;
        }
    }

    let mut else_ok = false;
    if (not_found := find_value(-5)) {
        if_ok = false;
    } else {
        else_ok = true;
    }

    // 2. Test walrus := in while loop
    let mut iter = SimpleIter { current: 0, max: 3 };
    let mut sum: i32 = 0;
    while (item := iter.next()) {
        sum += item;
    }

    // 3. Test shorthand bang error union return
    let bang_ok = (test_shorthand_bang(false) catch 0) == 100;
    let bang_err = (test_shorthand_bang(true) catch 999) == 999;

    if (if_ok && else_ok && sum == 3 && bang_ok && bang_err) {
        @print("All walrus binding and shorthand bang tests passed successfully!");
    } else {
        @print("Test failed: if_ok={}, else_ok={}, sum={d}, bang_ok={}, bang_err={}", if_ok, else_ok, sum, bang_ok, bang_err);
    }
}
