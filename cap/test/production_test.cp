import { Agent } from "cap:beam";

class ProductionAgent extends Agent {
    async fn onMessage(this, action_id: u32, payload: any): void {
        __zig__ { _ = ctx.payload; }
        @print("Agent ID: {d}", this.id);
        const test_path = "./prod_test.txt";
        @fs_write_file(test_path, "CP Production SDK Content");
        const content = await @fs_read_async(test_path);
        @print("Read complete: {s}", content);
    }
}
fn main(): void {
    @print("CP Standalone entry point.");
}
