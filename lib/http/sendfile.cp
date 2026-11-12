/**
 * CP HTTP SendFile Kernel Zero-Copy File Upload (cap:http)
 * Ported from Bun.js http/SendFile.cp
 */

const std = @import("std");

export const SendFileStatus = enum {
    Done,
    Again,
    Error,
};

export const SendFile = struct {
    fd: i32 = -1,
    remain: usize = 0,
    offset: usize = 0,
    content_size: usize = 0,

    pub fn init(fd: i32, size: usize) SendFile {
        var sf = SendFile{};
        sf.fd = fd;
        sf.remain = size;
        sf.offset = 0;
        sf.content_size = size;
        return sf;
    }

    pub fn isDone(self: SendFile) bool {
        return self.remain == 0;
    }

    pub fn writeToSocket(self: *SendFile, socket_fd: i32) SendFileStatus {
        if (self.remain == 0) return .Done;

        var signed_offset: i64 = @as(i64, @intCast(self.offset));
        const val = std.os.linux.sendfile(socket_fd, self.fd, &signed_offset, self.remain);
        self.offset = @as(usize, @intCast(signed_offset));

        if (val > 0) {
            const sent: usize = @as(usize, @intCast(val));
            if (sent >= self.remain) {
                self.remain = 0;
                return .Done;
            } else {
                self.remain -= sent;
                return .Again;
            }
        } else if (val == 0) {
            return .Done;
        } else {
            return .Error;
        }
    }
};
