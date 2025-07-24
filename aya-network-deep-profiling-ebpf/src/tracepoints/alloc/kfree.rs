use crate::free;

free!(
    kmalloc,
    kfree
);

free!(
    kmem_cache,
    kmem_cache_free
);