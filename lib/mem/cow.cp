// lib/mem/cow.cp
// Copy-on-Write byte slice (absorbed from bun/src/ptr/CowSlice.cp & Cow.cp)

export struct CowBytes {
    pub ptr: [*]u8! = undefined;
    pub length: usize = 0;
    pub is_owned: bool = false;

    pub static fn initBorrowed(data: []const u8): CowBytes {
        zig {
            return CowBytes{
                .ptr = @constCast(data.ptr),
                .length = data.len,
                .is_owned = false,
            };
        }
    }

    pub static fn initOwned(data: []const u8): CowBytes {
        zig {
            const buf = std.heap.page_allocator.dupe(u8, data) catch unreachable;
            return CowBytes{
                .ptr = buf.ptr,
                .length = data.len,
                .is_owned = true,
            };
        }
    }

    pub fn borrow(this): CowBytes {
        return CowBytes{
            ptr: this.ptr,
            length: this.length,
            is_owned: false,
        };
    }

    pub fn isOwned(this): bool {
        return this.is_owned;
    }

    pub fn len(this): usize {
        return this.length;
    }

    pub fn slice(this): []const u8 {
        if (this.length == 0) {
            return &[];
        }
        let s: []const u8 = &[];
        zig {
            s = self.ptr[0..self.length];
        }
        return s;
    }

    // Ensures the buffer is uniquely owned on the heap before mutation (Copy-on-Write)
    pub fn ensureOwned(*this): void {
        if (!this.is_owned && this.length > 0) {
            zig {
                const duped = std.heap.page_allocator.dupe(u8, self.ptr[0..self.length]) catch unreachable;
                self.ptr = duped.ptr;
                self.is_owned = true;
            }
        } else if (!this.is_owned) {
            this.is_owned = true;
        }
    }

    pub fn setByte(*this, index: usize, val: u8): bool {
        if (index >= this.length) {
            return false;
        }
        this.ensureOwned();
        unsafe {
            this.ptr[index] = val;
        }
        return true;
    }

    pub fn release(*this): void {
        zig {
            if (self.is_owned and self.length > 0) {
                std.heap.page_allocator.free(self.ptr[0..self.length]);
            }
            self.length = 0;
            self.is_owned = false;
        }
    }
}
