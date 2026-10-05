/**
 * CP File System SDK (cap:fs)
 * 使用 @is_beam 进行显式双模分支逻辑重构
 */

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
