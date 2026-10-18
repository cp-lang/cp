import { print } from "cap:io";

// ============================================================================
// 1. Named Error Enum, Error Union (E1 | E2), and Declarative `from` Conversion
// ============================================================================
enum FileErr {
    FileNotFound,
    PermDenied,
    ReadEOF
}

enum NetErr {
    Timeout,
    ConnReset
}

enum AppErr {
    Io(FileErr) from FileErr,
    Net(NetErr) from NetErr,
    InvalidConfig
}

// ============================================================================
// 2. Tri-State Return (`?string ! FileErr`), Direct Value, `null`, `Err(...)`,
//    and Postfix `?` Bubbling
// ============================================================================
fn readFile(path: string): ?string ! FileErr {
    if (path == "") {
        return Err(FileErr.FileNotFound);
    }
    if (path == "secret.txt") {
        // Contextual shorthand `.Variant` inside Err(...)
        return Err(.PermDenied);
    }
    if (path == "empty.txt") {
        return null;
    }
    return "hello_cp";
}

fn load(path: string): ?string ! FileErr {
    let txt = readFile(path)?;
    return "prefix:" + txt;
}

// ============================================================================
// 3. Anonymous Error Set `!{ EmptyInput, Overflow }` & Type Inference
// ============================================================================
fn parsePositive(n: i32): i32 ! { EmptyInput, Overflow } {
    if (n == 0) {
        return Err(.EmptyInput);
    }
    if (n > 1000) {
        return Err(.Overflow);
    }
    return n * 2;
}

// ============================================================================
// 4. Declarative `from` Error Conversion & `try { ... } catch err => { ... }`
// ============================================================================
fn fetchRemote(host: string): string ! NetErr {
    if (host == "bad") {
        return Err(NetErr.Timeout);
    }
    return "remote_ok";
}

fn runAppTask(path: string, host: string): string ! AppErr {
    // `readFile(path)?` bubbles FileErr -> automatically converted to AppErr.Io(FileErr) via `from FileErr`!
    let local_txt = readFile(path)? ?? "empty_fallback";
    // `fetchRemote(host)?` bubbles NetErr -> automatically converted to AppErr.Net(NetErr) via `from NetErr`!
    let remote_txt = fetchRemote(host)?;
    return local_txt + "+" + remote_txt;
}

// Auto-bubbling `try { ... } catch err => { ... }` region (omitting trailing `?` inside `try`)
fn runTryRegion(path: string): string {
    let result = "init";
    try {
        // Inside `try { ... }`, fallible calls automatically bubble without trailing `?`!
        let a = load(path);
        let unwrapped = a ?? "was_null";
        result = "try_ok:" + unwrapped;
    } catch err => {
        result = switch err {
            FileErr.FileNotFound => "caught:not_found",
            FileErr.PermDenied => "caught:perm_denied",
            FileErr.ReadEOF => "caught:eof"
        };
    }
    return result;
}

// Built-in Unified `Error` struct (`!Error` or `!: Error`)
fn runUnifiedError(mode: i32): i32 ! Error {
    if (mode == 1) {
        // Bubbling domain error FileErr into unified `Error` automatically converts!
        let s = readFile("")?;
        return 1;
    }
    if (mode == 2) {
        return Err(Error { code: 503, msg: "service_unavailable" });
    }
    return 200;
}

// ============================================================================
// 5. Optional Chaining (`?.`), `? default`, `? return default`, and `??`
// ============================================================================
class CityInfo {
    zip: i32;
    fn init(z: i32) {
        _.zip = z;
    }
    fn getZip(): i32 {
        return _.zip;
    }
}

class Address {
    city: string;
    info: ?CityInfo;
    fn init(c: string, move inf: ?CityInfo) {
        _.city = c;
        _.info = inf;
    }
}

class UserProfile {
    name: string;
    addr: ?Address;
    fn init(n: string, move a: ?Address) {
        _.name = n;
        _.addr = a;
    }
}

fn testFallbackReturn(path: string): string {
    // `? return default` catches both `Err` and `null` and returns early from function
    let content = readFile(path) ? return "early_return_default";
    return "got:" + content;
}

// ============================================================================
// 6. Built-in `Box<dyn Trait>` Fat Pointer, VTable Dispatch, `downcast<T>()`,
//    and Automatic Scope Drop (`release()`)
// ============================================================================
trait PluginError {
    fn code(): i32;
    fn summary(): string;
}

class DbTimeoutError implements PluginError {
    retry_After_ms: i32;
    host_tag: string;

    fn init(ms: i32, tag: string) {
        _.retry_After_ms = ms;
        _.host_tag = tag;
    }
    fn code(): i32 {
        return 504;
    }
    fn summary(): string {
        return "DbTimeout:" + _.host_tag;
    }
    fn release() {
        @print("DbTimeoutError released cleanly (retry_ms={d})", _.retry_After_ms);
    }
}

class AuthFailedError implements PluginError {
    user_id: i32;

    fn init(uid: i32) {
        _.user_id = uid;
    }
    fn code(): i32 {
        return 401;
    }
    fn summary(): string {
        return "AuthFailed";
    }
    fn release() {
        @print("AuthFailedError released cleanly (uid={d})", _.user_id);
    }
}

