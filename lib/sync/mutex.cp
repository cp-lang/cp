// Mutex synchronization primitive in CP
// Ported from Bun.js core threading architecture
import { Futex } from "./futex";
const std = @import("std");

export struct Mutex {
    impl: Impl = .{},

    const unlocked: u32 = 0b00;
    const locked: u32 = 0b01;
    const contended: u32 = 0b11;

    pub const Impl = if (@import("builtin").mode == .debug and !@import("builtin").single_threaded) DebugImpl else ReleaseImpl;
    pub const ReleaseImpl = if (@import("builtin").os.tag == std.Target.Os.Tag.windows) WindowsImpl else if (@import("builtin").os.tag.isDarwin()) DarwinImpl else FutexImpl;
    pub const ExternImpl = ReleaseImpl.Type;

    const DebugImpl = struct {
        locking_thread: std.atomic.Value(std.Thread.Id) = std.atomic.Value(std.Thread.Id).init(0),
        impl: ReleaseImpl = .{},

        inline fn tryLock(this: *DebugImpl) bool {
            const locking = this.impl.tryLock();
            if (locking) {
                this.locking_thread.store(std.Thread.getCurrentId(), .unordered);
            }
            return locking;
        }

        inline fn lock(this: *DebugImpl) void {
            const current_id = std.Thread.getCurrentId();
            if (this.locking_thread.load(.unordered) == current_id and current_id != 0) {
                @panic("Deadlock detected");
            }
            this.impl.lock();
            this.locking_thread.store(current_id, .unordered);
        }

        inline fn unlock(this: *DebugImpl) void {
            std.debug.assert(this.locking_thread.load(.unordered) == std.Thread.getCurrentId());
            this.locking_thread.store(0, .unordered);
            this.impl.unlock();
        }
    };

    const WindowsImpl = struct {
        srwlock: std.os.windows.SRWLOCK = .{},

        fn tryLock(this: *WindowsImpl) bool {
            return std.os.windows.kernel32.TryAcquireSRWLockExclusive(&this.srwlock) != std.os.windows.FALSE;
        }

        fn lock(this: *WindowsImpl) void {
            std.os.windows.kernel32.AcquireSRWLockExclusive(&this.srwlock);
        }

        fn unlock(this: *WindowsImpl) void {
            std.os.windows.kernel32.ReleaseSRWLockExclusive(&this.srwlock);
        }

        pub const Type = std.os.windows.SRWLOCK;
    };

    const DarwinImpl = struct {
        oul: std.c.os_unfair_lock = .{},

        fn tryLock(this: *DarwinImpl) bool {
            return std.c.os_unfair_lock_trylock(&this.oul);
        }

        fn lock(this: *DarwinImpl) void {
            std.c.os_unfair_lock_lock(&this.oul);
        }

        fn unlock(this: *DarwinImpl) void {
            std.c.os_unfair_lock_unlock(&this.oul);
        }

        pub const Type = std.c.os_unfair_lock;
    };

    const FutexImpl = struct {
        state: std.atomic.Value(u32) = std.atomic.Value(u32).init(unlocked),

        fn lock(this: *FutexImpl) void {
            if (!this.tryLock()) {
                this.lockSlow();
            }
        }

        fn tryLock(this: *FutexImpl) bool {
            if (comptime @import("builtin").target.cpu.arch.isX86()) {
                const locked_bit = @ctz(locked);
                return this.state.bitSet(locked_bit, .acquire) == 0;
            }

            return this.state.cmpxchgWeak(unlocked, locked, .acquire, .monotonic) == null;
        }

        fn lockSlow(this: *FutexImpl) void {
            @branchHint(.cold);

            if (this.state.load(.monotonic) == contended) {
                Futex.waitForever(&this.state, contended);
            }

            while (this.state.swap(contended, .acquire) != unlocked) {
                Futex.waitForever(&this.state, contended);
            }
        }

        fn unlock(this: *FutexImpl) void {
            const state = this.state.swap(unlocked, .release);
            std.debug.assert(state != unlocked);

            if (state == contended) {
                Futex.wake(&this.state, 1);
            }
        }

        pub const Type = u32;
    };

    /// Tries to acquire the mutex without blocking the caller's thread.
    pub fn tryLock(this: *Mutex): bool {
        return this.impl.tryLock();
    }

    /// Acquires the mutex, blocking the caller's thread until it can.
    pub fn lock(this: *Mutex): void {
        this.impl.lock();
    }

    /// Releases the mutex which was previously acquired with `lock()` or `tryLock()`.
    pub fn unlock(this: *Mutex): void {
        this.impl.unlock();
    }

    /// Scoped RAII lock execution helper
    pub fn withLock(this: *Mutex, cb: any): void {
        this.lock();
        defer this.unlock();
        cb();
    }

    pub fn release(this: *Mutex): void {
        _ = this;
    }
    pub fn deinit(this: *Mutex): void {
        this.release();
    }
}
