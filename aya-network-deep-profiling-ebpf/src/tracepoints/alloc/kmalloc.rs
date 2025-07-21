use crate::tracepoints::alloc::utils::should_profile_stack_id;
use crate::{log_time, ALLOCATIONS, TEMP_ALLOCATIONS};
use aya_ebpf::helpers::bpf_ktime_get_ns;
use aya_ebpf::macros::{tracepoint};
use aya_ebpf::programs::{TracePointContext};
use aya_log_ebpf::{trace};
use aya_network_deep_profiling_common::{AllocInfo, AllocType, Function};
use crate::utils::context::get_full_ctx;
use crate::utils::function::register_function;
use crate::utils::log::{log_ctx, LogType};

// cat /sys/kernel/debug/tracing/events/kmem/kmalloc/format
#[tracepoint]
pub fn tracepoint_kmalloc(ctx: TracePointContext) -> u32 {
    try_tracepoint_kmalloc(ctx).unwrap_or(0)
}

fn try_tracepoint_kmalloc(ctx: TracePointContext) -> Result<u32, u32> {
    let fctx = get_full_ctx(ctx)?;

    if !should_profile_stack_id(fctx.cpuid) {
        return Err(0);
    }

    //log_ctx(LogType::Trace, &fctx.ctx, Function::Kmalloc.as_str(), None);
    register_function(&fctx.stack_id, Function::Kmalloc)?;

    /*
    let call_site: u64 = unsafe { fctx.ctx.read_at(8).map_err(|_| 0u32)? };
    trace!(&fctx.ctx, "\tcall site: {:X}", call_site);
    */

    /* ----- */

    let size: u64 = unsafe { fctx.ctx.read_at(32).map_err(|_| 0u32)? };
    let ptr: u64 = unsafe { fctx.ctx.read_at(16).map_err(|_| 0u32)? };

    let time = unsafe { bpf_ktime_get_ns() };
    let alloc_info = AllocInfo {
        size,
        alloc_type: AllocType::Malloc,
        timestamp: time,
        stack_id: fctx.stack_id,
        pid: fctx.pid,
    };

    TEMP_ALLOCATIONS.insert(&ptr, &alloc_info, 0).map_err(|_| 0u32)?;
    ALLOCATIONS.push(&alloc_info, 0).map_err(|_| 0u32)?;

    Ok(0)
}

log_time!(
    bpf_map_kmalloc_node,
    mempool_kmalloc,
    __traceiter_kmalloc,
    __probestub_kmalloc,
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