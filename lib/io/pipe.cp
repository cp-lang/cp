/**
 * CP Native OS Pipe (cap:io)
 * 原生管道读取器与写入器，满足 Reader / Writer / Closer Trait 契约
 */

const std = @import("std");
import { Reader, Writer, Closer } from "./traits";

/**
 * 原生非阻塞/阻塞管道读取器
 */
export class PipeReader implements Reader, Closer {
    fd: i32;
    closed: bool;

    pub fn init(fd: i32) {
        _.fd = fd;
        _.closed = false;
    }

    pub fn read(mut _, *buf: []u8): usize! {
        if (_.closed || _.fd < 0) {
            return err.Closed;
        }
        if (buf.len == 0) {
            return 0;
        }
        const rc = std.c.read(_.fd, buf.ptr, buf.len);
        if (rc < 0) {
            return err.EndOfStream;
        }
        return rc as usize;
    }

    pub fn close(mut _): void {
        if (!_.closed && _.fd >= 0) {
            _ = std.c.close(_.fd);
            _.closed = true;
            _.fd = -1;
        }
    }

    pub fn getFd(_): i32 {
        return _.fd;
    }

    pub fn isClosed(_): bool {
        return _.closed;
    }
}

/**
 * 原生非阻塞/阻塞管道写入器
 */
export class PipeWriter implements Writer, Closer {
    fd: i32;
    closed: bool;

    pub fn init(fd: i32) {
        _.fd = fd;
        _.closed = false;
    }

    pub fn write(mut _, data: []const u8): usize! {
        if (_.closed || _.fd < 0) {
            return err.Closed;
        }
        if (data.len == 0) {
            return 0;
        }
        const rc = std.c.write(_.fd, data.ptr, data.len);
        if (rc < 0) {
            return err.ShortWrite;
        }
        return rc as usize;
    }

    pub fn close(mut _): void {
        if (!_.closed && _.fd >= 0) {
            _ = std.c.close(_.fd);
            _.closed = true;
            _.fd = -1;
        }
    }

    pub fn getFd(_): i32 {
        return _.fd;
    }

    pub fn isClosed(_): bool {
        return _.closed;
    }
}

/**
 * 管道工厂：创建一对相互连通的 PipeReader 与 PipeWriter
 */
export class Pipe {
    reader: PipeReader;
    writer: PipeWriter;

    pub fn init(reader: PipeReader, writer: PipeWriter) {
        _.reader = reader;
        _.writer = writer;
    }

    pub static fn create(): Pipe! {
        var fds: [2]i32 = undefined;
        const rc = std.c.pipe(&fds);
        if (rc != 0) {
            return err.Closed;
        }
        let r = new PipeReader(fds[0]);
        let w = new PipeWriter(fds[1]);
        return new Pipe(r, w);
    }
}
