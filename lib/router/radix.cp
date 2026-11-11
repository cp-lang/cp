// lib/router/radix.cp
// High-performance Radix-Tree / Trie router for CP Web applications
const std = @import("std");
import { ParamMap } from "./param_map";
import { RouteMatch } from "./route";

export fn RadixNode(comptime T: type): type {
    return struct {
        const Self = @This();

        segment: []const u8 = "",
        is_param: bool = false,
        is_wildcard: bool = false,
        param_name: []const u8 = "",
        wildcard_name: []const u8 = "",
        is_terminal: bool = false,
        handler: ?T = null,

        children: [16]?*Self = undefined,
        children_len: usize = 0,
        param_child: ?*Self = null,
        wildcard_child: ?*Self = null,

        pub fn create(allocator: std.mem.Allocator, segment: []const u8) *Self {
            const node = allocator.create(Self) catch unreachable;
            node.segment = segment;
            node.is_param = false;
            node.is_wildcard = false;
            node.param_name = "";
            node.wildcard_name = "";
            node.is_terminal = false;
            node.handler = null;
            node.children_len = 0;
            node.param_child = null;
            node.wildcard_child = null;
            var i: usize = 0;
            while (i < 16) : (i += 1) {
                node.children[i] = null;
            }
            return node;
        }

        pub fn findStaticChild(self: *const Self, seg: []const u8): ?*Self {
            var i: usize = 0;
            while (i < self.children_len) : (i += 1) {
                if (self.children[i]) |c| {
                    if (std.mem.eql(u8, c.segment, seg)) {
                        return c;
                    }
                }
            }
            return null;
        }

        pub fn addStaticChild(self: *Self, child: *Self): void {
            if (self.children_len < 16) {
                self.children[self.children_len] = child;
                self.children_len += 1;
            }
        }
    };
}

