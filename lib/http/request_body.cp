/**
 * CP HTTP Request Body (cap:http)
 * Ported from Bun.js http/HTTPRequestBody.cp
 */

export enum HTTPRequestBodyTag {
    Empty,
    Bytes,
    Stream,
}

export const HTTPRequestBody = struct {
    tag: HTTPRequestBodyTag = HTTPRequestBodyTag.Empty,
    bytes: []const u8 = "",

    pub fn fromBytes(data: []const u8) HTTPRequestBody {
        var body = HTTPRequestBody{};
        body.tag = HTTPRequestBodyTag.Bytes;
        body.bytes = data;
        return body;
    }

    pub fn empty() HTTPRequestBody {
        return HTTPRequestBody{};
    }

    pub fn len(self: HTTPRequestBody) usize {
        return switch (self.tag) {
            .Bytes => self.bytes.len,
            else => 0,
        };
    }

    pub fn isStream(self: HTTPRequestBody) bool {
        return self.tag == .Stream;
    }
};
