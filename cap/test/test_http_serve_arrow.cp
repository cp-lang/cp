import { serve, Response, Request } from "cap:http";
import { print } from "cap:io";

fn main(): void {
    let server = serve({
        port: 8129,
        fetch: (req: Request) => {
            return new Response("Hello from inline arrow handler!");
        }
    });
    @print("Arrow Server created on port: {}", server.port);
}
