import { test, expect_eq } from "cap:test";
import { serve, Response, Request, json } from "cap:http";
import { print } from "cap:io";

fn handle(req: Request): Response {
    if (req.url == "/json") {
        return json({ status: "ok", framework: "CP-BEAM" });
    }
    return new Response("Hello from Bun-style CP HTTP Server!");
}

fn main(): void {
    let server = serve({
        port: 8128,
        fetch: handle
    });
    @print("Server created successfully on port: {}", server.port);
}
