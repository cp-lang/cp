// lib/mem/tagged_ptr.cp
// 64-bit NaN-boxing / upper-16-bit & low-3-bit Tagged Pointers (absorbed from bun/src/ptr/tagged_pointer.cp)

export const TAG_SHIFT: u6 = 48;
export const ADDR_MASK: u64 = 0x0000FFFFFFFFFFFF;
export const LOW_TAG_MASK: usize = 7;

// Upper 16-bit tagged pointer (48-bit canonical address + 16-bit tag payload)
export struct TaggedPointer {
    raw: u64 = 0;

    pub static fn pack(addr: usize, tag_val: u16): TaggedPointer {
        const masked_addr = (addr as u64) & ADDR_MASK;
        const shifted_tag = (tag_val as u64) << 48;
        return TaggedPointer{
            raw: masked_addr | shifted_tag,
        };
    }

    pub static fn fromPtr(ptr: *!?, tag_val: u16): TaggedPointer {
        if (p := ptr) {
            let addr: usize = 0;
            unsafe {
                addr = @intFromPtr(p);
            }
            return TaggedPointer.pack(addr, tag_val);
        }
        return TaggedPointer.pack(0, tag_val);
    }

    pub static fn fromBits(raw_bits: u64): TaggedPointer {
        return TaggedPointer{ raw: raw_bits };
    }

    pub fn address(this): usize {
        return (this.raw & ADDR_MASK) as usize;
    }

    pub fn tag(this): u16 {
        return (this.raw >> 48) as u16;
    }

    pub fn withTag(this, new_tag: u16): TaggedPointer {
        return TaggedPointer.pack(this.address(), new_tag);
    }

    pub fn isNull(this): bool {
        return (this.raw & ADDR_MASK) == 0;
    }

    pub fn bits(this): u64 {
        return this.raw;
    }

    pub fn toRawPtr(this): *!? {
        const addr = this.address();
        if (addr == 0) {
            return null;
        }
        unsafe {
            return addr as *!?;
        }
    }
}

// Low 3-bit tagged pointer for 8-byte aligned allocations
export struct LowTagPointer {
    raw: usize = 0;

    pub static fn pack(aligned_addr: usize, low_tag: u8): LowTagPointer {
        const clean_addr = aligned_addr & ~LOW_TAG_MASK;
        const tag_bits = (low_tag as usize) & LOW_TAG_MASK;
        return LowTagPointer{
            raw: clean_addr | tag_bits,
        };
    }

    pub fn address(this): usize {
        return this.raw & ~LOW_TAG_MASK;
    }

    pub fn tag(this): u8 {
        return (this.raw & LOW_TAG_MASK) as u8;
    }

    pub fn withTag(this, new_tag: u8): LowTagPointer {
        return LowTagPointer.pack(this.address(), new_tag);
    }

    pub fn isNull(this): bool {
        return this.address() == 0;
    }
}
