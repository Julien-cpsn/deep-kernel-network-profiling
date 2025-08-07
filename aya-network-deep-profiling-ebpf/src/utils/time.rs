macro_rules! log_time {
    ($($name:expr, $function_enumeration:expr),*) => {
        paste::paste! {
            $(
                pub fn [<log_ $name:snake:lower _time>](function: aya_network_deep_profiling_common::$function_enumeration, direction: aya_network_deep_profiling_common::FunctionDirection, depth: u32, cpuid: u32) -> Result<(), u32> {
                    let time = unsafe { aya_ebpf::helpers::bpf_ktime_get_ns() };
                    let function_call = aya_network_deep_profiling_common::FunctionCall {
                        function,
                        direction,
                        depth,
                        cpuid
                    };

                    crate::[<$name:snake:upper _FUNCTIONS_EXECUTION_TIMES>].insert(&time, &function_call, 0).map_err(|_| 0u32)
                }
            )*
        }
    };
}

log_time!(
    kernel, KernelFunction,
    user, UserFunction,
    alloc, Alloc
);