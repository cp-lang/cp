// con/test_async_tuple.cp
import { print } from "std/io";

fn get_tuple(): [i32, string] {
    return [42, "hello"];
}

async fn async_tuple(): [i32, i32] {
    return [100, 200];
}

async fn main(): ?void {
    // Test Tuple destructuring (positional)
    let [x, s]: [i32, string] = get_tuple();
    __zig__ {
        std.debug.print("Tuple: x={d}, s={s}\n", .{ ctx.x, ctx.s });
    }

    // Test Object-like (Named) anonymous struct
    let obj: {w: i32, h: i32} = { w: 1920, h: 1080 };
    let { w, h }: {w: i32, h: i32} = obj;
    __zig__ {
        std.debug.print("Object: w={d}, h={d}\n", .{ ctx.w, ctx.h });
    }

    // Test async/await with Tuple
    let [a, b]: [i32, i32] = await async_tuple();
    __zig__ {
        std.debug.print("Async Tuple: a={d}, b={d}\n", .{ ctx.a, ctx.b });
    }
}
