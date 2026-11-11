// lib/dotenv/loader.cp
// High-performance, zero-allocation .env loader in CP
const std = @import("std");

pub const Entry = struct {
    key: []const u8,
    value: []const u8,
};

pub const DotEnv = struct {
    entries: [128]Entry = undefined,
    count: usize = 0,
    buf: [4096]u8 = undefined,
    buf_len: usize = 0,

    pub fn parse(content: []const u8): DotEnv {
        var env = DotEnv{};
        env.parseContent(content);
        return env;
    }

    pub fn get(self: *const DotEnv, key: []const u8): ?[]const u8 {
        var i: usize = self.count;
        while (i > 0) {
            i -= 1;
            if (std.mem.eql(u8, self.entries[i].key, key)) {
                return self.entries[i].value;
            }
        }
        return null;
    }

    pub fn getOr(self: *const DotEnv, key: []const u8, default_val: []const u8): []const u8 {
        if (self.get(key)) |val| {
            return val;
        }
        return default_val;
    }

    pub fn has(self: *const DotEnv, key: []const u8): bool {
        return self.get(key) != null;
    }

    fn parseContent(self: *DotEnv, content: []const u8) void {
        var line_iter = std.mem.splitScalar(u8, content, '\n');
        while (line_iter.next()) |raw_line| {
            var line = std.mem.trim(u8, raw_line, " \t\r");
            if (line.len == 0 or line[0] == '#') continue;

            // Strip "export " prefix if present
            if (std.mem.startsWith(u8, line, "export ") or std.mem.startsWith(u8, line, "export\t")) {
                line = std.mem.trim(u8, line[6..], " \t");
            }

            // Find delimiter '=' or ':'
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
            const val = self.parseValue(raw_val);

            self.put(raw_key, val);
        }
    }

    fn parseValue(self: *DotEnv, raw: []const u8) []const u8 {
        if (raw.len == 0) return "";

        // Double quoted: expand \n, \r, \t, etc.
        if (raw[0] == '"') {
            const end_idx = std.mem.lastIndexOfScalar(u8, raw, '"');
            if (end_idx != null and end_idx.? > 0) {
                const inner = raw[1..end_idx.?];
                return self.unescapeDoubleQuoted(inner);
            }
        }

        // Single quoted: raw literal
        if (raw[0] == '\'') {
            const end_idx = std.mem.lastIndexOfScalar(u8, raw, '\'');
            if (end_idx != null and end_idx.? > 0) {
                return raw[1..end_idx.?];
            }
        }

        // Unquoted: strip trailing inline comments
        var end = raw.len;
        var i: usize = 0;
        while (i < raw.len) : (i += 1) {
            if (raw[i] == '#' and i > 0 and (raw[i - 1] == ' ' or raw[i - 1] == '\t')) {
                end = i - 1;
                break;
            }
        }

        return std.mem.trim(u8, raw[0..end], " \t\r");
    }

    fn unescapeDoubleQuoted(self: *DotEnv, inner: []const u8) []const u8 {
        const start_buf_pos = self.buf_len;
        var i: usize = 0;
        while (i < inner.len) {
            if (inner[i] == '\\' and i + 1 < inner.len) {
                const next_char = inner[i + 1];
                switch (next_char) {
                    'n' => self.appendChar('\n'),
                    'r' => self.appendChar('\r'),
                    't' => self.appendChar('\t'),
                    '\\' => self.appendChar('\\'),
                    '"' => self.appendChar('"'),
                    else => {
                        self.appendChar('\\');
                        self.appendChar(next_char);
                    },
                }
                i += 2;
            } else {
                self.appendChar(inner[i]);
                i += 1;
            }
        }
        return self.buf[start_buf_pos..self.buf_len];
    }

    fn appendChar(self: *DotEnv, c: u8) void {
        if (self.buf_len < self.buf.len) {
            self.buf[self.buf_len] = c;
            self.buf_len += 1;
        }
    }

    fn put(self: *DotEnv, key: []const u8, value: []const u8) void {
        // If key exists, update value
        var i: usize = 0;
        while (i < self.count) : (i += 1) {
            if (std.mem.eql(u8, self.entries[i].key, key)) {
                self.entries[i].value = value;
                return;
            }
        }

        if (self.count < self.entries.len) {
            self.entries[self.count] = Entry{
                .key = key,
                .value = value,
            };
            self.count += 1;
        }
    }
};
