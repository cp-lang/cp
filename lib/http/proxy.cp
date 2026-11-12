/**
 * CP HTTP Proxy Tunnel (cap:http)
 * RFC 7231 HTTP CONNECT Tunneling
 * Ported from Bun.js http/ProxyTunnel.cp
 */

const std = @import("std");

export const ProxyTunnelSocketType = enum {
    Tcp,
    Ssl,
    None,
};

export const ProxyTunnel = struct {
    proxy_host: []const u8 = "",
    proxy_port: u16 = 8080,
    target_host: []const u8 = "",
    target_port: u16 = 443,
    socket_fd: i32 = -1,
    socket_type: ProxyTunnelSocketType = ProxyTunnelSocketType.None,
    is_established: bool = false,
    auth_header: []const u8 = "",

    pub fn init(proxy_host: []const u8, proxy_port: u16, target_host: []const u8, target_port: u16) ProxyTunnel {
        var tunnel = ProxyTunnel{};
        tunnel.proxy_host = proxy_host;
        tunnel.proxy_port = proxy_port;
        tunnel.target_host = target_host;
        tunnel.target_port = target_port;
        tunnel.socket_fd = -1;
        tunnel.socket_type = .None;
        tunnel.is_established = false;
        tunnel.auth_header = "";
        return tunnel;
    }

    pub fn formatConnectRequest(self: ProxyTunnel, buf: []u8) ![]const u8 {
        return try std.fmt.bufPrint(
            buf,
            "CONNECT {s}:{d} HTTP/1.1\r\nHost: {s}:{d}\r\nProxy-Connection: keep-alive\r\nUser-Agent: CP/1.0\r\n\r\n",
            .{ self.target_host, self.target_port, self.target_host, self.target_port },
        );
    }

    pub fn verifyConnectResponse(response: []const u8) bool {
        if (response.len < 12) return false;
        if (std.mem.startsWith(u8, response, "HTTP/1.1 200") or std.mem.startsWith(u8, response, "HTTP/1.0 200")) {
            return true;
        }
        return false;
    }

    pub fn close(self: *ProxyTunnel) void {
        if (self.socket_fd >= 0) {
            _ = std.os.linux.close(self.socket_fd);
            self.socket_fd = -1;
        }
        self.is_established = false;
    }
};
