mod args;
mod memory;
mod time;
mod utils;

use std::collections::HashMap;
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use aya::maps::{Queue, StackTraceMap, HashMap as EHashMap};
use aya::programs::{KProbe, TracePoint};
use clap::Parser;
use libc::{clock_gettime, timespec, CLOCK_BOOTTIME};
use log::{debug, warn, info};
use once_cell::sync::Lazy;
use pretty_env_logger::env_logger;
use tokio::signal;
use aya_network_deep_profiling_common::{AllocInfo, Function, FunctionWithDirection, Malloc, FUNCTIONS, MALLOCS};
use aya_network_deep_profiling_common::Function::{Kfree, Kmalloc};
use crate::args::Args;
use crate::memory::handle_memory_usage;
use crate::time::handle_execution_times;
use crate::utils::CPU_FREQUENCY;

static ARGS: Lazy<Args> = Lazy::new(Args::parse);

static TRACEPOINTS: Lazy<Vec<(&str, (&str, &str))>> = Lazy::new(|| vec![
    ("tracepoint_kmalloc", ("kmem", "kmalloc")),
    ("tracepoint_kfree", ("kmem", "kfree")),
]);

static PROBES: Lazy<Vec<(String, Vec<String>)>> = Lazy::new(|| {
    let mut probes = Vec::new();

    for function in FUNCTIONS {
        if function == Kmalloc.as_str() || function == Kfree.as_str() {
            continue;
        }

        probes.push((format!("probe_enter_{function}"), vec![function.replace("_p_", ".")]));
        probes.push((format!("probe_ret_{function}"), vec![function.replace("_p_", ".")]));
    }

    for kmalloc_variant in MALLOCS {
        probes.push((format!("probe_enter_{kmalloc_variant}"), vec![kmalloc_variant.replace("_p_", ".")]));
        probes.push((format!("probe_ret_{kmalloc_variant}"), vec![kmalloc_variant.replace("_p_", ".")]));
    }

    probes
});

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::builder().filter_level(ARGS.verbosity.log_level_filter()).init();

    /* --------- eBPF setup ----------- */

    // Bump the memlock rlimit. This is needed for older kernels that don't use the
    // new memcg based accounting, see https://lwn.net/Articles/837122/
    let rlim = libc::rlimit {
        rlim_cur: libc::RLIM_INFINITY,
        rlim_max: libc::RLIM_INFINITY,
    };
    let ret = unsafe { libc::setrlimit(libc::RLIMIT_MEMLOCK, &rlim) };
    if ret != 0 {
        debug!("remove limit on locked memory failed, ret is: {ret}");
    }

    // This will include your eBPF object file as raw bytes at compile-time and load it at
    // runtime. This approach is recommended for most real-world use cases. If you would
    // like to specify the eBPF program at runtime rather than at compile-time, you can
    // reach for `Bpf::load_file` instead.
    let mut ebpf = aya::Ebpf::load(aya::include_bytes_aligned!(concat!(
        env!("OUT_DIR"),
        "/aya-network-deep-profiling"
    )))?;
    if let Err(e) = aya_log::EbpfLogger::init(&mut ebpf) {
        // This can happen if you remove all log statements from your eBPF program.
        warn!("failed to initialize eBPF logger: {e}");
    }

    /* --------- Main preparation ----------- */

    let mut ts = timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    unsafe { clock_gettime(CLOCK_BOOTTIME, &mut ts)};

    /* --------- Tracepoints setup ----------- */

    let mut tracepoint_links = HashMap::new();

    for (program, (category, name)) in TRACEPOINTS.iter() {
        let tracepoint: &mut TracePoint = ebpf.program_mut(program).unwrap().try_into()?;
        tracepoint.load()?;
        info!("Attaching program {program} to tracepoint {category}:{name}");
        let link_id = tracepoint.attach(category, name)?;
        tracepoint_links.insert((program, (category, name)), link_id);
    }

    /* --------- Probes setup ----------- */

    let mut probe_links = HashMap::new();

    for (program, functions) in PROBES.iter() {
        let probe: &mut KProbe = ebpf.program_mut(program).unwrap().try_into()?;
        probe.load()?;

        for function in functions {
            info!("Attaching program {program} to function {function}");
            let link_id = probe.attach(function, 0)?;
            probe_links.insert((program, function), link_id);
        }
    }

    let ctrl_c = signal::ctrl_c();
    println!("Waiting for Ctrl-C...");
    ctrl_c.await?;
    println!("Exiting...");

    /* --------- Tracepoints end ----------- */

    for ((program, (category, name)), link_id) in tracepoint_links {
        info!("Detaching program {program} from tracepoint {category}:{name}");
        let tracepoint: &mut TracePoint = ebpf.program_mut(program).unwrap().try_into()?;
        tracepoint.detach(link_id)?;
    }

    /* --------- Probes end ----------- */

    for ((program, function), link_id) in probe_links {
        info!("Detaching program {program} from function {function}");
        let probe: &mut KProbe = ebpf.program_mut(program).unwrap().try_into()?;
        probe.detach(link_id)?;
    }

    /* --------- Main program ----------- */

    let initial_time = (ts.tv_sec * 1_000_000_000 + ts.tv_nsec) as u64;

    let mut allocations: Queue<_, AllocInfo> = Queue::try_from(ebpf.take_map("ALLOCATIONS").unwrap())?;
    let registered_functions: EHashMap<_, i64, Function> = EHashMap::try_from(ebpf.take_map("REGISTERED_FUNCTIONS").unwrap())?;
    let stack_traces = StackTraceMap::try_from(ebpf.take_map("STACK_TRACES").unwrap())?;

    let allocations = handle_memory_usage(&mut allocations, &registered_functions, &stack_traces, initial_time)?;

    let allocations_json = serde_json::to_string(&allocations)?;

    let allocations_json_file_path = env::current_dir()?.join("shared").join("allocations.json");
    let mut result_file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(allocations_json_file_path)?;

    result_file.write_all(allocations_json.as_bytes())?;

    println!();
    println!("CPU frequency: {} Hz", *CPU_FREQUENCY);

    let function_times: EHashMap<_, u64, FunctionWithDirection<Function>> = EHashMap::try_from(ebpf.take_map("FUNCTIONS_EXECUTION_TIMES").unwrap())?;
    let mut function_times: Vec<(u64, FunctionWithDirection<Function>)> = function_times
        .iter()
        .filter_map(|t| t.ok())
        .collect();
    function_times.sort_by(|(a, _), (b, _)| a.cmp(b));

    let function_execution_times = handle_execution_times(function_times, initial_time);

    println!();

    let malloc_times: EHashMap<_, u64, FunctionWithDirection<Malloc>> = EHashMap::try_from(ebpf.take_map("MALLOCS_EXECUTION_TIMES").unwrap())?;
    let mut malloc_times: Vec<(u64, FunctionWithDirection<Malloc>)> = malloc_times
        .iter()
        .filter_map(|t| t.ok())
        .collect();
    malloc_times.sort_by(|(a, _), (b, _)| a.cmp(b));

    let malloc_execution_times = handle_execution_times(malloc_times, initial_time);


    println!("Writing results to file...");
    let execution_times = [function_execution_times, malloc_execution_times].concat();
    let execution_times_json = serde_json::to_string(&execution_times)?;

    let execution_times_file_path = env::current_dir()?.join("shared").join("execution_times.json");
    let mut result_file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(execution_times_file_path)?;

    result_file.write_all(execution_times_json.as_bytes())?;

    Ok(())
}