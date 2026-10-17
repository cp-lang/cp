/**
 * CP HTTP API (cap:http)
 * 深度对标 Web Fetch & Bun.serve | 显式双模调度
 */

import { SystemAction } from "cap:beam";

export class Request {
    method: string;
    url: string;
    body: string;

    fn init(this, method: string, url: string, body: string): void {
        this.method = method;
        this.url = url;
        this.body = body;
    }

    fn text(this): string {
        return this.body;
    }

    fn json(this): any {
        return @json_parse(this.body);
    }
}

export class Response {
    status: i32;
    body: string;
    contentType: string;

    fn init(this, body: string): void {
        this.status = 200;
        this.body = body;
        this.contentType = "text/plain";
    }

    fn text(this): string {
        return this.body;
    }

    fn json(this): any {
        return @json_parse(this.body);
    }

    fn ok(this): bool {
        if (this.status >= 200) { if (this.status < 300) { return true; } } return false;
    }
}

export class Server {
    port: i32;
    fd: i32;

    fn init(this, port: i32, fd: i32): void {
        this.port = port;
        this.fd = fd;
    }

    fn stop(this): void {
        @net_close(this.fd);
    }
}

/**
 * 生产级 fetch 实现
 */
export async fn fetch(url: string, options: any): Response {
    let raw_body = "";

    if (@is_beam) {
        // --- BEAM 模式：外包给 Bun I/O Gateway ---
        raw_body = await @yield_to_beam(@intFromEnum(SystemAction.HTTP_FETCH), url);
    } else {
        // --- Native 模式：调用本地 Zig HTTP Client ---
        raw_body = await @http_fetch(url, options);
    }

    let res = new Response(raw_body);
    return res;
}

/**
 * 快速 JSON 响应构造函数 (对标 Bun 的 Response.json)
 */
export fn json(data: any): Response {
    let raw_json = @json_stringify(data);
    let res = new Response(raw_json);
    res.contentType = "application/json";
    return res;
}

/**
 * 极速 HTTP Server (对标 Bun.serve)
 */
export fn serve(config: any): Server {
    if (@is_beam) {
        @yield_to_beam(@intFromEnum(SystemAction.HTTP_SERVE), config);
        let s = new Server(3000, 0);
        return s;
    } else {
        return @http_serve(config);
    }
}
