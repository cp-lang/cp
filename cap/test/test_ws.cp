// cap/test/test_ws.cp
const std = @import("std");
import { test, expect_eq } from "cap:test";
import { Opcode, CloseCode, FrameHeader, FrameBuilder, Mask } from "cap:ws";

fn testOpcodes(): !void {
    expect_eq(Opcode.Text.isControl(), false);
    expect_eq(Opcode.Binary.isControl(), false);
    expect_eq(Opcode.Ping.isControl(), true);
    expect_eq(Opcode.Pong.isControl(), true);
    expect_eq(Opcode.Close.isControl(), true);
}

fn testFrameHeaderEncodeDecode(): !void {
    // 1. Short unmasked text frame
    const h1 = FrameHeader{
        .fin = true,
        .opcode = .Text,
        .masked = false,
        .payload_len = 11,
    };
    var buf1: [32]u8 = undefined;
    const len1 = try h1.encode(&buf1);
    expect_eq(len1, 2);
    expect_eq(buf1[0], 0x81); // 0x80 (FIN) | 0x01 (Text)
    expect_eq(buf1[1], 11);

    const decoded1 = try FrameHeader.decode(buf1[0..len1]);
    expect_eq(decoded1.fin, true);
    expect_eq(decoded1.opcode, Opcode.Text);
    expect_eq(decoded1.masked, false);
    expect_eq(decoded1.payload_len, 11);

    // 2. Medium masked binary frame (payload_len == 300, 16-bit extended length)
    let mask_key: [u8; 4] = [0x12 as u8, 0x34 as u8, 0x56 as u8, 0x78 as u8];
    const h2 = FrameHeader{
        .fin = true,
        .opcode = .Binary,
        .masked = true,
        .payload_len = 300,
        .mask_key = mask_key,
    };
    var buf2: [32]u8 = undefined;
    const len2 = try h2.encode(&buf2);
    expect_eq(len2, 2 + 2 + 4); // base 2 + ext 2 + mask 4 = 8 bytes
    expect_eq(buf2[0], 0x82); // 0x80 (FIN) | 0x02 (Binary)
    expect_eq(buf2[1], 0x80 | 126); // MASK bit | 126

    const decoded2 = try FrameHeader.decode(buf2[0..len2]);
    expect_eq(decoded2.fin, true);
    expect_eq(decoded2.opcode, Opcode.Binary);
    expect_eq(decoded2.masked, true);
    expect_eq(decoded2.payload_len, 300);
    expect_eq(std.mem.eql(u8, &decoded2.mask_key, &mask_key), true);
}

fn testFrameBuilderAndMasking(): !void {
    var frame_buf: [256]u8 = undefined;
    const message = "WebSocket in CP!";
    let mask_key: [u8; 4] = [0xAA as u8, 0xBB as u8, 0xCC as u8, 0xDD as u8];

    // Build masked text frame
    const frame_len = try FrameBuilder.buildText(&frame_buf, message, mask_key);
    expect_eq(frame_len, 2 + 4 + message.len);

    // Decode header
    const header = try FrameHeader.decode(frame_buf[0..frame_len]);
    expect_eq(header.opcode, Opcode.Text);
    expect_eq(header.masked, true);
    expect_eq(header.payload_len, message.len);

    // Unmask payload and verify content
    var payload: [64]u8 = [0 as u8; 64];
    @memcpy(payload[0..message.len], frame_buf[header.header_len .. frame_len]);
    Mask.apply(payload[0..message.len], header.mask_key);

    expect_eq(std.mem.eql(u8, payload[0..message.len], message), true);
}

fn testControlFrames(): !void {
    var ping_buf: [64]u8 = undefined;
    const ping_len = try FrameBuilder.buildPing(&ping_buf, "heartbeat", null);
    const ping_header = try FrameHeader.decode(ping_buf[0..ping_len]);
    expect_eq(ping_header.opcode, Opcode.Ping);
    expect_eq(ping_header.payload_len, 9);

    var close_buf: [64]u8 = undefined;
    const close_len = try FrameBuilder.buildClose(&close_buf, .NormalClosure, "bye", null);
    const close_header = try FrameHeader.decode(close_buf[0..close_len]);
    expect_eq(close_header.opcode, Opcode.Close);
    expect_eq(close_header.payload_len, 2 + 3); // 2 bytes code + 3 bytes "bye"
}

fn main(): void {
    test("WebSocket Opcode classifications", () => {
        try testOpcodes();
    });
    test("WebSocket FrameHeader encode and decode", () => {
        try testFrameHeaderEncodeDecode();
    });
    test("WebSocket FrameBuilder and XOR masking roundtrip", () => {
        try testFrameBuilderAndMasking();
    });
    test("WebSocket Ping and Close control frames", () => {
        try testControlFrames();
    });
}
