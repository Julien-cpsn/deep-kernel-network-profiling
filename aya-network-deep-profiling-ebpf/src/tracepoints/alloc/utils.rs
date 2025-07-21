use crate::{ACTIVE_FUNCTIONS};

pub fn should_profile_stack_id(cpuid: u32) -> bool {
    match unsafe { ACTIVE_FUNCTIONS.get(&cpuid) } {
        None => false,
        Some(active) => *active
    }
}