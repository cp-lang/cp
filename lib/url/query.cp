// lib/url/query.cp
// URLSearchParams query string parser and serializer
const std = @import("std");

pub const QueryParam = struct {
    key: []const u8,
    value: []const u8,
};

pub const URLSearchParams = struct {
    entries: [32]QueryParam = undefined,
    dynamic_entries: ?[]QueryParam = null,
    len: usize = 0,
    allocator: std.mem.Allocator = undefined,

    pub fn init(allocator: std.mem.Allocator): URLSearchParams {
        return URLSearchParams{
            .entries = undefined,
            .dynamic_entries = null,
            .len = 0,
            .allocator = allocator,
        };
    }

    pub fn deinit(self: *URLSearchParams): void {
        if (self.dynamic_entries) |dyn| {
            self.allocator.free(dyn);
            self.dynamic_entries = null;
        }
        self.len = 0;
    }

    pub fn count(self: *const URLSearchParams): usize {
        return self.len;
    }

    pub fn get(self: *const URLSearchParams, key: []const u8): ?[]const u8 {
        var i: usize = 0;
        while (i < self.len) : (i += 1) {
            const entry = if (self.dynamic_entries) |dyn| dyn[i] else self.entries[i];
            if (std.mem.eql(u8, entry.key, key)) {
                return entry.value;
            }
        }
        return null;
    }

    pub fn has(self: *const URLSearchParams, key: []const u8): bool {
        return self.get(key) != null;
    }

    pub fn append(self: *URLSearchParams, key: []const u8, value: []const u8) void {
        if (self.len < 32 and self.dynamic_entries == null) {
            self.entries[self.len] = QueryParam{ .key = key, .value = value };
            self.len += 1;
            return;
        }
        if (self.dynamic_entries == null) {
            const new_slice = self.allocator.alloc(QueryParam, 64) catch return;
            var i: usize = 0;
            while (i < self.len) : (i += 1) {
                new_slice[i] = self.entries[i];
            }
            new_slice[self.len] = QueryParam{ .key = key, .value = value };
            self.dynamic_entries = new_slice;
            self.len += 1;
        } else if (self.dynamic_entries) |dyn| {
            if (self.len < dyn.len) {
                dyn[self.len] = QueryParam{ .key = key, .value = value };
                self.len += 1;
            } else {
                const new_slice = self.allocator.realloc(dyn, dyn.len * 2) catch return;
                new_slice[self.len] = QueryParam{ .key = key, .value = value };
                self.dynamic_entries = new_slice;
                self.len += 1;
            }
        }
    }

    pub fn set(self: *URLSearchParams, key: []const u8, value: []const u8) void {
        var i: usize = 0;
        while (i < self.len) : (i += 1) {
            if (self.dynamic_entries) |dyn| {
                if (std.mem.eql(u8, dyn[i].key, key)) {
                    dyn[i].value = value;
                    return;
                }
            } else {
                if (std.mem.eql(u8, self.entries[i].key, key)) {
                    self.entries[i].value = value;
                    return;
                }
            }
        }
        self.append(key, value);
    }

    pub fn delete(self: *URLSearchParams, key: []const u8) void {
        var i: usize = 0;
        while (i < self.len) {
            const matches = if (self.dynamic_entries) |dyn|
                std.mem.eql(u8, dyn[i].key, key)
            else
                std.mem.eql(u8, self.entries[i].key, key);

            if (matches) {
                var j = i;
                while (j + 1 < self.len) : (j += 1) {
                    if (self.dynamic_entries) |dyn| {
                        dyn[j] = dyn[j + 1];
                    } else {
                        self.entries[j] = self.entries[j + 1];
                    }
                }
                self.len -= 1;
            } else {
                i += 1;
            }
        }
    }

    pub fn parse(query_str: []const u8, allocator: std.mem.Allocator): URLSearchParams {
        var params = URLSearchParams.init(allocator);
        var q = query_str;
        if (q.len > 0 and q[0] == '?') q = q[1..];

        var iter = std.mem.splitScalar(u8, q, '&');
        while (iter.next()) |pair| {
            if (pair.len == 0) continue;
            var eq_idx: ?usize = null;
            var i: usize = 0;
            while (i < pair.len) : (i += 1) {
                if (pair[i] == '=') {
                    eq_idx = i;
                    break;
                }
            }
            if (eq_idx) |idx| {
                params.append(pair[0..idx], pair[idx + 1 ..]);
            } else {
                params.append(pair, "");
            }
        }
        return params;
    }

    pub fn toString(self: *const URLSearchParams, buf: *[1024]u8): []const u8 {
        var out_idx: usize = 0;
        var i: usize = 0;
        while (i < self.len) : (i += 1) {
            const entry = if (self.dynamic_entries) |dyn| dyn[i] else self.entries[i];
            if (i > 0 and out_idx < 1024) {
                buf[out_idx] = '&';
                out_idx += 1;
            }
            @memcpy(buf[out_idx .. out_idx + entry.key.len], entry.key);
            out_idx += entry.key.len;
            buf[out_idx] = '=';
            out_idx += 1;
            @memcpy(buf[out_idx .. out_idx + entry.value.len], entry.value);
            out_idx += entry.value.len;
        }
        return buf[0..out_idx];
    }
};
