import { test, expect_eq } from "cap:test";
import { listen, Server } from "cap:net";
import { print } from "cap:io";

async fn acceptLoop(s: Server): void {
    let client = await s.accept();
    @spawn(() => {
        let c = client; // Capture client
        let req = await c.read();
        let res = "HTTP/1.1 200 OK\r\nContent-Length: 12\r\n\r\nHello CP Web";
        await c.write(res);
        c.close();
    });
    // Tail-call to accept next connection
    await acceptLoop(s);
}

async fn startServer(): void {
    let s = listen(8080);
    @print("Server listening on port 8080...");
    await acceptLoop(s);
}

fn main(): void {
    @print("cap:http async tail-call OOP compiles successfully!");
}
