// con/test_stdlib.cp
import { print } from "std/io";
import { get_arena } from "std/mem";

fn main(): ?void {
    // Testing standard library IO wrapper
    print("Standard Library IO check: status={s}", "ok");

    // Testing memory management
    let arena = get_arena();
    let ptr = arena.alloc(1024);
    
    __zig__ {
        std.debug.print("Allocated 1024 bytes at: {*}\n", .{ctx.ptr});
    }

    arena.free();
}
