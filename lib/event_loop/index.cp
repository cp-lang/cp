// lib/event_loop/index.cp
// Unified Event Loop Subsystem (cap:event_loop)
// High-performance monotonic timer heap, 16-byte lock-free tasks, deferred microtasks & I/O poller

export {
    Timer,
    TimerHeap,
    TimerCallback,
} from "./timer";

export {
    ConcurrentTask,
    ConcurrentQueue,
    TaskFn,
} from "./concurrent_task";

export {
    DeferredTaskQueue,
    DeferredEntry,
    DeferredFn,
} from "./deferred_task";

export {
    PollEngine,
    PollEvents,
    PollCallback,
    FilePollRegistration,
} from "./file_poll";

export {
    EventLoop,
} from "./loop";
