// lib/http/mime.cp
// Fast MIME type mapping based on file extension
const std = @import("std");

pub const MimeType = struct {
    pub fn fromExtension(ext: []const u8): []const u8 {
        var e = ext;
        if (e.len > 0 and e[0] == '.') e = e[1..];

        if (std.ascii.eqlIgnoreCase(e, "html") or std.ascii.eqlIgnoreCase(e, "htm")) return "text/html; charset=utf-8";
        if (std.ascii.eqlIgnoreCase(e, "json")) return "application/json; charset=utf-8";
        if (std.ascii.eqlIgnoreCase(e, "js") or std.ascii.eqlIgnoreCase(e, "mjs")) return "application/javascript; charset=utf-8";
        if (std.ascii.eqlIgnoreCase(e, "css")) return "text/css; charset=utf-8";
        if (std.ascii.eqlIgnoreCase(e, "txt") or std.ascii.eqlIgnoreCase(e, "text")) return "text/plain; charset=utf-8";
        if (std.ascii.eqlIgnoreCase(e, "png")) return "image/png";
        if (std.ascii.eqlIgnoreCase(e, "jpg") or std.ascii.eqlIgnoreCase(e, "jpeg")) return "image/jpeg";
        if (std.ascii.eqlIgnoreCase(e, "gif")) return "image/gif";
        if (std.ascii.eqlIgnoreCase(e, "svg")) return "image/svg+xml";
        if (std.ascii.eqlIgnoreCase(e, "webp")) return "image/webp";
        if (std.ascii.eqlIgnoreCase(e, "ico")) return "image/x-icon";
        if (std.ascii.eqlIgnoreCase(e, "wasm")) return "application/wasm";
        if (std.ascii.eqlIgnoreCase(e, "pdf")) return "application/pdf";
        if (std.ascii.eqlIgnoreCase(e, "xml")) return "application/xml";
        if (std.ascii.eqlIgnoreCase(e, "zip")) return "application/zip";
        if (std.ascii.eqlIgnoreCase(e, "tar")) return "application/x-tar";
        if (std.ascii.eqlIgnoreCase(e, "gz")) return "application/gzip";
        if (std.ascii.eqlIgnoreCase(e, "mp3")) return "audio/mpeg";
        if (std.ascii.eqlIgnoreCase(e, "mp4")) return "video/mp4";
        if (std.ascii.eqlIgnoreCase(e, "woff")) return "font/woff";
        if (std.ascii.eqlIgnoreCase(e, "woff2")) return "font/woff2";

        return "application/octet-stream";
    }

    pub fn fromPath(path: []const u8): []const u8 {
        var dot_idx: ?usize = null;
        var i: usize = path.len;
        while (i > 0) {
            i -= 1;
            if (path[i] == '.') {
                dot_idx = i;
                break;
            }
            if (path[i] == '/' or path[i] == '\\') {
                break;
            }
        }
        if (dot_idx) |idx| {
            return fromExtension(path[idx + 1 ..]);
        }
        return "application/octet-stream";
    }
};
