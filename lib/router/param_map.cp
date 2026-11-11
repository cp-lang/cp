// lib/router/param_map.cp
// High-performance route parameter dictionary with inline stack storage
const std = @import("std");

pub const Param = struct {
    name: []const u8,
    value: []const u8,
};

pub const ParamMap = struct {
    entries: [16]Param = undefined,
    len: usize = 0,

    pub fn init(): ParamMap {
        return ParamMap{
            .entries = undefined,
            .len = 0,
        };
    }

    pub fn count(self: *const ParamMap): usize {
        return self.len;
    }

    pub fn isEmpty(self: *const ParamMap): bool {
        return self.len == 0;
    }

    pub fn get(self: *const ParamMap, name: []const u8): ?[]const u8 {
        var i: usize = 0;
        while (i < self.len) : (i += 1) {
            if (std.mem.eql(u8, self.entries[i].name, name)) {
                return self.entries[i].value;
            }
        }
        return null;
    }

    pub fn at(self: *const ParamMap, index: usize): ?Param {
        if (index >= self.len) return null;
        return self.entries[index];
    }

    pub fn has(self: *const ParamMap, name: []const u8): bool {
        return self.get(name) != null;
    }

    pub fn append(self: *ParamMap, name: []const u8, value: []const u8): void {
        if (self.len < 16) {
            self.entries[self.len] = Param{ .name = name, .value = value };
            self.len += 1;
        }
    }
};
