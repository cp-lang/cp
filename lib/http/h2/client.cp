/**
 * CP HTTP/2 Client Session & Stream Manager (cap:http)
 * Ported from Bun.js http/H2Client.cp & http/h2_client/
 */

const std = @import("std");
import { FrameType, FrameHeader, ErrorCode, SettingsType, DEFAULT_WINDOW_SIZE, DEFAULT_MAX_FRAME_SIZE } from "./frame";

export const H2StreamState = enum {
    Idle,
    ReservedLocal,
    ReservedRemote,
    Open,
    HalfClosedLocal,
    HalfClosedRemote,
    Closed,
};

export const H2Stream = struct {
    id: u32 = 0,
    state: H2StreamState = H2StreamState.Idle,
    window_size: i32 = DEFAULT_WINDOW_SIZE,

    pub fn isOpen(self: H2Stream) bool {
        return self.state == .Open or self.state == .HalfClosedRemote;
    }

    pub fn close(self: *H2Stream) void {
        self.state = .Closed;
    }
};

export const H2ClientSession = struct {
    next_stream_id: u32 = 1,
    peer_window_size: i32 = DEFAULT_WINDOW_SIZE,
    local_window_size: i32 = DEFAULT_WINDOW_SIZE,
    max_frame_size: u32 = DEFAULT_MAX_FRAME_SIZE,
    max_concurrent_streams: u32 = 100,
    active_streams: usize = 0,
    is_goaway: bool = false,

    pub fn newStream(self: *H2ClientSession) H2Stream {
        const stream_id = self.next_stream_id;
        self.next_stream_id += 2; // Client stream IDs are odd (1, 3, 5...)
        self.active_streams += 1;
        var s = H2Stream{};
        s.id = stream_id;
        s.state = .Open;
        s.window_size = self.peer_window_size;
        return s;
    }

    pub fn canCreateStream(self: H2ClientSession) bool {
        return !self.is_goaway and (self.active_streams < self.max_concurrent_streams);
    }

    pub fn onGoaway(self: *H2ClientSession) void {
        self.is_goaway = true;
    }
};
