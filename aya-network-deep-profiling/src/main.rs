mod args;
mod memory;
mod time;
mod utils;

use std::collections::HashMap;
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use aya::maps;
use aya::maps::{Queue, StackTraceMap, HashMap as EHashMap, PerCpuHashMap};
use aya::programs::{KProbe, TracePoint, Xdp, XdpFlags};
use aya::programs::xdp::XdpLinkId;
use clap::Parser;
use getifaddrs::getifaddrs;
use libc::{clock_gettime, timespec, CLOCK_BOOTTIME};
use log::{debug, warn, info};
use once_cell::sync::Lazy;
use pretty_env_logger::env_logger;
use tokio::signal;
use aya_network_deep_profiling_common::{AllocInfo, Function, FunctionCall, Alloc, FUNCTIONS, ALLOCS, TRACEPOINTS, EthHeader, EtherHeaderType};
use crate::args::Args;
use crate::memory::{collect_queue, handle_memory_usage};
use crate::time::{filter_times, handle_execution_times};
use crate::utils::CPU_FREQUENCY;

static ARGS: Lazy<Args> = Lazy::new(Args::parse);

static TRACEPOINTS_: Lazy<Vec<(String, Vec<(&str, &str)>)>> = Lazy::new(|| {
    let mut tracepoints = Vec::new();

    for tracepoint in TRACEPOINTS {
        tracepoints.push((format!("tracepoint_{tracepoint}"), vec![("kmem", tracepoint)]));
    }

    tracepoints
});

