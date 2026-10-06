import { print } from "cap:io";

async fn test_move(): void {
    let big_data = "Super heavy string";
    
    @spawn(() => {
        let local_data = move big_data;
        @print("Inside closure, local_data = {s}", local_data);
    });

    // This should fail to compile!
    @print("Can I use it? {s}", big_data);
}

fn main(): void {
    @spawn(test_move);
}
