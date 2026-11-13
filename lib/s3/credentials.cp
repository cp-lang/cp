// ============================================================================
// lib/s3/credentials.cp — S3 凭证模型、校验与 Path-Style / Virtual-Hosted 端点拼接
// 吸收自 Bun src/s3_signing/credentials.cp
// ============================================================================

import { S3Acl, S3StorageClass, S3SignErrorCode } from "./types.cp";

export struct S3Credentials {
    access_key_id: []const u8 = "",
    secret_access_key: []const u8 = "",
    region: []const u8 = "us-east-1",
    endpoint: []const u8 = "s3.us-east-1.amazonaws.com",
    bucket: []const u8 = "",
    session_token: []const u8 = "",
    acl: S3Acl? = null,
    storage_class: S3StorageClass? = null,
    insecure_http: bool = false,
    virtual_hosted_style: bool = false,

    pub fn validate(&this): S3SignErrorCode {
        if (this.access_key_id.len == 0 || this.secret_access_key.len == 0 || this.bucket.len == 0) {
            return S3SignErrorCode.MissingCredentials;
        }
        if (this.endpoint.len == 0) {
            return S3SignErrorCode.InvalidEndpoint;
        }
        return S3SignErrorCode.Ok;
    }

    pub fn effectiveRegion(&this): []const u8 {
        if (this.region.len > 0) { return this.region; }
        return "us-east-1";
    }

    pub fn effectiveEndpoint(&this): []const u8 {
        if (this.endpoint.len > 0) { return this.endpoint; }
        return "s3.us-east-1.amazonaws.com";
    }

    pub static fn isValidMethod(method: []const u8): bool {
        return method == "GET" || method == "PUT" || method == "DELETE" || method == "HEAD" || method == "POST";
    }

    /// 构造 S3 请求的 Host 头与 Canonical URI（写入调用方栈缓冲区）
    /// - Virtual-Hosted Style: Host = `<bucket>.<endpoint>`, URI = `/<key>`
    /// - Path Style:           Host = `<endpoint>`,          URI = `/<bucket>/<key>`
    pub fn formatHost(&this, out: []u8): usize {
        let ep = this.effectiveEndpoint();
        if (!this.virtual_hosted_style) {
            if (out.len < ep.len) { return 0; }
            copyBytes(out, 0, ep);
            return ep.len;
        }
        let needed = this.bucket.len + 1 + ep.len;
        if (out.len < needed) { return 0; }
        let pos = copyBytes(out, 0, this.bucket);
        out[pos] = 46; // '.'
        pos += 1;
        pos = copyBytes(out, pos, ep);
        return pos;
    }

    pub fn formatCanonicalUri(&this, key: []const u8, out: []u8): usize {
        let clean_key = if (key.len > 0 && key[0] == 47) key[1..key.len] else key;
        if (this.virtual_hosted_style) {
            let needed = 1 + clean_key.len;
            if (out.len < needed) { return 0; }
            out[0] = 47; // '/'
            return copyBytes(out, 1, clean_key);
        }
        let needed = 1 + this.bucket.len + 1 + clean_key.len;
        if (out.len < needed) { return 0; }
        out[0] = 47; // '/'
        let pos = copyBytes(out, 1, this.bucket);
        out[pos] = 47; // '/'
        pos += 1;
        return copyBytes(out, pos, clean_key);
    }
}

fn copyBytes(dst: []u8, offset: usize, src: []const u8): usize {
    let i: usize = 0;
    while (i < src.len) {
        dst[offset + i] = src[i];
        i += 1;
    }
    return offset + src.len;
}
