import { connect, listen } from "cap:net";
import { print } from "cap:io";

fn main(): ?void {
    @print("--- Native Reactor Async Network Test ---");
    let worker = @spawn(() => {
        @print("Worker: Connecting to 127.0.0.1:8889...");
        let socket = connect("127.0.0.1", 8889);
        
        @print("Worker: Connected! Waiting for data...");
        // Use the builtin directly as class method state machines are WIP
        let data = await @net_read_async(socket.handle);
        
        @print("Worker: Received {} bytes: {s}", data.length, data);
        socket.close();
    });

    @print("Main: Spawned worker {}, continuing local execution.", worker);
    
    // Give the reactor a chance to run
    let counter = 0;
    while (counter < 5) {
        @print("Main: Tick {d}...", counter);
        counter = counter + 1;
    }
}
