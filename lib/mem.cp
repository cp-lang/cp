export class Arena {
    fn alloc(this, size: usize): any {
        return @alloc(size);
    }
    fn free(this): void {
        @free();
    }
}

export fn get_arena(): Arena {
    return @get_arena();
}

export class SharedTensor {
    size: usize,
    ptr: any,

    fn init(this, size: usize): void {
        this.size = size;
        __zig__ {
            const st = ZigSharedTensor.create(std.heap.page_allocator, @intCast(size)) catch unreachable;
            self.ptr = @ptrCast(st);
        }
    }

    fn retain(this): void {
        __zig__ {
            const st: *ZigSharedTensor = @ptrCast(@alignCast(self.ptr));
            st.retain();
        }
    }

    fn release(this): void {
        __zig__ {
            const st: *ZigSharedTensor = @ptrCast(@alignCast(self.ptr));
            std.debug.print("[Auto-Drop Triggered] Releasing SharedTensor!\n", .{});
            st.release();
        }
    }

    fn get_size(this): usize {
        return this.size;
    }
}
