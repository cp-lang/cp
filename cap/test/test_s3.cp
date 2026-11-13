import { test, expect_eq } from "cap:test";
import {
    S3Acl,
    S3StorageClass,
    S3SignErrorCode,
    S3Error,
    S3Credentials,
    SigV4Signer
} from "cap:s3";

fn main(): void {
    test("1. S3Acl, S3StorageClass & S3Error mapping", () => {
        expect_eq(S3Acl.toString(S3Acl.PublicRead), "public-read");
        expect_eq(S3Acl.toString(S3Acl.BucketOwnerFullControl), "bucket-owner-full-control");
        let parsed_acl = S3Acl.fromString("authenticated-read");
        expect_eq(parsed_acl != null, true);
        if (parsed_acl) |acl| {
            expect_eq(acl == S3Acl.AuthenticatedRead, true);
        }

        expect_eq(S3StorageClass.toString(S3StorageClass.IntelligentTiering), "INTELLIGENT_TIERING");
        let parsed_sc = S3StorageClass.fromString("DEEP_ARCHIVE");
        expect_eq(parsed_sc != null, true);
        if (parsed_sc) |sc| {
            expect_eq(sc == S3StorageClass.DeepArchive, true);
        }

        let s3_err = S3Error.fromSignError(S3SignErrorCode.MissingCredentials);
        expect_eq(s3_err.code, "ERR_S3_MISSING_CREDENTIALS");
    });

    test("2. S3Credentials validation & Path-Style vs Virtual-Hosted URL formatting", () => {
        let empty_creds = S3Credentials{};
        expect_eq(empty_creds.validate() == S3SignErrorCode.MissingCredentials, true);

        let creds = S3Credentials{
            access_key_id: "AKIAIOSFODNN7EXAMPLE",
            secret_access_key: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
            region: "us-east-1",
            endpoint: "s3.amazonaws.com",
            bucket: "examplebucket",
            virtual_hosted_style: true,
        };
        expect_eq(creds.validate() == S3SignErrorCode.Ok, true);
        expect_eq(S3Credentials.isValidMethod("GET"), true);
        expect_eq(S3Credentials.isValidMethod("TRACE"), false);

        let host_buf: [64]u8 = undefined;
        let host_len = creds.formatHost(host_buf[0..64]);
        expect_eq(host_buf[0..host_len], "examplebucket.s3.amazonaws.com");

        let uri_buf: [64]u8 = undefined;
        let uri_len = creds.formatCanonicalUri("test.txt", uri_buf[0..64]);
        expect_eq(uri_buf[0..uri_len], "/test.txt");

        // Path-style 模式
        let path_creds = S3Credentials{
            access_key_id: "AKIAIOSFODNN7EXAMPLE",
            secret_access_key: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
            endpoint: "localhost:9000",
            bucket: "mybucket",
            virtual_hosted_style: false,
        };
        let p_uri_len = path_creds.formatCanonicalUri("photos/2026.jpg", uri_buf[0..64]);
        expect_eq(uri_buf[0..p_uri_len], "/mybucket/photos/2026.jpg");
    });

    test("3. AWS SigV4 (AWS4-HMAC-SHA256) official test vector verification", () => {
        // 空字符串 SHA-256 标准向量
        let empty_hex: [64]u8 = undefined;
        let h_len = SigV4Signer.sha256Hex("", empty_hex[0..64]);
        expect_eq(h_len, 64 as usize);
        expect_eq(
            empty_hex[0..64],
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );

        // AWS S3 官方文档 GET Object 标准测试向量：
        // Date: 20130524T000000Z, Region: us-east-1, Service: s3
        let creds = S3Credentials{
            access_key_id: "AKIAIOSFODNN7EXAMPLE",
            secret_access_key: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
            region: "us-east-1",
            endpoint: "s3.amazonaws.com",
            bucket: "examplebucket",
            virtual_hosted_style: true,
        };
        let canonical_request = "GET\n/test.txt\n\nhost:examplebucket.s3.amazonaws.com\nrange:bytes=0-9\nx-amz-content-sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\nx-amz-date:20130524T000000Z\n\nhost;range;x-amz-content-sha256;x-amz-date\ne3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

        let sig_hex: [64]u8 = undefined;
        let sig_len = SigV4Signer.signS3CanonicalRequest(
            &creds,
            "20130524",
            "20130524T000000Z",
            canonical_request,
            sig_hex[0..64]
        );
        expect_eq(sig_len, 64 as usize);
        expect_eq(
            sig_hex[0..64],
            "f0e8bdb87c964420e857bd35b5d6ed310bd44f0170aba48dd91039c6036bdb41"
        );
    });
}
