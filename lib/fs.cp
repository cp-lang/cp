/**
 * CP File System SDK (cap:fs)
 * 使用 @is_beam 进行显式双模分支逻辑重构
 */
const std = @import("std");
const builtin = @import("builtin");

import { SystemAction } from "cap:beam";

/**
 * 生产级异步读取
 * 逻辑对开发者完全透明
 */
export async fn readFile(path: string): string {
    if (@is_beam) {
        // --- BEAM 模式：委托 I/O 网关 ---
        // 发送 FS_READ 请求并让出执行权，等待唤醒
        return await @yield_to_beam(@intFromEnum(SystemAction.FS_READ), path);
    } else {
        // --- Native 模式：直连 Zig std.fs ---
        // 在原生 Reactor 环境下，此操作由编译器映射为同步读取或非阻塞 Poll
        return @fs_read_file(path);
    }
}

/**
 * 生产级异步写入
 */
export async fn writeFile(path: string, content: string): void {
    if (@is_beam) {
        // 构造包含 path 和 content 的载荷（此处简化为 path）
        // 真实的复杂载荷将通过 SharedTensor 或专用编码处理
        await @yield_to_beam(@intFromEnum(SystemAction.FS_WRITE), path);
    } else {
        @fs_write_file(path, content);
    }
}

/**
 * 生产级同步读取
 */
export fn readFileSync(path: string): string {
    return @fs_read_file(path);
}

/**
 * 生产级同步写入
 */
export fn writeFileSync(path: string, content: string): void {
    @fs_write_file(path, content);
}

/**
 * 目录操作
 */
export fn mkdir(path: string): void {
    @fs_mkdir(path);
}

export fn exists(path: string): bool {
    return @fs_exists(path);
}

export fn remove(path: string): void {
    @fs_remove(path);
}

export fn rmdir(path: string): void {
    @fs_rmdir(path);
}

export fn rename(oldPath: string, newPath: string): void {
    @fs_rename(oldPath, newPath);
}

export fn copyFile(src: string, dest: string): void {
    @fs_copy_file(src, dest);
}

// 异步版
export async fn removeAsync(path: string): void {
    if (@is_beam) {
        await @yield_to_beam(@intFromEnum(SystemAction.FS_REMOVE), path);
    } else {
        @fs_remove(path);
    }
}

export async fn rmdirAsync(path: string): void {
    if (@is_beam) {
        await @yield_to_beam(@intFromEnum(SystemAction.FS_RMDIR), path);
    } else {
        @fs_rmdir(path);
    }
}

export async fn renameAsync(oldPath: string, newPath: string): void {
    if (@is_beam) {
        // Mock payload passing
        await @yield_to_beam(@intFromEnum(SystemAction.FS_RENAME), oldPath);
    } else {
        @fs_rename(oldPath, newPath);
    }
}

export async fn copyFileAsync(src: string, dest: string): void {
    if (@is_beam) {
        await @yield_to_beam(@intFromEnum(SystemAction.FS_COPY), src);
    } else {
        @fs_copy_file(src, dest);
    }
}

export fn statSize(path: string): i32 {
    return @fs_stat_size(path);
}

export fn isDirectory(path: string): bool {
    return @fs_is_dir(path);
}

var tmp_counter: std.atomic.Value(u64) = std.atomic.Value(u64).init(1);

pub const FileSystem = struct {
    pub fn tmpDir(): []const u8 {
        if (std.c.getenv("TMPDIR")) |p| return std.mem.span(p);
        if (std.c.getenv("TEMP")) |p| return std.mem.span(p);
        if (std.c.getenv("TMP")) |p| return std.mem.span(p);
        if (builtin.os.tag == .windows) return "C:\\Temp";
        return "/tmp";
    }

    pub fn tmpPath(prefix: []const u8, suffix: []const u8, dest: []u8): []const u8 {
        const dir = FileSystem.tmpDir();
        const id = tmp_counter.fetchAdd(1, .monotonic);
        var seed: u64 = id;
        if (builtin.os.tag == .linux) {
            var ts: std.os.linux.timespec = undefined;
            _ = std.os.linux.clock_gettime(.MONOTONIC, &ts);
            seed ^= @as(u64, @intCast(ts.sec)) * std.time.ns_per_s + @as(u64, @intCast(ts.nsec));
        }
        return std.fmt.bufPrint(dest, "{s}/{s}_{x}{s}", .{ dir, prefix, seed, suffix }) catch "";
    }

    pub fn writeAtomic(dest: string, content: string): void {
        var tmp_buf: [1024]u8 = undefined;
        const dir = std.fs.path.dirname(dest) orelse FileSystem.tmpDir();
        const base = std.fs.path.basename(dest);
        const id = tmp_counter.fetchAdd(1, .monotonic);
        const tmp_name = std.fmt.bufPrint(&tmp_buf, "{s}/.{s}.tmp_{x}", .{ dir, base, id }) catch return;

        writeFileSync(tmp_name, content);
        rename(tmp_name, dest);
    }
};

pub const tmpDir = FileSystem.tmpDir;
pub const tmpPath = FileSystem.tmpPath;
pub const writeAtomic = FileSystem.writeAtomic;
