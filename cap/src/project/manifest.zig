const std = @import("std");

pub const Manifest = struct {
    name: []const u8,
    version: []const u8,
    scripts: std.StringHashMap([]const u8),
    dependencies: std.StringHashMap([]const u8),

    pub fn parse(allocator: std.mem.Allocator, contents: []const u8) !Manifest {
        var parsed = try std.json.parseFromSlice(std.json.Value, allocator, contents, .{});
        defer parsed.deinit();

        const root = parsed.value.object;
        
        const name = try allocator.dupe(u8, root.get("name").?.string);
        const version = try allocator.dupe(u8, root.get("version").?.string);
        
        var scripts = std.StringHashMap([]const u8).init(allocator);
        if (root.get("scripts")) |s| {
            var iter = s.object.iterator();
            while (iter.next()) |entry| {
                try scripts.put(try allocator.dupe(u8, entry.key_ptr.*), try allocator.dupe(u8, entry.value_ptr.string));
            }
        }

        var deps = std.StringHashMap([]const u8).init(allocator);
        if (root.get("dependencies")) |d| {
            var iter = d.object.iterator();
            while (iter.next()) |entry| {
                try deps.put(try allocator.dupe(u8, entry.key_ptr.*), try allocator.dupe(u8, entry.value_ptr.string));
            }
        }

        return Manifest{
            .name = name,
            .version = version,
            .scripts = scripts,
            .dependencies = deps,
        };
    }

    pub fn save(self: *Manifest, allocator: std.mem.Allocator, path: []const u8) !void {
        var out_file = try std.fs.cwd().createFile(path, .{});
        defer out_file.close();

        var arena = std.heap.ArenaAllocator.init(allocator);
        defer arena.deinit();
        const aa = arena.allocator();

        var root = std.json.Value{ .object = std.json.ObjectMap.init(aa) };
        try root.object.put("name", std.json.Value{ .string = self.name });
        try root.object.put("version", std.json.Value{ .string = self.version });

        var scripts_map = std.json.ObjectMap.init(aa);
        var script_iter = self.scripts.iterator();
        while (script_iter.next()) |entry| {
            try scripts_map.put(entry.key_ptr.*, std.json.Value{ .string = entry.value_ptr.* });
        }
        try root.object.put("scripts", std.json.Value{ .object = scripts_map });

        var deps_map = std.json.ObjectMap.init(aa);
        var dep_iter = self.dependencies.iterator();
        while (dep_iter.next()) |entry| {
            try deps_map.put(entry.key_ptr.*, std.json.Value{ .string = entry.value_ptr.* });
        }
        try root.object.put("dependencies", std.json.Value{ .object = deps_map });

        var buf: [4096]u8 = undefined;
        var writer = out_file.writer(&buf);
        try std.json.Stringify.value(root, .{ .whitespace = .indent_2 }, &writer.interface);
        try writer.end();
    }

    pub fn addDependency(self: *Manifest, allocator: std.mem.Allocator, name: []const u8, version_range: []const u8) !void {
        const name_dup = try allocator.dupe(u8, name);
        const ver_dup = try allocator.dupe(u8, version_range);
        if (self.dependencies.get(name)) |old_v| {
            allocator.free(old_v);
        }
        try self.dependencies.put(name_dup, ver_dup);
    }

    pub fn deinit(self: *Manifest, allocator: std.mem.Allocator) void {
        allocator.free(self.name);
        allocator.free(self.version);
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
    }
};