fn invokePlugin(kind: i32): i32 ! Box<dyn PluginError> {
    if (kind == 1) {
        let db_err = new DbTimeoutError(1500, "db-primary");
        return Err(Box.new(db_err));
    }
    if (kind == 2) {
        let auth_err = new AuthFailedError(99);
        return Err(Box.new(auth_err));
    }
    return 42;
}

fn main() {
    // --- A. Tri-state switch pattern matching on `?string ! FileErr` ---
    let r1 = load("data.txt");
    let out1 = switch r1 {
        content => "ok:" + content,
        null => "empty",
        Err(FileErr.FileNotFound) => "err:not_found",
        Err(FileErr.PermDenied) => "err:perm_denied",
        Err(e) => "err:other"
    };
    @print("1. load(data.txt) => {s}", out1);

    let r2 = load("empty.txt");
    let out2 = switch r2 {
        content => "ok:" + content,
        null => "empty",
        Err(e) => "err"
    };
    @print("2. load(empty.txt) => {s}", out2);

    let r3 = load("");
    let out3 = switch r3 {
        content => "ok:" + content,
        null => "empty",
        Err(FileErr.FileNotFound) => "err:not_found",
        Err(*) => "err:wildcard"
    };
    @print("3. load('') => {s}", out3);

    // --- B. `? default`, `? return default`, and `??` ---
    let fb1 = readFile("") ? "fallback_on_err";
    let fb2 = readFile("empty.txt") ? "fallback_on_null";
    let fb3 = readFile("data.txt") ? "fallback_unused";
    @print("4. ? default => {s} | {s} | {s}", fb1, fb2, fb3);

    let early1 = testFallbackReturn("");
    let early2 = testFallbackReturn("empty.txt");
    let early3 = testFallbackReturn("data.txt");
    @print("5. ? return default => {s} | {s} | {s}", early1, early2, early3);

    // Inline `catch` + pure `??` null coalescing
    let c1 = (readFile("secret.txt") catch err => "caught_secret") ?? "null_coalesced";
    let c2 = (readFile("empty.txt") catch err => "caught_err") ?? "null_coalesced";
    @print("6. catch + ?? => {s} | {s}", c1, c2);

    // --- C. Anonymous Error Set, Declarative `from`, and `try { ... } catch` ---
    let anon_explicit: i32 ! { EmptyInput, Overflow } = parsePositive(21);
    let anon_inferred = parsePositive(0);
    let anon_msg = switch anon_inferred {
        v => v,
        Err(.EmptyInput) => -1,
        Err(.Overflow) => -2
    };
    @print("7. anon error set => ok={d}, err={d}", anon_explicit ? 0, anon_msg);

    let app_ok = runAppTask("data.txt", "good") ? "app_failed";
    let app_err_res = runAppTask("", "good");
    let app_err_tag = switch app_err_res {
        val => "ok:" + val,
        Err(AppErr.Io(fe)) => "app_io_converted",
        Err(AppErr.Net(ne)) => "app_net_converted",
        Err(AppErr.InvalidConfig) => "app_invalid_cfg"
    };
    @print("8. declarative from => {s} | {s}", app_ok, app_err_tag);

    let tr1 = runTryRegion("data.txt");
    let tr2 = runTryRegion("secret.txt");
    @print("9. try/catch region => {s} | {s}", tr1, tr2);

    let u_err = runUnifiedError(2);
    let u_code = switch u_err {
        v => v,
        Err(e) => e.code
    };
    @print("10. unified Error struct => code={d}", u_code);

    // --- D. Optional Chaining `?.` ---
    let info = new CityInfo(94105);
    let addr = new Address("SF", info);
    let user_with_addr: ?UserProfile = new UserProfile("Alice", addr);
    let user_no_addr: ?UserProfile = new UserProfile("Bob", null);
    let null_user: ?UserProfile = null;

    let city1 = user_with_addr?.addr?.city ?? "unknown_city";
    let zip1 = user_with_addr?.addr?.info?.getZip() ?? -1;
    let city2 = user_no_addr?.addr?.city ?? "unknown_city";
    let city3 = null_user?.addr?.city ?? "unknown_city";
    @print("11. optional chaining => city1={s}, zip1={d}, city2={s}, city3={s}", city1, zip1, city2, city3);

    // --- E. `Box<dyn Trait>` Fat Pointer & `downcast<T>()` ---
    let p_res = invokePlugin(1);
    switch p_res {
        v => {
            @print("Plugin succeeded: {d}", v);
        },
        Err(box_err) => {
            @print("12. Box<dyn PluginError> vtable: code={d}, summary={s}", box_err.code(), box_err.summary());
            // Runtime O(1) downcast<DbTimeoutError> returning `?&DbTimeoutError`
            let db_ref = box_err.downcast<DbTimeoutError>();
            let retry_ms = db_ref?.retry_After_ms ?? -1;
            let wrong_ref = box_err.downcast<AuthFailedError>();
            let wrong_uid = wrong_ref?.user_id ?? -999;
            @print("13. Box downcast<T>: retry_ms={d}, wrong_downcast={d}", retry_ms, wrong_uid);
        }
    }

    // --- F. `switch` Guards (`if`), Ranges (`..=`), and OR (`|`) Patterns ---
    let score = 85;
    let grade = switch score {
        90..=100 => "A",
        80..=89 if score >= 85 => "B+",
        80..=89 => "B",
        60 | 70 => "Pass",
        * => "F"
    };
    @print("14. switch range & guard => grade={s}", grade);
}
