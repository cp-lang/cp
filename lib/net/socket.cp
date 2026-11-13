// lib/net/socket.cp
// SafeSocket with RAII, socket options, long-timeout splitting, and legacy Socket compatibility
// Absorbed from Bun src/uws_sys/socket.cp and src/uws_sys/us_socket_t.cp

const std = @import("std");
import { SocketKind, SocketCloseCode, SendStatus } from "./_usockets";

export enum SocketState: u8 {
    Connecting = 0,
    Connected = 1,
    Closing = 2,
    Closed = 3,
    Detached = 4,
}

export struct SocketStats {
    pub bytes_read: u64 = 0;
    pub bytes_written: u64 = 0;
    pub write_calls: u64 = 0;
    pub read_calls: u64 = 0;
}

export class SafeSocket {
    pub fd: i32 = -1;
    pub kind: SocketKind = SocketKind.closed;
    pub state: SocketState = SocketState.Closed;
    pub is_tls: bool = false;
    pub nodelay: bool = false;
    pub keepalive_enabled: bool = false;
    pub keepalive_delay: u32 = 0;
    pub timeout_sec: u32 = 0;
    pub long_timeout_min: u32 = 0;
    pub paused: bool = false;
    pub read_shutdown: bool = false;
    pub write_shutdown: bool = false;
    pub close_code: SocketCloseCode = SocketCloseCode.normal;
    pub stats: SocketStats = SocketStats {};

    pub static fn create(fd: i32, kind: SocketKind): *SafeSocket {
        let ptr = std.heap.page_allocator.create(SafeSocket) catch unreachable;
        ptr.fd = fd;
        ptr.kind = kind;
        if (kind == SocketKind.established_tcp || kind == SocketKind.established_tls || kind == SocketKind.udp) {
            ptr.state = SocketState.Connected;
        } else if (kind == SocketKind.connecting_tcp || kind == SocketKind.connecting_tls) {
            ptr.state = SocketState.Connecting;
        } else {
            ptr.state = SocketState.Closed;
        }
        ptr.is_tls = (kind == SocketKind.established_tls || kind == SocketKind.connecting_tls || kind == SocketKind.listen_tls);
        ptr.nodelay = false;
        ptr.keepalive_enabled = false;
        ptr.keepalive_delay = 0;
        ptr.timeout_sec = 0;
        ptr.long_timeout_min = 0;
        ptr.paused = false;
        ptr.read_shutdown = false;
        ptr.write_shutdown = false;
        ptr.close_code = SocketCloseCode.normal;
        ptr.stats = SocketStats {};
        return ptr;
    }

    pub fn setNoDelay(self: *SafeSocket, enabled: bool): void {
        if (self.isClosed()) {
            return;
        }
        self.nodelay = enabled;
    }

    pub fn setKeepAlive(self: *SafeSocket, enabled: bool, delay: u32): bool {
        if (self.isClosed()) {
            return false;
        }
        self.keepalive_enabled = enabled;
        if (enabled) {
            self.keepalive_delay = delay;
        } else {
            self.keepalive_delay = 0;
        }
        return true;
    }

    // Absorbed from Bun SocketHandler.setTimeout:
    // uSockets short timeout supports <= 240s (4-second granularity);
    // timeouts > 240s automatically promote to minute-level long_timeout!
    pub fn setTimeout(self: *SafeSocket, seconds: u32): void {
        if (self.isClosed()) {
            return;
        }
        if (seconds > 240) {
            self.timeout_sec = 0;
            self.long_timeout_min = seconds / 60;
        } else {
            self.timeout_sec = seconds;
            self.long_timeout_min = 0;
        }
    }

    pub fn setLongTimeout(self: *SafeSocket, minutes: u32): void {
        if (self.isClosed()) {
            return;
        }
        self.long_timeout_min = minutes;
    }

    pub fn pause(self: *SafeSocket): void {
        if (self.isClosed()) {
            return;
        }
        self.paused = true;
    }

    pub fn resumeSocket(self: *SafeSocket): void {
        if (self.isClosed()) {
            return;
        }
        self.paused = false;
    }

    pub fn detach(self: *SafeSocket): void {
        self.state = SocketState.Detached;
        self.kind = SocketKind.closed;
        self.fd = -1;
    }

    pub fn isDetached(self: *SafeSocket): bool {
        return self.state == SocketState.Detached || self.fd == -1;
    }

    pub fn isClosed(self: *SafeSocket): bool {
        return self.state == SocketState.Closed || self.state == SocketState.Detached || self.kind == SocketKind.closed;
    }

    pub fn isEstablished(self: *SafeSocket): bool {
        return self.state == SocketState.Connected &&
            (self.kind == SocketKind.established_tcp || self.kind == SocketKind.established_tls);
    }

    pub fn recordWrite(self: *SafeSocket, n: usize): SendStatus {
        if (self.isClosed() || self.write_shutdown) {
            return SendStatus.dropped;
        }
        self.stats.bytes_written += n as u64;
        self.stats.write_calls += 1;
        return SendStatus.success;
    }

    pub fn recordRead(self: *SafeSocket, n: usize): void {
        if (self.isClosed() || self.read_shutdown) {
            return;
        }
        self.stats.bytes_read += n as u64;
        self.stats.read_calls += 1;
    }

    pub fn shutdown(self: *SafeSocket): void {
        if (self.isClosed()) {
            return;
        }
        self.write_shutdown = true;
    }

    pub fn shutdownRead(self: *SafeSocket): void {
        if (self.isClosed()) {
            return;
        }
        self.read_shutdown = true;
    }

    pub fn close(self: *SafeSocket, code: SocketCloseCode): void {
        if (self.state == SocketState.Closed || self.state == SocketState.Detached) {
            return;
        }
        self.close_code = code;
        self.state = SocketState.Closed;
        self.kind = SocketKind.closed;
        self.fd = -1;
    }

    pub fn release(self: *SafeSocket): void {
        if (!self.isClosed()) {
            self.close(SocketCloseCode.normal);
        }
        std.heap.page_allocator.destroy(self);
    }
}

// ============================================================================
// Backward-Compatible Actor / Async Socket (from legacy lib/net.cp)
// ============================================================================
export class Socket {
    pub fd: i32;

    pub fn init(fd: i32): void {
        this.fd = fd;
    }

    pub async fn read(): string {
        return await @net_read_async(this.fd);
    }

    pub async fn write(data: string): i32 {
        return await @net_write_async(this.fd, data);
    }

    pub fn close(): void {
        @net_close(this.fd);
    }
}
