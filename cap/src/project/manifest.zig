const std = @import("std");

pub const OptimizeMode = enum {
    Debug,
    ReleaseSafe,
    ReleaseFast,
    ReleaseSmall,

    pub fn fromString(str: []const u8) ?OptimizeMode {
        if (std.ascii.eqlIgnoreCase(str, "debug")) return .Debug;
        if (std.ascii.eqlIgnoreCase(str, "safe") or std.ascii.eqlIgnoreCase(str, "releasesafe")) return .ReleaseSafe;
        if (std.ascii.eqlIgnoreCase(str, "fast") or std.ascii.eqlIgnoreCase(str, "releasefast")) return .ReleaseFast;
        if (std.ascii.eqlIgnoreCase(str, "small") or std.ascii.eqlIgnoreCase(str, "releasesmall")) return .ReleaseSmall;
        return null;
    }

    pub fn toZigFlag(self: OptimizeMode) []const u8 {
        return switch (self) {
            .Debug => "Debug",
            .ReleaseSafe => "ReleaseSafe",
            .ReleaseFast => "ReleaseFast",
            .ReleaseSmall => "ReleaseSmall",
        };
    }

    pub fn toConfigString(self: OptimizeMode) []const u8 {
        return switch (self) {
            .Debug => "debug",
            .ReleaseSafe => "safe",
            .ReleaseFast => "fast",
            .ReleaseSmall => "small",
        };
    }
};

pub const BuildConfig = struct {
    opt: ?OptimizeMode = null,
    out_dir: ?[]const u8 = null,
    target: ?[]const u8 = null,
    targets: [][]const u8 = &.{},
    c_flags: [][]const u8 = &.{},
    link_libs: [][]const u8 = &.{},

    pub fn deinit(self: *BuildConfig, allocator: std.mem.Allocator) void {
        if (self.out_dir) |od| allocator.free(od);
        if (self.target) |t| allocator.free(t);
        for (self.targets) |t| allocator.free(t);
        if (self.targets.len > 0) allocator.free(self.targets);
        for (self.c_flags) |cf| allocator.free(cf);
        if (self.c_flags.len > 0) allocator.free(self.c_flags);
        for (self.link_libs) |ll| allocator.free(ll);
        if (self.link_libs.len > 0) allocator.free(self.link_libs);
    }
};

pub const NestConfig = struct {
    mailbox_capacity: ?u64 = null,
    reduction_slice: ?u64 = null,
};

