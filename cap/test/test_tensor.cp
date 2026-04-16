// con/test_tensor.cp
import { print } from "std/io";

fn main(): ?void {
    @print("--- Tensor and Arena Test ---");

    // Test Tensor Initialization (mapping to @tensor_init)
    let shape = [2, 3];
    let t = @tensor_init(shape);

    __zig__ {
        std.debug.print("Tensor initialized. Shape: {d}x{d}, Data ptr: {*}\n", .{ctx.t.shape[0], ctx.t.shape[1], ctx.t.data.ptr});
    }

    // Test Agent-Local Allocation
    let buffer = @alloc(null, 100);
    @print("Allocated buffer size: 100");

    __zig__ {
        std.debug.print("Buffer pointer: {*}\n", .{ctx.buffer.ptr});
    }
}
