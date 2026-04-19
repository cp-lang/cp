/**
 * CP-to-BEAM 核心桥接契约 (RFC-002 集成版)
 */

export enum SystemAction {
    NULL_ACTION = 0,    // 防御性预留
    WAKE = 1,           // 状态恢复/冷启动
    SLEEP = 2,          // 强制休眠，保存状态至 ETS
    STOP = 3,           // 销毁 Agent

    // --- 委托 I/O 指令 (10-89 自动路由至 I/O 网关 99990001) ---
    
    // 网络 I/O (10-19)
    NET_CONNECT_REQ = 10,
    NET_CONNECT_RES = 11,
    NET_WRITE_REQ = 12,
    NET_READ_REQ = 13,
    NET_READ_RES = 14,

    // 文件 I/O (20-29)
    FS_READ_REQ = 20,
    FS_READ_RES = 21,
    FS_WRITE_REQ = 22,
    FS_WRITE_RES = 23,

    // 高层 HTTP (30-39)
    HTTP_FETCH_REQ = 30,
    HTTP_FETCH_RES = 31,

    // --- 统一错误回传 (99) ---
    IO_ERROR = 99, 
}

export class Env {}
export class Term {}

export class Agent {
    id: i32,
    env: Env,
    
    fn send(this, target_id: u32, action_id: u32, payload: any): void {
        __zig__ {
            const payload_bin = beam.Beam.makeBinary(self.env.?, "{}") catch unreachable;
            _ = beam.Beam.makeFastMessage(self.env.?, target_id, action_id, payload_bin) catch unreachable;
            _ = payload;
        }
    }

    fn onMessage(this, action_id: u32, payload: any): void {
        __zig__ {
            _ = action_id;
            _ = payload;
            // No discard for self, but return ok
        }
    }
}
