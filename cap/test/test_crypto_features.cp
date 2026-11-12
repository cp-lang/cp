// cap/test/test_crypto_features.cp
// Comprehensive End-to-End Test Suite for Industrial Cryptography Subsystem (cap:crypto)

import { test, expect_eq } from "cap:test";
import {
    sha256,
    sha512,
    sha1,
    md5,
    createHash,
    HashAlgorithm,
    hmacSha256,
    createHmac,
    HmacAlgorithm,
    randomBytes,
    randomUUID,
    randomInt,
    pbkdf2Sync,
    Aes256Gcm,
    ChaCha20Poly1305,
} from "cap:crypto";

fn testRandomAndUUID(): !void {
    const bytes = randomBytes(32);
    expect_eq(bytes.len, 32);

    const uuid = randomUUID();
    expect_eq(uuid.len, 36);
    expect_eq(uuid[8], 45);  // '-'
    expect_eq(uuid[13], 45); // '-'
    expect_eq(uuid[18], 45); // '-'
    expect_eq(uuid[23], 45); // '-'
    expect_eq(uuid[14], 52); // '4' (UUID v4)

    const r_val = randomInt(10, 50);
    expect_eq(r_val >= 10, true);
    expect_eq(r_val < 50, true);
}

fn testHashAlgorithms(): !void {
    // 1. One-shot standard test vectors
    const h256 = sha256("hello world");
    expect_eq(h256, "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9");

    const h_md5 = md5("hello world");
    expect_eq(h_md5, "5eb63bbbe01eeed093cb22bb8f5acdc3");

    const h_sha1 = sha1("hello world");
    expect_eq(h_sha1, "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed");

    const h512 = sha512("hello world");
    expect_eq(h512.len, 128);

    // 2. Streaming Hasher
    var stream_hasher = createHash("sha256").?;
    stream_hasher.update("hello ");
    stream_hasher.update("world");
    const stream_digest = stream_hasher.digest();
    expect_eq(stream_digest, h256);
}

fn testHmacAndPbkdf2(): !void {
    // 1. One-shot HMAC-SHA256
    const hmac_res = hmacSha256("secret", "hello world");
    expect_eq(hmac_res.len, 64);

    // 2. Streaming HMAC
    var stream_hmac = createHmac("sha256", "secret").?;
    stream_hmac.update("hello ");
    stream_hmac.update("world");
    const stream_digest = stream_hmac.digest();
    expect_eq(stream_digest, hmac_res);

    // 3. PBKDF2 Key Derivation
    const derived = pbkdf2Sync("password", "salt", 1000, 32);
    expect_eq(derived, "632c2812e46d4604102ba7618e9d6d7d2f8128f6266b4a03264d2a0460b7dcb3");
}

fn testAeadCiphers(): !void {
    // 1. AES-256-GCM Round-trip
    var aes_key: [32]u8 = undefined;
    @memset(&aes_key, 1);
    var aes_nonce: [12]u8 = undefined;
    @memset(&aes_nonce, 2);

    const plaintext = "Hello Secret World!";
    var ciphertext: [19]u8 = undefined;
    var tag: [16]u8 = undefined;

    Aes256Gcm.encrypt(&aes_key, &aes_nonce, plaintext, "auth-data", &ciphertext, &tag);

    var decrypted: [19]u8 = undefined;
    const ok = Aes256Gcm.decrypt(&aes_key, &aes_nonce, &ciphertext, tag, "auth-data", &decrypted);
    expect_eq(ok, true);
    expect_eq(decrypted[0..19], plaintext);

    // 2. ChaCha20-Poly1305 Round-trip
    var chacha_key: [32]u8 = undefined;
    @memset(&chacha_key, 3);
    var chacha_nonce: [12]u8 = undefined;
    @memset(&chacha_nonce, 4);

    var chacha_ciphertext: [19]u8 = undefined;
    var chacha_tag: [16]u8 = undefined;

    ChaCha20Poly1305.encrypt(&chacha_key, &chacha_nonce, plaintext, "", &chacha_ciphertext, &chacha_tag);

    var chacha_decrypted: [19]u8 = undefined;
    const chacha_ok = ChaCha20Poly1305.decrypt(&chacha_key, &chacha_nonce, &chacha_ciphertext, chacha_tag, "", &chacha_decrypted);
    expect_eq(chacha_ok, true);
    expect_eq(chacha_decrypted[0..19], plaintext);
}

fn main(): void {
    test("CSPRNG Random & RFC 4122 v4 UUID Generation", () => {
        testRandomAndUUID()?;
    });

    test("Cryptographic Hashing (SHA256, SHA512, SHA1, MD5 & Streaming)", () => {
        testHashAlgorithms()?;
    });

    test("HMAC Message Authentication & PBKDF2 Key Derivation", () => {
        testHmacAndPbkdf2()?;
    });

    test("AEAD Symmetric Ciphers (AES-256-GCM & ChaCha20-Poly1305)", () => {
        testAeadCiphers()?;
    });
}
