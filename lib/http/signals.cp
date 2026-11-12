/**
 * CP HTTP Atomic Signals & Flags (cap:http)
 * Ported from Bun.js http/Signals.cp
 */

const std = @import("std");

export const Signals = struct {
    header_progress: ?*std.atomic.Value(bool) = null,
    response_body_streaming: ?*std.atomic.Value(bool) = null,
    aborted: ?*std.atomic.Value(bool) = null,
    cert_errors: ?*std.atomic.Value(bool) = null,
    upgraded: ?*std.atomic.Value(bool) = null,

    pub fn isAborted(self: Signals) bool {
        if (self.aborted) |ptr| {
            return ptr.load(.monotonic);
        }
        return false;
    }

    pub fn isStreaming(self: Signals) bool {
        if (self.response_body_streaming) |ptr| {
            return ptr.load(.monotonic);
        }
        return false;
    }

    pub fn isUpgraded(self: Signals) bool {
        if (self.upgraded) |ptr| {
            return ptr.load(.monotonic);
        }
        return false;
    }

    pub fn abort(self: Signals) void {
        if (self.aborted) |ptr| {
            ptr.store(true, .monotonic);
        }
    }
};

export const SignalStore = struct {
    header_progress: std.atomic.Value(bool) = std.atomic.Value(bool).init(false),
    response_body_streaming: std.atomic.Value(bool) = std.atomic.Value(bool).init(false),
    aborted: std.atomic.Value(bool) = std.atomic.Value(bool).init(false),
    cert_errors: std.atomic.Value(bool) = std.atomic.Value(bool).init(false),
    upgraded: std.atomic.Value(bool) = std.atomic.Value(bool).init(false),

    pub fn toSignals(self: *SignalStore) Signals {
        var s = Signals{};
        s.header_progress = &self.header_progress;
        s.response_body_streaming = &self.response_body_streaming;
        s.aborted = &self.aborted;
        s.cert_errors = &self.cert_errors;
        s.upgraded = &self.upgraded;
        return s;
    }
};
