// con/test_spawn_arrow.cp
import { print } from "std/io";

fn main(): ?void {
    @print("--- Spawn Arrow Test ---");

    // Spawn an anonymous async arrow function
    let pid = @spawn(() => {
        @print("Hello from spawned arrow! pid={d}", @self());
    });

    @print("Spawned actor with pid={d}", pid);
}
