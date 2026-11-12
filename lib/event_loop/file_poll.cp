// lib/event_loop/file_poll.cp
// Cross-platform I/O file descriptor poller based on std.posix.poll

const std = @import("std");

pub const PollEvents = struct {
    pub const Readable: u32 = 1;
    pub const Writable: u32 = 2;
    pub const Error: u32 = 4;
    pub const Hup: u32 = 8;
};

export type PollCallback = *const fn (fd: i32, events: u32, ctx: ?*anyopaque) void;

export struct FilePollRegistration {
    pub fd: i32 = -1;
    pub events: u32 = 0;
    pub callback: PollCallback = undefined;
    pub ctx: ?*anyopaque = null;
}

export class PollEngine {
    regs: []FilePollRegistration = &.{};
    poll_fds: []std.posix.pollfd = &.{};
    capacity: usize = 0;
    len: usize = 0;

    pub static fn init(): *PollEngine {
        let pe = std.heap.page_allocator.create(PollEngine) catch unreachable;
        pe.capacity = 32;
        pe.len = 0;
        pe.regs = (std.heap.page_allocator.alloc(FilePollRegistration, 32) catch unreachable);
        pe.poll_fds = (std.heap.page_allocator.alloc(std.posix.pollfd, 32) catch unreachable);
        return pe;
    }

    pub fn count(self: *PollEngine): usize {
        return self.len;
    }

    pub fn register(self: *PollEngine, fd: i32, events: u32, cb: PollCallback, ctx: ?*anyopaque): bool {
        // If already registered, update
        for (0..self.len) |i| {
            if (self.regs[i].fd == fd) {
                self.regs[i].events = events;
                self.regs[i].callback = cb;
                self.regs[i].ctx = ctx;
                var z_events: i16 = 0;
                if ((events & PollEvents.Readable) != 0) z_events |= std.posix.POLL.IN;
                if ((events & PollEvents.Writable) != 0) z_events |= std.posix.POLL.OUT;
                self.poll_fds[i].events = z_events;
                return true;
            }
        }

        if (self.len >= self.capacity) {
            const new_cap = self.capacity * 2;
            var new_regs = std.heap.page_allocator.alloc(FilePollRegistration, new_cap) catch unreachable;
            var new_fds = std.heap.page_allocator.alloc(std.posix.pollfd, new_cap) catch unreachable;
            for (0..self.len) |i| {
                new_regs[i] = self.regs[i];
                new_fds[i] = self.poll_fds[i];
            }
            std.heap.page_allocator.free(self.regs);
            std.heap.page_allocator.free(self.poll_fds);
            self.regs = new_regs;
            self.poll_fds = new_fds;
            self.capacity = new_cap;
        }

        const idx = self.len;
        self.len += 1;
        self.regs[idx].fd = fd;
        self.regs[idx].events = events;
        self.regs[idx].callback = cb;
        self.regs[idx].ctx = ctx;

        var z_events: i16 = 0;
        if ((events & PollEvents.Readable) != 0) z_events |= std.posix.POLL.IN;
        if ((events & PollEvents.Writable) != 0) z_events |= std.posix.POLL.OUT;

        self.poll_fds[idx].fd = fd;
        self.poll_fds[idx].events = z_events;
        self.poll_fds[idx].revents = 0;
        return true;
    }

    pub fn unregister(self: *PollEngine, fd: i32): bool {
        if (self.len == 0) return false;
        for (0..self.len) |i| {
            if (self.regs[i].fd == fd) {
                self.len -= 1;
                if (i < self.len) {
                    self.regs[i] = self.regs[self.len];
                    self.poll_fds[i] = self.poll_fds[self.len];
                }
                return true;
            }
        }
        return false;
    }

    pub fn poll(self: *PollEngine, timeout_ms: i32): usize {
        if (self.len == 0) return 0;

        let slice = self.poll_fds[0..self.len];
        for (0..self.len) |i| {
            slice[i].revents = 0;
        }

        const ready_count: usize = std.posix.poll(slice, timeout_ms) catch @as(usize, 0);
        if (ready_count == 0) return 0;

        var dispatched: usize = 0;
        var i: usize = 0;
        while (i < self.len) {
            const revents = self.poll_fds[i].revents;
            if (revents != 0) {
                var flags: u32 = 0;
                if ((revents & std.posix.POLL.IN) != 0) flags |= PollEvents.Readable;
                if ((revents & std.posix.POLL.OUT) != 0) flags |= PollEvents.Writable;
                if ((revents & std.posix.POLL.ERR) != 0) flags |= PollEvents.Error;
                if ((revents & std.posix.POLL.HUP) != 0) flags |= PollEvents.Hup;

                const reg = self.regs[i];
                reg.callback(reg.fd, flags, reg.ctx);
                dispatched += 1;
            }
            i += 1;
        }
        return dispatched;
    }
}
