use aya_network_deep_profiling_common::Function;
use crate::{ACTIVE_FUNCTIONS, REGISTERED_FUNCTIONS};

pub fn set_function_active(cpuid: &u32, state: bool) -> Result<(), u32> {
    match ACTIVE_FUNCTIONS.get_ptr_mut(cpuid) {
        None => ACTIVE_FUNCTIONS.insert(cpuid, &state, 0).map_err(|_| 0u32)?,
        Some(active) => unsafe { *active = state }
    };

    Ok(())
}

pub fn register_function(stack_id: &i64, function_name: Function) -> Result<(), u32> {
    if REGISTERED_FUNCTIONS.get_ptr(stack_id).is_none() {
        REGISTERED_FUNCTIONS.insert(stack_id, &function_name, 0).map_err(|_| 0u32)?;
    }

    Ok(())
}
