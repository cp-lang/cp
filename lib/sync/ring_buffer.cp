// Bounded MPMC lock-free ring buffer in CP
// Based on Dmitry Vyukov's MPMC queue algorithm
const std = @import("std");

export fn MpmcQueue(comptime T: type, comptime Capacity: usize): type {
    return struct {
        const Self = @This();
        const Slot = struct {
            turn: std.atomic.Value(usize),
            storage: T,
        };

        slots: [Capacity]Slot = undefined,
        head: std.atomic.Value(usize) = .{ .raw = 0 },
        tail: std.atomic.Value(usize) = .{ .raw = 0 },

        pub fn init(): Self {
            var self: Self = undefined;
            self.head = .{ .raw = 0 };
            self.tail = .{ .raw = 0 };
            var i: usize = 0;
            while (i < Capacity) : (i += 1) {
                self.slots[i] = .{
                    .turn = .{ .raw = i },
                    .storage = undefined,
                };
            }
            return self;
        }

        pub fn tryPush(self: *Self, item: T): bool {
            var head = self.head.load(.monotonic);
            while (true) {
                const slot = &self.slots[head % Capacity];
                const turn = slot.turn.load(.acquire);
                const diff = @as(i64, @intCast(turn)) - @as(i64, @intCast(head));
                if (diff == 0) {
                    if (self.head.cmpxchgWeak(head, head + 1, .monotonic, .monotonic)) |next_head| {
                        head = next_head;
                    } else {
                        slot.storage = item;
                        slot.turn.store(head + 1, .release);
                        return true;
                    }
                } else if (diff < 0) {
                    return false;
                } else {
                    head = self.head.load(.monotonic);
                }
            }
        }

        pub fn tryPop(self: *Self): ?T {
            var tail = self.tail.load(.monotonic);
            while (true) {
                const slot = &self.slots[tail % Capacity];
                const turn = slot.turn.load(.acquire);
                const diff = @as(i64, @intCast(turn)) - @as(i64, @intCast(tail + 1));
                if (diff == 0) {
                    if (self.tail.cmpxchgWeak(tail, tail + 1, .monotonic, .monotonic)) |next_tail| {
                        tail = next_tail;
                    } else {
                        const item = slot.storage;
                        slot.turn.store(tail + Capacity, .release);
                        return item;
                    }
                } else if (diff < 0) {
                    return null;
                } else {
                    tail = self.tail.load(.monotonic);
                }
            }
        }

        pub fn isEmpty(self: *Self): bool {
            const head = self.head.load(.monotonic);
            const tail = self.tail.load(.monotonic);
            return head == tail;
        }

        pub fn isFull(self: *Self): bool {
            const head = self.head.load(.monotonic);
            const tail = self.tail.load(.monotonic);
            return (head - tail) >= Capacity;
        }

        pub fn len(self: *Self): usize {
            const head = self.head.load(.monotonic);
            const tail = self.tail.load(.monotonic);
            if (head >= tail) {
                return head - tail;
            }
            return 0;
        }
    };
}

export fn RingBuffer(comptime T: type, comptime Capacity: usize): type {
    return MpmcQueue(T, Capacity);
}
