import { print } from "std/io";

fn main(): ?void {
    @print("--- Actor Concurrency Test ---");

    let worker = @spawn((msg) => {
        match (msg) {
            .ping |sender| => {
                @print("Worker: Received ping from {any}, replying pong...", sender);
                @reply(sender, { pong: @self() });
            }
            ! => {
                @print("Worker: Unknown message");
            }
        }
    });

    @print("Main: Sending ping to worker {any}...", worker);
    @reply(worker, { ping: @self() });

    @print("Main: Waiting for reply...");
    let response = @receive();
    
    match (response) {
        .pong |pid| => {
            @print("Main: Received pong from worker {any}", pid);
        }
        ! => {
            @print("Main: Unknown reply");
        }
    }

    @print("Main: Finished.");
}