use aya_ebpf::helpers::bpf_ktime_get_ns;
use crate::tracepoints::alloc::utils::should_profile_stack_id;
use crate::{ALLOCATIONS, TEMP_ALLOCATIONS};
use aya_ebpf::macros::tracepoint;
use aya_ebpf::programs::TracePointContext;
use aya_log_ebpf::trace;
use aya_network_deep_profiling_common::{AllocInfo, AllocType, Function};
use crate::utils::context::get_full_ctx;
use crate::utils::function::register_function;
use crate::utils::log::{log_ctx, LogType};

#[tracepoint]
pub fn tracepoint_kfree(ctx: TracePointContext) -> u32 {
    try_tracepoint_kfree(ctx).unwrap_or(0)
}

fn try_tracepoint_kfree(ctx: TracePointContext) -> Result<u32, u32> {
    let fctx = get_full_ctx(ctx)?;

    if !should_profile_stack_id(fctx.cpuid) {
        return Err(0);
    }
    
    //log_ctx(LogType::Trace, &fctx.ctx, Function::Kfree.as_str(), None);
    register_function(&fctx.stack_id, Function::Kfree)?;

    /*
    let call_site: u64 = unsafe { fctx.ctx.read_at(8).map_err(|_| 0u32)? };
    trace!(&fctx.ctx, "\tcall site: {:X}", call_site);
    */
    
    /* ----- */

    let ptr: u64 = unsafe { fctx.ctx.read_at(16).map_err(|_| 0u32)? };
    let alloc_info = unsafe { *TEMP_ALLOCATIONS.get(&ptr).ok_or(0u32)? };
    let time = unsafe { bpf_ktime_get_ns() };

    let alloc_info = AllocInfo {
        size: alloc_info.size,
        alloc_type: AllocType::Free,
        timestamp: time,
        stack_id: alloc_info.stack_id,
        pid: alloc_info.pid,
    };

    TEMP_ALLOCATIONS.remove(&ptr).map_err(|_| 0u32)?;
    ALLOCATIONS.push(&alloc_info, 0).map_err(|_| 0u32)?;

    Ok(0)
}