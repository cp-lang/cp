// lib/which/which.cp
// Cross-platform executable binary locator in CP
const std = @import("std");
const builtin = @import("builtin");

pub const WhichOptions = struct {
    path: ?[]const u8 = null,
    cwd: ?[]const u8 = null,
};

pub const Which = struct {
    pub fn isExecutable(filepath: []const u8): bool {
        if (filepath.len == 0 or filepath.len >= 4096) return false;
        var path_z: [4096:0]u8 = undefined;
        @memcpy(path_z[0..filepath.len], filepath);
        path_z[filepath.len] = 0;

        if (builtin.os.tag == .windows) {
            const has_ext = std.mem.endsWith(u8, filepath, ".exe") or
                std.mem.endsWith(u8, filepath, ".cmd") or
                std.mem.endsWith(u8, filepath, ".bat");
            if (has_ext) {
                std.fs.accessAbsolute(filepath) catch return false;
                return true;
            }
            return false;
        } else {
            return std.c.access(&path_z, 1) == 0;
        }
    }

    pub fn which(bin: []const u8, options: WhichOptions, out_buf: []u8): ?[]const u8 {
        if (bin.len == 0 or bin.len >= out_buf.len) return null;

        // 1. Absolute path
        if (std.fs.path.isAbsolute(bin)) {
            if (Which.isExecutable(bin)) {
                @memcpy(out_buf[0..bin.len], bin);
                return out_buf[0..bin.len];
            }
            return null;
        }

        // 2. Relative path with separator (e.g. ./app or bin/tool)
        if (std.mem.indexOfScalar(u8, bin, '/') != null or std.mem.indexOfScalar(u8, bin, '\\') != null) {
            if (options.cwd) |cwd| {
                const combined = std.fmt.bufPrint(out_buf, "{s}/{s}", .{ std.mem.trimEnd(u8, cwd, "/\\"), bin }) catch return null;
                if (Which.isExecutable(combined)) {
                    return combined;
                }
            } else {
                if (Which.isExecutable(bin)) {
                    @memcpy(out_buf[0..bin.len], bin);
                    return out_buf[0..bin.len];
                }
            }
            return null;
        }

        // 3. Search in PATH
        const path_val = options.path orelse (if (std.c.getenv("PATH")) |p| std.mem.span(p) else return null);
        const sep: u8 = if (builtin.os.tag == .windows) ';' else ':';

        var iter = std.mem.tokenizeScalar(u8, path_val, sep);
        while (iter.next()) |segment| {
            if (segment.len == 0) continue;

            const candidate = std.fmt.bufPrint(out_buf, "{s}/{s}", .{ std.mem.trimEnd(u8, segment, "/\\"), bin }) catch continue;
            if (Which.isExecutable(candidate)) {
                return candidate;
            }

            // On Windows, also test .exe, .cmd, .bat if no extension
            if (builtin.os.tag == .windows and std.mem.indexOfScalar(u8, bin, '.') == null) {
                const exts = [_][]const u8{ ".exe", ".cmd", ".bat" };
                for (exts) |ext| {
                    const win_cand = std.fmt.bufPrint(out_buf, "{s}/{s}{s}", .{ std.mem.trimEnd(u8, segment, "/\\"), bin, ext }) catch continue;
                    if (Which.isExecutable(win_cand)) {
                        return win_cand;
                    }
                }
            }
        }

        return null;
    }
};
