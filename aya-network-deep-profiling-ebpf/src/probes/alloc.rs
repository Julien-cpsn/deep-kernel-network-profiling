use crate::log_time;

log_time!(
    bpf_map_kmalloc_node,
    mempool_kmalloc,
    kmalloc_size_roundup,
    free_large_kmalloc,
    ___kmalloc_large_node,
    __kmalloc_large_noprof,
    __kmalloc_large_node_noprof,
    __kmalloc_noprof,
    __kmalloc_node_track_caller_noprof,
    __kmalloc_cache_node_noprof,
    __kmalloc_node_noprof,
    __kmalloc_cache_noprof,
    bio_kmalloc,
    devm_kmalloc_match,
    devm_kmalloc_release,
    devm_kmalloc,
    sock_kmalloc,
    kmalloc_reserve,
    kmalloc_fix_flags
);

log_time!(
    kfree,
    kfree_skbmem,
    kfree_skb_list_reason,
    __kfree_skb,
    kfree_skb_partial,
    __napi_kfree_skb,
    dev_kfree_skb_irq_reason,
    dev_kfree_skb_any_reason
);