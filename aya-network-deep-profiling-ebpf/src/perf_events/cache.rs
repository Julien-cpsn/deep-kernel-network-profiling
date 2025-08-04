use crate::utils::function::should_profile_stack_id;
//use crate::CACHE_MISSES;
use aya_ebpf::bindings::bpf_perf_event_value;
use aya_ebpf::helpers::{bpf_get_smp_processor_id, bpf_ktime_get_ns};
use aya_ebpf::macros::perf_event;
use aya_ebpf::programs::PerfEventContext;
use aya_ebpf::EbpfContext;

// NOT USED
/*
#[perf_event]
pub fn perf_l1d_misses(ctx: PerfEventContext) -> u32 {
    match try_perf_l1d_misses(ctx) {
        Ok(ret) => ret,
        Err(ret) => {
            unsafe {
                aya_ebpf::bpf_printk!(b"Error in perf_l1d_misses");
            }
            ret
        }
    }
}

fn try_perf_l1d_misses(ctx: PerfEventContext) -> Result<u32, u32> {
    let event_data = ctx.as_ptr() as *const bpf_perf_event_value;
    let value = unsafe { (*event_data).counter };
    let cpuid = unsafe { bpf_get_smp_processor_id() };

    if !should_profile_stack_id(cpuid) {
        return Err(0);
    }

    let time = unsafe { bpf_ktime_get_ns() };
    CACHE_MISSES.insert(&time, &value, 0).map_err(|_| 0u32)?;

    Ok(0)
}*/