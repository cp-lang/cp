/**
 * CP HTTP Header Value Iterator (cap:http)
 * Ported from Bun.js http/HeaderValueIterator.cp
 */

const std = @import("std");

export const HeaderValueIterator = struct {
    raw: []const u8 = "",
    pos: usize = 0,

    pub fn init(input: []const u8) HeaderValueIterator {
        var it = HeaderValueIterator{};
        it.raw = std.mem.trim(u8, input, " \t");
        it.pos = 0;
        return it;
    }

    pub fn next(self: *HeaderValueIterator) ?[]const u8 {
        if (self.pos >= self.raw.len) {
            return null;
        }

        const start = self.pos;
        var end = start;
        while (end < self.raw.len and self.raw[end] != ',') : (end += 1) {}

        const slice = std.mem.trim(u8, self.raw[start..end], " \t");
        self.pos = if (end < self.raw.len) end + 1 else self.raw.len;

        if (slice.len == 0) {
            return self.next();
        }
        return slice;
    }
};
