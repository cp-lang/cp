import { writeFileSync, mkdir } from "cap:fs";
import { print } from "cap:io";

fn main(): ?void {
    @print("--- Async File System Test ---");

    let test_dir = "./test_fs_async";
    let test_file = "./test_fs_async/hello.txt";
    let content = "Async I/O in Native Reactor!";

    mkdir(test_dir);
    writeFileSync(test_file, content);

    let worker = @spawn(() => {
        @print("Worker: Reading file asynchronously...");
        // Calling builtin directly to trigger CPS state machine transformation
        let data = await @fs_read_async("./test_fs_async/hello.txt");
        @print("Worker: Read complete. Content: {s}", data);
    });

    @print("Main: Spawned worker, continuing...");
}
