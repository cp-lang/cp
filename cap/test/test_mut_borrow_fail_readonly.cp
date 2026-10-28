import { print } from "cap:io";

fn main(): void {
    let v = [10, 20];
    let r = &v; // read-only borrow
    r.push(30); // ERROR: Cannot perform container method 'push' through read-only borrow 'r'
    print(`${v.length}`);
}
