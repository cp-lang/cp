/**
 * CP HTTP Context & Keep-Alive Connection Pool (cap:http)
 * Ported from Bun.js http/HTTPContext.cp
 */

const std = @import("std");

pub const MAX_POOLED_SOCKETS: usize = 64;

export const PooledSocket = struct {
    fd: i32 = -1,
    hostname: []const u8 = "",
    port: u16 = 80,
    is_ssl: bool = false,
    in_use: bool = false,
    last_used: i64 = 0,

    pub fn isValid(self: PooledSocket) bool {
        return self.fd >= 0 and self.in_use;
    }

    pub fn close(self: *PooledSocket) void {
        if (self.fd >= 0) {
            _ = std.os.linux.close(self.fd);
            self.fd = -1;
        }
        self.in_use = false;
    }
};

export const HTTPContext = struct {
    pool: [MAX_POOLED_SOCKETS]PooledSocket = undefined,
    pool_count: usize = 0,
    max_idle_seconds: i64 = 300,

    pub fn init() HTTPContext {
        var ctx = HTTPContext{};
        ctx.pool_count = 0;
        ctx.max_idle_seconds = 300;
        var i: usize = 0;
        while (i < MAX_POOLED_SOCKETS) : (i += 1) {
            ctx.pool[i] = PooledSocket{};
        }
        return ctx;
    }

    pub fn acquireSocket(self: *HTTPContext, hostname: []const u8, port: u16, is_ssl: bool) ?i32 {
        var i: usize = 0;
        while (i < self.pool_count) : (i += 1) {
            if (!self.pool[i].in_use and self.pool[i].port == port and self.pool[i].is_ssl == is_ssl) {
                if (std.mem.eql(u8, self.pool[i].hostname, hostname)) {
                    self.pool[i].in_use = true;
                    return self.pool[i].fd;
                }
            }
        }
        return null;
    }

    pub fn releaseSocket(self: *HTTPContext, fd: i32, hostname: []const u8, port: u16, is_ssl: bool) bool {
        var i: usize = 0;
        // Check if already in pool
        while (i < self.pool_count) : (i += 1) {
            if (self.pool[i].fd == fd) {
                self.pool[i].in_use = false;
                self.pool[i].last_used = 0;
                return true;
            }
        }
        // Add new socket to pool if capacity allows
        if (self.pool_count < MAX_POOLED_SOCKETS) {
            self.pool[self.pool_count] = PooledSocket{
                .fd = fd,
                .hostname = hostname,
                .port = port,
                .is_ssl = is_ssl,
                .in_use = false,
                .last_used = 0,
            };
            self.pool_count += 1;
            return true;
        }
        return false;
    }

    pub fn closeDeadSocket(self: *HTTPContext, fd: i32) void {
        var i: usize = 0;
        while (i < self.pool_count) : (i += 1) {
            if (self.pool[i].fd == fd) {
                self.pool[i].close();
                // Shift remaining sockets
                var j: usize = i;
                while (j + 1 < self.pool_count) : (j += 1) {
                    self.pool[j] = self.pool[j + 1];
                }
                self.pool_count -= 1;
                return;
            }
        }
    }

    pub fn deinit(self: *HTTPContext) void {
        var i: usize = 0;
        while (i < self.pool_count) : (i += 1) {
            self.pool[i].close();
        }
        self.pool_count = 0;
    }
};
