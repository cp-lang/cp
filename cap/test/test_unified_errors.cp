import { print } from "cap:io";

// Comprehensive unified errors test suite

// 1. Dedicated error set (returns directly without err())
err FileErr {
    NotFound,
    AccessDenied,
}

fn open_file(flag: i32): i32 ! FileErr {
    if (flag == 1) {
        return FileErr.NotFound;
    }
    if (flag == 2) {
        return .AccessDenied;
    }
    return 100;
}

// 2. Normal enum error (requires err() wrapping)
enum HttpError {
    BadRequest,
    ServerError,
}

fn handle_http(code: i32): i32 ! HttpError {
    if (code == 400) {
        return err(HttpError.BadRequest);
    }
    if (code == 500) {
        return err(.ServerError);
    }
    return 200;
}

// 3. Dynamic business error with Err and Error alias
fn fetch_user(id: i32): string ! Err {
    if (id == 404) {
        return err(404, "User Not Found");
    }
    if (id == 500) {
        return err("Database Timeout");
    }
    return "Alice";
}

fn check_system(status: i32): i32 ! Error {
    if (status < 0) {
        return err(status, "System Failure");
    }
    return 1;
}

// 4. AnyErr shorthand (T!)
fn test_anyerr(code: i32): i32! {
    if (code == 0) {
        return err.NotFound;
    }
    return 42;
}

// 5. Dynamic error with Trait and downcast
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

fn fail_with_box(status: i32): i32 ! Box<dyn CustomErr> {
    if (status != 0) {
        let boxed: Box<dyn CustomErr> = Box.new(new NetError(status));
        return err(boxed);
    }
    return 123;
}

fn test_bubble_file(): i32 ! Err {
    let f = open_file(1) ?;
    return f;
}

fn test_bubble_http(): i32 ! Err {
    let h = handle_http(400) ?;
    return h;
}

pub fn main(): void {
    // Test 1: FileErr matching
    let res1 = open_file(1);
    let msg1 = switch res1 {
        FileErr.NotFound => "file_not_found",
        FileErr.AccessDenied => "access_denied",
        val => "file_ok",
    };
    print(msg1);

    let res2 = open_file(2);
    let msg2 = switch res2 {
        .NotFound => "file_not_found",
        .AccessDenied => "access_denied",
        val => "file_ok",
    };
    print(msg2);

    // Test 2: HttpError enum matching
    let h1 = handle_http(400);
    let h_msg1 = switch h1 {
        err(HttpError.BadRequest) => "bad_request",
        err(HttpError.ServerError) => "server_err",
        ok_code => "http_ok",
    };
    print(h_msg1);

    let h2 = handle_http(500);
    let h_msg2 = switch h2 {
        err(.BadRequest) => "bad_request",
        err(.ServerError) => "server_err",
        ok_code => "http_ok",
    };
    print(h_msg2);

    // Test 3: Dynamic Err matching
    let u1 = fetch_user(404);
    let u_msg1 = switch u1 {
        err(404, "User Not Found") => "matched_404_msg",
        err(e) => "other_err",
        name => name,
    };
    print(u_msg1);

    let u2 = fetch_user(500);
    let u_msg2 = switch u2 {
        err("Database Timeout") => "matched_timeout_msg",
        err(e) => "other_err",
        name => name,
    };
    print(u_msg2);

    // Test 4: AnyErr switch matching
    let a1 = test_anyerr(0);
    let a_msg1 = switch a1 {
        err.NotFound => "anyerr_not_found",
        val => "anyerr_ok",
    };
    print(a_msg1);

    let a2 = test_anyerr(1);
    let a_msg2 = switch a2 {
        err.NotFound => "anyerr_not_found",
        val => "anyerr_ok",
    };
    print(a_msg2);

    // Test 5: Dynamic error downcast matching
    let b_res = fail_with_box(503);
    let b_msg = switch b_res {
        err(e: NetError) => e.status,
        val => val,
    };
    print(b_msg);

    // Test 6: Error bubbling and auto-conversion to Err
    let bf = test_bubble_file();
    let bf_msg = switch bf {
        err(e) => e.msg,
        val => "ok",
    };
    print(bf_msg);

    let bh = test_bubble_http();
    let bh_msg = switch bh {
        err(e) => e.msg,
        val => "ok",
    };
    print(bh_msg);

    print("ALL_UNIFIED_ERROR_TESTS_PASSED");
}
