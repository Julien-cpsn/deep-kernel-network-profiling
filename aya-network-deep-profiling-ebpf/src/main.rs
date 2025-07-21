#![no_std]
#![no_main]
#![allow(
    clippy::all,
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unnecessary_transmutes,
)]

pub mod probes;
pub mod tracepoints;
pub mod bindings;
pub mod utils;

use aya_ebpf::macros::map;
use aya_ebpf::maps::{HashMap, PerCpuHashMap, Queue, StackTrace};
use aya_network_deep_profiling_common::{AllocInfo, Function, FunctionWithDirection, Malloc};

const MAX_ENTRIES: u32 = 10000;

#[map]
pub static REGISTERED_FUNCTIONS: HashMap<i64, Function> = HashMap::with_max_entries(100, 0);

#[map]
static ACTIVE_FUNCTIONS: PerCpuHashMap<u32, bool> = PerCpuHashMap::with_max_entries(MAX_ENTRIES, 0);

#[map]
pub static ALLOCATIONS: Queue<AllocInfo> = Queue::with_max_entries(MAX_ENTRIES, 0);

#[map]
static TEMP_ALLOCATIONS: HashMap<u64, AllocInfo> = HashMap::with_max_entries(MAX_ENTRIES, 0);

#[map]
pub static STACK_TRACES: StackTrace = StackTrace::with_max_entries(MAX_ENTRIES, 0);

#[map]
pub static FUNCTIONS_EXECUTION_TIMES: HashMap<u64, FunctionWithDirection<Function>> = HashMap::with_max_entries(MAX_ENTRIES, 0);

#[map]
pub static MALLOCS_EXECUTION_TIMES: HashMap<u64, FunctionWithDirection<Malloc>> = HashMap::with_max_entries(MAX_ENTRIES, 0);

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[unsafe(link_section = "license")]
#[unsafe(no_mangle)]
static LICENSE: [u8; 13] = *b"Dual MIT/GPL\0";
