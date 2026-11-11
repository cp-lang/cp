/**
 * CP Standard JSON & Serde (cap:json / cap:serde)
 * 工业级零拷贝序列化与反序列化体系
 */

import { JsonTag, JsonField, JsonValue } from "./value";
import { JsonParser, JsonError } from "./parser";
import { JsonBufferPool } from "./pool";
const std = @import("std");

export class JSON {
    pub static fn parse(allocator: std.mem.Allocator, src: []const u8): !JsonValue {
        return JsonParser.parse(allocator, src);
    }

    pub fn stringify(this, obj: any): string {
        return @json_stringify(obj);
    }
}

export const json = new JSON();
