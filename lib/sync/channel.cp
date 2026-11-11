// Thread-safe bounded channel in CP
// High-throughput CSP communication primitive
import { Mutex } from "./mutex";
import { Condition } from "./condition";
const std = @import("std");

export fn Channel(comptime T: type): type {
    return struct {
        const Self = @This();

        mutex: Mutex = .{},
        not_empty: Condition = .{},
        not_full: Condition = .{},
        buf: []T = undefined,
        head: usize = 0,
        tail: usize = 0,
        count: usize = 0,
        capacity: usize = 0,
        is_closed: bool = false,
        allocator: std.mem.Allocator = undefined,

        pub fn init(allocator: std.mem.Allocator, capacity: usize): !Self {
            const cap = if (capacity == 0) 1 else capacity;
            const buf = try allocator.alloc(T, cap);
            return Self{
                .mutex = .{},
                .not_empty = .{},
                .not_full = .{},
                .buf = buf,
                .head = 0,
                .tail = 0,
                .count = 0,
                .capacity = cap,
                .is_closed = false,
                .allocator = allocator,
            };
        }

        pub fn deinit(self: *Self): void {
            self.close();
            if (self.capacity > 0) {
                self.allocator.free(self.buf);
                self.buf = &.{};
                self.capacity = 0;
            }
            self.mutex.deinit();
            self.not_empty.deinit();
            self.not_full.deinit();
        }

        pub fn close(self: *Self): void {
            self.mutex.lock();
            defer self.mutex.unlock();

            if (self.is_closed) return;
            self.is_closed = true;
            self.not_full.broadcast();
            self.not_empty.broadcast();
        }

        pub fn isClosed(self: *Self): bool {
            self.mutex.lock();
            defer self.mutex.unlock();
            return self.is_closed;
        }

        pub fn len(self: *Self): usize {
            self.mutex.lock();
            defer self.mutex.unlock();
            return self.count;
        }

        pub fn send(self: *Self, item: T): !void {
            self.mutex.lock();
            defer self.mutex.unlock();

            while (self.count == self.capacity and !self.is_closed) {
                self.not_full.wait(&self.mutex);
            }

            if (self.is_closed) {
                return error.Closed;
            }

            self.buf[self.tail] = item;
            self.tail = (self.tail + 1) % self.capacity;
            self.count += 1;
            self.not_empty.signal();
        }

        pub fn trySend(self: *Self, item: T): !bool {
            self.mutex.lock();
            defer self.mutex.unlock();

            if (self.is_closed) return error.Closed;
            if (self.count == self.capacity) return false;

            self.buf[self.tail] = item;
            self.tail = (self.tail + 1) % self.capacity;
            self.count += 1;
            self.not_empty.signal();
            return true;
        }

        pub fn recv(self: *Self): !?T {
            self.mutex.lock();
            defer self.mutex.unlock();

            while (self.count == 0 and !self.is_closed) {
                self.not_empty.wait(&self.mutex);
            }

            if (self.count == 0 and self.is_closed) {
                return null;
            }

            const item = self.buf[self.head];
            self.head = (self.head + 1) % self.capacity;
            self.count -= 1;
            self.not_full.signal();
            return item;
        }

        pub fn tryRecv(self: *Self): ?T {
            self.mutex.lock();
            defer self.mutex.unlock();

            if (self.count == 0) return null;

            const item = self.buf[self.head];
            self.head = (self.head + 1) % self.capacity;
            self.count -= 1;
            self.not_full.signal();
            return item;
        }
    };
}
