// ============================================================================
// lib/s3/types.cp — S3 访问控制列表 (ACL)、存储类型 (StorageClass) 与错误码
// 吸收自 Bun src/s3_signing/acl.cp, storage_class.cp, error.cp
// ============================================================================

export enum S3Acl: u8 {
    Private = 0,
    PublicRead = 1,
    PublicReadWrite = 2,
    AwsExecRead = 3,
    AuthenticatedRead = 4,
    BucketOwnerRead = 5,
    BucketOwnerFullControl = 6,
    LogDeliveryWrite = 7,

    pub static fn toString(acl: S3Acl): []const u8 {
        if (acl == S3Acl.Private) { return "private"; }
        if (acl == S3Acl.PublicRead) { return "public-read"; }
        if (acl == S3Acl.PublicReadWrite) { return "public-read-write"; }
        if (acl == S3Acl.AwsExecRead) { return "aws-exec-read"; }
        if (acl == S3Acl.AuthenticatedRead) { return "authenticated-read"; }
        if (acl == S3Acl.BucketOwnerRead) { return "bucket-owner-read"; }
        if (acl == S3Acl.BucketOwnerFullControl) { return "bucket-owner-full-control"; }
        return "log-delivery-write";
    }

    pub static fn fromString(text: []const u8): S3Acl? {
        if (text == "private") { return S3Acl.Private; }
        if (text == "public-read") { return S3Acl.PublicRead; }
        if (text == "public-read-write") { return S3Acl.PublicReadWrite; }
        if (text == "aws-exec-read") { return S3Acl.AwsExecRead; }
        if (text == "authenticated-read") { return S3Acl.AuthenticatedRead; }
        if (text == "bucket-owner-read") { return S3Acl.BucketOwnerRead; }
        if (text == "bucket-owner-full-control") { return S3Acl.BucketOwnerFullControl; }
        if (text == "log-delivery-write") { return S3Acl.LogDeliveryWrite; }
        return null;
    }
}

export enum S3StorageClass: u8 {
    Standard = 0,
    StandardIa = 1,
    IntelligentTiering = 2,
    ExpressOnezone = 3,
    OnezoneIa = 4,
    Glacier = 5,
    GlacierIr = 6,
    ReducedRedundancy = 7,
    Outposts = 8,
    DeepArchive = 9,
    Snow = 10,

    pub static fn toString(sc: S3StorageClass): []const u8 {
        if (sc == S3StorageClass.Standard) { return "STANDARD"; }
        if (sc == S3StorageClass.StandardIa) { return "STANDARD_IA"; }
        if (sc == S3StorageClass.IntelligentTiering) { return "INTELLIGENT_TIERING"; }
        if (sc == S3StorageClass.ExpressOnezone) { return "EXPRESS_ONEZONE"; }
        if (sc == S3StorageClass.OnezoneIa) { return "ONEZONE_IA"; }
        if (sc == S3StorageClass.Glacier) { return "GLACIER"; }
        if (sc == S3StorageClass.GlacierIr) { return "GLACIER_IR"; }
        if (sc == S3StorageClass.ReducedRedundancy) { return "REDUCED_REDUNDANCY"; }
        if (sc == S3StorageClass.Outposts) { return "OUTPOSTS"; }
        if (sc == S3StorageClass.DeepArchive) { return "DEEP_ARCHIVE"; }
        return "SNOW";
    }

    pub static fn fromString(text: []const u8): S3StorageClass? {
        if (text == "STANDARD") { return S3StorageClass.Standard; }
        if (text == "STANDARD_IA") { return S3StorageClass.StandardIa; }
        if (text == "INTELLIGENT_TIERING") { return S3StorageClass.IntelligentTiering; }
        if (text == "EXPRESS_ONEZONE") { return S3StorageClass.ExpressOnezone; }
        if (text == "ONEZONE_IA") { return S3StorageClass.OnezoneIa; }
        if (text == "GLACIER") { return S3StorageClass.Glacier; }
        if (text == "GLACIER_IR") { return S3StorageClass.GlacierIr; }
        if (text == "REDUCED_REDUNDANCY") { return S3StorageClass.ReducedRedundancy; }
        if (text == "OUTPOSTS") { return S3StorageClass.Outposts; }
        if (text == "DEEP_ARCHIVE") { return S3StorageClass.DeepArchive; }
        if (text == "SNOW") { return S3StorageClass.Snow; }
        return null;
    }
}

export enum S3SignErrorCode: u8 {
    Ok = 0,
    MissingCredentials = 1,
    InvalidMethod = 2,
    InvalidPath = 3,
    InvalidEndpoint = 4,
    InvalidSessionToken = 5,
    SignError = 6,
}

export struct S3Error {
    code: []const u8 = "",
    message: []const u8 = "",

    pub static fn fromSignError(kind: S3SignErrorCode): S3Error {
        if (kind == S3SignErrorCode.MissingCredentials) {
            return S3Error{
                code: "ERR_S3_MISSING_CREDENTIALS",
                message: "Missing S3 credentials. 'accessKeyId', 'secretAccessKey', 'bucket', and 'endpoint' are required",
            };
        }
        if (kind == S3SignErrorCode.InvalidMethod) {
            return S3Error{
                code: "ERR_S3_INVALID_METHOD",
                message: "Method must be GET, PUT, DELETE or HEAD when using s3:// protocol",
            };
        }
        if (kind == S3SignErrorCode.InvalidPath) {
            return S3Error{
                code: "ERR_S3_INVALID_PATH",
                message: "Invalid S3 bucket, key combination",
            };
        }
        if (kind == S3SignErrorCode.InvalidEndpoint) {
            return S3Error{
                code: "ERR_S3_INVALID_ENDPOINT",
                message: "Invalid S3 endpoint",
            };
        }
        if (kind == S3SignErrorCode.InvalidSessionToken) {
            return S3Error{
                code: "ERR_S3_INVALID_SESSION_TOKEN",
                message: "Invalid session token",
            };
        }
        return S3Error{
            code: "ERR_S3_INVALID_SIGNATURE",
            message: "Failed to retrieve S3 content. Are the credentials correct?",
        };
    }
}
