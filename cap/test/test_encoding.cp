// cap/test/test_encoding.cp
import { test, expect_eq } from "cap:test";
import { Base64 } from "cap:encoding";
const std = @import("std");

fn testBase64Standard(): !void {
    var out_buf: [128]u8 = undefined;

    // 1 byte: "M" -> "TQ=="
    const len1 = Base64.encode(&out_buf, "M");
    expect_eq(std.mem.eql(u8, out_buf[0..len1], "TQ=="), true);

    // 2 bytes: "Ma" -> "TWE="
    const len2 = Base64.encode(&out_buf, "Ma");
    expect_eq(std.mem.eql(u8, out_buf[0..len2], "TWE="), true);

    // 3 bytes: "Man" -> "TWFu"
    const len3 = Base64.encode(&out_buf, "Man");
    expect_eq(std.mem.eql(u8, out_buf[0..len3], "TWFu"), true);

    // Multi-block: "Hello, CP Concurrency!"
    const msg = "Hello, CP Concurrency!";
    const enc_len = Base64.encode(&out_buf, msg);

    var dec_buf: [128]u8 = undefined;
    const dec_len = try Base64.decode(&dec_buf, out_buf[0..enc_len]);
    expect_eq(std.mem.eql(u8, dec_buf[0..dec_len], msg), true);
}

fn testBase64Alloc(): !void {
    const allocator = std.heap.page_allocator;

    const source = "Antigravity + CP Language Native Fast Serde";
    const encoded = try Base64.encodeAlloc(allocator, source);
    defer allocator.free(encoded);

    const decoded = try Base64.decodeAlloc(allocator, encoded);
    defer allocator.free(decoded);

    expect_eq(std.mem.eql(u8, decoded, source), true);
}

fn testBase64URLSafe(): !void {
    const allocator = std.heap.page_allocator;

    // Characters that would produce '+' or '/' in standard Base64:
    // 0xFB, 0xFF, 0xFE -> standard Base64 has '+' and '/'
    const binary_data = "\xFB\xFF\xFE\x00\x3F";

    const enc_url = try Base64.encodeURLSafeAlloc(allocator, binary_data);
    defer allocator.free(enc_url);

    // Ensure no '+' or '/' in URL-safe encoded output
    expect_eq(std.mem.indexOfScalar(u8, enc_url, '+') == null, true);
    expect_eq(std.mem.indexOfScalar(u8, enc_url, '/') == null, true);
    expect_eq(std.mem.indexOfScalar(u8, enc_url, '=') == null, true); // No padding

    const decoded = try Base64.decodeAlloc(allocator, enc_url);
    defer allocator.free(decoded);

    expect_eq(std.mem.eql(u8, decoded, binary_data), true);
}

fn main(): void {
    test("Base64 standard encoding and decoding with padding", () => {
        try testBase64Standard();
    });
    test("Base64 dynamic memory allocation encodeAlloc and decodeAlloc", () => {
        try testBase64Alloc();
    });
    test("Base64 URL-safe encoding without padding characters", () => {
        try testBase64URLSafe();
    });
}