static PROBES: Lazy<Vec<(String, Vec<String>)>> = Lazy::new(|| {
    let mut probes = Vec::new();

    for function in FUNCTIONS {
        let kernel_function = function.replace("_p_", ".");

        probes.push((format!("probe_enter_{function}"), vec![kernel_function.clone()]));
        probes.push((format!("probe_ret_{function}"), vec![kernel_function]));
    }

    for alloc_variant in ALLOCS {
        let kernel_alloc = alloc_variant.replace("_p_", ".");

        probes.push((format!("probe_enter_{alloc_variant}"), vec![kernel_alloc.clone()]));
        probes.push((format!("probe_ret_{alloc_variant}"), vec![kernel_alloc]));
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

    for (program, tracepoints) in TRACEPOINTS_.iter() {
        info!("Attaching program {program} to tracepoints: {tracepoints:?}");
        let tracepoint: &mut TracePoint = ebpf.program_mut(program).unwrap().try_into()?;
        tracepoint.load()?;

        for (category, name) in tracepoints {
            let link_id = tracepoint.attach(category, name)?;
            tracepoint_links.insert((program, (category, name)), link_id);
        }
    }

    /* --------- Probes setup ----------- */

    let mut probe_links = HashMap::new();

    for (program, functions) in PROBES.iter() {
        info!("Attaching program {program} to functions: {functions:?}");
        let probe: &mut KProbe = ebpf.program_mut(program).unwrap().try_into()?;
        probe.load()?;

        for function in functions {
            let link_id = probe.attach(function, 0)?;
            probe_links.insert((program, function), link_id);
        }
    }

    /* --------- XDP setup ----------- */

    let mut interfaces_links: HashMap<String, XdpLinkId> = HashMap::new();
    let program: &mut Xdp = ebpf.program_mut("xdp_packet_log").unwrap().try_into()?;
    program.load()?;

    for interface in getifaddrs()? {
        if interfaces_links.contains_key(&interface.name) {
            continue;
        }

        info!("Attaching program xdp_packet_log to iface {}", interface.name);
        let link_id = program.attach(&interface.name, XdpFlags::default())?;
        interfaces_links.insert(interface.name.clone(), link_id);
    }

    /* --------- Perf event ----------- */

    /*
    let mut perf_events_links: HashMap<String, PerfEventLinkId> = HashMap::new();
    let program: &mut PerfEvent = ebpf.program_mut("perf_l1d_misses").unwrap().try_into()?;
    program.load()?;

    // Config for L1D cache misses: type=L1D, op=READ, result=MISS
    let config = (PERF_COUNT_HW_CACHE_L1D as u64) |
        ((PERF_COUNT_HW_CACHE_OP_READ as u64) << 8) |
        ((PERF_COUNT_HW_CACHE_RESULT_MISS as u64) << 16);


    perf_events_links.insert(String::from("perf_l1d_misses"), link_id);
    for cpu in online_cpus().map_err(|(message, error)| { println!("{message}"); error })? {
        info!("Attaching program perf_l1d_misses to CPU {cpu}");

        let link_id = program.attach(
            PerfTypeId::HwCache,
            config,
            // Monitor all processes
            PerfEventScope::AllProcessesOneCpu { cpu },
            // Sample period
            SamplePolicy::Period(10_000),
            false,
        )?;
    }*/

    /* --------- Wait ----------- */

    let initial_time = (ts.tv_sec * 1_000_000_000 + ts.tv_nsec) as u64;

    let ctrl_c = signal::ctrl_c();
    warn!("Waiting for Ctrl-C...");
    ctrl_c.await?;
    warn!("Exiting...");

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

    /* --------- XDP end ----------- */

    for (interface, link_id) in interfaces_links {
        info!("Detaching program xdp_packet_log from iface {interface}");
        let xdp: &mut Xdp = ebpf.program_mut("xdp_packet_log").unwrap().try_into()?;
        xdp.detach(link_id)?;
    }

    /* --------- Perf events end ----------- */

    /*
    for (program, link_id) in perf_events_links {
        info!("Detaching perf event program {program}");
        let perf_event: &mut PerfEvent = ebpf.program_mut(&program).unwrap().try_into()?;
        perf_event.detach(link_id)?;
    }*/

    /* --------- Main program ----------- */

    let mut kmalloc_allocations: Queue<_, AllocInfo> = Queue::try_from(ebpf.take_map("KMALLOC_ALLOCATIONS").unwrap())?;
    let mut kmem_cache_allocations: Queue<_, AllocInfo> = Queue::try_from(ebpf.take_map("KMEM_CACHE_ALLOCATIONS").unwrap())?;
    let registered_functions: EHashMap<_, i64, u16> = EHashMap::try_from(ebpf.take_map("REGISTERED_FUNCTIONS").unwrap())?;
    let stack_traces = StackTraceMap::try_from(ebpf.take_map("STACK_TRACES").unwrap())?;

    let kmalloc_allocations = collect_queue(&mut kmalloc_allocations, initial_time);
    let kmem_cache_allocations = collect_queue(&mut kmem_cache_allocations, initial_time);
    let mut allocations = [kmalloc_allocations, kmem_cache_allocations].concat();

    handle_memory_usage(&mut allocations, &registered_functions, &stack_traces, initial_time)?;

    let allocations_json = serde_json::to_string(&allocations)?;

    info!("Writing results to file...");

    let allocations_json_file_path = env::current_dir()?.join("shared").join("allocations.json");
    let mut result_file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(allocations_json_file_path)?;

    result_file.write_all(allocations_json.as_bytes())?;

    println!();
    println!("CPU frequency: {} Hz", *CPU_FREQUENCY);

    //let cache_misses: maps::HashMap<_, u64, u64> = maps::HashMap::try_from(ebpf.take_map("CACHE_MISSES").unwrap())?;

    let function_execution_times: PerCpuHashMap<_, u64, FunctionCall<Function>> = PerCpuHashMap::try_from(ebpf.take_map("FUNCTIONS_EXECUTION_TIMES").unwrap())?;
    let function_execution_times = filter_times(function_execution_times, initial_time);
    let function_execution_times = handle_execution_times(function_execution_times, initial_time);

    println!();

    let alloc_execution_times: PerCpuHashMap<_, u64, FunctionCall<Alloc>> = PerCpuHashMap::try_from(ebpf.take_map("ALLOCS_EXECUTION_TIMES").unwrap())?;
    let alloc_execution_times = filter_times(alloc_execution_times, initial_time);
    let alloc_execution_times = handle_execution_times(alloc_execution_times, initial_time);

    let execution_times = [function_execution_times, alloc_execution_times].concat();
    let execution_times_json = serde_json::to_string(&execution_times)?;

    let execution_times_file_path = env::current_dir()?.join("shared").join("execution_times.json");
    let mut result_file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(execution_times_file_path)?;

    result_file.write_all(execution_times_json.as_bytes())?;

    let xdp_times: maps::HashMap<_, u64, EthHeader> = maps::HashMap::try_from(ebpf.take_map("XDP_TIMES").unwrap())?;
    let xdp_times: Vec<(u64, String)> = xdp_times
        .iter()
        .filter_map(|x| match x {
            Ok(x) => {
                let new_time = x.0.saturating_sub(initial_time);

                let eth_type = match x.1.ether_type {
                    EtherHeaderType::Loop => "Loop",
                    EtherHeaderType::Ipv4 => "IPv4",
                    EtherHeaderType::Arp => "ARP",
                    EtherHeaderType::Ipv6 => "IPv6",
                    EtherHeaderType::FibreChannel => "FibreChannel",
                    EtherHeaderType::Infiniband => "Infiniband",
                    EtherHeaderType::LoopbackIeee8023 => "LoopbackIeee8023",
                };

                let src = format!("{:0>2X}:{:0>2X}:{:0>2X}:{:0>2X}:{:0>2X}:{:0>2X}", x.1.src_addr[0], x.1.src_addr[1], x.1.src_addr[2], x.1.src_addr[3], x.1.src_addr[4], x.1.src_addr[5]);
                let dst = format!("{:0>2X}:{:0>2X}:{:0>2X}:{:0>2X}:{:0>2X}:{:0>2X}", x.1.dst_addr[0], x.1.dst_addr[1], x.1.dst_addr[2], x.1.dst_addr[3], x.1.dst_addr[4], x.1.dst_addr[5]);
                let info = format!("{eth_type}, SRC: {src}, DST: {dst}");

                Some((new_time, info))
            }
            Err(_) => None
        })
        .collect();
    let xdp_times_json = serde_json::to_string(&xdp_times)?;

    let xdp_times_file_path = env::current_dir()?.join("shared").join("xdp_times.json");
    let mut result_file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(xdp_times_file_path)?;

    result_file.write_all(xdp_times_json.as_bytes())?;

    Ok(())
}