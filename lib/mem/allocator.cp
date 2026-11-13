// lib/mem/allocator.cp
// High-performance Arena, StackFallbackAllocator, BumpArena, MaybeOwnedBuf, and SharedTensor
// Absorbed from bun/src/bun_alloc/ (BufferFallbackAllocator, MaxHeapAllocator, maybe_owned, MimallocArena)

export class Arena {
    pub fn alloc(this, size: usize): *! {
        return @alloc(size);
    }
    pub fn free(this): void {
        @free();
    }
}

export fn get_arena(): Arena {
    return @get_arena();
}

export class SharedTensor {
    pub size: usize = 0;
    pub ref_count: usize = 0;
    pub ptr: [*]u8! = undefined;

    pub fn init(*this, size: usize): void {
        this.size = size;
        this.ref_count = 1;
        if (size > 0) {
            zig {
                const buf = std.heap.page_allocator.alloc(u8, size) catch unreachable;
                @memset(buf, 0);
                self.ptr = buf.ptr;
            }
        }
    }

    pub fn retain(*this): void {
        if (this.ref_count > 0) {
            zig {
                _ = @atomicRmw(usize, &self.ref_count, .Add, 1, .monotonic);
            }
        }
    }

    pub fn release(*this): void {
        if (this.ref_count == 0) {
            return;
        }
        let prev: usize = 0;
        zig {
            prev = @atomicRmw(usize, &self.ref_count, .Sub, 1, .acq_rel);
        }
        if (prev == 1) {
            zig {
                if (self.size > 0) {
                    std.heap.page_allocator.free(self.ptr[0..self.size]);
                }
                self.size = 0;
            }
        }
    }

    pub fn get_size(this): usize {
        return this.size;
    }

    pub fn get_ref_count(this): usize {
        return this.ref_count;
    }
}

// Stack-first allocator with seamless fallback to heap when stack capacity is exceeded
// (Absorbed from bun/src/bun_alloc/BufferFallbackAllocator.cp)
export struct StackFallbackAllocator<const CAP: usize> {
    buffer: [CAP]u8 = undefined;
    stack_used: usize = 0;
    heap_allocs: usize = 0;
    heap_bytes: usize = 0;

    pub static fn init(): StackFallbackAllocator<CAP> {
        return StackFallbackAllocator<CAP>{
            buffer: undefined,
            stack_used: 0,
            heap_allocs: 0,
            heap_bytes: 0,
        };
    }

    pub fn allocStack(*this, len: usize): ([]u8)? {
        if (this.stack_used + len > CAP) {
            return null;
        }
        const start = this.stack_used;
        this.stack_used += len;
        return this.buffer[start .. start + len];
    }

    pub fn recordFallback(*this, len: usize): void {
        this.heap_allocs += 1;
        this.heap_bytes += len;
    }

    pub fn stackRemaining(this): usize {
        return CAP - this.stack_used;
    }

    pub fn reset(*this): void {
        this.stack_used = 0;
    }
}

// Contiguous region bump allocator (Absorbed from bun/src/bun_alloc/MaxHeapAllocator.cp)
export struct BumpArena {
    pub ptr: [*]u8! = undefined;
    pub cap: usize = 0;
    pub offset: usize = 0;

    pub static fn initCapacity(capacity: usize): BumpArena {
        const cap = if (capacity == 0) 4096 else capacity;
        zig {
            const buf = std.heap.page_allocator.alloc(u8, cap) catch unreachable;
            return BumpArena{
                .ptr = buf.ptr,
                .cap = cap,
                .offset = 0,
            };
        }
    }

    pub fn allocBytes(*this, len: usize): ([]u8)? {
        if (this.offset + len > this.cap) {
            return null;
        }
        const start = this.offset;
        this.offset += len;
        let out: []u8 = &[];
        zig {
            out = self.ptr[start .. start + len];
        }
        return out;
    }

    pub fn allocAligned(*this, len: usize, align_pow2: usize): ([]u8)? {
        const mask = align_pow2 - 1;
        const aligned_offset = (this.offset + mask) & ~mask;
        if (aligned_offset + len > this.cap) {
            return null;
        }
        this.offset = aligned_offset + len;
        let out: []u8 = &[];
        zig {
            out = self.ptr[aligned_offset .. aligned_offset + len];
        }
        return out;
    }

    pub fn used(this): usize {
        return this.offset;
    }

    pub fn remaining(this): usize {
        return this.cap - this.offset;
    }

    pub fn reset(*this): void {
        this.offset = 0;
    }

    pub fn release(*this): void {
        zig {
            if (self.cap > 0) {
                std.heap.page_allocator.free(self.ptr[0..self.cap]);
                self.cap = 0;
                self.offset = 0;
            }
        }
    }
}

// Ownership-aware buffer wrapper (Absorbed from bun/src/bun_alloc/maybe_owned.cp)
export struct MaybeOwnedBuf {
    data: []const u8 = &[];
    owned: bool = false;

    pub static fn initBorrowed(buf: []const u8): MaybeOwnedBuf {
        return MaybeOwnedBuf{
            data: buf,
            owned: false,
        };
    }

    pub static fn initOwnedCopy(buf: []const u8): MaybeOwnedBuf {
        let duped: []const u8 = &[];
        zig {
            duped = std.heap.page_allocator.dupe(u8, buf) catch unreachable;
        }
        return MaybeOwnedBuf{
            data: duped,
            owned: true,
        };
    }

    pub fn isOwned(this): bool {
        return this.owned;
    }

    pub fn slice(this): []const u8 {
        return this.data;
    }

    pub fn len(this): usize {
        return this.data.len;
    }

    pub fn release(*this): void {
        if (this.owned && this.data.len > 0) {
            zig {
                std.heap.page_allocator.free(self.data);
            }
        }
        this.data = &[];
        this.owned = false;
    }
}
