// Thread-local memory pool for JSON parsing (Ported from Bun.js HashMapPool)
const std = @import("std");

export struct JsonBufferPool {
    const BufferList = std.SinglyLinkedList([4096]u8);

    pub static fn acquire(allocator: std.mem.Allocator): *BufferList.Node {
        zig {
            const S = struct {
                threadlocal var pool_head: ?*BufferList.Node = null;
                threadlocal var is_initialized: bool = false;
            };
            if (S.is_initialized) {
                if (S.pool_head) |node| {
                    S.pool_head = node.next;
                    return node;
                }
            }
            const node = allocator.create(BufferList.Node) catch unreachable;
            S.is_initialized = true;
            return node;
        }
    }

    pub static fn release(node: *BufferList.Node): void {
        zig {
            const S = struct {
                threadlocal var pool_head: ?*BufferList.Node = null;
            };
            node.next = S.pool_head;
            S.pool_head = node;
        }
    }
}
