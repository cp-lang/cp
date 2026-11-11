// Atomic reference-counted slice in CP
// Enables zero-copy, lock-free, immutable data sharing across threads and actors
const std = @import("std");

export fn ArcSlice(comptime T: type): type {
    const Header = struct {
        ref_count: std.atomic.Value(usize),
        allocator: std.mem.Allocator,
    };

    return struct {
        const Self = @This();

        items: []T = &.{},
        header: ?*Header = null,

        pub fn init(allocator: std.mem.Allocator, data: []const T): !Self {
            const hdr = try allocator.create(Header);
            errdefer allocator.destroy(hdr);

            const items = try allocator.alloc(T, data.len);
            errdefer allocator.free(items);

            hdr.* = .{
                .ref_count = std.atomic.Value(usize).init(1),
                .allocator = allocator,
            };
            @memcpy(items, data);

            return Self{
                .items = items,
                .header = hdr,
            };
        }

        pub fn clone(self: *const Self): Self {
            if (self.header) |hdr| {
                _ = hdr.ref_count.fetchAdd(1, .monotonic);
            }
            return Self{
                .items = self.items,
                .header = self.header,
            };
        }

        pub fn release(self: *Self): void {
            if (self.header) |hdr| {
                self.header = null;
                if (hdr.ref_count.fetchSub(1, .acq_rel) == 1) {
                    hdr.allocator.free(self.items);
                    hdr.allocator.destroy(hdr);
                }
            }
        }

        pub fn deinit(self: *Self): void {
            self.release();
        }

        pub fn slice(self: *const Self): []const T {
            return self.items;
        }

        pub fn len(self: *const Self): usize {
            return self.items.len;
        }

        pub fn refCount(self: *const Self): usize {
            if (self.header) |hdr| {
                return hdr.ref_count.load(.monotonic);
            }
            return 0;
        }

        pub fn get(self: *const Self, index: usize): ?T {
            if (index >= self.items.len) return null;
            return self.items[index];
        }
    };
}
