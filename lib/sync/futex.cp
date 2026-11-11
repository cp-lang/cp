// Futex thread synchronization primitive in CP
// Ported from Bun.js core threading architecture
const std = @import("std");

export struct Futex {
    pub const UnsupportedImpl = struct {
        fn wait(ptr: *const std.atomic.Value(u32), expect: u32, timeout: ?u64) error{Timeout}!void {
            return unsupported(.{ ptr, expect, timeout });
        }

        fn wake(ptr: *const std.atomic.Value(u32), max_waiters: u32) void {
            return unsupported(.{ ptr, max_waiters });
        }

        fn unsupported(unused: anytype) noreturn {
            _ = unused;
            @compileError("Unsupported operating system " ++ @tagName(@import("builtin").target.os.tag));
        }
    };

    pub const WindowsImpl = struct {
        fn wait(ptr: *const std.atomic.Value(u32), expect: u32, timeout: ?u64) error{Timeout}!void {
            var timeout_value: std.os.windows.LARGE_INTEGER = undefined;
            var timeout_ptr: ?*const std.os.windows.LARGE_INTEGER = null;

            if (timeout) |delay| {
                timeout_value = @as(std.os.windows.LARGE_INTEGER, @intCast(delay / 100));
                timeout_value = -timeout_value;
                timeout_ptr = &timeout_value;
            }

            const rc = std.os.windows.ntdll.RtlWaitOnAddress(
                ptr,
                &expect,
                @sizeOf(@TypeOf(expect)),
                timeout_ptr,
            );

            switch (rc) {
                .SUCCESS => {},
                .TIMEOUT => {
                    std.debug.assert(timeout != null);
                    return error.Timeout;
                },
                else => @panic("Unexpected RtlWaitOnAddress() return code"),
            }
        }

        fn wake(ptr: *const std.atomic.Value(u32), max_waiters: u32) void {
            const address: ?*const anyopaque = ptr;
            std.debug.assert(max_waiters != 0);

            switch (max_waiters) {
                1 => std.os.windows.ntdll.RtlWakeAddressSingle(address),
                else => std.os.windows.ntdll.RtlWakeAddressAll(address),
            }
        }
    };

    pub const DarwinImpl = struct {
        fn wait(ptr: *const std.atomic.Value(u32), expect: u32, timeout: ?u64) error{Timeout}!void {
            const supports_ulock_wait2 = @import("builtin").target.os.version_range.semver.min.major >= 11;

            var timeout_ns: u64 = 0;
            if (timeout) |delay| {
                std.debug.assert(delay != 0);
                timeout_ns = delay;
            }

            var timeout_overflowed = false;

            const addr: *const anyopaque = ptr;
            const flags: std.c.UL = .{
                .op = .COMPARE_AND_WAIT,
                .NO_ERRNO = true,
            };
            const status = blk: {
                if (supports_ulock_wait2) {
                    break :blk std.c.__ulock_wait2(flags, addr, expect, timeout_ns, 0);
                }

                const timeout_us: u32 = if (timeout_ns == 0)
                    0
                else
                    std.math.cast(u32, (timeout_ns + 999) / 1000) orelse blk_overflow: {
                        timeout_overflowed = true;
                        break :blk_overflow std.math.maxInt(u32);
                    };

                break :blk std.c.__ulock_wait(flags, addr, expect, timeout_us);
            };

            if (status >= 0) return;
            switch (@as(std.c.E, @enumFromInt(-status))) {
                .INTR => {},
                .FAULT => @panic("__ulock_wait() returned EFAULT unexpectedly"),
                .NOENT => {},
                .TIMEDOUT => {
                    if (timeout_overflowed) return;
                    std.debug.assert(timeout != null);
                    return error.Timeout;
                },
                else => @panic("Unexpected __ulock_wait() return code"),
            }
        }

        fn wake(ptr: *const std.atomic.Value(u32), max_waiters: u32) void {
            const addr: *const anyopaque = ptr;
            const flags: std.c.UL = .{
                .op = .COMPARE_AND_WAIT,
                .WAKE_ALL = max_waiters > 1,
                .NO_ERRNO = true,
            };

            const status = std.c.__ulock_wake(flags, addr, 0);
            if (status >= 0) return;
            switch (@as(std.c.E, @enumFromInt(-status))) {
                .NOENT => {},
                .FAULT => @panic("__ulock_wake() returned EFAULT unexpectedly"),
                else => @panic("Unexpected __ulock_wake() return code"),
            }
        }
    };

