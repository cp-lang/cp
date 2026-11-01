import { test, expect_eq } from "cap:test";
import { print } from "cap:io";
import { VecDeque, BinaryHeap, BTreeMap, BTreeSet, HashMap, HashSet } from "cap:collections";

fn mutate_deque(*q: VecDeque, val: i32): void {
    q.push_back(val);
}

fn query_deque(&q: VecDeque): usize {
    return q.len();
}

fn mutate_heap(*h: BinaryHeap, val: i32): void {
    h.push(val);
}

fn mutate_btree(*b: BTreeMap, k: string, v: i32): void {
    b.insert(k, v);
}

fn main(): void {
    print("=== [Test 1] VecDeque Double-Ended Ring Buffer ===");
    let mut q = new VecDeque();
    expect_eq(q.is_empty(), true);

    q.push_back(10);
    q.push_back(20);
    q.push_front(5);
    // Queue is now: [5, 10, 20]
    expect_eq(q.len(), 3);
    expect_eq(q.is_empty(), false);

    let pf = q.peek_front();
    let pb = q.peek_back();
    expect_eq(pf, 5);
    expect_eq(pb, 20);

    let front_val = q.pop_front();
    expect_eq(front_val, 5);
    let back_val = q.pop_back();
    expect_eq(back_val, 20);
    expect_eq(q.len(), 1);

    // Test mutable borrow and read-only borrow functions
    mutate_deque(*q, 99);
    expect_eq(query_deque(&q), 2);

    print("VecDeque basic operations verified successfully!");

    print("=== [Test 2] VecDeque Dual-Track Stream Iterator for...of ===");
    let mut stream_q = new VecDeque();
    stream_q.push_back(1);
    stream_q.push_back(2);
    stream_q.push_back(3);
    let mut sum_q = 0;
    for item of stream_q {
        sum_q = sum_q + item;
    }
    expect_eq(sum_q, 6);
    expect_eq(stream_q.is_empty(), true);
    print("VecDeque for...of stream iteration verified!");

    print("=== [Test 3] BinaryHeap Max-Heap Priority Queue ===");
    let mut heap = new BinaryHeap();
    expect_eq(heap.is_empty(), true);

    mutate_heap(*heap, 15);
    heap.push(42);
    heap.push(8);
    heap.push(27);
    expect_eq(heap.len(), 4);
    expect_eq(heap.peek(), 42);

    // Pops should yield items in descending priority order
    let p1 = heap.pop();
    let p2 = heap.pop();
    let p3 = heap.pop();
    let p4 = heap.pop();
    expect_eq(p1, 42);
    expect_eq(p2, 27);
    expect_eq(p3, 15);
    expect_eq(p4, 8);
    expect_eq(heap.is_empty(), true);
    print("BinaryHeap max-heap operations verified!");

    print("=== [Test 4] BinaryHeap Priority Stream Iterator ===");
    let mut sort_heap = new BinaryHeap();
    sort_heap.push(10);
    sort_heap.push(50);
    sort_heap.push(30);
    let mut heap_popped_first = 0;
    for v of sort_heap {
        if (heap_popped_first == 0) {
            heap_popped_first = v.to_int();
        }
    }
    expect_eq(heap_popped_first, 50);
    expect_eq(sort_heap.is_empty(), true);
    print("BinaryHeap priority iteration verified!");

    print("=== [Test 5] BTreeMap Ordered Key-Value Map ===");
    let mut btree = new BTreeMap();
    expect_eq(btree.is_empty(), true);

    btree.insert("banana", 20);
    mutate_btree(*btree, "apple", 10);
    btree.insert("cherry", 30);
    expect_eq(btree.len(), 3);

    expect_eq(btree.get("apple"), 10);
    expect_eq(btree.get("banana"), 20);
    expect_eq(btree.get("cherry"), 30);
    expect_eq(btree.has("banana"), true);
    expect_eq(btree.has("durian"), false);

    expect_eq(btree.min_key(), "apple");
    expect_eq(btree.max_key(), "cherry");

    let keys = btree.keys();
    expect_eq(keys.len, 3);
    expect_eq(keys[0], "apple");
    expect_eq(keys[1], "banana");
    expect_eq(keys[2], "cherry");

    let removed = btree.remove("banana");
    expect_eq(removed, 20);
    expect_eq(btree.len(), 2);
    expect_eq(btree.has("banana"), false);
    print("BTreeMap sorted key-value operations verified!");

    print("=== [Test 6] BTreeSet Ordered Set ===");
    let mut bset = new BTreeSet();
    bset.add(30);
    bset.add(10);
    bset.add(20);
    bset.add(10); // Duplicate, should not increase size
    expect_eq(bset.len(), 3);

    expect_eq(bset.min(), 10);
    expect_eq(bset.max(), 30);
    expect_eq(bset.has(20), true);
    expect_eq(bset.has(99), false);

    let vals = bset.values();
    expect_eq(vals.len, 3);
    expect_eq(vals[0], 10);
    expect_eq(vals[1], 20);
    expect_eq(vals[2], 30);

    let rem_res = bset.remove(20);
    expect_eq(rem_res, true);
    expect_eq(bset.len(), 2);
    expect_eq(bset.has(20), false);
    print("BTreeSet sorted set operations verified!");

    print("=== [Test 7] HashMap & HashSet Parity Aliases ===");
    let mut hm = new HashMap();
    hm.set("lang", "CP");
    expect_eq(hm.get("lang"), "CP");
    expect_eq(hm.size, 1);

    let mut hs = new HashSet();
    hs.add(100);
    expect_eq(hs.has(100), true);
    expect_eq(hs.size, 1);
    print("HashMap & HashSet aliases verified!");

    print("🎉 All Standard Collections Matrix tests passed 100%!");
}
