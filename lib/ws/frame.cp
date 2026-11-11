// lib/ws/frame.cp
// RFC 6455 WebSocket Frame Header & Builder in CP
const std = @import("std");
import { Opcode, CloseCode } from "./opcode";
import { Mask } from "./mask";

pub const FrameHeader = struct {
    fin: bool = true,
    opcode: Opcode = .Text,
    masked: bool = false,
    payload_len: usize = 0,
    mask_key: [4]u8 = .{ 0, 0, 0, 0 },
    header_len: usize = 2,

    pub fn encode(self: FrameHeader, out: []u8) !usize {
        var needed_len: usize = 2;
        if (self.payload_len >= 126 and self.payload_len <= 0xFFFF) {
            needed_len += 2;
        } else if (self.payload_len > 0xFFFF) {
            needed_len += 8;
        }
        if (self.masked) {
            needed_len += 4;
        }

        if (out.len < needed_len) return error.BufferTooSmall;

        // Byte 0: FIN + RSV + Opcode
        var b0: u8 = @intFromEnum(self.opcode);
        if (self.fin) b0 |= 0x80;
        out[0] = b0;

        // Byte 1: Mask bit + 7-bit length
        var b1: u8 = 0;
        if (self.masked) b1 |= 0x80;

        var offset: usize = 2;
        if (self.payload_len <= 125) {
            b1 |= @as(u8, @truncate(self.payload_len));
            out[1] = b1;
        } else if (self.payload_len <= 0xFFFF) {
            b1 |= 126;
            out[1] = b1;
            std.mem.writeInt(u16, out[offset .. offset + 2][0..2], @as(u16, @truncate(self.payload_len)), .big);
            offset += 2;
        } else {
            b1 |= 127;
            out[1] = b1;
            std.mem.writeInt(u64, out[offset .. offset + 8][0..8], @as(u64, self.payload_len), .big);
            offset += 8;
        }

        // Masking Key
        if (self.masked) {
            @memcpy(out[offset .. offset + 4], &self.mask_key);
            offset += 4;
        }

        return offset;
    }

    pub fn decode(buf: []const u8) !FrameHeader {
        if (buf.len < 2) return error.IncompleteHeader;

        const b0 = buf[0];
        const b1 = buf[1];

        const fin = (b0 & 0x80) != 0;
        const opcode_val: u4 = @truncate(b0 & 0x0F);
        const opcode: Opcode = @enumFromInt(opcode_val);
        const masked = (b1 & 0x80) != 0;
        const len7 = b1 & 0x7F;

        var offset: usize = 2;
        var payload_len: usize = 0;

        if (len7 <= 125) {
            payload_len = len7;
        } else if (len7 == 126) {
            if (buf.len < offset + 2) return error.IncompleteHeader;
            payload_len = std.mem.readInt(u16, buf[offset .. offset + 2][0..2], .big);
            offset += 2;
        } else {
            if (buf.len < offset + 8) return error.IncompleteHeader;
            payload_len = @as(usize, @truncate(std.mem.readInt(u64, buf[offset .. offset + 8][0..8], .big)));
            offset += 8;
        }

        var mask_key: [4]u8 = .{ 0, 0, 0, 0 };
        if (masked) {
            if (buf.len < offset + 4) return error.IncompleteHeader;
            @memcpy(&mask_key, buf[offset .. offset + 4]);
            offset += 4;
        }

        return FrameHeader{
            .fin = fin,
            .opcode = opcode,
            .masked = masked,
            .payload_len = payload_len,
            .mask_key = mask_key,
            .header_len = offset,
        };
    }
};

pub const FrameBuilder = struct {
    pub fn build(out: []u8, opcode: Opcode, payload: []const u8, mask: ?[4]u8) !usize {
        const header = FrameHeader{
            .fin = true,
            .opcode = opcode,
            .masked = mask != null,
            .payload_len = payload.len,
            .mask_key = if (mask) |k| k else .{ 0, 0, 0, 0 },
        };

        const header_len = try header.encode(out);
        if (out.len < header_len + payload.len) return error.BufferTooSmall;

        @memcpy(out[header_len .. header_len + payload.len], payload);

        if (mask) |k| {
            Mask.apply(out[header_len .. header_len + payload.len], k);
        }

        return header_len + payload.len;
    }

    pub fn buildText(out: []u8, text: []const u8, mask: ?[4]u8) !usize {
        return build(out, .Text, text, mask);
    }

    pub fn buildBinary(out: []u8, data: []const u8, mask: ?[4]u8) !usize {
        return build(out, .Binary, data, mask);
    }

    pub fn buildPing(out: []u8, payload: []const u8, mask: ?[4]u8) !usize {
        return build(out, .Ping, payload, mask);
    }

    pub fn buildPong(out: []u8, payload: []const u8, mask: ?[4]u8) !usize {
        return build(out, .Pong, payload, mask);
    }

    pub fn buildClose(out: []u8, code: CloseCode, reason: []const u8, mask: ?[4]u8) !usize {
        var payload_buf: [128]u8 = undefined;
        std.mem.writeInt(u16, payload_buf[0..2], @intFromEnum(code), .big);
        var total_len: usize = 2;
        if (reason.len > 0) {
            const copy_len = @min(reason.len, payload_buf.len - 2);
            @memcpy(payload_buf[2 .. 2 + copy_len], reason[0..copy_len]);
            total_len += copy_len;
        }
        return build(out, .Close, payload_buf[0..total_len], mask);
    }
};
