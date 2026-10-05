import { console } from "cap:io";
import { readFile, writeFile } from "cap:fs";
import { fetch } from "cap:http";
import { Agent } from "cap:beam";

class ComplexAgent extends Agent {
    async fn onMessage(this, action_id: u32, payload: any): void {
        __zig__ { _ = ctx.payload; }
        console.log("Agent received action {d}", action_id);
        
        await writeFile("./test.txt", "Production-grade CP-BEAM");
        const data = await readFile("./test.txt");
        @print("FS Test OK: {s}", data);
        
        const res = await fetch("https://google.com", null);
        @print("HTTP Test OK: {s}", res.text());
    }
}
fn main(): void {
    @print("--- CP SDK Multi-Library Test ---");
}
