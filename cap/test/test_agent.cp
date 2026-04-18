import { Agent, SystemAction } from "cap:beam";
import { print } from "cap:io";

class MyAgent extends Agent {
    fn onMessage(this, action_id: u32, payload: any): void {
        @print("MyAgent received message with action: {}", action_id);
    }
}
