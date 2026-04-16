import { print } from "std/io";

fn main(): ?void {
    @print("--- Actor Concurrency Test ---");

    let worker = @spawn((msg) => {
        match (msg) {
            .ping |sender| => {
                @print("Worker: Received ping from {d}, replying pong...", sender);
                @reply(sender, { pong: @self() });
            }
            ! => {
                @print("Worker: Unknown message");
            }
        }
    });

    @print("Main: Sending ping to worker {d}...", worker);
    @reply(worker, { ping: @self() });

    @print("Main: Waiting for reply...");
    let response = @receive();
    
    match (response) {
        .pong |pid| => {
            @print("Main: Received pong from worker {d}", pid);
        }
        ! => {
            @print("Main: Unknown reply");
        }
    }

    @print("Main: Finished.");
}