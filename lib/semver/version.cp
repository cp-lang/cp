// lib/semver/version.cp
// Semver 2.0 Semantic Version specification in CP
const std = @import("std");

pub const Version = struct {
    major: u32 = 0,
    minor: u32 = 0,
    patch: u32 = 0,
    prerelease: []const u8 = "",
    build: []const u8 = "",

    pub fn parse(input_str: []const u8): !Version {
        var str = std.mem.trim(u8, input_str, " \t\r\n");
        if (str.len == 0) return error.InvalidVersion;

        // Strip leading 'v' or 'V' or '='
        if (str[0] == 'v' or str[0] == 'V' or str[0] == '=') {
            str = str[1..];
            str = std.mem.trim(u8, str, " \t\r\n");
        }
        if (str.len == 0) return error.InvalidVersion;

        var v = Version{};

        // Extract build metadata (+...)
        if (std.mem.indexOfScalar(u8, str, '+')) |plus_idx| {
            v.build = str[plus_idx + 1 ..];
            str = str[0..plus_idx];
        }

        // Extract prerelease (-...)
        if (std.mem.indexOfScalar(u8, str, '-')) |dash_idx| {
            v.prerelease = str[dash_idx + 1 ..];
            str = str[0..dash_idx];
        }

        // Parse major.minor.patch
        var iter = std.mem.splitScalar(u8, str, '.');

        const major_str = iter.next() orelse return error.InvalidVersion;
        v.major = std.fmt.parseInt(u32, major_str, 10) catch return error.InvalidVersion;

        if (iter.next()) |minor_str| {
            v.minor = std.fmt.parseInt(u32, minor_str, 10) catch return error.InvalidVersion;
            if (iter.next()) |patch_str| {
                v.patch = std.fmt.parseInt(u32, patch_str, 10) catch return error.InvalidVersion;
            } else {
                v.patch = 0;
            }
        } else {
            v.minor = 0;
            v.patch = 0;
        }

        if (iter.next() != null) return error.InvalidVersion;

        return v;
    }

    pub fn compare(a: Version, b: Version): std.math.Order {
        if (a.major != b.major) return if (a.major < b.major) .lt else .gt;
        if (a.minor != b.minor) return if (a.minor < b.minor) .lt else .gt;
        if (a.patch != b.patch) return if (a.patch < b.patch) .lt else .gt;

        // Prerelease comparison per Semver 2.0
        // A version without prerelease has higher precedence than with prerelease
        if (a.prerelease.len == 0 and b.prerelease.len == 0) return .eq;
        if (a.prerelease.len == 0 and b.prerelease.len > 0) return .gt;
        if (a.prerelease.len > 0 and b.prerelease.len == 0) return .lt;

        // Both have prereleases: compare dot-separated identifiers
        var it_a = std.mem.splitScalar(u8, a.prerelease, '.');
        var it_b = std.mem.splitScalar(u8, b.prerelease, '.');

        while (true) {
            const next_a = it_a.next();
            const next_b = it_b.next();

            if (next_a == null and next_b == null) return .eq;
            if (next_a == null) return .lt; // fewer parts = lower precedence
            if (next_b == null) return .gt;

            const part_a = next_a.?;
            const part_b = next_b.?;

            const num_a = std.fmt.parseInt(u64, part_a, 10) catch null;
            const num_b = std.fmt.parseInt(u64, part_b, 10) catch null;

            if (num_a != null and num_b != null) {
                // Both numeric: compare numerically
                if (num_a.? != num_b.?) {
                    return if (num_a.? < num_b.?) .lt else .gt;
                }
            } else if (num_a != null and num_b == null) {
                // Numeric identifier has lower precedence than non-numeric
                return .lt;
            } else if (num_a == null and num_b != null) {
                return .gt;
            } else {
                // Both strings: ASCII order
                const ord = std.mem.order(u8, part_a, part_b);
                if (ord != .eq) return ord;
            }
        }
    }

    pub fn eql(a: Version, b: Version): bool {
        return compare(a, b) == .eq;
    }

    pub fn lt(a: Version, b: Version): bool {
        return compare(a, b) == .lt;
    }

    pub fn lte(a: Version, b: Version): bool {
        const ord = compare(a, b);
        return ord == .lt or ord == .eq;
    }

    pub fn gt(a: Version, b: Version): bool {
        return compare(a, b) == .gt;
    }

    pub fn gte(a: Version, b: Version): bool {
        const ord = compare(a, b);
        return ord == .gt or ord == .eq;
    }

    pub fn format(self: Version, buf: []u8) []const u8 {
        var len: usize = 0;
        const main_part = std.fmt.bufPrint(buf, "{d}.{d}.{d}", .{ self.major, self.minor, self.patch }) catch return "";
        len += main_part.len;

        if (self.prerelease.len > 0) {
            if (len + 1 + self.prerelease.len > buf.len) return "";
            buf[len] = '-';
            len += 1;
            @memcpy(buf[len .. len + self.prerelease.len], self.prerelease);
            len += self.prerelease.len;
        }

        if (self.build.len > 0) {
            if (len + 1 + self.build.len > buf.len) return "";
            buf[len] = '+';
            len += 1;
            @memcpy(buf[len .. len + self.build.len], self.build);
            len += self.build.len;
        }

        return buf[0..len];
    }
};
