// lib/crypto/random.cp
// Cryptographically secure pseudorandom number generator (CSPRNG) & RFC 4122 v4 UUID

const std = @import("std");

pub fn randomBytes(size: usize): []u8 {
    const buf = std.heap.page_allocator.alloc(u8, size) catch unreachable;
    _ = std.os.linux.getrandom(buf.ptr, size, 0);
    return buf;
}

pub fn randomUUID(): string {
    var b: [16]u8 = undefined;
    _ = std.os.linux.getrandom(&b, 16, 0);
    b[6] = (b[6] & 0x0f) | 0x40;
    b[8] = (b[8] & 0x3f) | 0x80;

    const hex = std.fmt.bytesToHex(b, .lower);
    var uuid_buf = std.heap.page_allocator.alloc(u8, 36) catch unreachable;
    @memcpy(uuid_buf[0..8], hex[0..8]);
    uuid_buf[8] = 45; // '-'
    @memcpy(uuid_buf[9..13], hex[8..12]);
    uuid_buf[13] = 45;
    @memcpy(uuid_buf[14..18], hex[12..16]);
    uuid_buf[18] = 45;
    @memcpy(uuid_buf[19..23], hex[16..20]);
    uuid_buf[23] = 45;
    @memcpy(uuid_buf[24..36], hex[20..32]);

    return uuid_buf;
}

pub fn randomInt(min: i64, max: i64): i64 {
    if (min >= max) {
        return min;
    }
    const range: u64 = @intCast(max - min);
    var b: [8]u8 = undefined;
    _ = std.os.linux.getrandom(&b, 8, 0);
    const val: u64 = std.mem.readInt(u64, &b, std.builtin.Endian.little);
    return min + @as(i64, @intCast(val % range));
}
