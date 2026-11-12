// lib/encoding/base64.cp
// High-performance Base64 encoder and decoder in pure CP
const std = @import("std");

const standard_alphabet: string = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const url_safe_alphabet: string = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

export class Base64 {
    pub static fn encodeLen(len: usize): usize {
        return ((len + 2) / 3) * 4;
    }

    pub static fn decodeLen(len: usize): usize {
        return (len / 4) * 3 + 4;
    }

    pub static fn encode(dest: []u8, source: []const u8): usize {
        return encodeWithAlphabet(standard_alphabet, dest, source, true);
    }

    pub static fn encodeURLSafe(dest: []u8, source: []const u8): usize {
        return encodeWithAlphabet(url_safe_alphabet, dest, source, false);
    }

    pub static fn encodeAlloc(allocator: std.mem.Allocator, source: []const u8): []u8! {
        const out_len = encodeLen(source.len);
        let dest = allocator.alloc(u8, out_len)?;
        const actual_len = encode(dest, source);
        return dest[0..actual_len];
    }

    pub static fn encodeURLSafeAlloc(allocator: std.mem.Allocator, source: []const u8): []u8! {
        const out_len = encodeLen(source.len);
        let dest = allocator.alloc(u8, out_len)?;
        const actual_len = encodeURLSafe(dest, source);
        return dest[0..actual_len];
    }

    static fn encodeWithAlphabet(alphabet: []const u8, dest: []u8, source: []const u8, pad: bool): usize {
        var acc: u32 = 0;
        var acc_len: u32 = 0;
        var out_idx: usize = 0;

        for (byte of source) {
            acc = (acc << 8) | @as(u32, byte);
            acc_len += 8;
            while (acc_len >= 6) {
                acc_len -= 6;
                const shift = @as(u5, @intCast(acc_len));
                const idx = @as(usize, (acc >> shift) & 0x3F);
                dest[out_idx] = alphabet[idx];
                out_idx += 1;
            }
        }

        if (acc_len > 0) {
            const shift = @as(u5, @intCast(6 - acc_len));
            const idx = @as(usize, (acc << shift) & 0x3F);
            dest[out_idx] = alphabet[idx];
            out_idx += 1;
        }

        if (pad) {
            while (out_idx % 4 != 0) {
                dest[out_idx] = '=';
                out_idx += 1;
            }
        }

        return out_idx;
    }

    pub static fn decode(dest: []u8, source: []const u8): usize! {
        return decodeUnified(dest, source);
    }

    pub static fn decodeURLSafe(dest: []u8, source: []const u8): usize! {
        return decodeUnified(dest, source);
    }

    pub static fn decodeAlloc(allocator: std.mem.Allocator, source: []const u8): []u8! {
        const max_len = decodeLen(source.len);
        let dest = allocator.alloc(u8, max_len)?;
        const actual_len = decodeUnified(dest, source)?;
        return dest[0..actual_len];
    }

    pub static fn decodeURLSafeAlloc(allocator: std.mem.Allocator, source: []const u8): []u8! {
        return decodeAlloc(allocator, source);
    }

    static fn decodeUnified(dest: []u8, source: []const u8): usize! {
        var decode_table: [256]u8 = undefined;
        for (i of 0..256) {
            decode_table[i] = 0xFF;
        }

        for (i of 0..64) {
            decode_table[standard_alphabet[i]] = @intCast(i);
        }
        // Support URL-safe '-' and '_' seamlessly
        decode_table['-'] = 62;
        decode_table['_'] = 63;

        var acc: u32 = 0;
        var acc_len: u32 = 0;
        var out_idx: usize = 0;

        for (c of source) {
            if (c == '=') break;
            if (c == ' ' || c == '\r' || c == '\n' || c == '\t') continue;
            const val = decode_table[c];
            if (val == 0xFF) return error.InvalidCharacter;

            acc = (acc << 6) | @as(u32, val);
            acc_len += 6;

            if (acc_len >= 8) {
                acc_len -= 8;
                const shift = @as(u5, @intCast(acc_len));
                dest[out_idx] = @truncate((acc >> shift) & 0xFF);
                out_idx += 1;
            }
        }

        return out_idx;
    }
}
