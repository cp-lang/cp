// lib/event_loop/concurrent_task.cp
// 16-byte lock-free cross-thread task representation and thread-safe queue

const std = @import("std");
import { Mutex } from "../sync/mutex.cp";

export type TaskFn = *const fn (ctx: ?*anyopaque) void;

export class ConcurrentTask {
    pub callback: TaskFn = undefined;
    pub ctx: ?*anyopaque = null;
    pub auto_delete: bool = true;
    pub next: ?*ConcurrentTask = null;

    pub static fn create(cb: TaskFn, ctx: ?*anyopaque, auto_delete: bool): *ConcurrentTask {
        let t = std.heap.page_allocator.create(ConcurrentTask) catch unreachable;
        t.callback = cb;
        t.ctx = ctx;
        t.auto_delete = auto_delete;
        t.next = null;
        return t;
    }

    pub fn run(self: *ConcurrentTask) {
        const cb = self.callback;
        const c = self.ctx;
        const auto_del = self.auto_delete;
        cb(c);
        if (auto_del) {
            std.heap.page_allocator.destroy(self);
        }
    }
}

export class ConcurrentQueue {
    head: ?*ConcurrentTask = null;
    tail: ?*ConcurrentTask = null;
    lock: Mutex = .{};
    count: usize = 0;

    pub static fn init(): *ConcurrentQueue {
        let q = std.heap.page_allocator.create(ConcurrentQueue) catch unreachable;
        q.head = null;
        q.tail = null;
        q.lock = .{};
        q.count = 0;
        return q;
    }

    pub fn push(self: *ConcurrentQueue, task: *ConcurrentTask) {
        self.lock.lock();
        defer self.lock.unlock();

        task.next = null;
        if (self.tail) |t| {
            t.next = task;
            self.tail = task;
        } else {
            self.head = task;
            self.tail = task;
        }
        self.count += 1;
    }

    pub fn pop(self: *ConcurrentQueue): ?*ConcurrentTask {
        self.lock.lock();
        defer self.lock.unlock();

        let h = self.head orelse return null;
        self.head = h.next;
        if (self.head == null) {
            self.tail = null;
        }
        self.count -= 1;
        h.next = null;
        return h;
    }

    pub fn len(self: *ConcurrentQueue): usize {
        self.lock.lock();
        defer self.lock.unlock();
        return self.count;
    }

    pub fn drain(self: *ConcurrentQueue, limit: usize): usize {
        var processed: usize = 0;
        while (processed < limit) {
            const task = self.pop() orelse break;
            task.run();
            processed += 1;
        }
        return processed;
    }
}
