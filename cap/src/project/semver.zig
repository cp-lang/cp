const std = @import("std");

pub const Version = struct {
    major: u32,
    minor: u32,
    patch: u32,

    pub fn parse(text: []const u8) !Version {
        var it = std.mem.splitScalar(u8, text, '.');
        const major = try std.fmt.parseInt(u32, it.next() orelse "0", 10);
        const minor = try std.fmt.parseInt(u32, it.next() orelse "0", 10);
        const patch = try std.fmt.parseInt(u32, it.next() orelse "0", 10);
        return Version{ .major = major, .minor = minor, .patch = patch };
    }

    pub fn format(self: Version, allocator: std.mem.Allocator) ![]const u8 {
        return std.fmt.allocPrint(allocator, "{d}.{d}.{d}", .{self.major, self.minor, self.patch});
    }

    pub fn compare(self: Version, other: Version) i8 {
        if (self.major > other.major) return 1;
        if (self.major < other.major) return -1;
        if (self.minor > other.minor) return 1;
        if (self.minor < other.minor) return -1;
        if (self.patch > other.patch) return 1;
        if (self.patch < other.patch) return -1;
        return 0;
    }
};

pub const VersionRange = struct {
    kind: enum { Exact, Caret, Tilde },
    version: Version,

    pub fn parse(text: []const u8) !VersionRange {
        if (text.len == 0) return error.EmptyVersion;
        if (text[0] == '^') {
            return VersionRange{ .kind = .Caret, .version = try Version.parse(text[1..]) };
        } else if (text[0] == '~') {
            return VersionRange{ .kind = .Tilde, .version = try Version.parse(text[1..]) };
        } else {
            return VersionRange{ .kind = .Exact, .version = try Version.parse(text) };
        }
    }

    pub fn satisfies(self: VersionRange, v: Version) bool {
        switch (self.kind) {
            .Exact => return self.version.compare(v) == 0,
            .Caret => {
                // ^1.2.3 means >=1.2.3 <2.0.0
                if (v.major != self.version.major) return false;
                return v.compare(self.version) >= 0;
            },
            .Tilde => {
                // ~1.2.3 means >=1.2.3 <1.3.0
                if (v.major != self.version.major) return false;
                if (v.minor != self.version.minor) return false;
                return v.compare(self.version) >= 0;
            },
        }
    }
};
