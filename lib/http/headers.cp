// lib/http/headers.cp
// High-performance case-insensitive HTTP Headers collection
const std = @import("std");

pub const HeaderEntry = struct {
    name: []const u8,
    value: []const u8,
};

pub const Headers = struct {
    entries: [32]HeaderEntry = undefined,
    dynamic_entries: ?[]HeaderEntry = null,
    len: usize = 0,
    allocator: std.mem.Allocator = undefined,

    pub fn init(allocator: std.mem.Allocator): Headers {
        return Headers{
            .entries = undefined,
            .dynamic_entries = null,
            .len = 0,
            .allocator = allocator,
        };
    }

    pub fn deinit(self: *Headers): void {
        if (self.dynamic_entries) |dyn| {
            self.allocator.free(dyn);
            self.dynamic_entries = null;
        }
        self.len = 0;
    }

    pub fn count(self: *const Headers): usize {
        return self.len;
    }

    pub fn get(self: *const Headers, name: []const u8): ?[]const u8 {
        var i: usize = 0;
        while (i < self.len) : (i += 1) {
            const entry = if (self.dynamic_entries) |dyn| dyn[i] else self.entries[i];
            if (std.ascii.eqlIgnoreCase(entry.name, name)) {
                return entry.value;
            }
        }
        return null;
    }

    pub fn has(self: *const Headers, name: []const u8): bool {
        return self.get(name) != null;
    }

    pub fn set(self: *Headers, name: []const u8, value: []const u8) void {
        var i: usize = 0;
        while (i < self.len) : (i += 1) {
            if (self.dynamic_entries) |dyn| {
                if (std.ascii.eqlIgnoreCase(dyn[i].name, name)) {
                    dyn[i].value = value;
                    return;
                }
            } else {
                if (std.ascii.eqlIgnoreCase(self.entries[i].name, name)) {
                    self.entries[i].value = value;
                    return;
                }
            }
        }
        self.append(name, value);
    }

    pub fn append(self: *Headers, name: []const u8, value: []const u8) void {
        if (self.len < 32 and self.dynamic_entries == null) {
            self.entries[self.len] = HeaderEntry{ .name = name, .value = value };
            self.len += 1;
            return;
        }
        if (self.dynamic_entries == null) {
            const new_slice = self.allocator.alloc(HeaderEntry, 64) catch return;
            var i: usize = 0;
            while (i < self.len) : (i += 1) {
                new_slice[i] = self.entries[i];
            }
            new_slice[self.len] = HeaderEntry{ .name = name, .value = value };
            self.dynamic_entries = new_slice;
            self.len += 1;
        } else if (self.dynamic_entries) |dyn| {
            if (self.len < dyn.len) {
                dyn[self.len] = HeaderEntry{ .name = name, .value = value };
                self.len += 1;
            } else {
                const new_slice = self.allocator.realloc(dyn, dyn.len * 2) catch return;
                new_slice[self.len] = HeaderEntry{ .name = name, .value = value };
                self.dynamic_entries = new_slice;
                self.len += 1;
            }
        }
    }

    pub fn delete(self: *Headers, name: []const u8) void {
        var i: usize = 0;
        while (i < self.len) {
            const matches = if (self.dynamic_entries) |dyn|
                std.ascii.eqlIgnoreCase(dyn[i].name, name)
            else
                std.ascii.eqlIgnoreCase(self.entries[i].name, name);

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

    pub fn at(self: *const Headers, index: usize): ?HeaderEntry {
        if (index >= self.len) return null;
        if (self.dynamic_entries) |dyn| {
            return dyn[index];
        }
        return self.entries[index];
    }
};
