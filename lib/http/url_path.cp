/**
 * CP HTTP URL Path Parser (cap:http)
 * Ported from Bun.js http_types/URLPath.cp
 */

const std = @import("std");

export const URLPath = struct {
    extname: []const u8 = "",
    path: []const u8 = "",
    pathname: []const u8 = "",
    first_segment: []const u8 = "",
    query_string: []const u8 = "",
    needs_redirect: bool = false,
    is_source_map: bool = false,

    pub fn isRoot(self: URLPath) bool {
        return self.path.len == 0 or (self.path.len == 1 and self.path[0] == '/') or std.mem.eql(u8, self.path, "index");
    }

    pub fn parse(raw_pathname: []const u8) URLPath {
        if (raw_pathname.len == 0) {
            return URLPath{};
        }

        var question_idx: i32 = -1;
        var period_idx: i32 = -1;
        var first_slash: i32 = -1;
        var last_slash: i32 = -1;

        const len = raw_pathname.len;
        var i: usize = 0;
        while (i < len) : (i += 1) {
            const c = raw_pathname[i];
            if (c == '?' and question_idx < 0) {
                question_idx = @as(i32, @intCast(i));
            } else if (c == '/' and question_idx < 0) {
                if (first_slash < 0 and i > 0) {
                    first_slash = @as(i32, @intCast(i));
                }
                last_slash = @as(i32, @intCast(i));
            } else if (c == '.' and question_idx < 0) {
                period_idx = @as(i32, @intCast(i));
            }
        }

        const pathname_end: usize = if (question_idx >= 0) @as(usize, @intCast(question_idx)) else len;
        const path_slice = raw_pathname[0..pathname_end];
        const query_slice = if (question_idx >= 0) raw_pathname[@as(usize, @intCast(question_idx))..len] else "";

        const ext_slice = if (period_idx > last_slash and period_idx >= 0)
            raw_pathname[@as(usize, @intCast(period_idx + 1))..pathname_end]
        else
            "";

        const first_seg_end: usize = if (first_slash > 0) @as(usize, @intCast(first_slash)) else pathname_end;
        const first_seg = if (first_seg_end > 1) raw_pathname[1..first_seg_end] else "";

        const is_map = std.mem.eql(u8, ext_slice, "map");

        var res = URLPath{};
        res.extname = ext_slice;
        res.path = path_slice;
        res.pathname = raw_pathname;
        res.first_segment = first_seg;
        res.query_string = query_slice;
        res.needs_redirect = false;
        res.is_source_map = is_map;
        return res;
    }
};
