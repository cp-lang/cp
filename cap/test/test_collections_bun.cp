// cap/test/test_collections_bun.cp
// Comprehensive end-to-end test suite for Bun-absorbed collections in cap:collections (lib/collections/)

import { test, expect_eq } from "cap:test";
import {
    BoundedArray,
    BabyList,
    OffsetByteList,
    LinearFifo,
    BitSet64,
    HiveArray,
    FastStringMap
} from "cap:collections";

fn main(): void {
    test("1. BoundedArray<T, CAP> zero-heap stack-allocated bounded array", () => {
        var arr = BoundedArray<i32, 8>.init();
        expect_eq(arr.capacity() as i64, 8);
        expect_eq(arr.isEmpty(), true);

        expect_eq(arr.append(10), true);
        expect_eq(arr.append(20), true);
        expect_eq(arr.append(30), true);
        expect_eq(arr.append(40), true);
        expect_eq(arr.len as i64, 4);
        expect_eq(arr.unusedCapacity() as i64, 4);
        expect_eq(arr.get(1) as i64, 20);

        arr.set(1, 25);
        expect_eq(arr.get(1) as i64, 25);

        const removed = arr.orderedRemove(1);
        expect_eq(removed as i64, 25);
        expect_eq(arr.len as i64, 3);
        expect_eq(arr.get(1) as i64, 30);

        const swapped = arr.swapRemove(0);
        expect_eq(swapped as i64, 10);
        expect_eq(arr.get(0) as i64, 40);

        if (popped := arr.pop()) {
            expect_eq(popped as i64, 30);
        } else {
            expect_eq(1, 0);
        }

        arr.clear();
        expect_eq(arr.isEmpty(), true);
    });

    test("2. BabyList<T> & OffsetByteList compact u32 dynamic list", () => {
        let list = BabyList<i64>.initCapacity(4);
        expect_eq(list.isEmpty(), true);
        expect_eq(list.capacity() as i64, 4);

        list.append(100);
        list.append(200);
        list.append(300);
        list.append(400);
        list.append(500);
        expect_eq(list.count() as i64, 5);
        expect_eq(list.capacity() as i64, 8);
        expect_eq(list.at(4), 500);

        if (last_val := list.pop()) {
            expect_eq(last_val, 500);
        } else {
            expect_eq(1, 0);
        }
        expect_eq(list.count() as i64, 4);
        list.release();

        let obl = OffsetByteList.init(16);
        obl.write("HTTP/1.1 ");
        obl.write("200 OK");
        expect_eq(obl.remainingLen() as i64, 15);

        const c1 = obl.consume(9);
        expect_eq(c1 as i64, 9);
        expect_eq(obl.remainingLen() as i64, 6);

        const c2 = obl.consume(6);
        expect_eq(c2 as i64, 6);
        expect_eq(obl.remainingLen() as i64, 0);
        expect_eq(obl.head as i64, 0);
        obl.release();
    });

    test("3. LinearFifo<T> power-of-two ring buffer FIFO", () => {
        let fifo = LinearFifo<i32>.init(4);
        expect_eq(fifo.isEmpty(), true);

        fifo.writeItem(11);
        fifo.writeItem(22);
        fifo.writeItem(33);
        expect_eq(fifo.readableLength() as i64, 3);

        if (p0 := fifo.peekItem(0)) {
            expect_eq(p0 as i64, 11);
        } else {
            expect_eq(1, 0);
        }

        if (r0 := fifo.readItem()) {
            expect_eq(r0 as i64, 11);
        } else {
            expect_eq(1, 0);
        }
        expect_eq(fifo.readableLength() as i64, 2);

        fifo.discard(1);
        expect_eq(fifo.readableLength() as i64, 1);

        if (r2 := fifo.readItem()) {
            expect_eq(r2 as i64, 33);
        } else {
            expect_eq(1, 0);
        }
        expect_eq(fifo.isEmpty(), true);
        fifo.release();
    });

    test("4. BitSet64 & HiveArray<T, CAP> O(1) hardware @ctz/@popCount slot pool", () => {
        var bs = BitSet64.initEmpty();
        expect_eq(bs.count() as i64, 0);
        bs.set(3);
        bs.set(7);
        expect_eq(bs.count() as i64, 2);
        expect_eq(bs.isSet(3), true);
        expect_eq(bs.isSet(4), false);

        if (first_set := bs.findFirstSet(64)) {
            expect_eq(first_set as i64, 3);
        } else {
            expect_eq(1, 0);
        }

        var hive = HiveArray<i64, 8>.init();
        expect_eq(hive.capacity() as i64, 8);
        expect_eq(hive.isEmpty(), true);

        const s0 = hive.acquire(1001) ?? 999;
        const s1 = hive.acquire(1002) ?? 999;
        const s2 = hive.acquire(1003) ?? 999;
        expect_eq(s0 as i64, 0);
        expect_eq(s1 as i64, 1);
        expect_eq(s2 as i64, 2);
        expect_eq(hive.usedCount() as i64, 3);

        expect_eq(hive.releaseSlot(1), true);
        expect_eq(hive.usedCount() as i64, 2);

        // Next acquire reuses slot 1 in O(1) via @ctz
        const s_reused = hive.acquire(2002) ?? 999;
        expect_eq(s_reused as i64, 1);
        if (val := hive.get(1)) {
            expect_eq(val, 2002);
        } else {
            expect_eq(1, 0);
        }
    });

    test("5. FastStringMap<V, CAP> length-filtered exact & case-insensitive lookup", () => {
        var smap = FastStringMap<i32, 16>.init();
        expect_eq(smap.put("Content-Type", 1), true);
        expect_eq(smap.put("Content-Length", 2), true);
        expect_eq(smap.put("Host", 3), true);
        expect_eq(smap.count() as i64, 3);

        expect_eq(smap.has("Host"), true);
        expect_eq(smap.has("Accept"), false);

        if (v1 := smap.get("Content-Length")) {
            expect_eq(v1 as i64, 2);
        } else {
            expect_eq(1, 0);
        }

        if (v_ci := smap.getCaseInsensitive("content-type")) {
            expect_eq(v_ci as i64, 1);
        } else {
            expect_eq(1, 0);
        }
    });
}
