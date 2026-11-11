// Ultra-fast zero-copy streaming JSON parser in CP
// String keys and values borrow directly from input slice without heap allocation
import { JsonValue, JsonField, JsonTag } from "./value";
const std = @import("std");

export error JsonError {
    UnexpectedEof,
    InvalidToken,
    ExpectedColon,
    ExpectedComma,
}

export struct JsonParser {
    src: []const u8 = "",
    pos: usize = 0,
    allocator: std.mem.Allocator = undefined,

    pub static fn init(allocator: std.mem.Allocator, src: []const u8): JsonParser {
        return JsonParser{
            .src = src,
            .pos = 0,
            .allocator = allocator,
        };
    }

    fn skipWhitespace(*this): void {
        while (pos < src.len) {
            const b = src[pos];
            if (b == ' ' or b == '\t' or b == '\r' or b == '\n') {
                pos += 1;
            } else {
                break;
            }
        }
    }

    fn peek(*this): ?u8 {
        skipWhitespace();
        if (pos >= src.len) return null;
        return src[pos];
    }

    fn next(*this): ?u8 {
        skipWhitespace();
        if (pos >= src.len) return null;
        const b = src[pos];
        pos += 1;
        return b;
    }

    fn parseString(*this): ![]const u8 {
        skipWhitespace();
        if (pos >= src.len or src[pos] != '"') {
            return error.InvalidToken;
        }
        pos += 1; // skip opening quote
        const start = pos;
        while (pos < src.len) {
            const b = src[pos];
            if (b == '\\') {
                pos += 2; // skip escape
            } else if (b == '"') {
                const slice = src[start..pos];
                pos += 1; // skip closing quote
                return slice;
            } else {
                pos += 1;
            }
        }
        return error.UnexpectedEof;
    }

    fn parseNumber(*this): !f64 {
        skipWhitespace();
        const start = pos;
        if (pos < src.len and src[pos] == '-') {
            pos += 1;
        }
        while (pos < src.len) {
            const b = src[pos];
            if ((b >= '0' and b <= '9') or b == '.' or b == 'e' or b == 'E' or b == '+' or b == '-') {
                pos += 1;
            } else {
                break;
            }
        }
        const num_str = src[start..pos];
        return std.fmt.parseFloat(f64, num_str) catch return error.InvalidToken;
    }

    pub fn parseValue(*this): !JsonValue {
        skipWhitespace();
        if (pos >= src.len) return error.UnexpectedEof;
        const ch = src[pos];

        if (ch == 'n') {
            if (pos + 4 <= src.len and std.mem.eql(u8, src[pos..pos + 4], "null")) {
                pos += 4;
                return JsonValue{ .tag = .Null };
            }
            return error.InvalidToken;
        }

        if (ch == 't') {
            if (pos + 4 <= src.len and std.mem.eql(u8, src[pos..pos + 4], "true")) {
                pos += 4;
                return JsonValue{ .tag = .Bool, .bool_val = true };
            }
            return error.InvalidToken;
        }

        if (ch == 'f') {
            if (pos + 5 <= src.len and std.mem.eql(u8, src[pos..pos + 5], "false")) {
                pos += 5;
                return JsonValue{ .tag = .Bool, .bool_val = false };
            }
            return error.InvalidToken;
        }

        if (ch == '"') {
            const s = try parseString();
            return JsonValue{ .tag = .String, .str_val = s };
        }

        if (ch == '-' or (ch >= '0' and ch <= '9')) {
            const n = try parseNumber();
            return JsonValue{ .tag = .Number, .num_val = n };
        }

        if (ch == '[') {
            return parseArray();
        }

        if (ch == '{') {
            return parseObject();
        }

        return error.InvalidToken;
    }

    fn parseArray(*this): !JsonValue {
        pos += 1; // skip '['
        skipWhitespace();

        var list: std.ArrayList(JsonValue) = .empty;
        errdefer list.deinit(allocator);

        if (peek()) |next_ch| {
            if (next_ch == ']') {
                pos += 1;
                return JsonValue{ .tag = .Array, .arr_val = try list.toOwnedSlice(allocator) };
            }
        }

        while (true) {
            const val = try parseValue();
            try list.append(allocator, val);

            skipWhitespace();
            if (pos >= src.len) return error.UnexpectedEof;
            const sep = src[pos];
            if (sep == ']') {
                pos += 1;
                break;
            } else if (sep == ',') {
                pos += 1;
            } else {
                return error.ExpectedComma;
            }
        }

        return JsonValue{ .tag = .Array, .arr_val = try list.toOwnedSlice(allocator) };
    }

    fn parseObject(*this): !JsonValue {
        pos += 1; // skip '{'
        skipWhitespace();

        var list: std.ArrayList(JsonField) = .empty;
        errdefer list.deinit(allocator);

        if (peek()) |next_ch| {
            if (next_ch == '}') {
                pos += 1;
                return JsonValue{ .tag = .Object, .obj_val = try list.toOwnedSlice(allocator) };
            }
        }

        while (true) {
            const key = try parseString();
            skipWhitespace();
            if (pos >= src.len or src[pos] != ':') {
                return error.ExpectedColon;
            }
            pos += 1; // skip ':'

            const val = try parseValue();
            try list.append(allocator, JsonField{ .key = key, .value = val });

            skipWhitespace();
            if (pos >= src.len) return error.UnexpectedEof;
            const sep = src[pos];
            if (sep == '}') {
                pos += 1;
                break;
            } else if (sep == ',') {
                pos += 1;
            } else {
                return error.ExpectedComma;
            }
        }

        return JsonValue{ .tag = .Object, .obj_val = try list.toOwnedSlice(allocator) };
    }

    pub static fn parse(allocator: std.mem.Allocator, src: []const u8): !JsonValue {
        var parser = JsonParser.init(allocator, src);
        return parser.parseValue();
    }
}
