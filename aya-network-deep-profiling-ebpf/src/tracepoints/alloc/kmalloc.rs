use crate::alloc;

// cat /sys/kernel/debug/tracing/events/kmem/kmalloc/format
alloc!(
    kmalloc,
    kmalloc
);

alloc!(
    kmem_cache,
    kmem_cache_alloc
);