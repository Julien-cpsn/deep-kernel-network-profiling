use crate::{FUNCTIONS_EXECUTION_TIMES, MALLOCS_EXECUTION_TIMES};
use aya_ebpf::helpers::bpf_ktime_get_ns;
use aya_network_deep_profiling_common::{Function, FunctionDirection, FunctionWithDirection, Malloc};

pub fn log_function_time(function: Function, direction: FunctionDirection) -> Result<(), u32> {
    let time = unsafe { bpf_ktime_get_ns() };
    let function_with_direction = FunctionWithDirection(function, direction);

    FUNCTIONS_EXECUTION_TIMES.insert(&time, &function_with_direction, 0).map_err(|_| 0u32)
}

pub fn log_malloc_time(function: Malloc, direction: FunctionDirection) -> Result<(), u32> {
    let time = unsafe { bpf_ktime_get_ns() };
    let function_with_direction = FunctionWithDirection(function, direction);

    MALLOCS_EXECUTION_TIMES.insert(&time, &function_with_direction, 0).map_err(|_| 0u32)
}