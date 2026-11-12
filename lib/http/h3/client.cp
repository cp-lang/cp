/**
 * CP HTTP/3 Client Session & Stream Manager (cap:http)
 * Ported from Bun.js http/H3Client.cp & http/h3_client/
 */

export const H3StreamType = enum {
    Control,
    QpackEncoder,
    QpackDecoder,
    RequestBidi,
};

export const H3Stream = struct {
    id: u64 = 0,
    stream_type: H3StreamType = H3StreamType.RequestBidi,
    closed: bool = false,

    pub fn close(self: *H3Stream) void {
        self.closed = true;
    }

    pub fn isOpen(self: H3Stream) bool {
        return !self.closed;
    }
};

export const H3ClientSession = struct {
    next_bidi_stream_id: u64 = 0, // Client-initiated bidi stream IDs (0, 4, 8, 12...)
    connected: bool = false,
    active_streams: usize = 0,

    pub fn newRequestStream(self: *H3ClientSession) H3Stream {
        const sid = self.next_bidi_stream_id;
        self.next_bidi_stream_id += 4;
        self.active_streams += 1;
        var s = H3Stream{};
        s.id = sid;
        s.stream_type = .RequestBidi;
        s.closed = false;
        return s;
    }
};
