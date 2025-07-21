#[macro_export]
macro_rules! profile_function {
    ($($function:ident),*) => {
        paste::paste! {
            $(
                #[aya_ebpf::macros::kprobe]
                pub fn [<probe_enter_ $function>](ctx: aya_ebpf::programs::ProbeContext) -> u32 {
                    [<probe_try_enter_ $function>](ctx).unwrap_or(0)
                }

                fn [<probe_try_enter_ $function>](ctx: aya_ebpf::programs::ProbeContext) -> Result<u32, u32> {
                    let fctx = crate::utils::context::get_full_ctx(ctx)?;
                    let function = aya_network_deep_profiling_common::Function::$function;
                    let direction = aya_network_deep_profiling_common::FunctionDirection::Entry;

                    //crate::utils::log::log_ctx(crate::utils::log::LogType::Debug, &fctx.ctx, function.as_str(), Some(direction));
                    crate::utils::function::set_function_active(&fctx.cpuid, true)?;
                    crate::utils::function::register_function(&fctx.stack_id, function)?;
                    crate::utils::time::log_function_time(function, direction)?;

                    Ok(0)
                }


                #[aya_ebpf::macros::kretprobe]
                pub fn [<probe_ret_ $function>](ctx: aya_ebpf::programs::RetProbeContext) -> u32 {
                    [<probe_try_ret_ $function>](ctx).unwrap_or(0)
                }

                fn [<probe_try_ret_ $function>](ctx: aya_ebpf::programs::RetProbeContext) -> Result<u32, u32> {
                    let fctx = crate::utils::context::get_full_ctx(ctx)?;
                    let function = aya_network_deep_profiling_common::Function::$function;
                    let direction = aya_network_deep_profiling_common::FunctionDirection::Exit;

                    //crate::utils::log::log_ctx(crate::utils::log::LogType::Debug, &fctx.ctx, function.as_str(), Some(direction));
                    crate::utils::function::set_function_active(&fctx.cpuid, false)?;
                    crate::utils::function::register_function(&fctx.stack_id, function)?;
                    crate::utils::time::log_function_time(function, direction)?;

                    Ok(0)
                }
            )*
        }
    };
}

#[macro_export]
macro_rules! log_time {
    ($($function:ident),*) => {
        paste::paste! {
            $(
                #[aya_ebpf::macros::kprobe]
                pub fn [<probe_enter_ $function>](_ctx: aya_ebpf::programs::ProbeContext) -> u32 {
                    let cpuid = unsafe { aya_ebpf::helpers::bpf_get_smp_processor_id() } as u32;

                    if !crate::tracepoints::alloc::utils::should_profile_stack_id(cpuid) {
                        return 0;
                    }

                    crate::utils::time::log_malloc_time(aya_network_deep_profiling_common::Malloc::$function, aya_network_deep_profiling_common::FunctionDirection::Entry).ok();
                    0
                }

                #[aya_ebpf::macros::kretprobe]
                pub fn [<probe_ret_ $function>](_ctx: aya_ebpf::programs::RetProbeContext) -> u32 {
                    let cpuid = unsafe { aya_ebpf::helpers::bpf_get_smp_processor_id() } as u32;

                    if !crate::tracepoints::alloc::utils::should_profile_stack_id(cpuid) {
                        return 0;
                    }

                    crate::utils::time::log_malloc_time(aya_network_deep_profiling_common::Malloc::$function, aya_network_deep_profiling_common::FunctionDirection::Exit).ok();
                    0
                }
            )*
        }
    };
}