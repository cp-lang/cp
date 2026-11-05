import { print } from "cap:io";

err FileErr {
    NotFound,
    PermissionDenied
}

err NetErr {
    Timeout,
    ConnectionReset
}

// 1. Named + Anonymous union: FileErr || err{Timeout, Canceled}
fn test_anon_union(flag: i32): string ! FileErr || err{Timeout, Canceled} {
    if flag == 1 {
        return FileErr.NotFound;
    }
    if flag == 2 {
        return err.Timeout;
    }
    return "ok_anon";
}

// 2. Named + Dynamic Err union: FileErr || Err
fn test_err_union(flag: i32): string ! FileErr || Err {
    if flag == 1 {
        return FileErr.NotFound;
    }
    if flag == 2 {
        return err(500, "internal server failure");
    }
    return "ok_err";
}

// 3. Walrus downcast vs Colon downcast test
trait CustomErr {
    fn code(): i32;
}

class NetError implements CustomErr {
    pub status: i32;
    pub fn init(s: i32) {
        this.status = s;
    }
    fn code(): i32 {
        return this.status;
    }
}

fn fail_with_box(s: i32): i32 ! Err {
    let ne = new NetError(s);
    let b: Box<dyn CustomErr> = Box.new(ne);
    let e = Err.init(s, "failed").with_cause(move b);
    return err(e);
}

pub fn main(): void {
    // 1. Test Anon Union pattern match
    let r1 = test_anon_union(2);
    let out1 = switch r1 {
        .Timeout => "matched_timeout",
        err(e) => "other_err",
        val => val,
    };
    print(out1);

    // 2. Test Named + Err Union with err(code, msg) and .NotFound
    let r2 = test_err_union(2);
    let out2 = switch r2 {
        err(500, msg) => msg,
        .NotFound => "not_found",
        err(e) => "other_err",
        val => val,
    };
    print(out2);

    let r2_named = test_err_union(1);
    let out2_named = switch r2_named {
        err(500, msg) => msg,
        .NotFound => "not_found_named",
        err(e) => "other_err",
        val => val,
    };
    print(out2_named);

    // 3. Test Colon downcast: err(e: NetError)
    let b_res1 = fail_with_box(503);
    let status1 = switch b_res1 {
        err(e: NetError) => e.status,
        val => val,
    };
    print(status1);

    // 4. Test Walrus downcast: err(e := NetError)
    let b_res2 = fail_with_box(504);
    let status2 = switch b_res2 {
        err(e := NetError) => e.status,
        val => val,
    };
    print(status2);

    print("ALL_TESTS_SUCCESS");
}
