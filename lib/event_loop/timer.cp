// lib/event_loop/timer.cp
// High-performance intrusive binary min-heap timer system

const std = @import("std");

export type TimerCallback = *const fn (ctx: ?*anyopaque) void;

export class Timer {
    pub id: u64 = 0;
    pub fire_at_ms: i64 = 0;
    pub interval_ms: i64 = 0;
    pub callback: TimerCallback = undefined;
    pub ctx: ?*anyopaque = null;
    pub heap_idx: usize = 0;
    pub active: bool = false;
    pub repeat: bool = false;

    pub static fn create(id: u64, delay_ms: i64, interval_ms: i64, cb: TimerCallback, ctx: ?*anyopaque): *Timer {
        let ptr = std.heap.page_allocator.create(Timer) catch unreachable;
        const now = TimerHeap.nowMs();
        ptr.id = id;
        ptr.fire_at_ms = now + delay_ms;
        ptr.interval_ms = interval_ms;
        ptr.callback = cb;
        ptr.ctx = ctx;
        ptr.heap_idx = 0;
        ptr.active = true;
        ptr.repeat = interval_ms > 0;
        return ptr;
    }

    pub fn destroy(self: *Timer) {
        self.active = false;
        std.heap.page_allocator.destroy(self);
    }
}

export class TimerHeap {
    items: []*Timer = &.{};
    capacity: usize = 0;
    len: usize = 0;

    pub static fn nowMs(): i64 {
        __zig__ {
            var ts: std.os.linux.timespec = undefined;
            _ = std.os.linux.clock_gettime(std.os.linux.CLOCK.MONOTONIC, &ts);
            const sec: i64 = @as(i64, @intCast(ts.sec));
            const nsec: i64 = @as(i64, @intCast(ts.nsec));
            return sec * 1000 + @divTrunc(nsec, 1000000);
        }
    }

    pub static fn sleepMs(ms: u64) void {
        __zig__ {
            const sec: i64 = @as(i64, @intCast(ms / 1000));
            const nsec: i64 = @as(i64, @intCast((ms % 1000) * 1000000));
            const ts = std.os.linux.timespec{ .sec = sec, .nsec = nsec };
            _ = std.os.linux.nanosleep(&ts, null);
        }
    }

    pub static fn init(): *TimerHeap {
        let heap = std.heap.page_allocator.create(TimerHeap) catch unreachable;
        heap.capacity = 64;
        heap.len = 0;
        heap.items = (std.heap.page_allocator.alloc(*Timer, 64) catch unreachable);
        return heap;
    }

    pub fn count(self: *TimerHeap): usize {
        return self.len;
    }

    pub fn isEmpty(self: *TimerHeap): bool {
        return self.len == 0;
    }

    pub fn peek(self: *TimerHeap): ?*Timer {
        if (self.len == 0) return null;
        return self.items[0];
    }

    pub fn push(self: *TimerHeap, timer: *Timer) {
        if (self.len >= self.capacity) {
            const new_cap = self.capacity * 2;
            var new_items = std.heap.page_allocator.alloc(*Timer, new_cap) catch unreachable;
            for (0..self.len) |i| {
                new_items[i] = self.items[i];
            }
            std.heap.page_allocator.free(self.items);
            self.items = new_items;
            self.capacity = new_cap;
        }

        const idx = self.len;
        self.len += 1;
        timer.heap_idx = idx;
        timer.active = true;
        self.items[idx] = timer;
        self.siftUp(idx);
    }

    pub fn pop(self: *TimerHeap): ?*Timer {
        if (self.len == 0) return null;
        let root = self.items[0];
        root.active = false;
        self.len -= 1;
        if (self.len > 0) {
            self.items[0] = self.items[self.len];
            self.items[0].heap_idx = 0;
            self.siftDown(0);
        }
        return root;
    }

    pub fn remove(self: *TimerHeap, timer: *Timer): bool {
        if (!timer.active or self.len == 0) return false;
        const idx = timer.heap_idx;
        if (idx >= self.len or self.items[idx] != timer) return false;

        timer.active = false;
        self.len -= 1;
        if (idx == self.len) {
            return true;
        }

        self.items[idx] = self.items[self.len];
        self.items[idx].heap_idx = idx;
        self.siftUp(idx);
        self.siftDown(idx);
        return true;
    }

    pub fn getTimeoutMs(self: *TimerHeap, now_ms: i64): ?i32 {
        if (self.len == 0) return null;
        const top = self.items[0];
        const diff = top.fire_at_ms - now_ms;
        if (diff <= 0) return 0;
        if (diff > 2147483647) return 2147483647;
        return @as(i32, @intCast(diff));
    }

    pub fn drainExpired(self: *TimerHeap, now_ms: i64, max_drain: usize): usize {
        var fired: usize = 0;
        while (fired < max_drain and self.len > 0) {
            const top = self.items[0];
            if (top.fire_at_ms > now_ms) break;

            let timer = self.pop().?;
            timer.callback(timer.ctx);
            fired += 1;

            if (timer.repeat and timer.active) {
                timer.fire_at_ms = now_ms + timer.interval_ms;
                self.push(timer);
            }
        }
        return fired;
    }

    fn siftUp(self: *TimerHeap, start_idx: usize) {
        var idx = start_idx;
        while (idx > 0) {
            const parent = (idx - 1) / 2;
            if (self.items[idx].fire_at_ms < self.items[parent].fire_at_ms) {
                const tmp = self.items[idx];
                self.items[idx] = self.items[parent];
                self.items[parent] = tmp;
                self.items[idx].heap_idx = idx;
                self.items[parent].heap_idx = parent;
                idx = parent;
            } else {
                break;
            }
        }
    }

    fn siftDown(self: *TimerHeap, start_idx: usize) {
        var idx = start_idx;
        while (true) {
            var smallest = idx;
            const left = idx * 2 + 1;
            const right = idx * 2 + 2;

            if (left < self.len and self.items[left].fire_at_ms < self.items[smallest].fire_at_ms) {
                smallest = left;
            }
            if (right < self.len and self.items[right].fire_at_ms < self.items[smallest].fire_at_ms) {
                smallest = right;
            }

            if (smallest != idx) {
                const tmp = self.items[idx];
                self.items[idx] = self.items[smallest];
                self.items[smallest] = tmp;
                self.items[idx].heap_idx = idx;
                self.items[smallest].heap_idx = smallest;
                idx = smallest;
            } else {
                break;
            }
        }
    }
}
