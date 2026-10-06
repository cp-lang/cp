fn main(): ?void {
    let msg = "Hello CP";
    let len: i32 = 0;
    __zig__ { ctx.len = @intCast(ctx.msg.len); }
    let [a, b] = [100, 200];
    @print("SUCCESS: Str={s} Len={d} a={d} b={d}", msg, len, a, b);
}