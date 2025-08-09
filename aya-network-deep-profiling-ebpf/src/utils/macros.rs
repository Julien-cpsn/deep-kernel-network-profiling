#[macro_export]
macro_rules! profile_function {
    (
        $function_type:expr,
        $probe_type:expr,
        $($function:ident),*$(,)?
    ) => {
        paste::paste! {
            $(
                #[aya_ebpf::macros::[<$probe_type:lower probe>]]
                pub fn [<probe_enter_ $function>](_ctx: aya_ebpf::programs::ProbeContext) -> u32 {
                    match [<probe_try_enter_ $function>]() {
                        Ok(ret) => ret,
                        Err(ret) => {
                            unsafe {
                                aya_ebpf::bpf_printk!(b"Error in $function");
                            }
                            ret
                        },
                    }
                }

                fn [<probe_try_enter_ $function>]() -> Result<u32, u32> {
                    let cpuid = unsafe { aya_ebpf::helpers::bpf_get_smp_processor_id() } as u32;
                    /*
                    let stack_id = match unsafe { crate::STACK_TRACES.get_stackid(&ctx, 0) } {
                        Ok(stack_id) => stack_id,
                        _ => return Err(0),
                    };*/

                    let function = aya_network_deep_profiling_common::[<$function_type:camel Function>]::$function;
                    let direction = aya_network_deep_profiling_common::FunctionDirection::Entry;

                    //crate::utils::log::log_ctx(crate::utils::log::LogType::Debug, &fctx.ctx, function.as_str(), Some(direction));
                    let depth = crate::utils::function::increment_depth(&cpuid)?;
                    crate::utils::time::[<log_ $function_type:snake:lower _time>](function, direction, depth, cpuid)?;
                    crate::utils::function::set_function_active(&cpuid, true)?;
                    //crate::utils::function::register_function(&stack_id, function.as_id())?;

                    Ok(0)
                }


                #[aya_ebpf::macros::[<$probe_type:lower retprobe>]]
                pub fn [<probe_ret_ $function>](_ctx: aya_ebpf::programs::RetProbeContext) -> u32 {
                    match [<probe_try_ret_ $function>]() {
                        Ok(ret) => ret,
                        Err(ret) => {
                            unsafe {
                                aya_ebpf::bpf_printk!(b"Error in $function");
                            }
                            ret
                        },
                    }
                }

                fn [<probe_try_ret_ $function>]() -> Result<u32, u32> {
                    let cpuid = unsafe { aya_ebpf::helpers::bpf_get_smp_processor_id() } as u32;

                    let function = aya_network_deep_profiling_common::[<$function_type:camel Function>]::$function;
                    let direction = aya_network_deep_profiling_common::FunctionDirection::Exit;

                    //crate::utils::log::log_ctx(crate::utils::log::LogType::Debug, &fctx.ctx, function.as_str(), Some(direction));
                    let depth = crate::utils::function::decrement_depth(&cpuid)?;
                    crate::utils::time::[<log_ $function_type:snake:lower _time>](function, direction, depth, cpuid)?;
                    crate::utils::function::set_function_active(&cpuid, false)?;

                    Ok(0)
                }
            )*
        }
    };
}

#[macro_export]
macro_rules! alloc {
    ($alloc_type:ident, $($function:ident),*) => {
        paste::paste! {
            $(
                #[aya_ebpf::macros::tracepoint]
                pub fn [<tracepoint_ $function>](ctx: aya_ebpf::programs::TracePointContext) -> u32 {
                    match [<try_tracepoint_ $function>](ctx) {
                        Ok(ret) => ret,
                        Err(ret) => {
                            unsafe {
                                aya_ebpf::bpf_printk!(b"Error in $function");
                            }
                            ret
                        },
                    }
                }

                fn [<try_tracepoint_ $function>](ctx: aya_ebpf::programs::TracePointContext) -> Result<u32, u32> {
                    let fctx = crate::utils::context::get_full_ctx(ctx)?;

                    if !crate::utils::function::should_profile_stack_id(fctx.cpuid) {
                        return Err(0);
                    }

                    let ptr_ptr: *const u64 = unsafe { fctx.ctx.read_at(16).map_err(|_| 0u32)? };
                    let ptr: u64 = unsafe { aya_ebpf::helpers::bpf_probe_read_kernel(ptr_ptr).map_err(|_| 0u32)? };
                    let size: u64 = unsafe { fctx.ctx.read_at(32).map_err(|_| 0u32)? };

                    let time = unsafe { aya_ebpf::helpers::bpf_ktime_get_ns() };
                    let alloc_info = aya_network_deep_profiling_common::AllocInfo {
                        alloc_type: aya_network_deep_profiling_common::AllocType::$alloc_type,
                        alloc_direction: aya_network_deep_profiling_common::AllocDirection::Alloc,
                        size,
                        timestamp: time,
                        stack_id: fctx.stack_id,
                        pid: fctx.pid,
                    };

                    crate::[<$alloc_type:upper _ALLOCATIONS>].push(&alloc_info, 0).map_err(|_| 0u32)?;
                    crate::[<TEMP_ $alloc_type:upper _ALLOCATIONS>].insert(&ptr, &alloc_info, 0).map_err(|_| 0u32)?;

                    //aya_log_ebpf::trace!(&fctx.ctx, "ALLOC {} at {:X}", size, ptr);

                    Ok(0)
                }
            )*
        }
    };
}

