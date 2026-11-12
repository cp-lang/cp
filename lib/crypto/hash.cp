// lib/crypto/hash.cp
// High-performance cryptographic hashing (SHA256, SHA512, SHA1, MD5)

const std = @import("std");

pub enum HashAlgorithm {
    Sha256,
    Sha512,
    Sha1,
    Md5,

    pub static fn fromString(str: []const u8): ?HashAlgorithm {
        if (std.mem.eql(u8, str, "sha256") or std.mem.eql(u8, str, "SHA256")) return HashAlgorithm.Sha256;
        if (std.mem.eql(u8, str, "sha512") or std.mem.eql(u8, str, "SHA512")) return HashAlgorithm.Sha512;
        if (std.mem.eql(u8, str, "sha1") or std.mem.eql(u8, str, "SHA1")) return HashAlgorithm.Sha1;
        if (std.mem.eql(u8, str, "md5") or std.mem.eql(u8, str, "MD5")) return HashAlgorithm.Md5;
        return null;
    }
}

pub fn sha256(data: string): string {
    var out: [32]u8 = undefined;
    std.crypto.hash.sha2.Sha256.hash(data, &out, .{});
    const hex = std.fmt.bytesToHex(out, .lower);
    const res = std.heap.page_allocator.alloc(u8, 64) catch unreachable;
    @memcpy(res, hex[0..64]);
    return res;
}

pub fn sha512(data: string): string {
    var out: [64]u8 = undefined;
    std.crypto.hash.sha2.Sha512.hash(data, &out, .{});
    const hex = std.fmt.bytesToHex(out, .lower);
    const res = std.heap.page_allocator.alloc(u8, 128) catch unreachable;
    @memcpy(res, hex[0..128]);
    return res;
}

pub fn sha1(data: string): string {
    var out: [20]u8 = undefined;
    std.crypto.hash.Sha1.hash(data, &out, .{});
    const hex = std.fmt.bytesToHex(out, .lower);
    const res = std.heap.page_allocator.alloc(u8, 40) catch unreachable;
    @memcpy(res, hex[0..40]);
    return res;
}

pub fn md5(data: string): string {
    var out: [16]u8 = undefined;
    std.crypto.hash.Md5.hash(data, &out, .{});
    const hex = std.fmt.bytesToHex(out, .lower);
    const res = std.heap.page_allocator.alloc(u8, 32) catch unreachable;
    @memcpy(res, hex[0..32]);
    return res;
}

export class Hash {
    algo: HashAlgorithm = HashAlgorithm.Sha256;
    h256: std.crypto.hash.sha2.Sha256 = undefined;
    h512: std.crypto.hash.sha2.Sha512 = undefined;
    h1: std.crypto.hash.Sha1 = undefined;
    hmd5: std.crypto.hash.Md5 = undefined;

    pub static fn init(algo: HashAlgorithm): *Hash {
        let h = std.heap.page_allocator.create(Hash) catch unreachable;
        h.* = Hash{};
        h.algo = algo;
        switch (algo) {
            .Sha256 => { h.h256 = std.crypto.hash.sha2.Sha256.init(.{}); },
            .Sha512 => { h.h512 = std.crypto.hash.sha2.Sha512.init(.{}); },
            .Sha1 => { h.h1 = std.crypto.hash.Sha1.init(.{}); },
            .Md5 => { h.hmd5 = std.crypto.hash.Md5.init(.{}); },
        }
        return h;
    }

    pub fn update(mut self: *Hash, data: string): *Hash {
        switch (self.algo) {
            .Sha256 => { self.h256.update(data); },
            .Sha512 => { self.h512.update(data); },
            .Sha1 => { self.h1.update(data); },
            .Md5 => { self.hmd5.update(data); },
        }
        return self;
    }

    pub fn digest(mut self: *Hash): string {
        switch (self.algo) {
            .Sha256 => {
                var out: [32]u8 = undefined;
                self.h256.final(&out);
                const hex = std.fmt.bytesToHex(out, .lower);
                const res = std.heap.page_allocator.alloc(u8, 64) catch unreachable;
                @memcpy(res, hex[0..64]);
                return res;
            },
            .Sha512 => {
                var out: [64]u8 = undefined;
                self.h512.final(&out);
                const hex = std.fmt.bytesToHex(out, .lower);
                const res = std.heap.page_allocator.alloc(u8, 128) catch unreachable;
                @memcpy(res, hex[0..128]);
                return res;
            },
            .Sha1 => {
                var out: [20]u8 = undefined;
                self.h1.final(&out);
                const hex = std.fmt.bytesToHex(out, .lower);
                const res = std.heap.page_allocator.alloc(u8, 40) catch unreachable;
                @memcpy(res, hex[0..40]);
                return res;
            },
            .Md5 => {
                var out: [16]u8 = undefined;
                self.hmd5.final(&out);
                const hex = std.fmt.bytesToHex(out, .lower);
                const res = std.heap.page_allocator.alloc(u8, 32) catch unreachable;
                @memcpy(res, hex[0..32]);
                return res;
            },
        }
    }
}

pub fn createHash(algo: string): ?*Hash {
    const parsed = HashAlgorithm.fromString(algo);
    if (parsed == null) {
        return null;
    }
    return Hash.init(parsed.?);
}
