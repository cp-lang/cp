// lib/ini/parser.cp
// High-performance INI file parser in CP
const std = @import("std");

pub const Property = struct {
    section: []const u8 = "",
    key: []const u8 = "",
    value: []const u8 = "",
};

pub const Ini = struct {
    props: [128]Property = undefined,
    count: usize = 0,

    pub fn parse(content: []const u8): Ini {
        var res = Ini{};
        res.parseContent(content);
        return res;
    }

    pub fn get(self: *const Ini, section: []const u8, key: []const u8): ?[]const u8 {
        var i: usize = self.count;
        while (i > 0) {
            i -= 1;
            if (std.mem.eql(u8, self.props[i].section, section) and
                std.mem.eql(u8, self.props[i].key, key))
            {
                return self.props[i].value;
            }
        }
        return null;
    }

    pub fn getGlobal(self: *const Ini, key: []const u8): ?[]const u8 {
        return self.get("", key);
    }

    pub fn getOr(self: *const Ini, section: []const u8, key: []const u8, default_val: []const u8): []const u8 {
        if (self.get(section, key)) |v| {
            return v;
        }
        return default_val;
    }

    pub fn hasSection(self: *const Ini, section: []const u8) bool {
        var i: usize = 0;
        while (i < self.count) : (i += 1) {
            if (std.mem.eql(u8, self.props[i].section, section)) {
                return true;
            }
        }
        return false;
    }

    fn parseContent(self: *Ini, content: []const u8) void {
        var current_section: []const u8 = "";
        var line_iter = std.mem.splitScalar(u8, content, '\n');

        while (line_iter.next()) |raw_line| {
            const line = std.mem.trim(u8, raw_line, " \t\r");
            if (line.len == 0 or line[0] == ';' or line[0] == '#') continue;

            // Section header: [section_name]
            if (line[0] == '[') {
                if (std.mem.lastIndexOfScalar(u8, line, ']')) |close_idx| {
                    if (close_idx > 1) {
                        current_section = std.mem.trim(u8, line[1..close_idx], " \t");
                    }
                }
                continue;
            }

            // Key-value pair
            var delim_idx: ?usize = null;
            var i: usize = 0;
            while (i < line.len) : (i += 1) {
                if (line[i] == '=' or line[i] == ':') {
                    delim_idx = i;
                    break;
                }
            }

            if (delim_idx == null) continue;
            const idx = delim_idx.?;

            const raw_key = std.mem.trim(u8, line[0..idx], " \t");
            if (raw_key.len == 0) continue;

            const raw_val = std.mem.trim(u8, line[idx + 1 ..], " \t");
            const val = parseValue(raw_val);

            self.put(current_section, raw_key, val);
        }
    }

    fn parseValue(raw: []const u8) []const u8 {
        if (raw.len == 0) return "";

        // Double quoted
        if (raw.len >= 2 and raw[0] == '"' and raw[raw.len - 1] == '"') {
            return raw[1 .. raw.len - 1];
        }

        // Single quoted
        if (raw.len >= 2 and raw[0] == '\'' and raw[raw.len - 1] == '\'') {
            return raw[1 .. raw.len - 1];
        }

        // Strip inline comments ; or #
        var end = raw.len;
        var i: usize = 0;
        while (i < raw.len) : (i += 1) {
            if ((raw[i] == ';' or raw[i] == '#') and i > 0 and (raw[i - 1] == ' ' or raw[i - 1] == '\t')) {
                end = i - 1;
                break;
            }
        }

        return std.mem.trim(u8, raw[0..end], " \t\r");
    }

    fn put(self: *Ini, section: []const u8, key: []const u8, value: []const u8) void {
        var i: usize = 0;
        while (i < self.count) : (i += 1) {
            if (std.mem.eql(u8, self.props[i].section, section) and
                std.mem.eql(u8, self.props[i].key, key))
            {
                self.props[i].value = value;
                return;
            }
        }

        if (self.count < self.props.len) {
            self.props[self.count] = Property{
                .section = section,
                .key = key,
                .value = value,
            };
            self.count += 1;
        }
    }
};
