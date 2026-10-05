import { test, expect_eq } from "cap:test";
import { print } from "cap:io";
import { self_pid, sender, send, receive, sleep } from "cap:actor";
import { PID } from "cap:beam";

async fn worker_entry(): void {
    @print("▶ Worker thread started. PID = {d}:{d}", @self().node_id, @self().local_id);

    // 1. Wait for ping message from main thread
    let msg1 = @receive();
    let from1 = @sender();
    @print("✔ Worker received message 1: '{s}' from {d}:{d}", msg1.string, from1.node_id, from1.local_id);

    // Reply pong 1
    @send(from1, "pong_1");

    // 2. Wait for ping message 2 from main thread
    let msg2 = @receive();
    let from2 = @sender();
    @print("✔ Worker received message 2: '{s}' from {d}:{d}", msg2.string, from2.node_id, from2.local_id);

    // Reply pong 2
    @send(from2, "pong_2");

    @print("✔ Worker thread finished successfully.");
}

async fn main(): void {
    @print("=== Test Cross-Thread Mailbox Channel (Inter-Worker Communication) ===");

    let main_self = @self();
    @print("Main thread PID: {d}:{d}", main_self.node_id, main_self.local_id);
    expect_eq(main_self.node_id, 0);

    // Spawn independent worker thread
    let worker_pid = @worker_spawn(worker_entry);
    @print("Spawned worker thread with PID: {d}:{d}", worker_pid.node_id, worker_pid.local_id);
    expect_eq(worker_pid.node_id, 1);
    expect_eq(worker_pid.local_id, 1);

    // Round 1: Send ping_1 to worker
    @print("Main sending 'ping_1' to worker...");
    @send(worker_pid, "ping_1");

    // Wait for worker reply
    let reply1 = @receive();
    let rep_sender1 = @sender();
    @print("Main received reply 1: '{s}' from {d}:{d}", reply1.string, rep_sender1.node_id, rep_sender1.local_id);
    expect_eq(reply1, "pong_1");
    expect_eq(rep_sender1.node_id, 1);

    // Round 2: Send ping_2 to worker
    @print("Main sending 'ping_2' to worker...");
    @send(worker_pid, "ping_2");

    // Wait for worker reply 2
    let reply2 = @receive();
    let rep_sender2 = @sender();
    @print("Main received reply 2: '{s}' from {d}:{d}", reply2.string, rep_sender2.node_id, rep_sender2.local_id);
    expect_eq(reply2, "pong_2");
    expect_eq(rep_sender2.node_id, 1);

    @print("🎉 Cross-thread channel ping-pong verified successfully!");
}
