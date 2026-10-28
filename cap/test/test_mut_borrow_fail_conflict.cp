import { print } from "cap:io";

fn main(): void {
    let m = new Map<string, i32>();
    let r = &m; // active read-only borrow
    m.set("k", 1); // ERROR: Cannot mutably borrow or mutate 'm' because it is already borrowed by 'r'
    print(`${r.size}`);
}
