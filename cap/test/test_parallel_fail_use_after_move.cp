import { print } from "cap:io";
import { spawn } from "cap:parallel";

@parallel
async fn main() {
    let data = "owned";
    let t = spawn(async () => {
        let taken = move data;
        return taken;
    });
    print(data);
}
