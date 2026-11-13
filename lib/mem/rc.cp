// lib/mem/rc.cp
// Deterministic Reference Counting & Weak Control Block (absorbed from bun/src/ptr/raw_ref_count.cp, ref_count.cp, weak_ptr.cp)

export enum DecrementResult {
    KeepAlive,
    ShouldDestroy,
}

// Single-threaded deterministic reference counter
export struct RefCounter {
    raw_value: u32 = 1;

    pub static fn init(initial_count: u32): RefCounter {
        return RefCounter{
            raw_value: initial_count,
        };
    }

    pub fn increment(*this): u32 {
        this.raw_value += 1;
        return this.raw_value;
    }

    pub fn decrement(*this): DecrementResult {
        this.raw_value -= 1;
        if (this.raw_value == 0) {
            return .ShouldDestroy;
        }
        return .KeepAlive;
    }

    pub fn count(this): u32 {
        return this.raw_value;
    }

    pub fn isUnique(this): bool {
        return this.raw_value == 1;
    }
}

// Thread-safe atomic reference counter
export struct AtomicRefCounter {
    raw_value: u32 = 1;

    pub static fn init(initial_count: u32): AtomicRefCounter {
        return AtomicRefCounter{
            raw_value: initial_count,
        };
    }

    pub fn increment(*this): u32 {
        let prev: u32 = 0;
        zig {
            prev = @atomicRmw(u32, &self.raw_value, .Add, 1, .monotonic);
        }
        return prev + 1;
    }

    pub fn decrement(*this): DecrementResult {
        let prev: u32 = 0;
        zig {
            prev = @atomicRmw(u32, &self.raw_value, .Sub, 1, .acq_rel);
        }
        if (prev == 1) {
            return .ShouldDestroy;
        }
        return .KeepAlive;
    }

    pub fn count(this): u32 {
        let cur: u32 = 0;
        zig {
            cur = @atomicLoad(u32, &self.raw_value, .monotonic);
        }
        return cur;
    }
}

// Weak reference control block (Absorbed from bun/src/ptr/weak_ptr.cp)
export struct WeakControlBlock {
    strong_count: u32 = 1;
    weak_count: u32 = 0;
    finalized: bool = false;

    pub static fn init(): WeakControlBlock {
        return WeakControlBlock{
            strong_count: 1,
            weak_count: 0,
            finalized: false,
        };
    }

    pub fn incStrong(*this): bool {
        if (this.finalized || this.strong_count == 0) {
            return false;
        }
        this.strong_count += 1;
        return true;
    }

    pub fn decStrong(*this): DecrementResult {
        this.strong_count -= 1;
        if (this.strong_count == 0) {
            this.finalized = true;
            return .ShouldDestroy;
        }
        return .KeepAlive;
    }

    pub fn incWeak(*this): u32 {
        this.weak_count += 1;
        return this.weak_count;
    }

    pub fn decWeak(*this): bool {
        this.weak_count -= 1;
        return this.finalized && this.weak_count == 0;
    }

    pub fn isAlive(this): bool {
        return !this.finalized && this.strong_count > 0;
    }

    pub fn canDeallocate(this): bool {
        return this.finalized && this.strong_count == 0 && this.weak_count == 0;
    }

    pub fn strongCount(this): u32 {
        return this.strong_count;
    }

    pub fn weakCount(this): u32 {
        return this.weak_count;
    }
}
