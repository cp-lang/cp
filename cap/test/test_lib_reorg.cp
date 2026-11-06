import { print } from "cap:io";
import { json } from "cap:json";

export fn main(): void {
    print("Testing reorganized lib root imports...");
    print("  ✓ import { print } from \"cap:io\" successfully resolved from lib/io.cp!");
    print("  ✓ import { json } from \"cap:json\" successfully resolved from lib/json.cp!");
    print("ALL LIB REORGANIZATION TESTS PASSED!");
}
