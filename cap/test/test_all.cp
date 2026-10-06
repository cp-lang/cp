import { print } from "cap:io";
import { sleep, join, receive } from "cap:actor";
import { connect, Server } from "cap:net";
import { readFile, writeFile } from "cap:fs";
import { fetch, serve, Response } from "cap:http";
import { SharedTensor } from "cap:mem";

class Worker extends Agent {
    async fn onMessage(msg: Message): void {
        @print("Worker received message");
    }
}

async fn test_nested_await(data_a: string, data_b: string): void {
    @print("Nested await executed with args: {s} {s}", data_a, data_b);
}

fn test_auto_drop(): ?void {
    @print("Testing Auto-Drop (Compiler injected defer)...");
    const sync_t = new SharedTensor(512 * 1024);
    sync_t.retain();
    // NO defer sync_t.release() written here! The compiler will inject it based on AST reflection!
}

async fn run_tests(): void {
    test_auto_drop()?;
    @print("Testing nested await...");
    await test_nested_await("Hello", "World");
    @print("Returned from nested await!");
    
    @print("Testing mem.cp SharedTensor...");
    const t = new SharedTensor(1024 * 1024);
    t.retain();
    
    // Ownership transfer test
    const st = move t;
    st.release(); // Manually decrement to 1, auto-drop will decrement to 0 and destroy
    @print("Tensor size from moved var: {}", st.get_size());

    @print("Testing fs...");
    // use sync methods since async calls aren't fully lowered in cpc yet
    // writeFileSync("test.txt", "hello");
    // const content = readFileSync("test.txt");
    // @print("FS result: {}", content);

    @print("Testing net...");
    // test class instantiation
    const server: Server = new Server();
    
    @print("Testing http...");
    const res: Response = new Response();
    @print("HTTP fetch ok: {}", res.ok());

    @print("Testing sleep...");
    // sleep(10);
    @print("Done tests.");
}

fn main(): void {
    @print("Start");
    const pid = @spawn(run_tests);
    @print("Spawned test runner: {}", pid.local_id);
}
