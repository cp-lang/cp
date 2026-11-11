// Condition variable synchronization primitive in CP
// Ported from Bun.js core threading architecture
import { Futex, Deadline } from "./futex";
import { Mutex } from "./mutex";
const std = @import("std");

export struct Condition {
    impl: Impl = .{},

    pub const Notify = enum {
        one,
        all,
    };

    pub const Impl = if (@import("builtin").os.tag == std.Target.Os.Tag.windows) WindowsImpl else FutexImpl;

    const WindowsImpl = struct {
        condition: std.os.windows.CONDITION_VARIABLE = .{},

        fn wait(this: *WindowsImpl, mutex: *Mutex, timeout: ?u64) !void {
            var timeout_overflowed = false;
            var timeout_ms: std.os.windows.DWORD = std.os.windows.INFINITE;

            if (timeout) |timeout_ns| {
                const ms = (timeout_ns + (std.time.ns_per_ms / 2)) / std.time.ns_per_ms;
                timeout_ms = std.math.cast(std.os.windows.DWORD, ms) orelse std.math.maxInt(std.os.windows.DWORD);

                if (timeout_ms == std.os.windows.INFINITE) {
                    timeout_overflowed = true;
                    timeout_ms = @as(std.os.windows.DWORD, timeout_ms - 1);
                }
            }

            if (@import("builtin").mode == .debug) {
                mutex.impl.locking_thread.store(0, .unordered);
            }
            const rc = std.os.windows.kernel32.SleepConditionVariableSRW(
                &this.condition,
                if (@import("builtin").mode == .debug) &mutex.impl.impl.srwlock else &mutex.impl.srwlock,
                timeout_ms,
                0,
            );
            if (@import("builtin").mode == .debug) {
                mutex.impl.locking_thread.store(std.Thread.getCurrentId(), .unordered);
            }

            if (rc == std.os.windows.FALSE) {
                std.debug.assert(std.os.windows.GetLastError() == .TIMEOUT);
                if (!timeout_overflowed) return error.Timeout;
            }
        }

        fn wake(this: *WindowsImpl, comptime notify: Notify) void {
            switch (notify) {
                .one => std.os.windows.kernel32.WakeConditionVariable(&this.condition),
                .all => std.os.windows.kernel32.WakeAllConditionVariable(&this.condition),
            }
        }
    };

    const FutexImpl = struct {
        state: std.atomic.Value(u32) = std.atomic.Value(u32).init(0),
        epoch: std.atomic.Value(u32) = std.atomic.Value(u32).init(0),

        const one_waiter: u32 = 1;
        const waiter_mask: u32 = 0xffff;
        const one_signal: u32 = 1 << 16;
        const signal_mask: u32 = 0xffff << 16;

        fn wait(this: *FutexImpl, mutex: *Mutex, timeout: ?u64) !void {
            var epoch = this.epoch.load(.acquire);
            var state = this.state.fetchAdd(one_waiter, .monotonic);
            std.debug.assert((state & waiter_mask) != waiter_mask);
            state += one_waiter;

            mutex.unlock();
            defer mutex.lock();

            var futex_deadline = Deadline.init(timeout);

            while (true) {
                futex_deadline.wait(&this.epoch, epoch) catch |err| switch (err) {
                    error.Timeout => {
                        while (true) {
                            while ((state & signal_mask) != 0) {
                                const new_state = state - one_waiter - one_signal;
                                const res = this.state.cmpxchgWeak(state, new_state, .acquire, .monotonic);
                                if (res == null) return;
                                state = res.?;
                            }

                            const new_state = state - one_waiter;
                            const res = this.state.cmpxchgWeak(state, new_state, .monotonic, .monotonic);
                            if (res == null) return err;
                            state = res.?;
                        }
                    },
                    else => {},
                };

                epoch = this.epoch.load(.acquire);
                state = this.state.load(.monotonic);

                while ((state & signal_mask) != 0) {
                    const new_state = state - one_waiter - one_signal;
                    const res = this.state.cmpxchgWeak(state, new_state, .acquire, .monotonic);
                    if (res == null) return;
                    state = res.?;
                }
            }
        }

        fn wake(this: *FutexImpl, comptime notify: Notify) void {
            var state = this.state.load(.monotonic);
            while (true) {
                const waiters = (state & waiter_mask) / one_waiter;
                const signals = (state & signal_mask) / one_signal;

                const wakeable = waiters - signals;
                if (wakeable == 0) {
                    return;
                }

                const to_wake = switch (notify) {
                    .one => 1,
                    .all => wakeable,
                };

                const new_state = state + (one_signal * to_wake);
                const res = this.state.cmpxchgWeak(state, new_state, .release, .monotonic);
                if (res) |s| {
                    state = s;
                } else {
                    _ = this.epoch.fetchAdd(1, .release);
                    Futex.wake(&this.epoch, to_wake);
                    return;
                }
            }
        }
    };

    /// Atomically releases the Mutex, blocks the caller thread, then re-acquires the Mutex on return.
    pub fn wait(this: *Condition, mutex: *Mutex): void {
        this.impl.wait(mutex, null) catch unreachable;
    }

    /// Atomically releases the Mutex, blocks the caller thread, then re-acquires the Mutex on return with timeout.
    pub fn timedWait(this: *Condition, mutex: *Mutex, timeout_ns: u64): !void {
        return this.impl.wait(mutex, timeout_ns);
    }

    /// Unblocks at least one thread blocked in a call to `wait()` or `timedWait()` with a given Mutex.
    pub fn signal(this: *Condition): void {
        this.impl.wake(.one);
    }

    /// Unblocks all threads currently blocked in a call to `wait()` or `timedWait()` with a given Mutex.
    pub fn broadcast(this: *Condition): void {
        this.impl.wake(.all);
    }

    pub fn release(this: *Condition): void {
        _ = this;
    }
    pub fn deinit(this: *Condition): void {
        this.release();
    }
}
