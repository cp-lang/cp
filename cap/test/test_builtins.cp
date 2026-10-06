// con/test_builtins.cp

fn main(): ?void {
    // Test @print builtin
    @print("Hello from CP Builtins! val={d}", 42);

    // Test @map builtin (High-performance inlining)
    let arr: i32[] = undefined;
    __zig__ { ctx.arr = &[_]i32{ 1, 2, 3, 4, 5 }; }
    let doubled: i32[] = @map(arr, x => x * 2);

    __zig__ {
        std.debug.print("Doubled array: ", .{});
        for (ctx.doubled) |v| {
            std.debug.print("{d} ", .{v});
        }
        std.debug.print("\n", .{});
    }
}
