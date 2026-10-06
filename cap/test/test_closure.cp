import { print } from "cap:io";
import { sleep } from "cap:actor";

async fn test_closure(): void {
    let outer_var = 999;
    
    @spawn(() => {
        // Will this fail to compile because outer_var is not in lambda's ctx?
        @print("Inside closure, outer_var = {}", outer_var);
    });
}

fn main(): void {
    @spawn(test_closure);
}
