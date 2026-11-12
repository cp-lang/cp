// cap/test/test_http_features.cp
import { test, expect_eq } from "cap:test";
import { Method, Headers, ETag, MimeType } from "cap:http";
const std = @import("std");

fn testMethod(): !void {
    const m_get = Method.GET;
    expect_eq(m_get.isIdempotent(), true);
    expect_eq(m_get.hasRequestBody(), false);
    expect_eq(std.mem.eql(u8, m_get.toString(), "GET"), true);

    const m_post = Method.POST;
    expect_eq(m_post.isIdempotent(), false);
    expect_eq(m_post.hasRequestBody(), true);

    const m_put = Method.PUT;
    expect_eq(m_put.isIdempotent(), true);
    expect_eq(m_put.hasRequestBody(), true);

    const parsed = Method.fromString("DELETE");
    expect_eq(parsed != null, true);
    expect_eq(parsed.? == Method.DELETE, true);

    const invalid = Method.fromString("INVALID");
    expect_eq(invalid == null, true);
}

fn testHeaders(): !void {
    const allocator = std.heap.page_allocator;
    var headers = Headers.init(allocator);
    defer headers.deinit();

    headers.append("Content-Type", "application/json");
    headers.append("Authorization", "Bearer token123");
    headers.append("X-Custom-Header", "InitialValue");

    expect_eq(headers.count(), 3);
    expect_eq(headers.has("content-type"), true);
    expect_eq(headers.has("CONTENT-TYPE"), true);
    expect_eq(std.mem.eql(u8, headers.get("content-type").?, "application/json"), true);
    expect_eq(std.mem.eql(u8, headers.get("AUTHORIZATION").?, "Bearer token123"), true);

    // Overwrite header via set()
    headers.set("x-custom-header", "UpdatedValue");
    expect_eq(headers.count(), 3);
    expect_eq(std.mem.eql(u8, headers.get("X-CUSTOM-HEADER").?, "UpdatedValue"), true);

    // Delete header
    headers.delete("authorization");
    expect_eq(headers.count(), 2);
    expect_eq(headers.has("authorization"), false);
}

fn testETag(): !void {
    var buf: [32]u8 = undefined;
    const tag1 = ETag.generate("Hello CP World", &buf);
    expect_eq(tag1.len > 2, true);

    // Exact matching
    expect_eq(ETag.ifNoneMatch(tag1, tag1), true);

    // Wildcard matching
    expect_eq(ETag.ifNoneMatch(tag1, "*"), true);

    // Weak tag matching
    const weak_tag = "W/\"12345\"";
    const strong_tag = "\"12345\"";
    expect_eq(ETag.ifNoneMatch(weak_tag, strong_tag), true);
    expect_eq(ETag.ifNoneMatch(strong_tag, "W/\"12345\", \"67890\""), true);
    expect_eq(ETag.ifNoneMatch(strong_tag, "\"99999\""), false);
}

fn testMimeType(): !void {
    expect_eq(std.mem.eql(u8, MimeType.fromExtension("html"), "text/html; charset=utf-8"), true);
    expect_eq(std.mem.eql(u8, MimeType.fromExtension(".json"), "application/json; charset=utf-8"), true);
    expect_eq(std.mem.eql(u8, MimeType.fromExtension("js"), "application/javascript; charset=utf-8"), true);
    expect_eq(std.mem.eql(u8, MimeType.fromExtension("png"), "image/png"), true);
    expect_eq(std.mem.eql(u8, MimeType.fromExtension("wasm"), "application/wasm"), true);

    expect_eq(std.mem.eql(u8, MimeType.fromPath("/assets/app.min.css"), "text/css; charset=utf-8"), true);
    expect_eq(std.mem.eql(u8, MimeType.fromPath("photo.jpg"), "image/jpeg"), true);
    expect_eq(std.mem.eql(u8, MimeType.fromPath("unknown.xyz"), "application/octet-stream"), true);
}

fn main(): void {
    test("HTTP Method enumeration and attributes", () => {
        testMethod()?;
    });
    test("Case-insensitive HTTP Headers collection", () => {
        testHeaders()?;
    });
    test("HTTP ETag generation and If-None-Match evaluation", () => {
        testETag()?;
    });
    test("MIME type extension and path detection", () => {
        testMimeType()?;
    });
}
