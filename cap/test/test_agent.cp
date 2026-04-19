import { Agent, SystemAction } from "cap:beam";
import { print } from "cap:io";

class MyAgent extends Agent {
    fn onMessage(this, action_id: u32, payload: any): void {
        __zig__ {
            _ = self;
            _ = payload;
        }
        @print("MyAgent onMessage: {d}", action_id);
    }
}
