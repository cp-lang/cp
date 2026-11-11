// lib/path/posix.cp
// POSIX path utilities in CP
const std = @import("std");

pub const PosixPath = struct {
    pub const sep: u8 = '/';
    pub const sep_str: []const u8 = "/";
    pub const delimiter: u8 = ':';

    pub fn isAbsolute(filepath: []const u8): bool {
        return filepath.len > 0 and filepath[0] == '/';
    }

    pub fn basename(filepath: []const u8, ext: ?[]const u8): []const u8 {
        var p = filepath;
        // Trim trailing slashes
        while (p.len > 1 and p[p.len - 1] == '/') {
            p = p[0 .. p.len - 1];
        }
        if (p.len == 0) return "";
        if (p.len == 1 and p[0] == '/') return "/";

        var last_slash: ?usize = null;
        var i: usize = p.len;
        while (i > 0) {
            i -= 1;
            if (p[i] == '/') {
                last_slash = i;
                break;
            }
        }

        var base = if (last_slash) |idx| p[idx + 1 ..] else p;
        if (ext) |e| {
            if (e.len > 0 and std.mem.endsWith(u8, base, e) and base.len > e.len) {
                base = base[0 .. base.len - e.len];
            }
        }
        return base;
    }

    pub fn dirname(filepath: []const u8): []const u8 {
        var p = filepath;
        while (p.len > 1 and p[p.len - 1] == '/') {
            p = p[0 .. p.len - 1];
        }
        if (p.len == 0 or std.mem.eql(u8, p, ".")) return ".";
        if (p.len == 1 and p[0] == '/') return "/";

        var last_slash: ?usize = null;
        var i: usize = p.len;
        while (i > 0) {
            i -= 1;
            if (p[i] == '/') {
                last_slash = i;
                break;
            }
        }

        if (last_slash) |idx| {
            if (idx == 0) return "/";
            return p[0..idx];
        }
        return ".";
    }

    pub fn extname(filepath: []const u8): []const u8 {
        const base = basename(filepath, null);
        if (base.len <= 1) return "";

        var i: usize = base.len;
        while (i > 0) {
            i -= 1;
            if (base[i] == '.') {
                if (i == 0) return ""; // dotfiles like .gitignore have no ext
                return base[i..];
            }
        }
        return "";
    }

    pub fn normalize(filepath: []const u8, buf: *[1024]u8): []const u8 {
        if (filepath.len == 0) return ".";

        const is_abs = isAbsolute(filepath);
        var segments: [64][]const u8 = undefined;
        var seg_count: usize = 0;

        var iter = std.mem.splitScalar(u8, filepath, '/');
        while (iter.next()) |seg| {
            if (seg.len == 0 or std.mem.eql(u8, seg, ".")) {
                continue;
            }
            if (std.mem.eql(u8, seg, "..")) {
                if (seg_count > 0 and !std.mem.eql(u8, segments[seg_count - 1], "..")) {
                    seg_count -= 1;
                } else if (!is_abs and seg_count < 64) {
                    segments[seg_count] = "..";
                    seg_count += 1;
                }
            } else if (seg_count < 64) {
                segments[seg_count] = seg;
                seg_count += 1;
            }
        }

        if (seg_count == 0) {
            return if (is_abs) "/" else ".";
        }

        var out_idx: usize = 0;
        if (is_abs) {
            buf[out_idx] = '/';
            out_idx += 1;
        }

        var s_i: usize = 0;
        while (s_i < seg_count) : (s_i += 1) {
            if (s_i > 0 or is_abs) {
                if (out_idx > 1 or (!is_abs and out_idx > 0)) {
                    buf[out_idx] = '/';
                    out_idx += 1;
                }
            }
            const seg = segments[s_i];
            @memcpy(buf[out_idx .. out_idx + seg.len], seg);
            out_idx += seg.len;
        }

        return buf[0..out_idx];
    }

    pub fn join(paths: []const []const u8, buf: *[1024]u8): []const u8 {
        var joined_buf: [1024]u8 = undefined;
        var joined_len: usize = 0;

        for (paths) |p| {
            if (p.len == 0) continue;
            if (joined_len > 0 and joined_buf[joined_len - 1] != '/' and p[0] != '/') {
                joined_buf[joined_len] = '/';
                joined_len += 1;
            }
            @memcpy(joined_buf[joined_len .. joined_len + p.len], p);
            joined_len += p.len;
        }

        return normalize(joined_buf[0..joined_len], buf);
    }
};
