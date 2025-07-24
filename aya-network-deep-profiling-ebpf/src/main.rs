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
pub mod xdp;
pub mod bindings;
pub mod utils;

use aya_ebpf::macros::map;
use aya_ebpf::maps::{HashMap, PerCpuHashMap, Queue, StackTrace};
use aya_network_deep_profiling_common::{AllocInfo, Function, FunctionCall, Alloc, STRING_AS_BYTES_MAX_LEN};

const MAX_ENTRIES: u32 = 500_000;

// Functions

#[map]
pub static REGISTERED_FUNCTIONS: HashMap<i64, [u8;STRING_AS_BYTES_MAX_LEN]> = HashMap::with_max_entries(500, 0);

#[map]
static ACTIVE_FUNCTIONS: PerCpuHashMap<u32, bool> = PerCpuHashMap::with_max_entries(MAX_ENTRIES, 0);

#[map]
pub static DEPTH_COUNTER: PerCpuHashMap<u32, u32> = PerCpuHashMap::with_max_entries(MAX_ENTRIES, 0);

#[map]
pub static STACK_TRACES: StackTrace = StackTrace::with_max_entries(MAX_ENTRIES, 0);

// Memory

#[map]
pub static KMALLOC_ALLOCATIONS: Queue<AllocInfo> = Queue::with_max_entries(MAX_ENTRIES, 0);

#[map]
static TEMP_KMALLOC_ALLOCATIONS: HashMap<u64, AllocInfo> = HashMap::with_max_entries(MAX_ENTRIES, 0);

#[map]
pub static KMEM_CACHE_ALLOCATIONS: Queue<AllocInfo> = Queue::with_max_entries(MAX_ENTRIES, 0);

#[map]
static TEMP_KMEM_CACHE_ALLOCATIONS: HashMap<u64, AllocInfo> = HashMap::with_max_entries(MAX_ENTRIES, 0);

// Execution times

#[map]
pub static FUNCTIONS_EXECUTION_TIMES: PerCpuHashMap<u64, FunctionCall<Function>> = PerCpuHashMap::with_max_entries(MAX_ENTRIES, 0);

#[map]
pub static ALLOCS_EXECUTION_TIMES: PerCpuHashMap<u64, FunctionCall<Alloc>> = PerCpuHashMap::with_max_entries(MAX_ENTRIES, 0);

#[map]
pub static XDP_TIMES: HashMap<u64, [u8;6]> = HashMap::with_max_entries(MAX_ENTRIES, 0);

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[unsafe(link_section = "license")]
#[unsafe(no_mangle)]
static LICENSE: [u8; 13] = *b"Dual MIT/GPL\0";
