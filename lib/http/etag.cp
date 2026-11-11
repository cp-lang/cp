// lib/http/etag.cp
// HTTP ETag and If-None-Match evaluation in CP
const std = @import("std");

pub const ParsedETag = struct {
    tag: []const u8,
    is_weak: bool,
};

pub const ETag = struct {
    pub fn parse(tag_str: []const u8): ParsedETag {
        var str = std.mem.trim(u8, tag_str, " \t");
        var is_weak = false;
        if (std.mem.startsWith(u8, str, "W/")) {
            is_weak = true;
            str = str[2..];
            str = std.mem.trimStart(u8, str, " \t");
        }
        if (str.len >= 2 and str[0] == '"' and str[str.len - 1] == '"') {
            str = str[1 .. str.len - 1];
        }
        return ParsedETag{
            .tag = str,
            .is_weak = is_weak,
        };
    }

    pub fn ifNoneMatch(etag: []const u8, if_none_match: []const u8): bool {
        const our_parsed = parse(etag);
        if (std.mem.eql(u8, std.mem.trim(u8, if_none_match, " \t"), "*")) {
            return true;
        }

        var iter = std.mem.splitScalar(u8, if_none_match, ',');
        while (iter.next()) |tag_str| {
            const parsed = parse(tag_str);
            if (std.mem.eql(u8, our_parsed.tag, parsed.tag)) {
                return true;
            }
        }
        return false;
    }

    pub fn generate(bytes: []const u8, out_buf: *[32]u8): []const u8 {
        const hash = std.hash.Wyhash.hash(0, bytes);
        return std.fmt.bufPrint(out_buf, "\"{x}\"", .{hash}) catch "\"0\"";
    }
};
