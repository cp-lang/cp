// WaitGroup synchronization primitive in CP
// Ported from Bun.js core threading architecture
import { Mutex } from "./mutex";
import { Condition } from "./condition";
const std = @import("std");

export struct WaitGroup {
    raw_count: std.atomic.Value(usize) = .{ .raw = 0 },
    mutex: Mutex = .{},
    cond: Condition = .{},

    pub fn init(): WaitGroup {
        return .{};
    }

    pub fn initWithCount(count: usize): WaitGroup {
        return .{ .raw_count = .{ .raw = count } };
    }

    pub fn addUnsynchronized(this: *WaitGroup, n: usize) {
        this.raw_count.raw = this.raw_count.raw + n;
    }

    pub fn add(this: *WaitGroup, n: usize) {
        _ = this.raw_count.fetchAdd(n, .monotonic);
    }

    pub fn addOne(this: *WaitGroup) {
        this.add(1);
    }

    pub fn finish(this: *WaitGroup) {
        if (this.raw_count.fetchSub(1, .acq_rel) > 1) {
            return;
        }

        this.mutex.lock();
        this.mutex.unlock();
        this.cond.signal();
    }

    pub fn done(this: *WaitGroup) {
        this.finish();
    }

    pub fn deinit(this: *WaitGroup) {
        _ = this;
    }

    pub fn wait(this: *WaitGroup) {
        this.mutex.lock();
        defer this.mutex.unlock();

        while (this.raw_count.load(.acquire) > 0) {
            this.cond.wait(&this.mutex);
        }
    }

    pub fn release(this: *WaitGroup) {
        this.cond.release();
    }
}
