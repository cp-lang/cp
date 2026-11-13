// lib/collections/bounded_array.cp
// Zero-heap stack-allocated bounded array with compile-time capacity
// Absorbed from Bun src/collections/bounded_array.cp

export struct BoundedArray<T, const CAP: usize> {
    pub buffer: [CAP]T = undefined;
    pub len: usize = 0;

    pub static fn init(): BoundedArray<T, CAP> {
        return BoundedArray<T, CAP> {
            buffer: undefined,
            len: 0,
        };
    }

    pub fn capacity(&this): usize {
        return CAP;
    }

    pub fn isEmpty(&this): bool {
        return this.len == 0;
    }

    pub fn isFull(&this): bool {
        return this.len >= CAP;
    }

    pub fn unusedCapacity(&this): usize {
        return CAP - this.len;
    }

    pub fn clear(*this): void {
        this.len = 0;
    }

    pub fn append(*this, item: T): bool {
        if (this.len >= CAP) {
            return false;
        }
        this.buffer[this.len] = item;
        this.len += 1;
        return true;
    }

    pub fn appendSlice(*this, items: []const T): bool {
        if (this.len + items.len > CAP) {
            return false;
        }
        var i: usize = 0;
        while (i < items.len) {
            this.buffer[this.len + i] = items[i];
            i += 1;
        }
        this.len += items.len;
        return true;
    }

    pub fn get(&this, i: usize): T {
        return this.buffer[i];
    }

    pub fn set(*this, i: usize, item: T): void {
        if (i < this.len) {
            this.buffer[i] = item;
        }
    }

    pub fn pop(*this): T? {
        if (this.len == 0) {
            return null;
        }
        this.len -= 1;
        return this.buffer[this.len];
    }

    pub fn orderedRemove(*this, idx: usize): T {
        const val = this.buffer[idx];
        var i: usize = idx;
        while (i + 1 < this.len) {
            this.buffer[i] = this.buffer[i + 1];
            i += 1;
        }
        this.len -= 1;
        return val;
    }

    pub fn swapRemove(*this, idx: usize): T {
        const val = this.buffer[idx];
        this.len -= 1;
        if (idx != this.len) {
            this.buffer[idx] = this.buffer[this.len];
        }
        return val;
    }
}
