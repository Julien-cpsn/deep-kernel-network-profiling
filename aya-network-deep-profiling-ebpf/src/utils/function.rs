use aya_network_deep_profiling_common::STRING_AS_BYTES_MAX_LEN;
use crate::{ACTIVE_FUNCTIONS, DEPTH_COUNTER, REGISTERED_FUNCTIONS};

pub fn set_function_active(cpuid: &u32, state: bool) -> Result<(), u32> {
    match ACTIVE_FUNCTIONS.get_ptr_mut(cpuid) {
        None => ACTIVE_FUNCTIONS.insert(cpuid, &state, 0).map_err(|_| 0u32)?,
        Some(active) => unsafe { *active = state }
    };

    Ok(())
}

pub fn increment_depth(cpuid: &u32) -> Result<u32, u32> {
    let depth = match DEPTH_COUNTER.get_ptr_mut(cpuid) {
        Some(ptr) => unsafe { *ptr },
        None => 0,
    };

    let new_depth = depth + 1;
    unsafe {
        match DEPTH_COUNTER.get_ptr_mut(cpuid) {
            Some(ptr) => *ptr = new_depth,
            None => DEPTH_COUNTER.insert(cpuid, &new_depth, 0).map_err(|_| 0u32)?,
        }
    };

    Ok(new_depth)
}

pub fn decrement_depth(cpuid: &u32) -> Result<u32, u32> {
    let depth = match DEPTH_COUNTER.get_ptr_mut(cpuid) {
        Some(ptr) => unsafe { *ptr },
        None => 0,
    };

    // Decrement depth (ensure it doesn't go negative)
    let new_depth = depth.saturating_sub(1);
    unsafe {
        match DEPTH_COUNTER.get_ptr_mut(cpuid) {
            Some(ptr) => *ptr = new_depth,
            None => DEPTH_COUNTER.insert(cpuid, &new_depth, 0).map_err(|_| 0u32)?,
        }
    };

    Ok(new_depth)
}

pub fn should_profile_stack_id(cpuid: u32) -> bool {
    match unsafe { ACTIVE_FUNCTIONS.get(&cpuid) } {
        None => false,
        Some(active) => *active
    }
}

pub fn register_function(stack_id: &i64, function_name: &str) -> Result<(), u32> {
    let mut array_tmp = [0u8;STRING_AS_BYTES_MAX_LEN];
    array_tmp[..function_name.len()].copy_from_slice(function_name.as_bytes());

    if REGISTERED_FUNCTIONS.get_ptr(stack_id).is_none() {
        REGISTERED_FUNCTIONS.insert(stack_id, &array_tmp, 0).map_err(|_| 0u32)?;
    }

    Ok(())
}
