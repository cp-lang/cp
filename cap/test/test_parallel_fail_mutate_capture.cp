import { spawn } from "cap:parallel";

@parallel
async fn main() {
    let mut count = 10;
    let t = spawn(async () => {
        count = 20;
        return "fail";
    });
}
