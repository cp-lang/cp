import { print } from "cap:io";

fn main(): ?void {
    @print("--- Distributed Transparent Routing Test ---");

    // 1. Local Message (Standard)
    let worker = @spawn(() => {
        let msg = @receive();
        @print("Local Worker: Received message!");
    });
    @reply(worker, { int: 42 });

    // 2. Remote Message (Simulation)
    // We manually craft a PID that points to Node ID 99
    let remote_pid: PID = @self();
    __zig__ {
        ctx.remote_pid = PID{ .node_id = 99, .local_id = 777 };
    }

    @print("Main: Sending message to REMOTE PID (Node 99, Local 777)...");
    @reply(remote_pid, { int: 100 });

    @print("Main: Finished.");
}
