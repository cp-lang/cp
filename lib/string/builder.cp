// lib/string/builder.cp
// HashedString, SmolStr (15-byte inline small string), and two-pass StringBuilder
// Absorbed from bun/src/string/HashedString.zig, SmolStr.zig, StringBuilder.zig

// Fast 32-bit FNV-1a hash for HashedString O(1) comparison
export fn hashBytes32(data: []const u8): u32 {
    let h: u32 = 2166136261;
    let i: usize = 0;
    while (i < data.len) {
        h = (h ^ (data[i] as u32)) *% 16777619;
        i += 1;
    }
    return if (h == 0) 1 else h;
}

// Pre-hashed string view for O(1) fast rejection in identifier/header tables
export struct HashedString {
    pub data: []const u8 = "";
    pub hash: u32 = 0;

    pub static fn init(buf: []const u8): HashedString {
        return HashedString{
            data: buf,
            hash: hashBytes32(buf),
        };
    }

    pub fn len(this): usize {
        return this.data.len;
    }

    pub fn eql(this, other: HashedString): bool {
        if (this.hash != other.hash || this.data.len != other.data.len) {
            return false;
        }
        let eq = false;
        zig {
            eq = std.mem.eql(u8, self.data, other.data);
        }
        return eq;
    }
}

export const SMOL_MAX_INLINE: usize = 15;

// Small-string optimized container: stores <= 15 bytes inline on the stack without heap allocation
export struct SmolStr {
    inline_buf: [15]u8 = undefined;
    heap_ptr: [*]u8! = undefined;
    length: u32 = 0;
    inlined: bool = true;

    pub static fn init(s: []const u8): SmolStr {
        if (s.len <= SMOL_MAX_INLINE) {
            let res = SmolStr{
                inline_buf: undefined,
                heap_ptr: undefined,
                length: s.len as u32,
                inlined: true,
            };
            let i: usize = 0;
            while (i < s.len) {
                res.inline_buf[i] = s[i];
                i += 1;
            }
            return res;
        }
        zig {
            const duped = std.heap.page_allocator.dupe(u8, s) catch unreachable;
            return SmolStr{
                .inline_buf = undefined,
                .heap_ptr = duped.ptr,
                .length = @intCast(s.len),
                .inlined = false,
            };
        }
    }

    pub fn isInlined(this): bool {
        return this.inlined;
    }

    pub fn len(this): usize {
        return this.length as usize;
    }

    pub fn slice(*this): []const u8 {
        const n = this.length as usize;
        if (n == 0) {
            return &[];
        }
        if (this.inlined) {
            return this.inline_buf[0..n];
        }
        let s: []const u8 = &[];
        zig {
            s = self.heap_ptr[0..n];
        }
        return s;
    }

    pub fn release(*this): void {
        if (!this.inlined && this.length > 0) {
            const n = this.length as usize;
            zig {
                std.heap.page_allocator.free(self.heap_ptr[0..n]);
            }
        }
        this.length = 0;
        this.inlined = true;
    }
}

// Two-pass zero-reallocation StringBuilder (Absorbed from bun/src/string/StringBuilder.zig)
export struct StringBuilder {
    pub ptr: [*]u8! = undefined;
    pub len: usize = 0;
    pub cap: usize = 0;
    pub allocated: bool = false;

    pub static fn init(): StringBuilder {
        return StringBuilder{
            ptr: undefined,
            len: 0,
            cap: 0,
            allocated: false,
        };
    }

    // Pass 1: accumulate required byte capacity
    pub fn count(*this, s: []const u8): void {
        this.cap += s.len;
    }

    pub fn countByte(*this): void {
        this.cap += 1;
    }

    // Allocate exact buffer once between Pass 1 and Pass 2
    pub fn allocate(*this): void {
        const c = if (this.cap == 0) 64 else this.cap;
        this.cap = c;
        this.len = 0;
        this.allocated = true;
        zig {
            const buf = std.heap.page_allocator.alloc(u8, c) catch unreachable;
            self.ptr = buf.ptr;
        }
    }

    // Pass 2: append slices into pre-counted buffer without any reallocation
    pub fn append(*this, s: []const u8): []const u8 {
        if (!this.allocated) {
            this.cap += s.len + 64;
            this.allocate();
        }
        if (this.len + s.len > this.cap) {
            const new_cap = (this.cap + s.len) * 2;
            const old_len = this.len;
            const old_cap = this.cap;
            zig {
                const new_buf = std.heap.page_allocator.alloc(u8, new_cap) catch unreachable;
                if (old_len > 0) {
                    @memcpy(new_buf[0..old_len], self.ptr[0..old_len]);
                }
                if (old_cap > 0) {
                    std.heap.page_allocator.free(self.ptr[0..old_cap]);
                }
                self.ptr = new_buf.ptr;
                self.cap = new_cap;
            }
        }
        const start = this.len;
        const n = s.len;
        zig {
            @memcpy(self.ptr[start .. start + n], s);
        }
        this.len += n;
        let out: []const u8 = &[];
        zig {
            out = self.ptr[start .. start + n];
        }
        return out;
    }

    pub fn appendByte(*this, b: u8): void {
        let single: [1]u8 = [b];
        _ = this.append(single[0..1]);
    }

    pub fn slice(this): []const u8 {
        if (!this.allocated || this.len == 0) {
            return &[];
        }
        let out: []const u8 = &[];
        zig {
            out = self.ptr[0..self.len];
        }
        return out;
    }

    pub fn reset(*this): void {
        this.len = 0;
    }

    pub fn release(*this): void {
        if (this.allocated && this.cap > 0) {
            zig {
                std.heap.page_allocator.free(self.ptr[0..self.cap]);
            }
        }
        this.len = 0;
        this.cap = 0;
        this.allocated = false;
    }
}
