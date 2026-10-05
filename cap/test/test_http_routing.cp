import { serve, Response, Request, json } from "cap:http";
import { print } from "cap:io";

fn handle(req: Request): Response {
    if (req.url == "/") {
        return new Response("Home Page");
    }
    if (req.url == "/about") {
        return new Response("About CP-BEAM");
    }
    if (req.url == "/api/status") {
        return json({ status: "healthy", version: "1.0.0" });
    }
    let not_found = new Response("404 Route Not Found");
    not_found.status = 404;
    return not_found;
}

fn main(): void {
    let server = serve({
        port: 8130,
        fetch: handle
    });
    @print("Routing Server started on port: {}", server.port);
}
