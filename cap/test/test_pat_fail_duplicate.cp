import { print } from "cap:io";

fn main(): void {
    // '_' is a normal identifier (receiver alias), NOT a wildcard ('*' is the wildcard).
    // Duplicate '_' in pattern matching / destructuring must fail at compile time.
    let [_, _] = [10, 20];
    print(`${_}`);
}
