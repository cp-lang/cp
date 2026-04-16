// con/test_matmul.cp
import { print } from "std/io";

fn main(): ?void {
    @print("--- High Performance MatMul Test ---");

    // Initialize 2x2 identity-like matrices
    let shape = [2, 2];
    let a = @tensor_init(shape);
    let b = @tensor_init(shape);
    let out = @tensor_init(shape);

    __zig__ {
        ctx.a.data[0] = 1.0; ctx.a.data[1] = 2.0;
        ctx.a.data[2] = 3.0; ctx.a.data[3] = 4.0;
        
        ctx.b.data[0] = 5.0; ctx.b.data[1] = 6.0;
        ctx.b.data[2] = 7.0; ctx.b.data[3] = 8.0;
    }

    // High performance builtin call
    @matmul(a, b, out);

    __zig__ {
        std.debug.print("Result[0,0] = {d} (Expected 19.0)\n", .{ctx.out.data[0]});
        std.debug.print("Result[0,1] = {d} (Expected 22.0)\n", .{ctx.out.data[1]});
        std.debug.print("Result[1,0] = {d} (Expected 43.0)\n", .{ctx.out.data[2]});
        std.debug.print("Result[1,1] = {d} (Expected 50.0)\n", .{ctx.out.data[3]});
    }
}
