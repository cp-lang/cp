// lib/event_loop/deferred_task.cp
// Deferred microtask queue for batching I/O and non-reentrant event execution

const std = @import("std");

export type DeferredFn = *const fn (ctx: ?*anyopaque) void;

export struct DeferredEntry {
    pub ctx: ?*anyopaque = null;
    pub task: DeferredFn = undefined;
}

export class DeferredTaskQueue {
    entries: []DeferredEntry = &.{};
    capacity: usize = 0;
    len: usize = 0;

    pub static fn init(): *DeferredTaskQueue {
        let q = std.heap.page_allocator.create(DeferredTaskQueue) catch unreachable;
        q.capacity = 32;
        q.len = 0;
        q.entries = (std.heap.page_allocator.alloc(DeferredEntry, 32) catch unreachable);
        return q;
    }

    pub fn count(self: *DeferredTaskQueue): usize {
        return self.len;
    }

    pub fn postTask(self: *DeferredTaskQueue, ctx: ?*anyopaque, task: DeferredFn): bool {
        // Check for duplicate context to deduplicate flush requests
        if (ctx != null) {
            for (0..self.len) |i| {
                if (self.entries[i].ctx == ctx) {
                    self.entries[i].task = task;
                    return true;
                }
            }
        }

        if (self.len >= self.capacity) {
            const new_cap = self.capacity * 2;
            var new_entries = std.heap.page_allocator.alloc(DeferredEntry, new_cap) catch unreachable;
            for (0..self.len) |i| {
                new_entries[i] = self.entries[i];
            }
            std.heap.page_allocator.free(self.entries);
            self.entries = new_entries;
            self.capacity = new_cap;
        }

        const idx = self.len;
        self.len += 1;
        self.entries[idx].ctx = ctx;
        self.entries[idx].task = task;
        return false;
    }

    pub fn unregisterTask(self: *DeferredTaskQueue, ctx: ?*anyopaque): bool {
        if (ctx == null or self.len == 0) return false;
        for (0..self.len) |i| {
            if (self.entries[i].ctx == ctx) {
                self.len -= 1;
                if (i < self.len) {
                    self.entries[i] = self.entries[self.len];
                }
                return true;
            }
        }
        return false;
    }

    pub fn run(self: *DeferredTaskQueue): usize {
        if (self.len == 0) return 0;
        const total = self.len;
        for (0..total) |i| {
            const entry = self.entries[i];
            entry.task(entry.ctx);
        }
        self.len = 0;
        return total;
    }
}
