import { connect, listen } from "cap:net";
import { print } from "cap:io";

fn main(): ?void {
    @print("--- Network Module Compilation Test ---");
    // Just test if it compiles and the classes are recognized
    // We won't actually connect to anything that might fail
    @print("Network module loaded successfully.");
}
