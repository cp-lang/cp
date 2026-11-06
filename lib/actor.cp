/**
 * CP Actor Management (cap:actor)
 */

import { PID } from "cap:beam";


/**
 * 自愿出让执行权。
 */
export async fn yield(): void {
    __zig__ {
        ctx.pc += 1;
        return;
    }
}

/**
 * 接收当前信箱的第一条消息。
 */
export async fn receive(): any {
    return await @receive();
}

/**
 * 暂停当前 Actor 的执行指定的毫秒数。
 */
export async fn sleep(ms: u32): void {
    if (@is_beam) {
        await @yield_to_beam(2, ms); // SystemAction.SLEEP = 2
    } else {
        __zig__ {
            const ts = std.os.linux.timespec{ .sec = @intCast(@divTrunc(ctx.ms, 1000)), .nsec = @intCast(@mod(ctx.ms, 1000) * 1000000) };
            _ = std.os.linux.nanosleep(&ts, null);
        }
    }
}

/**
 * 等待另一个 Actor 执行完毕。
 */
export async fn join(pid: PID): void {
    // 真实工业级实现应为 Monitor 机制。这里先作简单抽象。
    await @yield_to_beam(99, pid);
}

/**
 * 获取当前 Actor 的 PID。
 */
export fn self_pid(): PID {
    return @self();
}

/**
 * 获取当前接收到消息的发送者 PID。
 */
export fn sender(): PID {
    return @sender();
}

/**
 * 向目标 Actor 发送消息。
 */
export fn send(target: PID, msg: any): bool {
    return @send(target, msg);
}

/**
 * 向目标 Actor 回复消息。
 */
export fn reply(target: PID, msg: any): bool {
    return @reply(target, msg);
}