    pub const LinuxImpl = struct {
        fn wait(ptr: *const std.atomic.Value(u32), expect: u32, timeout: ?u64) error{Timeout}!void {
            var timeout_timespec: std.os.linux.timespec = undefined;
            var timeout_ptr: ?*const std.os.linux.timespec = null;

            if (timeout) |timeout_ns| {
                timeout_timespec = .{
                    .sec = @intCast(timeout_ns / std.time.ns_per_s),
                    .nsec = @intCast(timeout_ns % std.time.ns_per_s),
                };
                timeout_ptr = &timeout_timespec;
            }

            const rc = std.os.linux.futex_4arg(
                &ptr.raw,
                .{ .cmd = .WAIT, .private = true },
                expect,
                timeout_ptr,
            );

            switch (std.posix.errno(rc)) {
                .SUCCESS => {},
                .INVAL => {},
                .TIMEDOUT => {
                    std.debug.assert(timeout != null);
                    return error.Timeout;
                },
                .INTR => {},
                .AGAIN => {},
                .FAULT => @panic("futex_wait() returned EFAULT unexpectedly"),
                else => @panic("Unexpected futex_wait() return code"),
            }
        }

        fn wake(ptr: *const std.atomic.Value(u32), max_waiters: u32) void {
            const rc = std.os.linux.futex_3arg(
                &ptr.raw,
                .{ .cmd = .WAKE, .private = true },
                @bitCast(std.math.cast(i32, max_waiters) orelse std.math.maxInt(i32)),
            );

            switch (std.posix.errno(rc)) {
                .SUCCESS => {},
                .INVAL => {},
                .FAULT => @panic("futex_wake() returned EFAULT unexpectedly"),
                else => @panic("Unexpected futex_wake() return code"),
            }
        }
    };

    pub const FreebsdImpl = struct {
        fn wait(ptr: *const std.atomic.Value(u32), expect: u32, timeout: ?u64) error{Timeout}!void {
            var tm_size: usize = 0;
            var tm: std.c._umtx_time = undefined;
            var tm_ptr: ?*const std.c._umtx_time = null;

            if (timeout) |timeout_ns| {
                tm_ptr = &tm;
                tm_size = @sizeOf(@TypeOf(tm));

                tm.flags = 0;
                tm.clockid = .MONOTONIC;
                tm.timeout.sec = @as(@TypeOf(tm.timeout.sec), @intCast(timeout_ns / std.time.ns_per_s));
                tm.timeout.nsec = @as(@TypeOf(tm.timeout.nsec), @intCast(timeout_ns % std.time.ns_per_s));
            }

            const rc = std.c._umtx_op(
                @intFromPtr(&ptr.raw),
                @intFromEnum(std.c.UMTX_OP.WAIT_UINT_PRIVATE),
                @as(std.c.c_ulong, expect),
                tm_size,
                @intFromPtr(tm_ptr),
            );

            switch (std.posix.errno(rc)) {
                .SUCCESS => {},
                .FAULT => @panic("_umtx_op() WAIT returned EFAULT unexpectedly"),
                .INVAL => {},
                .TIMEDOUT => {
                    std.debug.assert(timeout != null);
                    return error.Timeout;
                },
                .INTR => {},
                else => @panic("Unexpected _umtx_op() WAIT return code"),
            }
        }

        fn wake(ptr: *const std.atomic.Value(u32), max_waiters: u32) void {
            const n: std.c.c_ulong = @min(max_waiters, std.math.maxInt(std.c.c_int));
            const rc = std.c._umtx_op(@intFromPtr(&ptr.raw), @intFromEnum(std.c.UMTX_OP.WAKE_PRIVATE), n, 0, 0);
            switch (std.posix.errno(rc)) {
                .SUCCESS => {},
                .FAULT => {},
                .INVAL => @panic("_umtx_op() WAKE returned EINVAL unexpectedly"),
                else => @panic("Unexpected _umtx_op() WAKE return code"),
            }
        }
    };

