// lib/event_loop/loop.cp
// Unified EventLoop integrating intrusive timers, deferred microtasks, cross-thread tasks, and I/O polling

const std = @import("std");
import { Timer, TimerHeap, TimerCallback } from "./timer";
import { ConcurrentTask, ConcurrentQueue, TaskFn } from "./concurrent_task";
import { DeferredTaskQueue, DeferredFn } from "./deferred_task";
import { PollEngine, PollEvents, PollCallback } from "./file_poll";

export class EventLoop {
    timers: *TimerHeap = undefined;
    deferred: *DeferredTaskQueue = undefined;
    poll_engine: *PollEngine = undefined;
    concurrent: *ConcurrentQueue = undefined;
    next_timer_id: u64 = 1;
    running: bool = false;
    stopped: bool = false;

    pub static fn init(): *EventLoop {
        let el = std.heap.page_allocator.create(EventLoop) catch unreachable;
        el.timers = TimerHeap.init();
        el.deferred = DeferredTaskQueue.init();
        el.poll_engine = PollEngine.init();
        el.concurrent = ConcurrentQueue.init();
        el.next_timer_id = 1;
        el.running = false;
        el.stopped = false;
        return el;
    }

    pub fn setTimeout(self: *EventLoop, cb: TimerCallback, delay_ms: i64, ctx: ?*anyopaque): *Timer {
        const id = self.next_timer_id;
        self.next_timer_id += 1;
        const timer = Timer.create(id, delay_ms, 0, cb, ctx);
        self.timers.push(timer);
        return timer;
    }

    pub fn setInterval(self: *EventLoop, cb: TimerCallback, interval_ms: i64, ctx: ?*anyopaque): *Timer {
        const id = self.next_timer_id;
        self.next_timer_id += 1;
        const timer = Timer.create(id, interval_ms, interval_ms, cb, ctx);
        self.timers.push(timer);
        return timer;
    }

    pub fn clearTimeout(self: *EventLoop, timer: *Timer): bool {
        return self.timers.remove(timer);
    }

    pub fn clearInterval(self: *EventLoop, timer: *Timer): bool {
        return self.timers.remove(timer);
    }

    pub fn deferTask(self: *EventLoop, task: DeferredFn, ctx: ?*anyopaque): bool {
        return self.deferred.postTask(ctx, task);
    }

    pub fn postTask(self: *EventLoop, cb: TaskFn, ctx: ?*anyopaque) {
        const task = ConcurrentTask.create(cb, ctx, true);
        self.concurrent.push(task);
    }

    pub fn watch(self: *EventLoop, fd: i32, events: u32, cb: PollCallback, ctx: ?*anyopaque): bool {
        return self.poll_engine.register(fd, events, cb, ctx);
    }

    pub fn unwatch(self: *EventLoop, fd: i32): bool {
        return self.poll_engine.unregister(fd);
    }

    pub fn hasActiveHandles(self: *EventLoop): bool {
        return self.timers.count() > 0 or self.poll_engine.count() > 0 or self.concurrent.len() > 0 or self.deferred.count() > 0;
    }

    pub fn tick(self: *EventLoop, timeout_override: ?i32): usize {
        var events_processed: usize = 0;

        // 1. Drain concurrent cross-thread tasks
        events_processed += self.concurrent.drain(64);

        // 2. Run deferred microtasks before sleeping/polling
        events_processed += self.deferred.run();

        // 3. Compute timeout until next timer
        const now = TimerHeap.nowMs();
        var timeout_ms: i32 = 0;
        if (timeout_override) |t| {
            timeout_ms = t;
        } else {
            const next_timeout = self.timers.getTimeoutMs(now);
            if (next_timeout) |t| {
                timeout_ms = t;
            } else if (self.poll_engine.count() > 0) {
                timeout_ms = 100; // default poll interval when waiting for I/O
            } else {
                timeout_ms = 0;
            }
        }

        // 4. Poll I/O file descriptors
        if (self.poll_engine.count() > 0) {
            events_processed += self.poll_engine.poll(timeout_ms);
        } else if (timeout_ms > 0) {
            // Sleep for timeout if no I/O descriptors
            TimerHeap.sleepMs(@as(u64, @intCast(timeout_ms)));
        }

        // 5. Drain expired timers
        const now_after = TimerHeap.nowMs();
        events_processed += self.timers.drainExpired(now_after, 64);

        // 6. Run any follow-up microtasks triggered by timers or I/O
        events_processed += self.deferred.run();

        return events_processed;
    }

    pub fn run(self: *EventLoop) {
        self.running = true;
        self.stopped = false;
        while (self.hasActiveHandles() and !self.stopped) {
            _ = self.tick(null);
        }
        self.running = false;
    }

    pub fn stop(self: *EventLoop) {
        self.stopped = true;
    }
}