pub const Manifest = struct {
    name: []const u8,
    version: []const u8,
    description: ?[]const u8 = null,
    main: ?[]const u8 = null,
    mode: ?[]const u8 = null,
    scripts: std.StringHashMap([]const u8),
    dependencies: std.StringHashMap([]const u8),
    dev_dependencies: std.StringHashMap([]const u8),
    build_config: ?BuildConfig = null,
    nest_config: ?NestConfig = null,

    pub fn parse(allocator: std.mem.Allocator, contents: []const u8) !Manifest {
        var parsed = try std.json.parseFromSlice(std.json.Value, allocator, contents, .{});
        defer parsed.deinit();

        const root = parsed.value.object;

        const name = try allocator.dupe(u8, if (root.get("name")) |n| (if (n == .string) n.string else "unknown") else "unknown");
        const version = try allocator.dupe(u8, if (root.get("version")) |v| (if (v == .string) v.string else "0.1.0") else "0.1.0");

        var description: ?[]const u8 = null;
        if (root.get("description")) |d| {
            if (d == .string) {
                description = try allocator.dupe(u8, d.string);
            }
        }

        var main: ?[]const u8 = null;
        if (root.get("main")) |m| {
            if (m == .string) {
                main = try allocator.dupe(u8, m.string);
            }
        }

        var mode: ?[]const u8 = null;
        if (root.get("mode")) |m| {
            if (m == .string) {
                mode = try allocator.dupe(u8, m.string);
            }
        }

        var scripts = std.StringHashMap([]const u8).init(allocator);
        if (root.get("scripts")) |s| {
            if (s == .object) {
                var iter = s.object.iterator();
                while (iter.next()) |entry| {
                    if (entry.value_ptr.* == .string) {
                        try scripts.put(try allocator.dupe(u8, entry.key_ptr.*), try allocator.dupe(u8, entry.value_ptr.string));
                    }
                }
            }
        }

        var deps = std.StringHashMap([]const u8).init(allocator);
        if (root.get("dependencies")) |d| {
            if (d == .object) {
                var iter = d.object.iterator();
                while (iter.next()) |entry| {
                    if (entry.value_ptr.* == .string) {
                        try deps.put(try allocator.dupe(u8, entry.key_ptr.*), try allocator.dupe(u8, entry.value_ptr.string));
                    }
                }
            }
        }

        var dev_deps = std.StringHashMap([]const u8).init(allocator);
        if (root.get("dev_dependencies")) |dd| {
            if (dd == .object) {
                var iter = dd.object.iterator();
                while (iter.next()) |entry| {
                    if (entry.value_ptr.* == .string) {
                        try dev_deps.put(try allocator.dupe(u8, entry.key_ptr.*), try allocator.dupe(u8, entry.value_ptr.string));
                    }
                }
            }
        }

        var build_cfg: ?BuildConfig = null;
        if (root.get("build")) |b| {
            if (b == .object) {
                var bc = BuildConfig{};
                if (b.object.get("opt")) |o| {
                    if (o == .string) {
                        bc.opt = OptimizeMode.fromString(o.string);
                    }
                }
                if (b.object.get("out_dir")) |od| {
                    if (od == .string) {
                        bc.out_dir = try allocator.dupe(u8, od.string);
                    }
                }
                if (b.object.get("target")) |t| {
                    if (t == .string) {
                        bc.target = try allocator.dupe(u8, t.string);
                    }
                }
                if (b.object.get("targets")) |ts| {
                    if (ts == .array) {
                        var list = try allocator.alloc([]const u8, ts.array.items.len);
                        for (ts.array.items, 0..) |item, i| {
                            list[i] = try allocator.dupe(u8, if (item == .string) item.string else "");
                        }
                        bc.targets = list;
                    }
                }
                if (b.object.get("c_flags")) |cfs| {
                    if (cfs == .array) {
                        var list = try allocator.alloc([]const u8, cfs.array.items.len);
                        for (cfs.array.items, 0..) |item, i| {
                            list[i] = try allocator.dupe(u8, if (item == .string) item.string else "");
                        }
                        bc.c_flags = list;
                    }
                }
                if (b.object.get("link_libs")) |lls| {
                    if (lls == .array) {
                        var list = try allocator.alloc([]const u8, lls.array.items.len);
                        for (lls.array.items, 0..) |item, i| {
                            list[i] = try allocator.dupe(u8, if (item == .string) item.string else "");
                        }
                        bc.link_libs = list;
                    }
                }
                build_cfg = bc;
            }
        }

        var nest_cfg: ?NestConfig = null;
        if (root.get("nest")) |n| {
            if (n == .object) {
                var nc = NestConfig{};
                if (n.object.get("mailbox_capacity")) |mc| {
                    if (mc == .integer) {
                        nc.mailbox_capacity = @intCast(mc.integer);
                    }
                }
                if (n.object.get("reduction_slice")) |rs| {
                    if (rs == .integer) {
                        nc.reduction_slice = @intCast(rs.integer);
                    }
                }
                nest_cfg = nc;
            }
        }

        return Manifest{
            .name = name,
            .version = version,
            .description = description,
            .main = main,
            .mode = mode,
            .scripts = scripts,
            .dependencies = deps,
            .dev_dependencies = dev_deps,
            .build_config = build_cfg,
            .nest_config = nest_cfg,
        };
    }

    pub fn save(self: *Manifest, allocator: std.mem.Allocator, path: []const u8) !void {
        var out_file = try @import("../main.zig").fs_shim.cwd().createFile(path, .{});
        defer out_file.close(@import("../main.zig").global_io);

        var arena = std.heap.ArenaAllocator.init(allocator);
        defer arena.deinit();
        const aa = arena.allocator();

        var root = std.json.Value{ .object = std.json.ObjectMap.init(aa, &.{}, &.{}) catch unreachable };
        try root.object.put(aa, "name", std.json.Value{ .string = self.name });
        try root.object.put(aa, "version", std.json.Value{ .string = self.version });
        if (self.description) |d| {
            try root.object.put(aa, "description", std.json.Value{ .string = d });
        }
        if (self.main) |m| {
            try root.object.put(aa, "main", std.json.Value{ .string = m });
        }
        if (self.mode) |m| {
            try root.object.put(aa, "mode", std.json.Value{ .string = m });
        }

        var scripts_map = std.json.ObjectMap.init(aa, &.{}, &.{}) catch unreachable;
        var script_iter = self.scripts.iterator();
        while (script_iter.next()) |entry| {
            try scripts_map.put(aa, entry.key_ptr.*, std.json.Value{ .string = entry.value_ptr.* });
        }
        try root.object.put(aa, "scripts", std.json.Value{ .object = scripts_map });

        var deps_map = std.json.ObjectMap.init(aa, &.{}, &.{}) catch unreachable;
        var dep_iter = self.dependencies.iterator();
        while (dep_iter.next()) |entry| {
            try deps_map.put(aa, entry.key_ptr.*, std.json.Value{ .string = entry.value_ptr.* });
        }
        try root.object.put(aa, "dependencies", std.json.Value{ .object = deps_map });

        if (self.dev_dependencies.count() > 0) {
            var dev_deps_map = std.json.ObjectMap.init(aa, &.{}, &.{}) catch unreachable;
            var dev_dep_iter = self.dev_dependencies.iterator();
            while (dev_dep_iter.next()) |entry| {
                try dev_deps_map.put(aa, entry.key_ptr.*, std.json.Value{ .string = entry.value_ptr.* });
            }
            try root.object.put(aa, "dev_dependencies", std.json.Value{ .object = dev_deps_map });
        }

        if (self.build_config) |bc| {
            var b_map = std.json.ObjectMap.init(aa, &.{}, &.{}) catch unreachable;
            if (bc.opt) |opt| {
                try b_map.put(aa, "opt", std.json.Value{ .string = opt.toConfigString() });
            }
            if (bc.out_dir) |od| {
                try b_map.put(aa, "out_dir", std.json.Value{ .string = od });
            }
            if (bc.target) |t| {
                try b_map.put(aa, "target", std.json.Value{ .string = t });
            }
            if (bc.targets.len > 0) {
                var arr = std.json.Array.init(aa);
                for (bc.targets) |t| {
                    try arr.append(std.json.Value{ .string = t });
                }
                try b_map.put(aa, "targets", std.json.Value{ .array = arr });
            }
            if (bc.c_flags.len > 0) {
                var arr = std.json.Array.init(aa);
                for (bc.c_flags) |cf| {
                    try arr.append(std.json.Value{ .string = cf });
                }
                try b_map.put(aa, "c_flags", std.json.Value{ .array = arr });
            }
            if (bc.link_libs.len > 0) {
                var arr = std.json.Array.init(aa);
                for (bc.link_libs) |ll| {
                    try arr.append(std.json.Value{ .string = ll });
                }
                try b_map.put(aa, "link_libs", std.json.Value{ .array = arr });
            }
            try root.object.put(aa, "build", std.json.Value{ .object = b_map });
        }

        if (self.nest_config) |nc| {
            var n_map = std.json.ObjectMap.init(aa, &.{}, &.{}) catch unreachable;
            if (nc.mailbox_capacity) |mc| {
                try n_map.put(aa, "mailbox_capacity", std.json.Value{ .integer = @intCast(mc) });
            }
            if (nc.reduction_slice) |rs| {
                try n_map.put(aa, "reduction_slice", std.json.Value{ .integer = @intCast(rs) });
            }
            try root.object.put(aa, "nest", std.json.Value{ .object = n_map });
        }

        var buf: [8192]u8 = undefined;
        var writer = out_file.writer(@import("../main.zig").global_io, &buf);
        try std.json.Stringify.value(root, .{ .whitespace = .indent_2 }, &writer.interface);
        try writer.end();
    }

    pub fn addDependency(self: *Manifest, allocator: std.mem.Allocator, name: []const u8, version_range: []const u8, is_dev: bool) !void {
        const name_dup = try allocator.dupe(u8, name);
        const ver_dup = try allocator.dupe(u8, version_range);
        if (is_dev) {
            if (self.dev_dependencies.fetchRemove(name)) |kv| {
                allocator.free(kv.key);
                allocator.free(kv.value);
            }
            try self.dev_dependencies.put(name_dup, ver_dup);
        } else {
            if (self.dependencies.fetchRemove(name)) |kv| {
                allocator.free(kv.key);
                allocator.free(kv.value);
            }
            try self.dependencies.put(name_dup, ver_dup);
        }
    }

    pub fn removeDependency(self: *Manifest, allocator: std.mem.Allocator, name: []const u8) bool {
        var found = false;
        if (self.dependencies.fetchRemove(name)) |kv| {
            allocator.free(kv.key);
            allocator.free(kv.value);
            found = true;
        }
        if (self.dev_dependencies.fetchRemove(name)) |kv| {
            allocator.free(kv.key);
            allocator.free(kv.value);
            found = true;
        }
        return found;
    }

    pub fn deinit(self: *Manifest, allocator: std.mem.Allocator) void {
        allocator.free(self.name);
        allocator.free(self.version);
        if (self.description) |d| allocator.free(d);
        if (self.main) |m| allocator.free(m);
        if (self.mode) |m| allocator.free(m);
        var script_iter = self.scripts.iterator();
        while (script_iter.next()) |entry| {
            allocator.free(entry.key_ptr.*);
            allocator.free(entry.value_ptr.*);
        }
        self.scripts.deinit();
        var dep_iter = self.dependencies.iterator();
        while (dep_iter.next()) |entry| {
            allocator.free(entry.key_ptr.*);
            allocator.free(entry.value_ptr.*);
        }
        self.dependencies.deinit();
        var dev_dep_iter = self.dev_dependencies.iterator();
        while (dev_dep_iter.next()) |entry| {
            allocator.free(entry.key_ptr.*);
            allocator.free(entry.value_ptr.*);
        }
        self.dev_dependencies.deinit();
        if (self.build_config) |*bc| {
            bc.deinit(allocator);
        }
    }
};
