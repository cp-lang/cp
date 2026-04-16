// con/test_builtins.cp

fn main(): ?void {
    // Test @print builtin
    @print("Hello from CP Builtins! val={d}", 42);

    // Test @map builtin (High-performance inlining)
    let arr = [1, 2, 3, 4, 5];
    let doubled = @map(arr, x => x * 2);

    __zig__ {
        std.debug.print("Doubled array: ", .{});
        for (ctx.doubled) |v| {
            std.debug.print("{d} ", .{v});
        }
        std.debug.print("\n", .{});
    }
}
