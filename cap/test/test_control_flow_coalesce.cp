import { print } from "cap:io";

fn get_opt(val: i32): i32? {
    if (val > 0) {
        return val;
    }
    return null;
}

fn get_err(val: i32): i32!Error {
    if (val > 0) {
        return val;
    }
    return Err("NegativeOrZero");
}

fn is_positive(val: i32): bool {
    return val > 0;
}

fn test_opt_coalesce(): i32 {
    let mut sum: i32 = 0;
    let mut i: i32 = -2;

    // 1. ?? continue
    while (i <= 3) {
        let cur = i;
        i += 1;
        let v = get_opt(cur) ?? continue;
        sum += v;
    }
    // 应该累加 1 + 2 + 3 = 6
    if (sum != 6) {
        return 1;
    }

    // 2. ?? break
    let mut count: i32 = 0;
    let mut j: i32 = 1;
    while (j < 10) {
        let mut cur: i32 = -1;
        if (j < 4) {
            cur = j;
        }
        j += 1;
        let v = get_opt(cur) ?? break;
        count += v;
    }
    // 应该累加 1 + 2 + 3 = 6
    if (count != 6) {
        return 2;
    }

    return 0;
}

fn test_err_coalesce(): i32 {
    let mut sum: i32 = 0;
    let mut i: i32 = -2;

    // 1. ? continue
    while (i <= 3) {
        let cur = i;
        i += 1;
        let v = get_err(cur) ? continue;
        sum += v;
    }
    // 应该累加 1 + 2 + 3 = 6
    if (sum != 6) {
        return 3;
    }

    // 2. ? break
    let mut count: i32 = 0;
    let mut j: i32 = 1;
    while (j < 10) {
        let mut cur: i32 = -1;
        if (j < 4) {
            cur = j;
        }
        j += 1;
        let v = get_err(cur) ? break;
        count += v;
    }
    // 应该累加 1 + 2 + 3 = 6
    if (count != 6) {
        return 4;
    }

    return 0;
}

fn test_logical_or_guards(): i32 {
    let mut sum: i32 = 0;
    let mut i: i32 = -2;

    // 1. || continue
    while (i <= 3) {
        let cur = i;
        i += 1;
        is_positive(cur) || continue;
        sum += cur;
    }
    if (sum != 6) {
        return 5;
    }

    // 2. || break
    let mut count: i32 = 0;
    let mut j: i32 = 1;
    while (j < 10) {
        let cur = j;
        j += 1;
        (cur < 4) || break;
        count += cur;
    }
    if (count != 6) {
        return 6;
    }

    return 0;
}

fn test_early_returns(x: i32): i32 {
    // 1. || return
    (x != -999) || return -999;

    // 2. ?? return
    let v1 = get_opt(x) ?? return -10;

    // 3. ? return
    let v2 = get_err(x) ? return -20;

    return v1 + v2;
}

fn main() {
    let r1 = test_opt_coalesce();
    if (r1 != 0) {
        print("test_opt_coalesce failed\n");
        return;
    }

    let r2 = test_err_coalesce();
    if (r2 != 0) {
        print("test_err_coalesce failed\n");
        return;
    }

    let r3 = test_logical_or_guards();
    if (r3 != 0) {
        print("test_logical_or_guards failed\n");
        return;
    }

    let ret_sentinel = test_early_returns(-999);
    if (ret_sentinel != -999) {
        print("test_early_returns sentinel failed\n");
        return;
    }

    let ret_neg = test_early_returns(-1);
    if (ret_neg != -10) {
        print("test_early_returns negative failed\n");
        return;
    }

    let ret_pos = test_early_returns(5);
    if (ret_pos != 10) {
        print("test_early_returns positive failed\n");
        return;
    }

    print("ALL_CONTROL_FLOW_COALESCE_TESTS_PASSED\n");
}