#[macro_export]
macro_rules! free {
    ($alloc_type:ident, $($function:ident),*) => {
        paste::paste! {
            $(
                #[aya_ebpf::macros::tracepoint]
                pub fn [<tracepoint_ $function>](ctx: aya_ebpf::programs::TracePointContext) -> u32 {
                    match [<try_tracepoint_ $function>](ctx) {
                        Ok(ret) => ret,
                        Err(ret) => {
                            unsafe {
                                aya_ebpf::bpf_printk!(b"Error in $function");
                            }
                            ret
                        },
                    }
                }

                fn [<try_tracepoint_ $function>](ctx: aya_ebpf::programs::TracePointContext) -> Result<u32, u32> {
                    let fctx = crate::utils::context::get_full_ctx(ctx)?;

                    if !crate::utils::function::should_profile_stack_id(fctx.cpuid) {
                        return Err(0);
                    }

                    let ptr_ptr: *const u64 = unsafe { fctx.ctx.read_at(16).map_err(|_| 0u32)? };
                    let ptr: u64 = unsafe { aya_ebpf::helpers::bpf_probe_read_kernel(ptr_ptr).map_err(|_| 0u32)? };
                    let alloc_info = unsafe { *crate::[<TEMP_ $alloc_type:upper _ALLOCATIONS>].get(&ptr).ok_or(0u32)? };
                    let time = unsafe { aya_ebpf::helpers::bpf_ktime_get_ns() };
                    
                    let alloc_info = aya_network_deep_profiling_common::AllocInfo {
                        alloc_type: aya_network_deep_profiling_common::AllocType::$alloc_type,
                        alloc_direction: aya_network_deep_profiling_common::AllocDirection::Free,
                        size: alloc_info.size,
                        timestamp: time,
                        stack_id: alloc_info.stack_id,
                        pid: alloc_info.pid,
                    };

                    crate::[<$alloc_type:upper _ALLOCATIONS>].push(&alloc_info, 0).map_err(|_| 0u32)?;
                    crate::[<TEMP_ $alloc_type:upper _ALLOCATIONS>].remove(&ptr).map_err(|_| 0u32)?;

                    //aya_log_ebpf::trace!(&fctx.ctx, "FREED {} at {:X}", *size, ptr);

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
                pub fn [<probe_enter_ $function>](ctx: aya_ebpf::programs::ProbeContext) -> u32 {
                    match [<probe_try_enter_ $function>](ctx) {
                        Ok(ret) => ret,
                        Err(ret) => {
                            unsafe {
                                aya_ebpf::bpf_printk!(b"Error in $function");
                            }
                            ret
                        },
                    }
                }

                pub fn [<probe_try_enter_ $function>](ctx: aya_ebpf::programs::ProbeContext) -> Result<u32, u32> {
                    let cpuid = unsafe { aya_ebpf::helpers::bpf_get_smp_processor_id() } as u32;

                    if !crate::utils::function::should_profile_stack_id(cpuid) {
                        return Ok(0);
                    }

                    let function = aya_network_deep_profiling_common::Alloc::$function;
                    let direction = aya_network_deep_profiling_common::FunctionDirection::Entry;

                    let depth = crate::utils::function::increment_depth(&cpuid)?;
                    crate::utils::time::log_alloc_time(function, direction, depth, cpuid)?;

                    let stack_id = match unsafe { crate::STACK_TRACES.get_stackid(&ctx, 0) } {
                        Ok(stack_id) => stack_id,
                        _ => return Err(0),
                    };
                    crate::utils::function::register_function(&stack_id, function.as_id())?;

                    Ok(0)
                }

                #[aya_ebpf::macros::kretprobe]
                pub fn [<probe_ret_ $function>](_ctx: aya_ebpf::programs::RetProbeContext) -> u32 {
                    match [<probe_try_ret_ $function>]() {
                        Ok(ret) => ret,
                        Err(ret) => {
                            unsafe {
                                aya_ebpf::bpf_printk!(b"Error in $function");
                            }
                            ret
                        },
                    }
                }

                pub fn [<probe_try_ret_ $function>]() -> Result<u32, u32> {
                    let cpuid = unsafe { aya_ebpf::helpers::bpf_get_smp_processor_id() } as u32;

                    if !crate::utils::function::should_profile_stack_id(cpuid) {
                        return Ok(0);
                    }

                    let function = aya_network_deep_profiling_common::Alloc::$function;
                    let direction = aya_network_deep_profiling_common::FunctionDirection::Exit;

                    let depth = crate::utils::function::decrement_depth(&cpuid)?;
                    crate::utils::time::log_alloc_time(function, direction, depth, cpuid)?;

                    Ok(0)
                }
            )*
        }
    };
}