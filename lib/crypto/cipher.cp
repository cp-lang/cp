// lib/crypto/cipher.cp
// Authenticated Encryption with Associated Data (AEAD) Ciphers: AES-256-GCM & ChaCha20-Poly1305

const std = @import("std");

export class Aes256Gcm {
    pub const KEY_LENGTH: usize = 32;
    pub const NONCE_LENGTH: usize = 12;
    pub const TAG_LENGTH: usize = 16;

    pub static fn encrypt(
        key: []const u8,
        nonce: []const u8,
        plaintext: string,
        ad: string,
        ciphertext: []u8,
        tag: *[16]u8,
    ): void {
        var k: [32]u8 = undefined;
        @memcpy(&k, key[0..32]);
        var n: [12]u8 = undefined;
        @memcpy(&n, nonce[0..12]);
        std.crypto.aead.aes_gcm.Aes256Gcm.encrypt(@constCast(ciphertext), tag, plaintext, ad, n, k);
    }

    pub static fn decrypt(
        key: []const u8,
        nonce: []const u8,
        ciphertext: []const u8,
        tag: [16]u8,
        ad: string,
        plaintext: []u8,
    ): bool {
        var k: [32]u8 = undefined;
        @memcpy(&k, key[0..32]);
        var n: [12]u8 = undefined;
        @memcpy(&n, nonce[0..12]);
        std.crypto.aead.aes_gcm.Aes256Gcm.decrypt(@constCast(plaintext), ciphertext, tag, ad, n, k) catch return false;
        return true;
    }
}

export class ChaCha20Poly1305 {
    pub const KEY_LENGTH: usize = 32;
    pub const NONCE_LENGTH: usize = 12;
    pub const TAG_LENGTH: usize = 16;

    pub static fn encrypt(
        key: []const u8,
        nonce: []const u8,
        plaintext: string,
        ad: string,
        ciphertext: []u8,
        tag: *[16]u8,
    ): void {
        var k: [32]u8 = undefined;
        @memcpy(&k, key[0..32]);
        var n: [12]u8 = undefined;
        @memcpy(&n, nonce[0..12]);
        std.crypto.aead.chacha_poly.ChaCha20Poly1305.encrypt(@constCast(ciphertext), tag, plaintext, ad, n, k);
    }

    pub static fn decrypt(
        key: []const u8,
        nonce: []const u8,
        ciphertext: []const u8,
        tag: [16]u8,
        ad: string,
        plaintext: []u8,
    ): bool {
        var k: [32]u8 = undefined;
        @memcpy(&k, key[0..32]);
        var n: [12]u8 = undefined;
        @memcpy(&n, nonce[0..12]);
        std.crypto.aead.chacha_poly.ChaCha20Poly1305.decrypt(@constCast(plaintext), ciphertext, tag, ad, n, k) catch return false;
        return true;
    }
}
