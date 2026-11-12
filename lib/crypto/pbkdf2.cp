// lib/crypto/pbkdf2.cp
// Password-Based Key Derivation Function 2 (PBKDF2)

const std = @import("std");

pub fn bytesToHex(bytes: []const u8): string {
    const charset = "0123456789abcdef";
    var result = std.heap.page_allocator.alloc(u8, bytes.len * 2) catch unreachable;
    for (bytes, 0..) |b, i| {
        result[i * 2 + 0] = charset[b >> 4];
        result[i * 2 + 1] = charset[b & 15];
    }
    return result;
}

pub fn pbkdf2Sync(password: string, salt: string, iterations: u32, keylen: usize): string {
    const key_buf = std.heap.page_allocator.alloc(u8, keylen) catch unreachable;
    std.crypto.pwhash.pbkdf2(key_buf, password, salt, iterations, std.crypto.auth.hmac.sha2.HmacSha256) catch unreachable;
    return bytesToHex(key_buf);
}

pub fn pbkdf2Raw(password: string, salt: string, iterations: u32, keylen: usize): []u8 {
    const key_buf = std.heap.page_allocator.alloc(u8, keylen) catch unreachable;
    std.crypto.pwhash.pbkdf2(key_buf, password, salt, iterations, std.crypto.auth.hmac.sha2.HmacSha256) catch unreachable;
    return key_buf;
}
