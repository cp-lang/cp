// lib/collections/linear_fifo.cp
// High-throughput power-of-two ring-buffer FIFO queue
// Absorbed from Bun src/collections/linear_fifo.cp

const std = @import("std");

export class LinearFifo<T> {
    pub buf: []T = &.{};
    pub cap: usize = 0;
    pub head: usize = 0;
    pub count: usize = 0;

    pub static fn init(initial_power_of_two_cap: usize): *LinearFifo<T> {
        var c: usize = 16;
        while (c < initial_power_of_two_cap) {
            c = c * 2;
        }
        let fifo = std.heap.page_allocator.create(@This()) catch unreachable;
        fifo.buf = (std.heap.page_allocator.alloc(T, c) catch unreachable);
        fifo.cap = c;
        fifo.head = 0;
        fifo.count = 0;
        return fifo;
    }

    pub fn readableLength(self: *LinearFifo<T>): usize {
        return self.count;
    }

    pub fn writableLength(self: *LinearFifo<T>): usize {
        return self.cap - self.count;
    }

    pub fn isEmpty(self: *LinearFifo<T>): bool {
        return self.count == 0;
    }

    pub fn ensureCapacity(self: *LinearFifo<T>, additional: usize): void {
        const needed = self.count + additional;
        if (needed <= self.cap) {
            return;
        }
        var new_cap = self.cap * 2;
        while (new_cap < needed) {
            new_cap = new_cap * 2;
        }
        var new_buf = std.heap.page_allocator.alloc(T, new_cap) catch unreachable;
        const mask = self.cap - 1;
        var i: usize = 0;
        while (i < self.count) {
            new_buf[i] = self.buf[(self.head + i) & mask];
            i += 1;
        }
        std.heap.page_allocator.free(self.buf);
        self.buf = new_buf;
        self.cap = new_cap;
        self.head = 0;
    }

    pub fn writeItem(self: *LinearFifo<T>, item: T): void {
        self.ensureCapacity(1);
        const mask = self.cap - 1;
        const write_idx = (self.head + self.count) & mask;
        self.buf[write_idx] = item;
        self.count += 1;
    }

    pub fn writeSlice(self: *LinearFifo<T>, items: []const T): void {
        if (items.len == 0) {
            return;
        }
        self.ensureCapacity(items.len);
        const mask = self.cap - 1;
        var i: usize = 0;
        while (i < items.len) {
            const write_idx = (self.head + self.count + i) & mask;
            self.buf[write_idx] = items[i];
            i += 1;
        }
        self.count += items.len;
    }

    pub fn peekItem(self: *LinearFifo<T>, offset: usize): T? {
        if (offset >= self.count) {
            return null;
        }
        const mask = self.cap - 1;
        return self.buf[(self.head + offset) & mask];
    }

    pub fn readItem(self: *LinearFifo<T>): T? {
        if (self.count == 0) {
            return null;
        }
        const mask = self.cap - 1;
        const val = self.buf[self.head];
        self.head = (self.head + 1) & mask;
        self.count -= 1;
        if (self.count == 0) {
            self.head = 0;
        }
        return val;
    }

    pub fn readSlice(self: *LinearFifo<T>, out: []T): usize {
        const to_read = if (out.len < self.count) out.len else self.count;
        const mask = self.cap - 1;
        var i: usize = 0;
        while (i < to_read) {
            out[i] = self.buf[(self.head + i) & mask];
            i += 1;
        }
        self.discard(to_read);
        return to_read;
    }

    pub fn discard(self: *LinearFifo<T>, n: usize): void {
        if (n >= self.count) {
            self.head = 0;
            self.count = 0;
            return;
        }
        const mask = self.cap - 1;
        self.head = (self.head + n) & mask;
        self.count -= n;
    }

    pub fn rewind(self: *LinearFifo<T>): void {
        self.head = 0;
        self.count = 0;
    }

    pub fn deinit(self: *LinearFifo<T>): void {
        if (self.cap > 0) {
            std.heap.page_allocator.free(self.buf);
            self.cap = 0;
            self.head = 0;
            self.count = 0;
        }
    }

    pub fn release(self: *LinearFifo<T>): void {
        self.deinit();
        std.heap.page_allocator.destroy(self);
    }
}
