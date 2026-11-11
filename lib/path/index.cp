// lib/path/index.cp
// Cross-platform path standard library for CP (cap:path)
const std = @import("std");
const builtin = @import("builtin");
import { PosixPath } from "./posix";
import { WindowsPath } from "./windows";

pub const Path = struct {
    pub const posix = PosixPath;
    pub const win32 = WindowsPath;

    pub const sep = if (builtin.os.tag == .windows) WindowsPath.sep else PosixPath.sep;
    pub const delimiter = if (builtin.os.tag == .windows) WindowsPath.delimiter else PosixPath.delimiter;

    pub fn isAbsolute(filepath: []const u8): bool {
        if (builtin.os.tag == .windows) {
            return WindowsPath.isAbsolute(filepath);
        } else {
            return PosixPath.isAbsolute(filepath);
        }
    }

    pub fn basename(filepath: []const u8, ext: ?[]const u8): []const u8 {
        if (builtin.os.tag == .windows) {
            return WindowsPath.basename(filepath, ext);
        } else {
            return PosixPath.basename(filepath, ext);
        }
    }

    pub fn dirname(filepath: []const u8): []const u8 {
        if (builtin.os.tag == .windows) {
            return WindowsPath.dirname(filepath);
        } else {
            return PosixPath.dirname(filepath);
        }
    }

    pub fn extname(filepath: []const u8): []const u8 {
        if (builtin.os.tag == .windows) {
            return WindowsPath.extname(filepath);
        } else {
            return PosixPath.extname(filepath);
        }
    }

    pub fn normalize(filepath: []const u8, buf: *[1024]u8): []const u8 {
        return PosixPath.normalize(filepath, buf);
    }

    pub fn join(paths: []const []const u8, buf: *[1024]u8): []const u8 {
        return PosixPath.join(paths, buf);
    }
};

pub const path = Path;
