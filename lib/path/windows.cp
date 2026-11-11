// lib/path/windows.cp
// Windows path utilities in CP
const std = @import("std");

pub const WindowsPath = struct {
    pub const sep: u8 = '\\';
    pub const sep_str: []const u8 = "\\";
    pub const delimiter: u8 = ';';

    pub fn isSeparator(c: u8): bool {
        return c == '/' or c == '\\';
    }

    pub fn isAbsolute(filepath: []const u8): bool {
        if (filepath.len >= 2 and isSeparator(filepath[0]) and isSeparator(filepath[1])) {
            return true; // UNC path \\server\share
        }
        if (filepath.len >= 3 and std.ascii.isAlphabetic(filepath[0]) and filepath[1] == ':' and isSeparator(filepath[2])) {
            return true; // Drive root C:\
        }
        return false;
    }

    pub fn basename(filepath: []const u8, ext: ?[]const u8): []const u8 {
        var p = filepath;
        while (p.len > 1 and isSeparator(p[p.len - 1])) {
            p = p[0 .. p.len - 1];
        }
        if (p.len == 0) return "";

        var last_slash: ?usize = null;
        var i: usize = p.len;
        while (i > 0) {
            i -= 1;
            if (isSeparator(p[i])) {
                last_slash = i;
                break;
            }
        }

        var base = if (last_slash) |idx| p[idx + 1 ..] else p;
        if (ext) |e| {
            if (e.len > 0 and std.ascii.endsWithIgnoreCase(base, e) and base.len > e.len) {
                base = base[0 .. base.len - e.len];
            }
        }
        return base;
    }

    pub fn dirname(filepath: []const u8): []const u8 {
        var p = filepath;
        while (p.len > 1 and isSeparator(p[p.len - 1])) {
            p = p[0 .. p.len - 1];
        }
        if (p.len == 0) return ".";

        var last_slash: ?usize = null;
        var i: usize = p.len;
        while (i > 0) {
            i -= 1;
            if (isSeparator(p[i])) {
                last_slash = i;
                break;
            }
        }

        if (last_slash) |idx| {
            if (idx == 0) return "\\";
            if (idx == 2 and p[1] == ':') return p[0..3]; // C:\
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
                if (i == 0) return "";
                return base[i..];
            }
        }
        return "";
    }
};
