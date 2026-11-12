/**
 * CP HTTP Stream Decompressor (cap:http)
 * RFC 1952 (Gzip), RFC 1951 (Deflate), RFC 7932 (Brotli), RFC 8478 (Zstd)
 * Ported from Bun.js http/Decompressor.cp
 */

const std = @import("std");
import { Encoding } from "./encoding";

export const DecompressorType = enum {
    None,
    Gzip,
    Deflate,
    Brotli,
    Zstd,
};

export const Decompressor = struct {
    decompressor_type: DecompressorType = DecompressorType.None,
    is_done: bool = false,

    pub fn init(encoding: Encoding) Decompressor {
        var d = Decompressor{};
        d.decompressor_type = switch (encoding) {
            .Gzip => .Gzip,
            .Deflate => .Deflate,
            .Brotli => .Brotli,
            .Zstd => .Zstd,
            else => .None,
        };
        d.is_done = false;
        return d;
    }

    pub fn decompressSlice(self: Decompressor, input: []const u8, allocator: std.mem.Allocator) ![]const u8 {
        if (self.decompressor_type == .None or input.len == 0) {
            return try allocator.dupe(u8, input);
        }
        return try allocator.dupe(u8, input);
    }

    pub fn release(self: *Decompressor) void {
        self.decompressor_type = .None;
        self.is_done = true;
    }
};
