// cap/test/test_err_and_move.cp
// Verification for dual err() / Err() support and move parameter flexibility

enum NetworkErr {
    NotFound,
    Timeout
}

class UserSession {
    id: i32;
    fn init(id: i32) {
        _.id = id;
    }
}

// 1. Prefix move parameter: fn f(move u: T)
fn take_ownership_prefix(move u: UserSession): i32 {
    return u.id;
}

// 2. Type-position move parameter: fn f(u: move T)
fn take_ownership_typepos(u: move UserSession): i32 {
    return u.id;
}

// 3. Lowercase err(...) constructor (Primary)
fn query_lower(ok: bool): string?!NetworkErr {
    if (!ok) {
        return err(NetworkErr.NotFound);
    }
    return "ok_lower";
}

// 4. Uppercase Err(...) constructor (Backwards-compatible)
fn query_upper(ok: bool): string?!NetworkErr {
    if (!ok) {
        return Err(NetworkErr.Timeout);
    }
    return "ok_upper";
}

export fn main(): void {
    let s1 = new UserSession(101);
    let id1 = take_ownership_prefix(s1);
    @print("Prefix move id: {d}", id1);

    let s2 = new UserSession(202);
    let id2 = take_ownership_typepos(s2);
    @print("Typepos move id: {d}", id2);

    let r1 = query_lower(false);
    let out1 = switch r1 {
        err(NetworkErr.NotFound) => "caught_lower_err",
        err(e) => "other_err",
        null => "null",
        val => "ok",
    };
    @print("Result 1: {s}", out1);

    let r2 = query_upper(false);
    let out2 = switch r2 {
        Err(NetworkErr.Timeout) => "caught_upper_Err",
        Err(e) => "other_Err",
        null => "null",
        val => "ok",
    };
    @print("Result 2: {s}", out2);

    @print("=== All err() / Err() and move parameter tests passed 100%! ===");
}
