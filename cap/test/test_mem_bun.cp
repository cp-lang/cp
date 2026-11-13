import { test, expect_eq } from "cap:test";
import {
    StackFallbackAllocator,
    BumpArena,
    MaybeOwnedBuf,
    SharedTensor,
    TaggedPointer,
    LowTagPointer,
    CowBytes,
    DecrementResult,
    RefCounter,
    AtomicRefCounter,
    WeakControlBlock,
} from "cap:mem";

fn main(): void {
    test("1. StackFallbackAllocator<CAP> zero-heap stack fast-path & fallback tracking", () => {
        let sfa = StackFallbackAllocator<64>.init();
        expect_eq(sfa.stackRemaining(), 64);

        if (b1 := sfa.allocStack(24)) {
            expect_eq(b1.len, 24);
            b1[0] = 0xAA;
            b1[23] = 0xBB;
            expect_eq(b1[0], 0xAA);
            expect_eq(b1[23], 0xBB);
        } else {
            expect_eq(true, false);
        }
        expect_eq(sfa.stackRemaining(), 40);

        if (b2 := sfa.allocStack(32)) {
            expect_eq(b2.len, 32);
        } else {
            expect_eq(true, false);
        }
        expect_eq(sfa.stackRemaining(), 8);

        // Requesting 16 bytes exceeds remaining 8 bytes on stack
        const over = sfa.allocStack(16);
        expect_eq(over == null, true);
        sfa.recordFallback(16);
        expect_eq(sfa.heap_allocs, 1);
        expect_eq(sfa.heap_bytes, 16);

        sfa.reset();
        expect_eq(sfa.stackRemaining(), 64);
    });

    test("2. BumpArena & MaybeOwnedBuf contiguous arena and ownership-aware buffer", () => {
        let ba = BumpArena.initCapacity(256);
        expect_eq(ba.used(), 0);
        expect_eq(ba.remaining(), 256);

        if (s1 := ba.allocBytes(13)) {
            expect_eq(s1.len, 13);
            s1[0] = 42;
            expect_eq(s1[0], 42);
        } else {
            expect_eq(true, false);
        }
        expect_eq(ba.used(), 13);

        // 8-byte aligned allocation should advance offset from 13 to 16, then + 16 = 32
        if (s2 := ba.allocAligned(16, 8)) {
            expect_eq(s2.len, 16);
        } else {
            expect_eq(true, false);
        }
        expect_eq(ba.used(), 32);

        ba.reset();
        expect_eq(ba.used(), 0);
        ba.release();

        let borrowed = MaybeOwnedBuf.initBorrowed("static-view");
        expect_eq(borrowed.isOwned(), false);
        expect_eq(borrowed.len(), 11);
        borrowed.release();

        let owned = MaybeOwnedBuf.initOwnedCopy("heap-copy");
        expect_eq(owned.isOwned(), true);
        expect_eq(owned.len(), 9);
        expect_eq(owned.slice(), "heap-copy");
        owned.release();
        expect_eq(owned.isOwned(), false);
    });

    test("3. TaggedPointer (48-bit addr + 16-bit tag) & LowTagPointer (low 3-bit tag)", () => {
        const fake_addr: usize = 0x00007FFF12345678;
        const tp = TaggedPointer.pack(fake_addr, 0xCAFE);
        expect_eq(tp.isNull(), false);
        expect_eq(tp.address(), fake_addr);
        expect_eq(tp.tag(), 0xCAFE);

        const tp2 = tp.withTag(0x0042);
        expect_eq(tp2.address(), fake_addr);
        expect_eq(tp2.tag(), 0x0042);

        const null_tp = TaggedPointer.pack(0, 99);
        expect_eq(null_tp.isNull(), true);
        expect_eq(null_tp.tag(), 99);

        const aligned_addr: usize = 0x100080;
        const ltp = LowTagPointer.pack(aligned_addr, 5);
        expect_eq(ltp.address(), aligned_addr);
        expect_eq(ltp.tag(), 5);
        const ltp2 = ltp.withTag(3);
        expect_eq(ltp2.address(), aligned_addr);
        expect_eq(ltp2.tag(), 3);
    });

    test("4. CowBytes Copy-on-Write lazy duplication & mutation isolation", () => {
        let orig = CowBytes.initBorrowed("hello");
        expect_eq(orig.isOwned(), false);
        expect_eq(orig.len(), 5);
        expect_eq(orig.slice(), "hello");

        let view = orig.borrow();
        expect_eq(view.isOwned(), false);
        expect_eq(view.slice(), "hello");

        // Mutating `view` triggers Copy-on-Write allocation without mutating `orig`
        expect_eq(view.setByte(0, 72), true); // 'H' = 72
        expect_eq(view.isOwned(), true);
        expect_eq(view.slice(), "Hello");
        expect_eq(orig.slice(), "hello");

        view.release();
        orig.release();
    });

    test("5. RefCounter, AtomicRefCounter, WeakControlBlock & SharedTensor", () => {
        let rc = RefCounter.init(1);
        expect_eq(rc.isUnique(), true);
        expect_eq(rc.increment(), 2);
        expect_eq(rc.isUnique(), false);
        expect_eq(rc.decrement() == DecrementResult.KeepAlive, true);
        expect_eq(rc.decrement() == DecrementResult.ShouldDestroy, true);

        let arc = AtomicRefCounter.init(1);
        expect_eq(arc.increment(), 2);
        expect_eq(arc.count(), 2);
        expect_eq(arc.decrement() == DecrementResult.KeepAlive, true);
        expect_eq(arc.decrement() == DecrementResult.ShouldDestroy, true);

        let wcb = WeakControlBlock.init();
        expect_eq(wcb.isAlive(), true);
        expect_eq(wcb.incWeak(), 1);
        expect_eq(wcb.decStrong() == DecrementResult.ShouldDestroy, true);
        expect_eq(wcb.isAlive(), false);
        expect_eq(wcb.canDeallocate(), false);
        expect_eq(wcb.decWeak(), true);
        expect_eq(wcb.canDeallocate(), true);

        const tensor = new SharedTensor(1024);
        expect_eq(tensor.get_size(), 1024);
        tensor.retain();
        expect_eq(tensor.get_ref_count(), 2);
        tensor.release();
        expect_eq(tensor.get_ref_count(), 1);
    });
}
