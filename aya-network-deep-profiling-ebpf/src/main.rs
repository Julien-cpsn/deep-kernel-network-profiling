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
//pub mod perf_events;
pub mod xdp;
pub mod classifiers;
pub mod bindings;
pub mod utils;

use aya_ebpf::macros::map;
use aya_ebpf::maps::{HashMap, PerCpuHashMap, Queue, StackTrace};
use aya_network_deep_profiling_common::{Alloc, AllocInfo, EthHeader, FunctionCall, KernelFunction, ThroughputStat, UserFunction};

const MAX_ENTRIES: u32 = 1_000_000;

// Functions

#[map]
pub static REGISTERED_FUNCTIONS: HashMap<i64, u16> = HashMap::with_max_entries(500, 0);

#[map]
static ACTIVE_FUNCTIONS: PerCpuHashMap<u32, bool> = PerCpuHashMap::with_max_entries(MAX_ENTRIES, 0);

#[map]
pub static DEPTH_COUNTER: PerCpuHashMap<u32, u32> = PerCpuHashMap::with_max_entries(MAX_ENTRIES, 0);

#[map]
pub static STACK_TRACES: StackTrace = StackTrace::with_max_entries(MAX_ENTRIES, 0);

/*
#[map]
static CACHE_MISSES: HashMap<u64, u64> = HashMap::with_max_entries(MAX_ENTRIES, 0);
*/

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
pub static KERNEL_FUNCTIONS_EXECUTION_TIMES: PerCpuHashMap<u64, FunctionCall<KernelFunction>> = PerCpuHashMap::with_max_entries(MAX_ENTRIES, 0);


#[map]
pub static USER_FUNCTIONS_EXECUTION_TIMES: PerCpuHashMap<u64, FunctionCall<UserFunction>> = PerCpuHashMap::with_max_entries(MAX_ENTRIES, 0);

#[map]
pub static ALLOC_FUNCTIONS_EXECUTION_TIMES: PerCpuHashMap<u64, FunctionCall<Alloc>> = PerCpuHashMap::with_max_entries(MAX_ENTRIES, 0);

#[map]
pub static XDP_TIMES: HashMap<u64, EthHeader> = HashMap::with_max_entries(MAX_ENTRIES, 0);

// Throughput stats

#[map]
pub static THROUGHPUT_STATS: Queue<ThroughputStat> = Queue::with_max_entries(MAX_ENTRIES, 0);


#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[unsafe(link_section = "license")]
#[unsafe(no_mangle)]
static LICENSE: [u8; 13] = *b"Dual MIT/GPL\0";
