// lib/net/tcp_stream.cp
// High-throughput TCP Stream with Corking & StreamBuffer
// Absorbed from Bun us_socket_stream_buffer_t and SocketHandler

const std = @import("std");
import { SocketKind, SocketCloseCode, SendStatus } from "./_usockets";
import { SocketAddress } from "./address";
import { SafeSocket } from "./socket";

export const STREAM_CORK_BUFFER_SIZE: usize = 16384;

export class StreamBuffer {
    pub buf: [16384]u8 = undefined;
    pub head: usize = 0;
    pub tail: usize = 0;

    pub static fn create(): *StreamBuffer {
        let ptr = std.heap.page_allocator.create(StreamBuffer) catch unreachable;
        ptr.head = 0;
        ptr.tail = 0;
        return ptr;
    }

    pub fn pendingLen(self: *StreamBuffer): usize {
        return self.tail - self.head;
    }

    pub fn availableSpace(self: *StreamBuffer): usize {
        return STREAM_CORK_BUFFER_SIZE - self.tail;
    }

    pub fn write(self: *StreamBuffer, data: []const u8): usize {
        if (self.head == self.tail) {
            self.head = 0;
            self.tail = 0;
        }
        const avail = self.availableSpace();
        const to_copy = if (data.len < avail) data.len else avail;
        var i: usize = 0;
        while (i < to_copy) {
            self.buf[self.tail + i] = data[i];
            i += 1;
        }
        self.tail += to_copy;
        return to_copy;
    }

    pub fn flushAll(self: *StreamBuffer): usize {
        const flushed = self.pendingLen();
        self.head = 0;
        self.tail = 0;
        return flushed;
    }

    pub fn reset(self: *StreamBuffer): void {
        self.head = 0;
        self.tail = 0;
    }

    pub fn destroy(self: *StreamBuffer): void {
        std.heap.page_allocator.destroy(self);
    }
}

export class TcpStream {
    pub sock: *SafeSocket;
    pub local_addr: SocketAddress;
    pub remote_addr: SocketAddress;
    pub corked: bool = false;
    pub send_buf: *StreamBuffer;
    pub recv_buf: [16384]u8 = undefined;
    pub recv_len: usize = 0;

    pub static fn init(fd: i32, local_addr: SocketAddress, remote_addr: SocketAddress): *TcpStream {
        let ptr = std.heap.page_allocator.create(TcpStream) catch unreachable;
        ptr.sock = SafeSocket.create(fd, SocketKind.established_tcp);
        ptr.local_addr = local_addr;
        ptr.remote_addr = remote_addr;
        ptr.corked = false;
        ptr.send_buf = StreamBuffer.create();
        ptr.recv_len = 0;
        return ptr;
    }

    pub fn cork(self: *TcpStream): void {
        if (self.sock.isClosed()) {
            return;
        }
        self.corked = true;
    }

    pub fn uncork(self: *TcpStream): usize {
        if (!self.corked) {
            return 0;
        }
        self.corked = false;
        return self.flush();
    }

    pub fn write(self: *TcpStream, data: []const u8): usize {
        if (self.sock.isClosed() || self.sock.write_shutdown) {
            return 0;
        }
        if (self.corked) {
            const buffered = self.send_buf.write(data);
            if (buffered == data.len) {
                return buffered;
            }
            const flushed = self.flush();
            let * = flushed;
            const rem = data.len - buffered;
            let * = self.sock.recordWrite(rem);
            return data.len;
        }
        const status = self.sock.recordWrite(data.len);
        if (status == SendStatus.success) {
            return data.len;
        }
        return 0;
    }

    pub fn flush(self: *TcpStream): usize {
        const pending = self.send_buf.flushAll();
        if (pending > 0) {
            let * = self.sock.recordWrite(pending);
        }
        return pending;
    }

    pub fn feedIncoming(self: *TcpStream, data: []const u8): usize {
        if (self.sock.isClosed() || self.sock.read_shutdown) {
            return 0;
        }
        const space = STREAM_CORK_BUFFER_SIZE - self.recv_len;
        const count = if (data.len < space) data.len else space;
        var i: usize = 0;
        while (i < count) {
            self.recv_buf[self.recv_len + i] = data[i];
            i += 1;
        }
        self.recv_len += count;
        self.sock.recordRead(count);
        return count;
    }

    pub fn readAvailable(self: *TcpStream, out: []u8): usize {
        const count = if (out.len < self.recv_len) out.len else self.recv_len;
        var i: usize = 0;
        while (i < count) {
            out[i] = self.recv_buf[i];
            i += 1;
        }
        const rem = self.recv_len - count;
        var j: usize = 0;
        while (j < rem) {
            self.recv_buf[j] = self.recv_buf[count + j];
            j += 1;
        }
        self.recv_len = rem;
        return count;
    }

    pub fn setNoDelay(self: *TcpStream, enabled: bool): void {
        self.sock.setNoDelay(enabled);
    }

    pub fn setKeepAlive(self: *TcpStream, enabled: bool, delay: u32): bool {
        return self.sock.setKeepAlive(enabled, delay);
    }

    pub fn setTimeout(self: *TcpStream, seconds: u32): void {
        self.sock.setTimeout(seconds);
    }

    pub fn close(self: *TcpStream): void {
        let * = self.flush();
        self.sock.close(SocketCloseCode.normal);
    }

    pub fn release(self: *TcpStream): void {
        self.close();
        self.send_buf.destroy();
        self.sock.release();
        std.heap.page_allocator.destroy(self);
    }
}
