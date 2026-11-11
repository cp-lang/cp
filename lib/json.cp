/**
 * CP Standard JSON Serializer & Deserializer (cap:json)
 * Zero-copy slice parsing & reflection stringification
 */

import { JsonTag, JsonField, JsonValue } from "./json/value";
import { JsonParser, JsonError } from "./json/parser";
import { JsonBufferPool } from "./json/pool";
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
