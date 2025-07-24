use crate::{ALLOCS_EXECUTION_TIMES, FUNCTIONS_EXECUTION_TIMES};
use aya_ebpf::helpers::bpf_ktime_get_ns;
use aya_network_deep_profiling_common::{Alloc, Function, FunctionCall, FunctionDirection};

pub fn log_function_time(function: Function, direction: FunctionDirection, depth: u32, cpuid: u32) -> Result<(), u32> {
    let time = unsafe { bpf_ktime_get_ns() };
    let function_call = FunctionCall {
        function,
        direction,
        depth,
        cpuid
    };

    FUNCTIONS_EXECUTION_TIMES.insert(&time, &function_call, 0).map_err(|_| 0u32)
}

pub fn log_alloc_time(function: Alloc, direction: FunctionDirection, depth: u32, cpuid: u32) -> Result<(), u32> {
    let time = unsafe { bpf_ktime_get_ns() };
    let function_call = FunctionCall {
        function,
        direction,
        depth,
        cpuid
    };

    ALLOCS_EXECUTION_TIMES.insert(&time, &function_call, 0).map_err(|_| 0u32)
}