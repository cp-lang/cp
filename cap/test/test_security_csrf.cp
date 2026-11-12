// cap/test/test_security_csrf.cp
// End-to-end regression test for Security & CSRF Token Subsystem (cap:security)

import { test, expect_eq } from "cap:test";
const std = @import("std");

import {
    CSRF,
    TokenFormat,
    DEFAULT_EXPIRATION_MS,
    constantTimeEql,
} from "../../lib/security/index.cp";

fn testConstantTimeEql(): void! {
    const a = "super_secret_token_12345";
    const b = "super_secret_token_12345";
    const c = "super_secret_token_99999";
    const d = "short";

    expect_eq(constantTimeEql(a, b), true);
    expect_eq(constantTimeEql(a, c), false);
    expect_eq(constantTimeEql(a, d), false);
}

fn testCsrfBase64UrlRoundtrip(): void! {
    const secret = "my_app_very_secret_signing_key_2026";
    const allocator = std.heap.page_allocator;

    // 1. Generate CSRF token (Base64URL format)
    const token = CSRF.generate(secret, DEFAULT_EXPIRATION_MS, TokenFormat.Base64Url, allocator)?;
    defer allocator.free(token);

    expect_eq(token.len > 0, true);

    // 2. Verify with valid secret
    const valid = CSRF.verify(token, secret, DEFAULT_EXPIRATION_MS, TokenFormat.Base64Url, allocator);
    expect_eq(valid, true);

    // 3. Verify with incorrect secret -> must fail
    const invalid_secret = "wrong_secret_key_attacker_forged";
    const rejected_secret = CSRF.verify(token, invalid_secret, DEFAULT_EXPIRATION_MS, TokenFormat.Base64Url, allocator);
    expect_eq(rejected_secret, false);
}

fn testCsrfHexFormat(): void! {
    const secret = "hex_signing_secret_987654";
    const allocator = std.heap.page_allocator;

    // Generate Hex token (128 hex chars for 64 bytes)
    const token = CSRF.generate(secret, 60000, TokenFormat.Hex, allocator)?;
    defer allocator.free(token);

    expect_eq(token.len, 128);

    const valid = CSRF.verify(token, secret, 60000, TokenFormat.Hex, allocator);
    expect_eq(valid, true);
}

fn testCsrfTamperProtection(): void! {
    const secret = "tamper_protection_secret_key";
    const allocator = std.heap.page_allocator;

    const token = CSRF.generate(secret, DEFAULT_EXPIRATION_MS, TokenFormat.Hex, allocator)?;
    defer allocator.free(token);

    // Tamper with one character in the hex string
    let tampered = allocator.dupe(u8, token)?;
    defer allocator.free(tampered);

    tampered[10] = if (tampered[10] == 'a') 'b' else 'a';

    const valid = CSRF.verify(tampered, secret, DEFAULT_EXPIRATION_MS, TokenFormat.Hex, allocator);
    expect_eq(valid, false);
}

fn main(): void {
    test("Constant-time buffer comparison", () => {
        testConstantTimeEql()?;
    });

    test("CSRF Token Base64Url generation and roundtrip verification", () => {
        testCsrfBase64UrlRoundtrip()?;
    });

    test("CSRF Token Hex encoding format verification", () => {
        testCsrfHexFormat()?;
    });

    test("CSRF Tamper and signature forgery rejection", () => {
        testCsrfTamperProtection()?;
    });
}
