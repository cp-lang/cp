import { print } from "cap:io";

class Tensor {
    size: i32;
    fn init(s: i32) { this.size = s; }
    fn release() { @print("Tensor released"); }
}

fn badNllMove() {
    let a = new Tensor(10);
    let r = &a;
    // 试图在借用 r 尚未结束前 Move 所有者 a！
    let b = a; // ❌ Error: Cannot move 'a' because it is currently borrowed by 'r'
    @print("Still using r after move: {d}", r.size);
}

fn main() {
    badNllMove();
}
