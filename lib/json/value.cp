// Zero-copy JSON value representation in CP
// String keys and string values borrow directly from source slice without heap duplication

export enum JsonTag {
    Null,
    Bool,
    Number,
    String,
    Array,
    Object,
}

export struct JsonField {
    key: []const u8 = "",
    value: JsonValue = .{},
}

export struct JsonValue {
    tag: JsonTag = .Null,
    bool_val: bool = false,
    num_val: f64 = 0,
    str_val: []const u8 = "",
    arr_val: []JsonValue = &.{},
    obj_val: []JsonField = &.{},

    pub fn isNull(self: *const JsonValue): bool {
        return self.tag == .Null;
    }

    pub fn asBool(self: *const JsonValue): ?bool {
        if (self.tag == .Bool) return self.bool_val;
        return null;
    }

    pub fn asNumber(self: *const JsonValue): ?f64 {
        if (self.tag == .Number) return self.num_val;
        return null;
    }

    pub fn asString(self: *const JsonValue): ?[]const u8 {
        if (self.tag == .String) return self.str_val;
        return null;
    }

    pub fn asArray(self: *const JsonValue): ?[]JsonValue {
        if (self.tag == .Array) return self.arr_val;
        return null;
    }

    pub fn asObject(self: *const JsonValue): ?[]JsonField {
        if (self.tag == .Object) return self.obj_val;
        return null;
    }

    pub fn len(self: *const JsonValue): usize {
        return switch (self.tag) {
            .Array => self.arr_val.len,
            .Object => self.obj_val.len,
            .String => self.str_val.len,
            else => 0,
        };
    }

    pub fn get(self: *const JsonValue, key: []const u8): ?JsonValue {
        if (self.tag != .Object) return null;
        for (self.obj_val) |field| {
            if (field.key.len == key.len) {
                var match_key = true;
                var i: usize = 0;
                while (i < key.len) {
                    if (field.key[i] != key[i]) {
                        match_key = false;
                        break;
                    }
                    i += 1;
                }
                if (match_key) return field.value;
            }
        }
        return null;
    }

    pub fn getIndex(self: *const JsonValue, idx: usize): ?JsonValue {
        if (self.tag != .Array or idx >= self.arr_val.len) return null;
        return self.arr_val[idx];
    }
}
