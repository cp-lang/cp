import { test, expect_eq } from "cap:test";
import { Futex, Mutex, Condition, WaitGroup, Channel, MpmcQueue, ConcurrentMap, ArcSlice, Wyhash } from "cap:sync";
const std = @import("std");

fn test_wyhash(): !void {
    const h1 = Wyhash.hash(0, "hello");
    const h2 = Wyhash.hash(0, "hello");
    const h3 = Wyhash.hash(0, "world");
    expect_eq(h1, h2);
    expect_eq(h1 != h3, true);
}

fn test_futex(): !void {
    var val: std.atomic.Value(u32) = .{ .raw = 100 };
    Futex.wake(&val, 1);
    expect_eq(val.raw, 100);
}

fn test_mutex(): !void {
    var mu: Mutex = .{};
    defer mu.deinit();

    expect_eq(mu.tryLock(), true);
    mu.unlock();

    mu.lock();
    mu.unlock();
}

fn test_wait_group(): !void {
    var wg: WaitGroup = .{};
    defer wg.deinit();

    wg.add(2);
    wg.done();
    wg.done();
    wg.wait();
}

fn test_mpmc(): !void {
    var q = MpmcQueue(i32, 4).init();

    expect_eq(q.tryPush(10), true);
    expect_eq(q.tryPush(20), true);
    expect_eq(q.tryPush(30), true);
    expect_eq(q.tryPush(40), true);
    expect_eq(q.tryPush(50), false); // full

    expect_eq(q.tryPop(), 10);
    expect_eq(q.tryPop(), 20);
    expect_eq(q.tryPop(), 30);
    expect_eq(q.tryPop(), 40);
    expect_eq(q.tryPop(), null); // empty
}

fn test_channel(): !void {
    const allocator = std.heap.page_allocator;
    var ch = try Channel(i32).init(allocator, 2);
    defer ch.deinit();

    try ch.send(100);
    try ch.send(200);

    expect_eq(ch.len(), 2);
    expect_eq(ch.tryRecv(), 100);
    expect_eq(ch.tryRecv(), 200);
    expect_eq(ch.tryRecv(), null);
}

fn test_concurrent_map(): !void {
    const allocator = std.heap.page_allocator;
    var map = ConcurrentMap(i32, i32).init(allocator);
    defer map.deinit();

    try map.put(1, 10);
    try map.put(2, 20);
    expect_eq(map.get(1), 10);
    expect_eq(map.get(2), 20);
    expect_eq(map.get(3), null);

    expect_eq(map.contains(1), true);
    expect_eq(map.remove(1), true);
    expect_eq(map.contains(1), false);
}

fn test_arc_slice(): !void {
    const allocator = std.heap.page_allocator;
    const data = "hello cp";
    var arc = try ArcSlice(u8).init(allocator, data);

    expect_eq(arc.len(), 8);
    expect_eq(arc.refCount(), 1);

    var cloned = arc.clone();
    expect_eq(arc.refCount(), 2);
    expect_eq(cloned.refCount(), 2);
    cloned.release();

    expect_eq(arc.refCount(), 1);
    expect_eq(arc.get(0), 'h');
    expect_eq(arc.get(1), 'e');
}

fn main(): void {
    test("Wyhash deterministic 64-bit hashing", () => {
        try test_wyhash();
    });

    test("Futex atomic operations", () => {
        try test_futex();
    });

    test("Mutex lock and unlock", () => {
        try test_mutex();
    });

    test("Condition signal and wait group", () => {
        try test_wait_group();
    });

    test("MpmcQueue bounded ring buffer", () => {
        try test_mpmc();
    });

    test("Channel bounded send and receive", () => {
        try test_channel();
    });

    test("ConcurrentMap thread-safe operations", () => {
        try test_concurrent_map();
    });

    test("ArcSlice atomic reference counted slice", () => {
        try test_arc_slice();
    });
}
