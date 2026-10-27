import { spawn } from "cap:parallel";

async fn main() {
    let mut x = 100;
    let b = &mut x;
    let task = spawn(async () => {
        b.* = 200;
    });
}
