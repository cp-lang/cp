import { print } from "cap:io";
import { test, expect_eq } from "cap:test";

// 1. Structure Escapes: @align(N) on struct and fields
@align(16)
struct AlignedBlock {
    @align(16) val: usize,
    other: u32,
}

// 2. @c declarations: block and single function (with alias and body)
@c {
    fn getpid(): i32;
}

@c("abs") fn c_abs(n: i32): i32;

@c export fn cp_add_three(x: i32): i32 {
    return x + 3;
}

// 3. VTable with @c fn(...) : Ret function pointer types
struct VTable {
    on_data: (@c fn(ctx: *any!, code: i32): void)?,
    on_close: @c fn(code: i32): void,
}

@c fn dummy_close(code: i32): void {
    _ = code;
}

// 4. Const generic class and type aliases
class App<const ssl: bool> {
    is_ssl: bool = ssl;

    fn get_ssl(this): bool {
        return this.is_ssl;
    }
}

type AppTCP = App<false>;
type AppSSL = App<true>;

// 5. Enum definition for name, value, and conversion testing
enum TaskState {
    Idle = 0,
    Running = 1,
    Completed = 2,
}

class Handler {
    fn onOpen(this): void {}
}

fn main(): void {
    test("Structure Escapes: @align(N) on struct and fields", () => {
        let block: AlignedBlock = AlignedBlock{ val: 42, other: 7 };
        expect_eq(block.val, 42);
        expect_eq(block.other, 7);
    });

    test("@c extern declarations and export callconv(.c)", () => {
        let pid = getpid();
        expect_eq(pid > 0, true);
        let neg = c_abs(-123);
        expect_eq(neg, 123);
        let added = cp_add_three(10);
        expect_eq(added, 13);
    });

    test("VTable with @c fn(...) : Ret function pointer types", () => {
        let vt: VTable = VTable{
            on_data: null,
            on_close: dummy_close,
        };
        expect_eq(vt.on_data == null, true);
    });

    test("Const generic class and type alias", () => {
        let tcp: AppTCP = AppTCP{};
        let ssl_app: AppSSL = AppSSL{};
        expect_eq(tcp.get_ssl(), false);
        expect_eq(ssl_app.get_ssl(), true);
    });

    test("cstr primitive type and .len property", () => {
        let cs: cstr = "hello cp";
        expect_eq(cs.len, 8);
        let s = string.from(cs);
        expect_eq(s, "hello cp");
    });

    test("[*]T! many-item raw pointer indexing and slicing", () => {
        let arr: [4]u8 = [65, 66, 67, 68];
        let many: [*]u8! = arr as [*]u8!;
        expect_eq(many[0], 65);
        expect_eq(many[1], 66);
        let slice = many[1..3];
        expect_eq(slice.len, 2);
        expect_eq(slice[0], 66);
        expect_eq(slice[1], 67);
    });

    test("Compile-time reflection builtins", () => {
        expect_eq(reflect.hasMethod(Handler, "onOpen"), true);
        expect_eq(reflect.hasMethod(Handler, "onClose"), false);
        expect_eq(reflect.hasField(AlignedBlock, "val"), true);
        expect_eq(reflect.has(Handler, "onOpen"), true);
    });

    test("@if comptime ternary expression", () => {
        let branch = @if (reflect.hasMethod(Handler, "onOpen")) 100 else 200;
        expect_eq(branch, 100);
        let branch2 = @if (!reflect.hasMethod(Handler, "onOpen")) 100 else 200;
        expect_eq(branch2, 200);
    });

    test("Enum .name and .value properties", () => {
        let st = TaskState.Running;
        expect_eq(st.name, "Running");
        expect_eq(st.value, 1);
        expect_eq(TaskState.Idle.name, "Idle");
        expect_eq(TaskState.Completed.value, 2);
    });

    test("string.from polymorphic conversion", () => {
        let st = TaskState.Running;
        expect_eq(string.from(st), "Running");
        expect_eq(string.from(true), "true");
        expect_eq(string.from(false), "false");
        expect_eq(string.from(12345), "12345");
    });

    test("Enum conversion via constructor, cast, and fromName", () => {
        let st_from_int = TaskState(2);
        expect_eq(st_from_int == TaskState.Completed, true);
        let st_from_as = 1 as TaskState;
        expect_eq(st_from_as == TaskState.Running, true);
        let st_opt = TaskState.fromName("Idle");
        expect_eq(st_opt.? == TaskState.Idle, true);
    });
}
