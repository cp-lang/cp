// Intrusive lock-free unbounded queue in CP
// Ported from Bun.js core threading architecture
const std = @import("std");

pub fn UnboundedQueue(comptime T: type, comptime next_field: std.meta.FieldEnum(T)): type {
    const field_info = std.meta.fieldInfo(T, next_field);
    const next_name = field_info.name;

    const cache_line_length = switch (@import("builtin").target.cpu.arch) {
        .x86_64, .aarch64, .powerpc64 => 128,
        .arm, .mips, .mips64, .riscv64 => 32,
        .s390x => 256,
        else => 64,
    };

    return struct {
        const Self = @This();

        inline fn getNext(item: *T): ?*T {
            return @field(item, next_name);
        }

        inline fn setNext(item: *T, ptr: ?*T): void {
            @field(item, next_name) = ptr;
        }

        inline fn atomicLoadNext(item: *T, ordering: std.builtin.AtomicOrder): ?*T {
            return @atomicLoad(?*T, &@field(item, next_name), ordering);
        }

        inline fn atomicStoreNext(item: *T, ptr: ?*T, ordering: std.builtin.AtomicOrder): void {
            @atomicStore(?*T, &@field(item, next_name), ptr, ordering);
        }

        pub const Batch = struct {
            pub const Iterator = struct {
                batch: Self.Batch,

                pub fn next(self: *Self.Batch.Iterator): ?*T {
                    if (self.batch.count == 0) return null;
                    const front = self.batch.front orelse unreachable;
                    self.batch.front = getNext(front);
                    self.batch.count -= 1;
                    return front;
                }
            };

            front: ?*T = null,
            last: ?*T = null,
            count: usize = 0,

            pub fn iterator(self: Self.Batch): Self.Batch.Iterator {
                return .{ .batch = self };
            }
        };

        pub const queue_padding_length = cache_line_length / 2;

        back: std.atomic.Value(?*T) align(queue_padding_length) = .init(null),
        front: std.atomic.Value(?*T) align(queue_padding_length) = .init(null),

        pub fn push(self: *Self, item: *T): void {
            self.pushBatch(item, item);
        }

        pub fn pushBatch(self: *Self, first: *T, last: *T): void {
            setNext(last, null);
            if (self.back.swap(last, .acq_rel)) |old_back| {
                atomicStoreNext(old_back, first, .release);
            } else {
                self.front.store(first, .release);
            }
        }

        pub fn pop(self: *Self): ?*T {
            var first = self.front.load(.acquire) orelse return null;
            const next_item = while (true) {
                const next_ptr = atomicLoadNext(first, .acquire);
                const maybe_first = self.front.cmpxchgWeak(
                    first,
                    next_ptr,
                    .acquire,
                    .monotonic,
                );
                if (maybe_first) |actual_first| {
                    first = actual_first;
                } else {
                    break next_ptr;
                }
            };

            if (next_item == null) {
                _ = self.back.cmpxchgStrong(first, null, .acq_rel, .monotonic);
            }

            return first;
        }

        pub fn popBatch(self: *Self): ?Self.Batch {
            var first = self.front.load(.acquire) orelse return null;
            while (true) {
                const maybe_first = self.front.cmpxchgWeak(
                    first,
                    null,
                    .acquire,
                    .monotonic,
                );
                if (maybe_first) |actual_first| {
                    first = actual_first;
                } else {
                    break;
                }
            }

            const last = self.back.swap(null, .acq_rel) orelse unreachable;
            var count: usize = 1;
            var curr = first;
            while (curr != last) {
                count += 1;
                while (atomicLoadNext(curr, .acquire)) |next_item| {
                    curr = next_item;
                    break;
                }
            }

            return Self.Batch{
                .front = first,
                .last = last,
                .count = count,
            };
        }
    };
}
