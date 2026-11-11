// lib/encoding/base64.cp
// High-performance Base64 encoder and decoder in CP
const std = @import("std");

pub const Base64 = struct {
    pub const standard_alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/".*;
    pub const url_safe_alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_".*;

    pub fn encodeLen(len: usize): usize {
        return ((len + 2) / 3) * 4;
    }

    pub fn decodeLen(len: usize): usize {
        return (len / 4) * 3 + 4;
    }

    pub fn encode(dest: []u8, source: []const u8) usize {
        return encodeWithAlphabet(&standard_alphabet, dest, source, true);
    }

    pub fn encodeURLSafe(dest: []u8, source: []const u8) usize {
        return encodeWithAlphabet(&url_safe_alphabet, dest, source, false);
    }

    pub fn encodeAlloc(allocator: std.mem.Allocator, source: []const u8) ![]u8 {
        const out_len = encodeLen(source.len);
        const dest = try allocator.alloc(u8, out_len);
        const actual_len = encode(dest, source);
        return dest[0..actual_len];
    }

    pub fn encodeURLSafeAlloc(allocator: std.mem.Allocator, source: []const u8) ![]u8 {
        const out_len = encodeLen(source.len);
        const dest = try allocator.alloc(u8, out_len);
        const actual_len = encodeURLSafe(dest, source);
        return dest[0..actual_len];
    }

    fn encodeWithAlphabet(alphabet: *const [64]u8, dest: []u8, source: []const u8, pad: bool) usize {
        var acc: u32 = 0;
        var acc_len: u32 = 0;
        var out_idx: usize = 0;

        for (source) |byte| {
            acc = (acc << 8) | @as(u32, byte);
            acc_len += 8;
            while (acc_len >= 6) {
                acc_len -= 6;
                const idx = @as(usize, (acc >> @intCast(acc_len)) & 0x3F);
                dest[out_idx] = alphabet[idx];
                out_idx += 1;
            }
        }

        if (acc_len > 0) {
            const shift = 6 - acc_len;
            const idx = @as(usize, (acc << @intCast(shift)) & 0x3F);
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

    pub fn decode(dest: []u8, source: []const u8) !usize {
        return decodeUnified(dest, source);
    }

    pub fn decodeURLSafe(dest: []u8, source: []const u8) !usize {
        return decodeUnified(dest, source);
    }

    pub fn decodeAlloc(allocator: std.mem.Allocator, source: []const u8) ![]u8 {
        const max_len = decodeLen(source.len);
        const dest = try allocator.alloc(u8, max_len);
        const actual_len = try decodeUnified(dest, source);
        return dest[0..actual_len];
    }

    pub fn decodeURLSafeAlloc(allocator: std.mem.Allocator, source: []const u8) ![]u8 {
        return decodeAlloc(allocator, source);
    }

    fn decodeUnified(dest: []u8, source: []const u8) !usize {
        var decode_table: [256]u8 = undefined;
        for (&decode_table) |*b| b.* = 0xFF;

        for (standard_alphabet, 0..) |c, i| {
            decode_table[c] = @intCast(i);
        }
        // Support URL-safe '-' and '_' seamlessly
        decode_table['-'] = 62;
        decode_table['_'] = 63;

        var acc: u32 = 0;
        var acc_len: u32 = 0;
        var out_idx: usize = 0;

        for (source) |c| {
            if (c == '=') break;
            if (c == ' ' or c == '\r' or c == '\n' or c == '\t') continue;
            const val = decode_table[c];
            if (val == 0xFF) return error.InvalidCharacter;

            acc = (acc << 6) | @as(u32, val);
            acc_len += 6;

            if (acc_len >= 8) {
                acc_len -= 8;
                dest[out_idx] = @truncate((acc >> @intCast(acc_len)) & 0xFF);
                out_idx += 1;
            }
        }

        return out_idx;
    }
};