    pub const WasmImpl = struct {
        fn wait(ptr: *const std.atomic.Value(u32), expect: u32, timeout: ?u64) !void {
            if (!comptime std.Target.wasm.featureSetHas(@import("builtin").target.cpu.features, .atomics)) {
                @compileError("WASI target missing cpu feature 'atomics'");
            }
            const to: i64 = if (timeout) |t| @intCast(t) else -1;
            const result = asm volatile (
                "local.get %[ptr]\nlocal.get %[expected]\nlocal.get %[timeout]\nmemory.atomic.wait32 0\nlocal.set %[ret]"
                : [ret] "=r" (-> u32),
                : [ptr] "r" (&ptr.raw),
                  [expected] "r" (@as(i32, @bitCast(expect))),
                  [timeout] "r" (to),
            );
            switch (result) {
                0 => {},
                1 => {},
                2 => return error.Timeout,
                else => @panic("Unexpected memory.atomic.wait32() return code"),
            }
        }

        fn wake(ptr: *const std.atomic.Value(u32), max_waiters: u32) void {
            if (!comptime std.Target.wasm.featureSetHas(@import("builtin").target.cpu.features, .atomics)) {
                @compileError("WASI target missing cpu feature 'atomics'");
            }
            std.debug.assert(max_waiters != 0);
            const woken_count = asm volatile (
                "local.get %[ptr]\nlocal.get %[waiters]\nmemory.atomic.notify 0\nlocal.set %[ret]"
                : [ret] "=r" (-> u32),
                : [ptr] "r" (&ptr.raw),
                  [waiters] "r" (max_waiters),
            );
            _ = woken_count;
        }
    };

    pub const Impl = switch (@import("builtin").os.tag) {
        .windows => WindowsImpl,
        .linux => LinuxImpl,
        .freebsd => FreebsdImpl,
        else => if (@import("builtin").os.tag.isDarwin())
            DarwinImpl
        else if (@import("builtin").target.isWasm())
            WasmImpl
        else
            UnsupportedImpl,
    };

    pub static fn wait(ptr: *const std.atomic.Value(u32), expect: u32, timeout_ns: ?u64): !void {
        if (timeout_ns) |t| {
            if (t == 0) {
                if (ptr.load(.seq_cst) != expect) return;
                return error.Timeout;
            }
        }
        return Impl.wait(ptr, expect, timeout_ns);
    }

    pub static fn waitForever(ptr: *const std.atomic.Value(u32), expect: u32) void {
        zig {
            Impl.wait(ptr, expect, null) catch unreachable;
        }
    }

    pub static fn wake(ptr: *const std.atomic.Value(u32), max_waiters: u32): void {
        if (max_waiters == 0) return;
        Impl.wake(ptr, max_waiters);
    }
}

export struct Deadline {
    timeout: ?u64,
    started_ns: u64,

    pub static fn init(expires_in_ns: ?u64): Deadline {
        var start: u64 = 0;
        if (expires_in_ns != null) {
            var ts: std.os.linux.timespec = undefined;
            _ = std.os.linux.clock_gettime(.MONOTONIC, &ts);
            start = @as(u64, @intCast(ts.sec)) * std.time.ns_per_s + @as(u64, @intCast(ts.nsec));
        }
        return Deadline{
            .timeout = expires_in_ns,
            .started_ns = start,
        };
    }

    pub fn wait(self: *Deadline, ptr: *const std.atomic.Value(u32), expect: u32): !void {
        if (self.timeout == null) {
            Futex.waitForever(ptr, expect);
            return;
        }
        const timeout_ns = self.timeout.?;
        var ts: std.os.linux.timespec = undefined;
        _ = std.os.linux.clock_gettime(.MONOTONIC, &ts);
        const now = @as(u64, @intCast(ts.sec)) * std.time.ns_per_s + @as(u64, @intCast(ts.nsec));
        const elapsed_ns = if (now > self.started_ns) now - self.started_ns else 0;
        const until_timeout_ns = if (timeout_ns > elapsed_ns) timeout_ns - elapsed_ns else 0;
        return Futex.wait(ptr, expect, until_timeout_ns);
    }
}
