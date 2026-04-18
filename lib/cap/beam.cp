/**
 * CP-to-BEAM 核心桥接契约 (RFC-002 集成版)
 */

export enum SystemAction {
    NULL_ACTION = 0,
    WAKE = 1,
    SLEEP = 2,
    STOP = 3
}

// 对应 sdk/beam.zig 中的不透明类型
export class Env {}
export class Term {}

export class Agent {
    id: i32,
    env: Env,
    
    fn send(this, target_id: u32, action_id: u32, payload: any): void {
        __zig__ {
            const beam = @import("beam.zig");
            const payload_bin = try beam.Beam.makeBinary(ctx.env, payload);
            const msg = try beam.Beam.makeFastMessage(ctx.env, target_id, action_id, payload_bin);
            // 真实的发送逻辑将由内核根据 NodeID 路由
        }
    }

    // 供业务 Agent 覆盖的核心回调
    fn onMessage(this, action_id: u32, payload: any): void {
        // 默认空实现
    }
}
