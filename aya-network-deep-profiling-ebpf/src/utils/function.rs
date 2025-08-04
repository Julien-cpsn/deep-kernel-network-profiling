use crate::{ACTIVE_FUNCTIONS, DEPTH_COUNTER, REGISTERED_FUNCTIONS};

pub fn set_function_active(cpuid: &u32, state: bool) -> Result<(), u32> {
    match ACTIVE_FUNCTIONS.get_ptr_mut(cpuid) {
        None => ACTIVE_FUNCTIONS.insert(cpuid, &state, 0).map_err(|_| 0u32)?,
        Some(active) => unsafe { *active = state }
    };

    Ok(())
}

pub fn increment_depth(cpuid: &u32) -> Result<u32, u32> {
    let new_depth = match DEPTH_COUNTER.get_ptr_mut(cpuid) {
        Some(ptr) => unsafe {
            *ptr += 1;
            *ptr
        },
        None => {
            DEPTH_COUNTER.insert(cpuid, &1, 0).map_err(|_| 0u32)?;
            1
        }
    };

    Ok(new_depth)
}

pub fn decrement_depth(cpuid: &u32) -> Result<u32, u32> {
    let new_depth = match DEPTH_COUNTER.get_ptr_mut(cpuid) {
        Some(ptr) => unsafe {
            *ptr = (*ptr).saturating_sub(1);
            *ptr
        },
        None => {
            DEPTH_COUNTER.insert(cpuid, &0, 0).map_err(|_| 0u32)?;
            0
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

pub fn register_function(stack_id: &i64, function_id: u16) -> Result<(), u32> {
    if REGISTERED_FUNCTIONS.get_ptr(stack_id).is_none() {
        REGISTERED_FUNCTIONS.insert(stack_id, &function_id, 0).map_err(|_| 0u32)?;
    }

    Ok(())
}
