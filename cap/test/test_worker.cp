import { test, expect_eq } from "cap:test";

import { print } from "cap:io";
import { sleep } from "cap:actor";

async fn worker_job(): void {
    @print("Hello from spawned worker thread!");
}

async fn main(): void {
    @worker_spawn(worker_job);
    @print("Main thread finishing up...");
    // Sleep a bit so main doesn't exit before worker prints
    await sleep(100);
}
