// lib/url/url.cp
// WHATWG URL parser and representation in CP
const std = @import("std");
import { URLSearchParams } from "./query";

pub const URL = struct {
    href: []const u8 = "",
    protocol: []const u8 = "",
    username: []const u8 = "",
    password: []const u8 = "",
    host: []const u8 = "",
    hostname: []const u8 = "",
    port: []const u8 = "",
    pathname: []const u8 = "/",
    search: []const u8 = "",
    hash: []const u8 = "",
    origin: []const u8 = "",
    searchParams: URLSearchParams = undefined,
    allocated_origin: ?[]const u8 = null,
    allocator: std.mem.Allocator = undefined,

    pub fn parse(input: []const u8, allocator: std.mem.Allocator): !URL {
        var url = URL{
            .href = input,
            .searchParams = URLSearchParams.init(allocator),
            .allocator = allocator,
        };

        var rem = input;

        // 1. Protocol (e.g. "https://", "http://", "file://")
        const proto_end = std.mem.indexOf(u8, rem, "://");
        if (proto_end) |p_idx| {
            url.protocol = input[0 .. p_idx + 1]; // "https:"
            rem = rem[p_idx + 3 ..]; // skip "://"
        } else if (std.mem.indexOf(u8, rem, ":")) |c_idx| {
            url.protocol = input[0 .. c_idx + 1];
            rem = rem[c_idx + 1 ..];
        }

        // 2. Hash ("#...")
        if (std.mem.indexOfScalar(u8, rem, '#')) |h_idx| {
            url.hash = rem[h_idx..];
            rem = rem[0..h_idx];
        }

        // 3. Search query ("?...")
        if (std.mem.indexOfScalar(u8, rem, '?')) |q_idx| {
            url.search = rem[q_idx..];
            url.searchParams = URLSearchParams.parse(url.search, allocator);
            rem = rem[0..q_idx];
        }

        // 4. Pathname
        var path_start: ?usize = null;
        var i: usize = 0;
        while (i < rem.len) : (i += 1) {
            if (rem[i] == '/') {
                path_start = i;
                break;
            }
        }

        var authority = rem;
        if (path_start) |p_start| {
            url.pathname = rem[p_start..];
            authority = rem[0..p_start];
        } else {
            url.pathname = "/";
        }

        // 5. Userinfo (username:password@)
        if (std.mem.indexOfScalar(u8, authority, '@')) |at_idx| {
            const userinfo = authority[0..at_idx];
            authority = authority[at_idx + 1 ..];

            if (std.mem.indexOfScalar(u8, userinfo, ':')) |colon_idx| {
                url.username = userinfo[0..colon_idx];
                url.password = userinfo[colon_idx + 1 ..];
            } else {
                url.username = userinfo;
            }
        }

        // 6. Host & Port
        url.host = authority;
        if (std.mem.lastIndexOfScalar(u8, authority, ':')) |port_idx| {
            if (!std.mem.endsWith(u8, authority, "]")) {
                url.hostname = authority[0..port_idx];
                url.port = authority[port_idx + 1 ..];
            } else {
                url.hostname = authority;
            }
        } else {
            url.hostname = authority;
        }

        // 7. Origin
        if (url.protocol.len > 0 and url.host.len > 0) {
            if (url.username.len == 0 and url.password.len == 0) {
                const end = @intFromPtr(url.host.ptr) + url.host.len;
                const start = @intFromPtr(url.protocol.ptr);
                if (end >= start) {
                    const len = end - start;
                    const ptr: [*]const u8 = @ptrFromInt(start);
                    url.origin = ptr[0..len];
                }
            } else {
                url.origin = try std.fmt.allocPrint(allocator, "{s}//{s}", .{ url.protocol, url.host });
                url.allocated_origin = url.origin;
            }
        }

        return url;
    }

    pub fn deinit(self: *URL): void {
        self.searchParams.deinit();
        if (self.allocated_origin) |orig| {
            self.allocator.free(orig);
            self.allocated_origin = null;
        }
    }
};
