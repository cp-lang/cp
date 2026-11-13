// ============================================================================
// lib/s3/signing.cp — AWS Signature Version 4 (AWS4-HMAC-SHA256) 签名引擎
// 吸收自 Bun src/s3_signing/credentials.cp
// ============================================================================

import { S3Credentials } from "./credentials.cp";

const HEX_LOWER: []const u8 = "0123456789abcdef";

/// AWS SigV4 SHA-256 与四级 HMAC-SHA256 派生签名工具
export struct SigV4Signer {
    /// 计算输入字节流的 SHA-256 并写入 64 字节小写十六进制缓冲区 `out_hex[0..64]`
    pub static fn sha256Hex(data: []const u8, out_hex: []u8): usize {
        if (out_hex.len < 64) { return 0; }
        let digest: [32]u8 = undefined;
        let d_slice = digest[0..32];
        if (data.len >= 0 && d_slice.len == 32) {
            zig {
                std.crypto.hash.sha2.Sha256.hash(data, d_slice[0..32], .{});
            }
        }
        return bytesToHexLower(d_slice, out_hex);
    }

    /// 计算 `HMAC-SHA256(key, message)` 并将 32 字节原始 MAC 写入 `out_mac[0..32]`
    pub static fn hmacSha256(key: []const u8, message: []const u8, out_mac: []u8): void {
        if (out_mac.len < 32) { return; }
        if (key.len >= 0 && message.len >= 0) {
            zig {
                std.crypto.auth.hmac.sha2.HmacSha256.create(out_mac[0..32], message, key);
            }
        }
    }

    /// 派生 AWS SigV4 四级签名密钥：
    /// `kDate    = HMAC("AWS4" + secret, date_yyyymmdd)`
    /// `kRegion  = HMAC(kDate, region)`
    /// `kService = HMAC(kRegion, service)`
    /// `kSigning = HMAC(kService, "aws4_request")`
    pub static fn deriveSigningKey(
        secret_access_key: []const u8,
        date_yyyymmdd: []const u8,
        region: []const u8,
        service: []const u8,
        out_signing_key: []u8
    ): bool {
        if (out_signing_key.len < 32 || secret_access_key.len > 240) { return false; }
        let k_secret_buf: [256]u8 = undefined;
        k_secret_buf[0] = 65; // 'A'
        k_secret_buf[1] = 87; // 'W'
        k_secret_buf[2] = 83; // 'S'
        k_secret_buf[3] = 52; // '4'
        let i: usize = 0;
        while (i < secret_access_key.len) {
            k_secret_buf[4 + i] = secret_access_key[i];
            i += 1;
        }
        let k_secret = k_secret_buf[0 .. 4 + secret_access_key.len];

        let k_date: [32]u8 = undefined;
        SigV4Signer.hmacSha256(k_secret, date_yyyymmdd, k_date[0..32]);

        let k_region: [32]u8 = undefined;
        SigV4Signer.hmacSha256(k_date[0..32], region, k_region[0..32]);

        let k_service: [32]u8 = undefined;
        SigV4Signer.hmacSha256(k_region[0..32], service, k_service[0..32]);

        SigV4Signer.hmacSha256(k_service[0..32], "aws4_request", out_signing_key[0..32]);
        return true;
    }

    /// 对 `string_to_sign` 计算最终 64 字节小写 Hex 签名
    pub static fn signStringToSign(
        signing_key: []const u8,
        string_to_sign: []const u8,
        out_sig_hex: []u8
    ): usize {
        if (signing_key.len < 32 || out_sig_hex.len < 64) { return 0; }
        let mac: [32]u8 = undefined;
        SigV4Signer.hmacSha256(signing_key[0..32], string_to_sign, mac[0..32]);
        return bytesToHexLower(mac[0..32], out_sig_hex);
    }

    /// 构造 Credential Scope: `<date>/<region>/<service>/aws4_request`
    pub static fn formatCredentialScope(
        date_yyyymmdd: []const u8,
        region: []const u8,
        service: []const u8,
        out: []u8
    ): usize {
        let needed = date_yyyymmdd.len + 1 + region.len + 1 + service.len + 13;
        if (out.len < needed) { return 0; }
        let pos: usize = 0;
        pos = appendSlice(out, pos, date_yyyymmdd);
        out[pos] = 47; // '/'
        pos += 1;
        pos = appendSlice(out, pos, region);
        out[pos] = 47; // '/'
        pos += 1;
        pos = appendSlice(out, pos, service);
        pos = appendSlice(out, pos, "/aws4_request");
        return pos;
    }

    /// 一键计算 S3 请求的 64 字节 Hex 签名
    pub static fn signS3CanonicalRequest(
        creds: &S3Credentials,
        date_yyyymmdd: []const u8,
        amz_date_iso8601: []const u8,
        canonical_request: []const u8,
        out_sig_hex: []u8
    ): usize {
        if (out_sig_hex.len < 64) { return 0; }
        let cr_hash_hex: [64]u8 = undefined;
        let h_len = SigV4Signer.sha256Hex(canonical_request, cr_hash_hex[0..64]);
        if (h_len != 64) { return 0; }

        let scope_buf: [128]u8 = undefined;
        let scope_len = SigV4Signer.formatCredentialScope(
            date_yyyymmdd,
            creds.effectiveRegion(),
            "s3",
            scope_buf[0..128]
        );
        if (scope_len == 0) { return 0; }

        // StringToSign = "AWS4-HMAC-SHA256\n" + amz_date + "\n" + scope + "\n" + sha256(CanonicalRequest)
        let sts_buf: [512]u8 = undefined;
        let pos: usize = 0;
        pos = appendSlice(sts_buf[0..512], pos, "AWS4-HMAC-SHA256\n");
        pos = appendSlice(sts_buf[0..512], pos, amz_date_iso8601);
        sts_buf[pos] = 10; // '\n'
        pos += 1;
        pos = appendSlice(sts_buf[0..512], pos, scope_buf[0..scope_len]);
        sts_buf[pos] = 10; // '\n'
        pos += 1;
        pos = appendSlice(sts_buf[0..512], pos, cr_hash_hex[0..64]);

        let signing_key: [32]u8 = undefined;
        if (!SigV4Signer.deriveSigningKey(
            creds.secret_access_key,
            date_yyyymmdd,
            creds.effectiveRegion(),
            "s3",
            signing_key[0..32]
        )) {
            return 0;
        }
        return SigV4Signer.signStringToSign(signing_key[0..32], sts_buf[0..pos], out_sig_hex);
    }
}

fn bytesToHexLower(bytes: []const u8, out: []u8): usize {
    let i: usize = 0;
    while (i < bytes.len) {
        let b = bytes[i];
        out[i * 2] = HEX_LOWER[(b >> 4) as usize];
        out[i * 2 + 1] = HEX_LOWER[(b & 0x0F) as usize];
        i += 1;
    }
    return bytes.len * 2;
}

fn appendSlice(dst: []u8, offset: usize, src: []const u8): usize {
    let i: usize = 0;
    while (i < src.len) {
        dst[offset + i] = src[i];
        i += 1;
    }
    return offset + src.len;
}
