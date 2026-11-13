// lib/collections/hive_array.cp
// O(1) BitSet64 and HiveArray slot pool with pointer/index stability
// Absorbed from Bun src/collections/hive_array.cp and bit_set.zig

export struct BitSet64 {
    pub mask: u64 = 0;

    pub static fn initEmpty(): BitSet64 {
        return BitSet64 { mask: 0 };
    }

    pub static fn initFull(): BitSet64 {
        return BitSet64 { mask: 0xFFFFFFFFFFFFFFFF };
    }

    pub fn isSet(&this, index: usize): bool {
        if (index >= 64) {
            return false;
        }
        const bit: u64 = (1 as u64) << index;
        return (this.mask & bit) != 0;
    }

    pub fn set(*this, index: usize): void {
        if (index < 64) {
            const bit: u64 = (1 as u64) << index;
            this.mask |= bit;
        }
    }

    pub fn unset(*this, index: usize): void {
        if (index < 64) {
            const bit: u64 = (1 as u64) << index;
            this.mask &= ~bit;
        }
    }

    pub fn count(&this): usize {
        return @popCount(this.mask) as usize;
    }

    pub fn findFirstUnset(&this, limit: usize): usize? {
        const inverted: u64 = ~this.mask;
        if (inverted == 0) {
            return null;
        }
        const idx = @ctz(inverted) as usize;
        if (idx >= limit) {
            return null;
        }
        return idx;
    }

    pub fn findFirstSet(&this, limit: usize): usize? {
        if (this.mask == 0) {
            return null;
        }
        const idx = @ctz(this.mask) as usize;
        if (idx >= limit) {
            return null;
        }
        return idx;
    }
}

export struct HiveArray<T, const CAP: usize> {
    pub buffer: [CAP]T = undefined;
    pub used: BitSet64 = BitSet64 {};

    pub static fn init(): HiveArray<T, CAP> {
        return HiveArray<T, CAP> {
            buffer: undefined,
            used: BitSet64.initEmpty(),
        };
    }

    pub fn capacity(&this): usize {
        return CAP;
    }

    pub fn usedCount(&this): usize {
        return this.used.count();
    }

    pub fn availableCount(&this): usize {
        return CAP - this.used.count();
    }

    pub fn isEmpty(&this): bool {
        return this.used.count() == 0;
    }

    pub fn isFull(&this): bool {
        return this.used.count() >= CAP;
    }

    // Claims an unused slot in O(1) using hardware @ctz, writes `val`, and returns its stable slot index
    pub fn acquire(*this, val: T): usize? {
        if (slot := this.used.findFirstUnset(CAP)) {
            this.used.set(slot);
            this.buffer[slot] = val;
            return slot;
        }
        return null;
    }

    pub fn get(&this, slot: usize): T? {
        if (!this.used.isSet(slot)) {
            return null;
        }
        return this.buffer[slot];
    }

    pub fn set(*this, slot: usize, val: T): bool {
        if (!this.used.isSet(slot)) {
            return false;
        }
        this.buffer[slot] = val;
        return true;
    }

    pub fn releaseSlot(*this, slot: usize): bool {
        if (!this.used.isSet(slot)) {
            return false;
        }
        this.used.unset(slot);
        return true;
    }

    pub fn clear(*this): void {
        this.used = BitSet64.initEmpty();
    }
}
