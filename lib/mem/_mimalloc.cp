// lib/mem/_mimalloc.cp
// Low-level Mimalloc C-FFI bindings and opaque heap handles (absorbed from bun/src/mimalloc_sys/mimalloc.cp)

export @opaque struct MiHeap {}

@c {
    pub fn mi_malloc(size: usize): *!?;
    pub fn mi_calloc(count: usize, size: usize): *!?;
    pub fn mi_realloc(p: *!?, newsize: usize): *!?;
    pub fn mi_expand(p: *!?, newsize: usize): *!?;
    pub fn mi_free(p: *!?): void;
    pub fn mi_malloc_small(size: usize): *!?;
    pub fn mi_zalloc_small(size: usize): *!?;
    pub fn mi_zalloc(size: usize): *!?;
    pub fn mi_mallocn(count: usize, size: usize): *!?;
    pub fn mi_reallocn(p: *!?, count: usize, size: usize): *!?;
    pub fn mi_reallocf(p: *!?, newsize: usize): *!?;
    pub fn mi_usable_size(p: *!?): usize;
    pub fn mi_good_size(size: usize): usize;
    pub fn mi_collect(force: bool): void;
    pub fn mi_version(): i32;
    pub fn mi_stats_reset(): void;
    pub fn mi_stats_merge(): void;
    pub fn mi_process_init(): void;
    pub fn mi_thread_init(): void;
    pub fn mi_thread_done(): void;
    pub fn mi_malloc_aligned(size: usize, alignment: usize): *!?;
    pub fn mi_zalloc_aligned(size: usize, alignment: usize): *!?;
    pub fn mi_calloc_aligned(count: usize, size: usize, alignment: usize): *!?;
    pub fn mi_realloc_aligned(p: *!?, newsize: usize, alignment: usize): *!?;

    pub fn mi_heap_new(): (*MiHeap!)?;
    pub fn mi_heap_delete(heap: *MiHeap!): void;
    pub fn mi_heap_destroy(heap: *MiHeap!): void;
    pub fn mi_heap_main(): *MiHeap!;
    pub fn mi_heap_contains(heap: *MiHeap!, p: *!?): bool;
    pub fn mi_heap_collect(heap: *MiHeap!, force: bool): void;
    pub fn mi_heap_malloc(heap: *MiHeap!, size: usize): *!?;
    pub fn mi_heap_zalloc(heap: *MiHeap!, size: usize): *!?;
    pub fn mi_heap_calloc(heap: *MiHeap!, count: usize, size: usize): *!?;
    pub fn mi_heap_realloc(heap: *MiHeap!, p: *!?, newsize: usize): *!?;
    pub fn mi_heap_malloc_aligned(heap: *MiHeap!, size: usize, alignment: usize): *!?;
}
