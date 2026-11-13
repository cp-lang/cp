// lib/collections/string_map.cp
// Length-filtered fast string lookup map
// Absorbed from Bun src/collections/comptime_string_map.cp & StaticHashMap.cp

export struct StringEntry<V> {
    pub key: []const u8 = "";
    pub value: V = undefined;
    pub occupied: bool = false;
}

export struct FastStringMap<V, const CAP: usize> {
    pub entries: [CAP]StringEntry<V> = undefined;
    pub len: usize = 0;

    pub static fn init(): FastStringMap<V, CAP> {
        var m = FastStringMap<V, CAP> {
            entries: undefined,
            len: 0,
        };
        var i: usize = 0;
        while (i < CAP) {
            m.entries[i].occupied = false;
            m.entries[i].key = "";
            i += 1;
        }
        return m;
    }

    pub static fn equalsBytes(a: []const u8, b: []const u8): bool {
        if (a.len != b.len) {
            return false;
        }
        var i: usize = 0;
        while (i < a.len) {
            if (a[i] != b[i]) {
                return false;
            }
            i += 1;
        }
        return true;
    }

    pub static fn equalsAsciiCaseInsensitive(a: []const u8, b: []const u8): bool {
        if (a.len != b.len) {
            return false;
        }
        var i: usize = 0;
        while (i < a.len) {
            const ca = if (a[i] >= 65 && a[i] <= 90) a[i] + 32 else a[i];
            const cb = if (b[i] >= 65 && b[i] <= 90) b[i] + 32 else b[i];
            if (ca != cb) {
                return false;
            }
            i += 1;
        }
        return true;
    }

    pub fn count(&this): usize {
        return this.len;
    }

    pub fn put(*this, key: []const u8, value: V): bool {
        var i: usize = 0;
        while (i < this.len) {
            if (this.entries[i].occupied && FastStringMap<V, CAP>.equalsBytes(this.entries[i].key, key)) {
                this.entries[i].value = value;
                return true;
            }
            i += 1;
        }
        if (this.len >= CAP) {
            return false;
        }
        this.entries[this.len] = StringEntry<V> {
            key: key,
            value: value,
            occupied: true,
        };
        this.len += 1;
        return true;
    }

    // Absorbed from Bun ComptimeStringMap: filters by key.len first before byte comparison
    pub fn get(&this, key: []const u8): V? {
        var i: usize = 0;
        while (i < this.len) {
            if (this.entries[i].occupied && this.entries[i].key.len == key.len) {
                if (FastStringMap<V, CAP>.equalsBytes(this.entries[i].key, key)) {
                    return this.entries[i].value;
                }
            }
            i += 1;
        }
        return null;
    }

    pub fn getCaseInsensitive(&this, key: []const u8): V? {
        var i: usize = 0;
        while (i < this.len) {
            if (this.entries[i].occupied && this.entries[i].key.len == key.len) {
                if (FastStringMap<V, CAP>.equalsAsciiCaseInsensitive(this.entries[i].key, key)) {
                    return this.entries[i].value;
                }
            }
            i += 1;
        }
        return null;
    }

    pub fn has(&this, key: []const u8): bool {
        var i: usize = 0;
        while (i < this.len) {
            if (this.entries[i].occupied && this.entries[i].key.len == key.len) {
                if (FastStringMap<V, CAP>.equalsBytes(this.entries[i].key, key)) {
                    return true;
                }
            }
            i += 1;
        }
        return false;
    }

    pub fn clear(*this): void {
        var i: usize = 0;
        while (i < this.len) {
            this.entries[i].occupied = false;
            i += 1;
        }
        this.len = 0;
    }
}
