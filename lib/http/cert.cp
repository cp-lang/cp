/**
 * CP HTTP TLS Certificate Info & Errors (cap:http)
 * Ported from Bun.js http/CertificateInfo.cp & HTTPCertError.cp
 */

const std = @import("std");

export const HTTPCertError = struct {
    error_no: i32 = 0,
    code: []const u8 = "",
    reason: []const u8 = "",
};

export const CertificateInfo = struct {
    cert: []const u8 = "",
    cert_error: HTTPCertError = HTTPCertError{},
    hostname: []const u8 = "",

    pub fn deinit(self: *CertificateInfo, allocator: std.mem.Allocator) void {
        if (self.cert.len > 0) allocator.free(self.cert);
        if (self.cert_error.code.len > 0) allocator.free(self.cert_error.code);
        if (self.cert_error.reason.len > 0) allocator.free(self.cert_error.reason);
        if (self.hostname.len > 0) allocator.free(self.hostname);
    }
};

export error InitError {
    FailedToOpenSocket,
    LoadCAFile,
    InvalidCAFile,
    InvalidCA,
}
