// lib/security/csrf.cp
// CSRF Token Generation and Verification in pure CP
// Uses CSPRNG nonces, HMAC-SHA256 signatures, and constant-time comparison

const std = @import("std");
import { randomBytes, fillRandom, hmacSha256Raw } from "../crypto/index.cp";
import { Base64 } from "../encoding/index.cp";

export enum TokenFormat {
    Base64Url,
    Base64,
    Hex,
}

export const DEFAULT_EXPIRATION_MS: u64 = 24 * 60 * 60 * 1000; // 24 hours

export fn constantTimeEql(a: []const u8, b: []const u8): bool {
    if (a.len != b.len) return false;
    var diff: u8 = 0;
    for (i of 0..a.len) {
        diff |= (a[i] ^ b[i]);
    }
    return diff == 0;
}

fn getWallTimeMs(): u64 {
    __zig__ {
        var ts: std.os.linux.timespec = undefined;
        _ = std.os.linux.clock_gettime(std.os.linux.CLOCK.REALTIME, &ts);
        const sec: u64 = @as(u64, @intCast(ts.sec));
        const nsec: u64 = @as(u64, @intCast(ts.nsec));
        return sec * 1000 + (nsec / 1000000);
    }
}

export class CSRF {
    pub static fn generateRaw(secret: []const u8, expires_in_ms: u64, out: []u8): usize! {
        if (out.len < 64) return error.BufferTooSmall;

        const now = getWallTimeMs();

        // 1. Write timestamp (8 bytes, Big-Endian)
        __zig__ {
            std.mem.writeInt(u64, out[0..8], now, .big);
        }

        // 2. Generate CSPRNG nonce (16 bytes)
        const nonce_slice = out[8..24];
        fillRandom(nonce_slice);

        // 3. Write expires_in_ms (8 bytes, Big-Endian)
        __zig__ {
            std.mem.writeInt(u64, out[24..32], expires_in_ms, .big);
        }

        // 4. Calculate HMAC-SHA256 over the 32-byte payload
        const payload = out[0..32];
        var digest: [32]u8 = undefined;
        hmacSha256Raw(&digest, secret, payload);

        // 5. Append HMAC digest (32 bytes)
        for (i of 0..32) {
            out[32 + i] = digest[i];
        }

        return 64;
    }

    pub static fn verifyRaw(raw_token: []const u8, secret: []const u8, max_age_ms: u64): bool {
        if (raw_token.len != 64) return false;

        const payload = raw_token[0..32];
        const received_hmac = raw_token[32..64];

        // 1. Verify HMAC signature in constant time
        var expected_hmac: [32]u8 = undefined;
        hmacSha256Raw(&expected_hmac, secret, payload);

        if (!constantTimeEql(&expected_hmac, received_hmac)) {
            return false;
        }

        // 2. Extract timestamp and expiration
        var timestamp: u64 = 0;
        var expires_in: u64 = 0;
        __zig__ {
            timestamp = std.mem.readInt(u64, raw_token[0..8], .big);
            expires_in = std.mem.readInt(u64, raw_token[24..32], .big);
        }

        const now = getWallTimeMs();

        // Reject tokens from far in the future (> 60s clock skew)
        if (timestamp > now + 60000) {
            return false;
        }

        // Check token's embedded expiration
        if (expires_in > 0 && now > timestamp + expires_in) {
            return false;
        }

        // Check caller's max_age_ms
        if (max_age_ms > 0 && now > timestamp + max_age_ms) {
            return false;
        }

        return true;
    }

    pub static fn generate(secret: []const u8, expires_in_ms: u64, format: TokenFormat, allocator: std.mem.Allocator): []const u8! {
        var raw: [64]u8 = undefined;
        __zig__ {
            @memset(&raw, 0);
        }
        const len = generateRaw(secret, expires_in_ms, raw[0..])?;
        const raw_slice = raw[0..len];

        switch (format) {
            .Base64Url => {
                return Base64.encodeURLSafeAlloc(allocator, raw_slice)?;
            },
            .Base64 => {
                return Base64.encodeAlloc(allocator, raw_slice)?;
            },
            .Hex => {
                const hex_chars = "0123456789abcdef";
                let out = allocator.alloc(u8, 128)?;
                for (i of 0..64) {
                    const b = raw_slice[i];
                    out[i * 2] = hex_chars[b >> 4];
                    out[i * 2 + 1] = hex_chars[b & 0x0F];
                }
                return out;
            },
        }
    }

    pub static fn verify(token: []const u8, secret: []const u8, max_age_ms: u64, format: TokenFormat, allocator: std.mem.Allocator): bool {
        if (token.len == 0) return false;

        var raw: [64]u8 = undefined;
        __zig__ {
            @memset(&raw, 0);
        }

        switch (format) {
            .Base64Url, .Base64 => {
                const decoded = Base64.decodeURLSafeAlloc(allocator, token) catch return false;
                defer allocator.free(decoded);
                if (decoded.len != 64) return false;
                for (i of 0..64) {
                    raw[i] = decoded[i];
                }
            },
            .Hex => {
                if (token.len != 128) return false;
                for (i of 0..64) {
                    const c1 = token[i * 2];
                    const c2 = token[i * 2 + 1];
                    const h1 = hexVal(c1) catch return false;
                    const h2 = hexVal(c2) catch return false;
                    raw[i] = (h1 << 4) | h2;
                }
            },
        }

        return verifyRaw(&raw, secret, max_age_ms);
    }
}

fn hexVal(c: u8): u8! {
    if (c >= '0' && c <= '9') return c - '0';
    if (c >= 'a' && c <= 'f') return c - 'a' + 10;
    if (c >= 'A' && c <= 'F') return c - 'A' + 10;
    return error.InvalidHex;
}
