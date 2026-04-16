// con/test/test_zero_copy.cp
import { print } from "std/io";

fn main(): ?void {
    @print("--- Zero-Copy Shared Memory Test ---");

    let shape = [1024, 1024]; 
    
    // 1. Initialize a Shared Tensor (RefCount=1)
    let st = @shared_tensor_init(shape);
    @print("Shared Tensor initialized.");

    // 2. Spawn a worker
    let worker = @spawn(() => {
        @print("Worker started, waiting for message...");
        let msg = @receive();
        @print("Worker received something!");
        match (msg) {
            .tensor |t| => {
                @print("Worker received tensor! Releasing...");
                @release(t); 
            }
            ! => {
                @print("Worker received unknown message");
            }
        }
    });

    @print("Sending tensor to worker...");
    @reply(worker, { tensor: st });

    // 3. Release main's reference
    @print("Main releasing its reference...");
    @release(st);

    @print("Main finished.");
}
