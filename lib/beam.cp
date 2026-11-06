export enum SystemAction {
    WAKE = 1, SLEEP = 2, STOP = 3, YIELD = 4,
    NET_CONNECT = 10, NET_READ = 11, NET_WRITE = 12, NET_LISTEN = 13,
    FS_READ = 20, FS_WRITE = 21, FS_MKDIR = 22, FS_REMOVE = 23, FS_RMDIR = 24, FS_RENAME = 25, FS_COPY = 26,
    HTTP_FETCH = 30, HTTP_SERVE = 31,
    ERROR = 99
}
export class PID { node_id: u32, local_id: u32 }
export class Message { sender: PID, action: u32, payload: any }
export class Agent {
    id: u32, env: any,
    fn send(this, target: PID, action: u32, payload: any): void {
        __zig__ {
            const beam = @import("beam.zig");
            const payload_bin = beam.Beam.makeBinary(self.env.?, "{}") catch unreachable;
            _ = beam.Beam.makeFastMessage(self.env.?, target.local_id, action, payload_bin) catch unreachable;
            _ = payload;
        }
    }
    async fn onMessage(this, msg: Message): void {
        __zig__ { _ = ctx.msg; }
    }
}
