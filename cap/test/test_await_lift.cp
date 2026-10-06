import { sleep } from "cap:actor";
import { print } from "cap:io";

async fn compute(): string {
    return "CP-BEAM Master";
}

async fn test_lifting(): void {
    @print("--- Test Context Lifting ---");
    // Local variables declared before await
    const a: i32 = 100;
    const b = 200;
    const msg = "Hello from before await";

    @print("Variables before await: a={}, b={}, msg={s}", a, b, msg);

    // This await will slice the state machine
    const res = await compute();

    // Now, after await (in case 1 of the state machine), we access a, b, msg
    // If context lifting fails, this will not compile or will contain garbage.
    let res_str: string = "";
    __zig__ {
        ctx.res_str = ctx.res.?.string;
    }
    @print("Variables after await: a={}, b={}, msg={s}, res={s}", a, b, msg, res_str);
    
    if (a == 100) {
        if (b == 200) {
            if (msg == "Hello from before await") {
                if (res_str == "CP-BEAM Master") {
                    @print("Context Lifting SUCCESS!");
                }
            }
        }
    }
}

fn main(): void {
    @spawn(test_lifting);
}
