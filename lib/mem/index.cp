// lib/mem/index.cp
// Unified Cap Memory & Smart Pointer Subsystem (cap:mem)

export {
    MiHeap,
    mi_malloc,
    mi_calloc,
    mi_realloc,
    mi_expand,
    mi_free,
    mi_malloc_small,
    mi_zalloc_small,
    mi_zalloc,
    mi_mallocn,
    mi_reallocn,
    mi_reallocf,
    mi_usable_size,
    mi_good_size,
    mi_collect,
    mi_version,
    mi_stats_reset,
    mi_stats_merge,
    mi_process_init,
    mi_thread_init,
    mi_thread_done,
    mi_malloc_aligned,
    mi_zalloc_aligned,
    mi_calloc_aligned,
    mi_realloc_aligned,
    mi_heap_new,
    mi_heap_delete,
    mi_heap_destroy,
    mi_heap_main,
    mi_heap_contains,
    mi_heap_collect,
    mi_heap_malloc,
    mi_heap_zalloc,
    mi_heap_calloc,
    mi_heap_realloc,
    mi_heap_malloc_aligned,
} from "./_mimalloc.cp";

export {
    Arena,
    get_arena,
    SharedTensor,
    StackFallbackAllocator,
    BumpArena,
    MaybeOwnedBuf,
} from "./allocator.cp";

export {
    TAG_SHIFT,
    ADDR_MASK,
    LOW_TAG_MASK,
    TaggedPointer,
    LowTagPointer,
} from "./tagged_ptr.cp";

export {
    CowBytes,
} from "./cow.cp";

export {
    DecrementResult,
    RefCounter,
    AtomicRefCounter,
    WeakControlBlock,
} from "./rc.cp";
