// lib/collections/baby_list.cp
// Ultra-compact u32-indexed dynamic list & OffsetByteList
// Absorbed from Bun src/collections/baby_list.cp

const std = @import("std");

export class BabyList<T> {
    pub items: []T = &.{};
    pub len: u32 = 0;
    pub cap: u32 = 0;

    pub static fn init(): *BabyList<T> {
        let list = std.heap.page_allocator.create(@This()) catch unreachable;
        list.items = (std.heap.page_allocator.alloc(T, 8) catch unreachable);
        list.len = 0;
        list.cap = 8;
        return list;
    }

    pub static fn initCapacity(initial_cap: u32): *BabyList<T> {
        const actual_cap: u32 = if (initial_cap > 0) initial_cap else 8;
        let list = std.heap.page_allocator.create(@This()) catch unreachable;
        list.items = (std.heap.page_allocator.alloc(T, actual_cap as usize) catch unreachable);
        list.len = 0;
        list.cap = actual_cap;
        return list;
    }

    pub fn isEmpty(self: *BabyList<T>): bool {
        return self.len == 0;
    }

    pub fn count(self: *BabyList<T>): usize {
        return self.len as usize;
    }

    pub fn capacity(self: *BabyList<T>): usize {
        return self.cap as usize;
    }

    pub fn ensureTotalCapacity(self: *BabyList<T>, needed: u32): void {
        if (needed <= self.cap) {
            return;
        }
        var new_cap: u32 = if (self.cap > 0) self.cap else 8;
        while (new_cap < needed) {
            new_cap = new_cap * 2;
        }
        var new_items = std.heap.page_allocator.alloc(T, new_cap as usize) catch unreachable;
        var i: usize = 0;
        const n = self.len as usize;
        while (i < n) {
            new_items[i] = self.items[i];
            i += 1;
        }
        if (self.cap > 0) {
            std.heap.page_allocator.free(self.items);
        }
        self.items = new_items;
        self.cap = new_cap;
    }

    pub fn append(self: *BabyList<T>, value: T): void {
        self.ensureTotalCapacity(self.len + 1);
        self.items[self.len as usize] = value;
        self.len += 1;
    }

    pub fn appendSlice(self: *BabyList<T>, values: []const T): void {
        const add_len = values.len as u32;
        if (add_len == 0) {
            return;
        }
        self.ensureTotalCapacity(self.len + add_len);
        const base = self.len as usize;
        var i: usize = 0;
        while (i < values.len) {
            self.items[base + i] = values[i];
            i += 1;
        }
        self.len += add_len;
    }

    pub fn at(self: *BabyList<T>, index: usize): T {
        return self.items[index];
    }

    pub fn set(self: *BabyList<T>, index: usize, value: T): void {
        if (index < (self.len as usize)) {
            self.items[index] = value;
        }
    }

    pub fn first(self: *BabyList<T>): T? {
        if (self.len == 0) {
            return null;
        }
        return self.items[0];
    }

    pub fn last(self: *BabyList<T>): T? {
        if (self.len == 0) {
            return null;
        }
        return self.items[(self.len - 1) as usize];
    }

    pub fn pop(self: *BabyList<T>): T? {
        if (self.len == 0) {
            return null;
        }
        self.len -= 1;
        return self.items[self.len as usize];
    }

    pub fn clearRetainingCapacity(self: *BabyList<T>): void {
        self.len = 0;
    }

    pub fn deinit(self: *BabyList<T>): void {
        if (self.cap > 0) {
            std.heap.page_allocator.free(self.items);
            self.cap = 0;
            self.len = 0;
        }
    }

    pub fn release(self: *BabyList<T>): void {
        self.deinit();
        std.heap.page_allocator.destroy(self);
    }
}

// Absorbed from Bun OffsetByteList: BabyList<u8> with a read head offset
export class OffsetByteList {
    pub list: *BabyList<u8>;
    pub head: u32 = 0;

    pub static fn init(initial_cap: u32): *OffsetByteList {
        let obl = std.heap.page_allocator.create(OffsetByteList) catch unreachable;
        obl.list = BabyList<u8>.initCapacity(initial_cap);
        obl.head = 0;
        return obl;
    }

    pub fn remainingLen(self: *OffsetByteList): u32 {
        if (self.head >= self.list.len) {
            return 0;
        }
        return self.list.len - self.head;
    }

    pub fn write(self: *OffsetByteList, bytes: []const u8): void {
        self.list.appendSlice(bytes);
    }

    pub fn consume(self: *OffsetByteList, n: u32): u32 {
        const rem = self.remainingLen();
        const step = if (n < rem) n else rem;
        self.head += step;
        if (self.head == self.list.len) {
            self.head = 0;
            self.list.clearRetainingCapacity();
        }
        return step;
    }

    pub fn reset(self: *OffsetByteList): void {
        self.head = 0;
        self.list.clearRetainingCapacity();
    }

    pub fn release(self: *OffsetByteList): void {
        self.list.release();
        std.heap.page_allocator.destroy(self);
    }
}
