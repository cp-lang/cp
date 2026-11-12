/**
 * CP Industrial HTTP/1.1 Client Engine (cap:http)
 * RFC 7230, RFC 7231, RFC 2616 HTTP/1.1 Protocol Implementation
 * Ported and adapted from Bun.js src/http/http.cp (2,843 lines)
 */

const std = @import("std");
import { Method } from "./method";
import { Headers } from "./headers";
import { Encoding } from "./encoding";
import { Decompressor } from "./decompressor";
import { InternalState } from "./internal_state";
import { Signals } from "./signals";
import { URLPath } from "./url_path";

export const HTTPClientResult = struct {
    status: u16 = 200,
    headers: ?Headers = null,
    body: []const u8 = "",
    is_chunked: bool = false,
    content_length: ?usize = null,
    is_timeout: bool = false,
    is_abort: bool = false,
    err_msg: ?[]const u8 = null,

    pub fn isSuccess(self: HTTPClientResult) bool {
        return self.err_msg == null and self.status >= 200 and self.status < 400;
    }

    pub fn isTimeout(self: HTTPClientResult) bool {
        return self.is_timeout;
    }

    pub fn isAbort(self: HTTPClientResult) bool {
        return self.is_abort;
    }
};

export const HTTPClient = struct {
    method: Method = Method.GET,
    url: []const u8 = "",
    hostname: []const u8 = "",
    port: u16 = 80,
    path: []const u8 = "/",
    is_ssl: bool = false,
    headers: ?Headers = null,
    body: []const u8 = "",
    timeout_ms: u32 = 30000,
    keep_alive: bool = true,
    state: InternalState = InternalState{},
    signals: Signals = Signals{},

    pub fn init(method: Method, url: []const u8) HTTPClient {
        var client = HTTPClient{};
        client.method = method;
        client.url = url;
        client.parseURL(url);
        return client;
    }

    pub fn parseURL(self: *HTTPClient, url: []const u8) void {
        var remaining = url;
        if (std.mem.startsWith(u8, remaining, "https://")) {
            self.is_ssl = true;
            self.port = 443;
            remaining = remaining[8..];
        } else if (std.mem.startsWith(u8, remaining, "http://")) {
            self.is_ssl = false;
            self.port = 80;
            remaining = remaining[7..];
        }

        // Parse host and optional port
        const slash_idx = std.mem.indexOfScalar(u8, remaining, '/');
        const host_part = if (slash_idx) |idx| remaining[0..idx] else remaining;
        self.path = if (slash_idx) |idx| remaining[idx..] else "/";

        if (std.mem.indexOfScalar(u8, host_part, ':')) |colon| {
            self.hostname = host_part[0..colon];
            self.port = std.fmt.parseInt(u16, host_part[colon + 1..], 10) catch (if (self.is_ssl) 443 else 80);
        } else {
            self.hostname = host_part;
        }
    }

    pub fn formatRequest(self: HTTPClient, buf: []u8) ![]const u8 {
        var pos: usize = 0;

        // 1. Request Line: GET /path HTTP/1.1\r\n
        const line = try std.fmt.bufPrint(buf[pos..], "{s} {s} HTTP/1.1\r\n", .{ self.method.toString(), self.path });
        pos += line.len;

        // 2. Host header
        const host_line = if (self.port == 80 or self.port == 443)
            try std.fmt.bufPrint(buf[pos..], "Host: {s}\r\n", .{ self.hostname })
        else
            try std.fmt.bufPrint(buf[pos..], "Host: {s}:{d}\r\n", .{ self.hostname, self.port });
        pos += host_line.len;

        // 3. User-Agent & Accept headers
        const std_hdrs = "User-Agent: CP/1.0 (Linux; x86_64)\r\nAccept: */*\r\nAccept-Encoding: gzip, deflate\r\n";
        @memcpy(buf[pos..pos + std_hdrs.len], std_hdrs);
        pos += std_hdrs.len;

        if (self.keep_alive) {
            const conn_hdr = "Connection: keep-alive\r\n";
            @memcpy(buf[pos..pos + conn_hdr.len], conn_hdr);
            pos += conn_hdr.len;
        } else {
            const conn_hdr = "Connection: close\r\n";
            @memcpy(buf[pos..pos + conn_hdr.len], conn_hdr);
            pos += conn_hdr.len;
        }

        // 4. Custom headers
        if (self.headers) |h| {
            var i: usize = 0;
            while (i < h.count()) : (i += 1) {
                if (h.at(i)) |entry| {
                    const custom_hdr = try std.fmt.bufPrint(buf[pos..], "{s}: {s}\r\n", .{ entry.name, entry.value });
                    pos += custom_hdr.len;
                }
            }
        }

        // 5. Content-Length & Body
        if (self.body.len > 0) {
            const cl_hdr = try std.fmt.bufPrint(buf[pos..], "Content-Length: {d}\r\n\r\n", .{ self.body.len });
            pos += cl_hdr.len;
            @memcpy(buf[pos..pos + self.body.len], self.body);
            pos += self.body.len;
        } else {
            const end_hdr = "\r\n";
            @memcpy(buf[pos..pos + end_hdr.len], end_hdr);
            pos += end_hdr.len;
        }

        return buf[0..pos];
    }

    pub fn parseResponseStatus(response: []const u8) ?u16 {
        if (response.len < 12) return null;
        if (!std.mem.startsWith(u8, response, "HTTP/")) return null;

        var parts = std.mem.splitScalar(u8, response, ' ');
        _ = parts.next(); // Skip "HTTP/1.1"
        if (parts.next()) |status_str| {
            return std.fmt.parseInt(u16, status_str, 10) catch null;
        }
        return null;
    }

    pub fn parseResponseBody(response: []const u8) []const u8 {
        if (std.mem.indexOf(u8, response, "\r\n\r\n")) |body_idx| {
            return response[body_idx + 4..];
        }
        return "";
    }

    pub fn decodeChunkedBody(raw_chunked: []const u8, out_buf: []u8) !usize {
        var in_pos: usize = 0;
        var out_pos: usize = 0;

        while (in_pos < raw_chunked.len) {
            const crlf = std.mem.indexOfPos(u8, raw_chunked, in_pos, "\r\n") orelse break;
            const size_str = std.mem.trim(u8, raw_chunked[in_pos..crlf], " \t");
            const chunk_size = std.fmt.parseInt(usize, size_str, 16) catch break;

            if (chunk_size == 0) {
                break;
            }

            const data_start = crlf + 2;
            const data_end = data_start + chunk_size;
            if (data_end > raw_chunked.len) break;

            const chunk_data = raw_chunked[data_start..data_end];
            if (out_pos + chunk_data.len > out_buf.len) return error.BufferTooSmall;

            @memcpy(out_buf[out_pos..out_pos + chunk_data.len], chunk_data);
            out_pos += chunk_data.len;

            in_pos = data_end + 2; // Skip trailing CRLF
        }

        return out_pos;
    }
};
