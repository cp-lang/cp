// cap/test/test_event_loop.cp
// End-to-end regression test for Unified Event Loop Subsystem (cap:event_loop)

import { test, expect_eq } from "cap:test";
const std = @import("std");

import {
    Timer,
    TimerHeap,
    ConcurrentTask,
    ConcurrentQueue,
    DeferredTaskQueue,
    PollEngine,
    PollEvents,
    EventLoop
} from "../../lib/event_loop/index.cp";

class CounterState {
    pub count: i32 = 0;

    pub static fn create(): *CounterState {
        let p = std.heap.page_allocator.create(CounterState) catch unreachable;
        p.count = 0;
        return p;
    }
}

fn timer_callback(ctx: ?*anyopaque) void {
    if (ctx) |p| {
        let state: *CounterState = @ptrCast(@alignCast(p));
        state.count += 1;
    }
}

fn deferred_callback(ctx: ?*anyopaque) void {
    if (ctx) |p| {
        let state: *CounterState = @ptrCast(@alignCast(p));
        state.count += 10;
    }
}

fn pipe_poll_callback(fd: i32, events: u32, ctx: ?*anyopaque) void {
    _ = fd;
    _ = events;
    if (ctx) |p| {
        let state: *CounterState = @ptrCast(@alignCast(p));
        state.count += 100;
    }
}

fn testTimerHeap(): !void {
    let heap = TimerHeap.init();
    expect_eq(heap.count(), 0);
    expect_eq(heap.isEmpty(), true);

    const state = CounterState.create();
    const state_ptr: *anyopaque = @ptrCast(state);

    // Push timers out of order: 50ms, 10ms, 30ms
    const t50 = Timer.create(1, 50, 0, timer_callback, state_ptr);
    const t10 = Timer.create(2, 10, 0, timer_callback, state_ptr);
    const t30 = Timer.create(3, 30, 0, timer_callback, state_ptr);

    heap.push(t50);
    heap.push(t10);
    heap.push(t30);

    expect_eq(heap.count(), 3);
    expect_eq(heap.isEmpty(), false);

    // Min-heap top should be t10 (lowest fire_at_ms)
    const top = heap.peek().?;
    expect_eq(top.id, 2);

    // Cancel t30 before it fires
    const removed = heap.remove(t30);
    expect_eq(removed, true);
    expect_eq(heap.count(), 2);

    // Sleep 15ms so t10 expires but t50 does not
    TimerHeap.sleepMs(15);
    const now = TimerHeap.nowMs();
    const drained = heap.drainExpired(now, 10);
    expect_eq(drained, 1);
    expect_eq(state.count, 1); // t10 fired!
    expect_eq(heap.count(), 1);

    // Sleep another 45ms so t50 expires
    TimerHeap.sleepMs(45);
    const now2 = TimerHeap.nowMs();
    const drained2 = heap.drainExpired(now2, 10);
    expect_eq(drained2, 1);
    expect_eq(state.count, 2); // t50 fired!
    expect_eq(heap.count(), 0);
}

fn testConcurrentQueue(): !void {
    let q = ConcurrentQueue.init();
    expect_eq(q.len(), 0);

    const state = CounterState.create();
    const state_ptr: *anyopaque = @ptrCast(state);

    const task1 = ConcurrentTask.create(timer_callback, state_ptr, true);
    const task2 = ConcurrentTask.create(timer_callback, state_ptr, true);
    const task3 = ConcurrentTask.create(timer_callback, state_ptr, true);

    q.push(task1);
    q.push(task2);
    q.push(task3);

    expect_eq(q.len(), 3);

    const processed = q.drain(10);
    expect_eq(processed, 3);
    expect_eq(state.count, 3);
    expect_eq(q.len(), 0);
}

fn testDeferredQueue(): !void {
    let dq = DeferredTaskQueue.init();
    expect_eq(dq.count(), 0);

    const state = CounterState.create();
    const state_ptr: *anyopaque = @ptrCast(state);

    // Post tasks, deduplicating on the same ctx pointer
    const dup1 = dq.postTask(state_ptr, deferred_callback);
    expect_eq(dup1, false); // first time: not found existing
    expect_eq(dq.count(), 1);

    const dup2 = dq.postTask(state_ptr, deferred_callback);
    expect_eq(dup2, true); // second time: found existing and replaced
    expect_eq(dq.count(), 1); // count remains 1 due to deduplication

    const executed = dq.run();
    expect_eq(executed, 1);
    expect_eq(state.count, 10);
    expect_eq(dq.count(), 0);
}

fn testPollEngine(): !void {
    let pe = PollEngine.init();
    expect_eq(pe.count(), 0);

    var r_fd: i32 = 0;
    var w_fd: i32 = 0;
    __zig__ {
        var fds: [2]i32 = undefined;
        _ = std.os.linux.pipe(&fds);
        r_fd = fds[0];
        w_fd = fds[1];
    }
    defer _ = std.os.linux.close(r_fd);
    defer _ = std.os.linux.close(w_fd);

    const state = CounterState.create();
    const state_ptr: *anyopaque = @ptrCast(state);

    // Register read end
    const reg_ok = pe.register(r_fd, PollEvents.Readable, pipe_poll_callback, state_ptr);
    expect_eq(reg_ok, true);
    expect_eq(pe.count(), 1);

    // Poll with 0 timeout when pipe is empty -> should not trigger
    const polled_empty = pe.poll(0);
    expect_eq(polled_empty, 0);
    expect_eq(state.count, 0);

    // Write a byte to the pipe
    const data = "x";
    _ = std.os.linux.write(w_fd, data.ptr, data.len);

    // Poll -> should be readable and invoke callback
    const polled_ready = pe.poll(50);
    expect_eq(polled_ready, 1);
    expect_eq(state.count, 100);

    // Unregister
    const unreg = pe.unregister(r_fd);
    expect_eq(unreg, true);
    expect_eq(pe.count(), 0);
}

fn testEventLoopIntegration(): !void {
    let loop = EventLoop.init();
    expect_eq(loop.hasActiveHandles(), false);

    const state = CounterState.create();
    const state_ptr: *anyopaque = @ptrCast(state);

    // Schedule a deferred task and a short timer
    _ = loop.deferTask(deferred_callback, state_ptr);
    const t = loop.setTimeout(timer_callback, 10, state_ptr);
    _ = t;

    expect_eq(loop.hasActiveHandles(), true);

    // First tick with 0 timeout: should run deferred microtask (+10) but timer not yet expired
    _ = loop.tick(0);
    expect_eq(state.count, 10);

    // Sleep 15ms and tick again: timer should fire (+1)
    TimerHeap.sleepMs(15);
    _ = loop.tick(0);
    expect_eq(state.count, 11);

    // All tasks completed
    expect_eq(loop.hasActiveHandles(), false);
}

fn main(): void {
    test("Intrusive Binary Min-Heap Timer Scheduling", () => {
        testTimerHeap()?;
    });

    test("16-byte Lock-Free Concurrent Task Queue", () => {
        testConcurrentQueue()?;
    });

    test("Deferred Microtask Queue and Deduplication", () => {
        testDeferredQueue()?;
    });

    test("I/O File Descriptor Polling Engine", () => {
        testPollEngine()?;
    });

    test("Unified EventLoop Integration Lifecycle", () => {
        testEventLoopIntegration()?;
    });
}