export fn Router(comptime T: type): type {
    return struct {
        const Self = @This();
        const Node = RadixNode(T);
        const Match = RouteMatch(T);

        trees: [8]?*Node = undefined,
        allocator: std.mem.Allocator,

        pub fn init(allocator: std.mem.Allocator) Self {
            var r = Self{
                .trees = undefined,
                .allocator = allocator,
            };
            var i: usize = 0;
            while (i < 8) : (i += 1) {
                r.trees[i] = Node.create(allocator, "");
            }
            return r;
        }

        pub fn methodIndex(method: []const u8) usize {
            if (std.mem.eql(u8, method, "GET")) return 0;
            if (std.mem.eql(u8, method, "POST")) return 1;
            if (std.mem.eql(u8, method, "PUT")) return 2;
            if (std.mem.eql(u8, method, "DELETE")) return 3;
            if (std.mem.eql(u8, method, "PATCH")) return 4;
            if (std.mem.eql(u8, method, "HEAD")) return 5;
            if (std.mem.eql(u8, method, "OPTIONS")) return 6;
            return 7; // ALL
        }

        pub fn insert(self: *Self, method: []const u8, path: []const u8, handler: T) void {
            const m_idx = methodIndex(method);
            if (self.trees[m_idx] == null) {
                self.trees[m_idx] = Node.create(self.allocator, "");
            }
            var curr = self.trees[m_idx].?;

            var p = path;
            if (p.len > 0 and p[0] == '/') p = p[1..];
            if (p.len > 0 and p[p.len - 1] == '/') p = p[0 .. p.len - 1];

            if (p.len == 0) {
                curr.is_terminal = true;
                curr.handler = handler;
                return;
            }

            var start: usize = 0;
            var i: usize = 0;
            while (i <= p.len) : (i += 1) {
                if (i == p.len or p[i] == '/') {
                    if (i > start) {
                        const seg = p[start..i];
                        curr = self.insertSegment(curr, seg);
                    }
                    start = i + 1;
                }
            }

            curr.is_terminal = true;
            curr.handler = handler;
        }

        fn insertSegment(self: *Self, parent: *Node, seg: []const u8) *Node {
            // Dynamic param :id or [id]
            if (seg.len > 1 and (seg[0] == ':' or (seg[0] == '[' and seg[seg.len - 1] == ']' and !std.mem.startsWith(u8, seg, "[...")))) {
                const name = if (seg[0] == ':') seg[1..] else seg[1 .. seg.len - 1];
                if (parent.param_child) |existing| {
                    return existing;
                }
                const child = Node.create(self.allocator, seg);
                child.is_param = true;
                child.param_name = name;
                parent.param_child = child;
                return child;
            }

            // Wildcard catch-all *path or [...path]
            if (seg.len > 0 and (seg[0] == '*' or (std.mem.startsWith(u8, seg, "[...") and seg[seg.len - 1] == ']'))) {
                const name = if (seg[0] == '*') (if (seg.len > 1) seg[1..] else "*") else seg[4 .. seg.len - 1];
                if (parent.wildcard_child) |existing| {
                    return existing;
                }
                const child = Node.create(self.allocator, seg);
                child.is_wildcard = true;
                child.wildcard_name = name;
                parent.wildcard_child = child;
                return child;
            }

            // Static segment
            if (parent.findStaticChild(seg)) |existing| {
                return existing;
            }
            const child = Node.create(self.allocator, seg);
            parent.addStaticChild(child);
            return child;
        }

        pub fn lookup(self: *const Self, method: []const u8, path: []const u8) ?Match {
            const m_idx = methodIndex(method);
            if (self.trees[m_idx]) |root| {
                if (self.lookupInTree(root, path)) |res| {
                    return res;
                }
            }
            if (m_idx != 7) {
                if (self.trees[7]) |all_root| {
                    return self.lookupInTree(all_root, path);
                }
            }
            return null;
        }

        fn lookupInTree(self: *const Self, root: *Node, path: []const u8) ?Match {
            _ = self;
            var params = ParamMap.init();
            var p = path;
            if (p.len > 0 and p[0] == '/') p = p[1..];
            if (p.len > 0 and p[p.len - 1] == '/') p = p[0 .. p.len - 1];

            if (p.len == 0) {
                if (root.is_terminal and root.handler != null) {
                    return Match{ .handler = root.handler.?, .params = params };
                }
                return null;
            }

            var curr = root;
            var start: usize = 0;
            var i: usize = 0;

            while (i <= p.len) : (i += 1) {
                if (i == p.len or p[i] == '/') {
                    if (i > start) {
                        const seg = p[start..i];

                        if (curr.findStaticChild(seg)) |sc| {
                            curr = sc;
                        } else if (curr.param_child) |pc| {
                            params.append(pc.param_name, seg);
                            curr = pc;
                        } else if (curr.wildcard_child) |wc| {
                            const remaining = p[start..];
                            params.append(wc.wildcard_name, remaining);
                            if (wc.is_terminal and wc.handler != null) {
                                return Match{ .handler = wc.handler.?, .params = params };
                            }
                            return null;
                        } else {
                            return null;
                        }
                    }
                    start = i + 1;
                }
            }

            if (curr.is_terminal and curr.handler != null) {
                return Match{ .handler = curr.handler.?, .params = params };
            }
            return null;
        }

        pub fn get(self: *Self, path: []const u8, handler: T) void {
            self.insert("GET", path, handler);
        }

        pub fn post(self: *Self, path: []const u8, handler: T) void {
            self.insert("POST", path, handler);
        }

        pub fn put(self: *Self, path: []const u8, handler: T) void {
            self.insert("PUT", path, handler);
        }

        pub fn delete(self: *Self, path: []const u8, handler: T) void {
            self.insert("DELETE", path, handler);
        }

        pub fn patch(self: *Self, path: []const u8, handler: T) void {
            self.insert("PATCH", path, handler);
        }

        pub fn all(self: *Self, path: []const u8, handler: T) void {
            self.insert("ALL", path, handler);
        }
    };
}
