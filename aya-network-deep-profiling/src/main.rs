mod args;
mod memory;
mod time;
mod xdp;
mod throughput;
mod utils;

use std::collections::HashMap;
use std::env;
use std::fs::{OpenOptions};
use std::io::Write;
use aya::maps;
use aya::maps::{Queue, StackTraceMap, HashMap as EHashMap, PerCpuHashMap};
use aya::programs::{KProbe, SchedClassifier, TcAttachType, TracePoint, UProbe, Xdp, XdpFlags};
use aya::programs::tc::SchedClassifierLinkId;
use aya::programs::xdp::XdpLinkId;
use clap::Parser;
use getifaddrs::getifaddrs;
use libc::{clock_gettime, timespec, CLOCK_BOOTTIME};
use log::{debug, warn, info};
use once_cell::sync::Lazy;
use pretty_env_logger::env_logger;
use serde::Serialize;
use tokio::signal;
use aya_network_deep_profiling_common::{AllocInfo, KernelFunction, FunctionCall, Alloc, KERNEL_FUNCTIONS, ALLOCS, TRACEPOINTS, USER_FUNCTIONS, UserFunction, USER_FUNCTION_VARIANTS, ThroughputStat, EthHeader};
use crate::args::Args;
use crate::memory::{handle_memory_usage};
use crate::throughput::{process_throughput, ThroughputRow};
use crate::time::{filter_times, handle_execution_times, ExecutionTimeRow};
use crate::utils::CPU_FREQUENCY;
use crate::xdp::process_xdp;

static ARGS: Lazy<Args> = Lazy::new(Args::parse);

static FUNCTIONS: Lazy<Vec<&str>> = Lazy::new(|| [KERNEL_FUNCTIONS.to_vec(), USER_FUNCTIONS.to_vec(), ALLOCS.to_vec()].concat());

static TRACEPOINTS_: Lazy<Vec<(String, Vec<(&str, &str)>)>> = Lazy::new(|| {
    let mut tracepoints = Vec::new();

    for tracepoint in TRACEPOINTS {
        tracepoints.push((format!("tracepoint_{tracepoint}"), vec![("kmem", tracepoint)]));
    }

    tracepoints
});

static KERNEL_PROBES: Lazy<Vec<(String, Vec<String>)>> = Lazy::new(|| {
    let mut probes = Vec::new();

    for function in KERNEL_FUNCTIONS {
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

static USER_PROBES: Lazy<Vec<(String, Vec<(String, String)>)>> = Lazy::new(|| {
    let mut probes = Vec::new();

    for function in USER_FUNCTION_VARIANTS {
        let user_function = function.as_str().replace("_p_", ".");
        let lib_path = function.get_lib_path().to_string();

        probes.push((
            format!("probe_enter_{function}"),
            vec![
                (user_function.clone(), lib_path.clone()),
            ])
        );
        probes.push((
            format!("probe_ret_{function}"),
            vec![
                (user_function.clone(), lib_path),
            ])
        );
    }

    probes
});

#[derive(Serialize)]
pub struct JsonData {
    pub allocations: Vec<AllocInfo>,
    pub execution_times: Vec<ExecutionTimeRow>,
    pub xdp_times: Vec<(u64, String)>,
    pub throughput: Vec<ThroughputRow>,
}

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

    let mut kernel_probe_links = HashMap::new();
    for (program, functions) in KERNEL_PROBES.iter() {
        info!("Attaching program {program} to functions: {functions:?}");
        let probe: &mut KProbe = ebpf.program_mut(program).unwrap().try_into()?;
        probe.load()?;

        for function in functions {
            let link_id = probe.attach(function, 0)?;
            kernel_probe_links.insert((program, function), link_id);
        }
    }

    let mut user_probe_links = HashMap::new();
    for (program, functions) in USER_PROBES.iter() {
        info!("Attaching program {program} to functions: {functions:?}");
        let probe: &mut UProbe = ebpf.program_mut(program).unwrap().try_into()?;
        probe.load()?;

        for (function, lib) in functions {
            let link_id = probe.attach(Some(function), 0, lib, None)?;
            user_probe_links.insert((program, function), link_id);
        }
    }

    /* --------- XDP setup ----------- */

    let interfaces = getifaddrs()?.collect::<Vec<_>>();

    let mut xdp_links: HashMap<String, XdpLinkId> = HashMap::new();
    let program: &mut Xdp = ebpf.program_mut("xdp_packet_log").unwrap().try_into()?;
    program.load()?;

    for interface in &interfaces {
        if xdp_links.contains_key(&interface.name) {
            continue;
        }

        info!("Attaching program xdp_packet_log to iface {}", interface.name);
        let link_id = program.attach(&interface.name, XdpFlags::default())?;
        xdp_links.insert(interface.name.clone(), link_id);
    }

    /* --------- Classifiers setup --------- */

    let mut classifiers_links: HashMap<(String, TcAttachType), SchedClassifierLinkId> = HashMap::new();
    let ingress_program: &mut SchedClassifier = ebpf.program_mut("tc_ingress").unwrap().try_into()?;
    ingress_program.load()?;

    for interface in &interfaces {
        if classifiers_links.contains_key(&(interface.name.clone(), TcAttachType::Ingress)) {
            continue;
        }

        info!("Attaching program tc_ingress to iface {}", interface.name);
        let link_id = ingress_program.attach(&interface.name, TcAttachType::Ingress)?;
        classifiers_links.insert((interface.name.clone(), TcAttachType::Ingress), link_id);
    }

    let egress_program: &mut SchedClassifier = ebpf.program_mut("tc_egress").unwrap().try_into()?;
    egress_program.load()?;

    for interface in &interfaces {
        if classifiers_links.contains_key(&(interface.name.clone(), TcAttachType::Egress)) {
            continue;
        }

        info!("Attaching program tc_egress to iface {}", interface.name);
        let link_id = egress_program.attach(&interface.name, TcAttachType::Egress)?;
        classifiers_links.insert((interface.name.clone(), TcAttachType::Egress), link_id);
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

    for ((program, function), link_id) in kernel_probe_links {
        info!("Detaching program {program} from function {function}");
        let probe: &mut KProbe = ebpf.program_mut(program).unwrap().try_into()?;
        probe.detach(link_id)?;
    }

    for ((program, function), link_id) in user_probe_links {
        info!("Detaching program {program} from function {function}");
        let probe: &mut UProbe = ebpf.program_mut(program).unwrap().try_into()?;
        probe.detach(link_id)?;
    }

    /* --------- XDP end ----------- */
    let xdp: &mut Xdp = ebpf.program_mut("xdp_packet_log").unwrap().try_into()?;

    for (interface, link_id) in xdp_links {
        info!("Detaching program xdp_packet_log from iface {interface}");
        xdp.detach(link_id)?;
    }

    /* --------- XDP end ----------- */
    for ((interface, direction), link_id) in classifiers_links {
        let program_name = match direction {
            TcAttachType::Ingress => "tc_ingress",
            TcAttachType::Egress => "tc_egress",
            TcAttachType::Custom(_) => unreachable!()
        };

        info!("Detaching program {program_name} from iface {interface}");

        let program: &mut SchedClassifier = ebpf.program_mut(program_name).unwrap().try_into()?;
        program.detach(link_id)?;
    }

    /* --------- Perf events end ----------- */

    /*
    for (program, link_id) in perf_events_links {
        info!("Detaching perf event program {program}");
        let perf_event: &mut PerfEvent = ebpf.program_mut(&program).unwrap().try_into()?;
        perf_event.detach(link_id)?;
    }*/

    /* --------- Main program ----------- */
    info!("Gathering data...");

    let mut kmalloc_allocations: Queue<_, AllocInfo> = Queue::try_from(ebpf.take_map("KMALLOC_ALLOCATIONS").unwrap())?;
    let mut kmem_cache_allocations: Queue<_, AllocInfo> = Queue::try_from(ebpf.take_map("KMEM_CACHE_ALLOCATIONS").unwrap())?;
    let registered_functions: EHashMap<_, i64, u16> = EHashMap::try_from(ebpf.take_map("REGISTERED_FUNCTIONS").unwrap())?;
    let stack_traces = StackTraceMap::try_from(ebpf.take_map("STACK_TRACES").unwrap())?;

    let kmalloc_allocations = memory::collect_queue(&mut kmalloc_allocations, initial_time);
    let kmem_cache_allocations = memory::collect_queue(&mut kmem_cache_allocations, initial_time);
    let mut allocations = [kmalloc_allocations, kmem_cache_allocations].concat();

    handle_memory_usage(&mut allocations, &registered_functions, &stack_traces, initial_time)?;

    println!();
    println!("CPU frequency: {} Hz", *CPU_FREQUENCY);

    //let cache_misses: maps::HashMap<_, u64, u64> = maps::HashMap::try_from(ebpf.take_map("CACHE_MISSES").unwrap())?;

    let kernel_functions_execution_times: PerCpuHashMap<_, u64, FunctionCall<KernelFunction>> = PerCpuHashMap::try_from(ebpf.take_map("KERNEL_FUNCTIONS_EXECUTION_TIMES").unwrap())?;
    let kernel_functions_execution_times = filter_times(kernel_functions_execution_times, initial_time);
    let kernel_functions_execution_times = handle_execution_times(kernel_functions_execution_times, initial_time);

    println!();

    let user_functions_execution_times: PerCpuHashMap<_, u64, FunctionCall<UserFunction>> = PerCpuHashMap::try_from(ebpf.take_map("USER_FUNCTIONS_EXECUTION_TIMES").unwrap())?;
    let user_functions_execution_times = filter_times(user_functions_execution_times, initial_time);
    let user_functions_execution_times = handle_execution_times(user_functions_execution_times, initial_time);

    println!();

    let alloc_functions_execution_times: PerCpuHashMap<_, u64, FunctionCall<Alloc>> = PerCpuHashMap::try_from(ebpf.take_map("ALLOC_FUNCTIONS_EXECUTION_TIMES").unwrap())?;
    let alloc_functions_execution_times = filter_times(alloc_functions_execution_times, initial_time);
    let alloc_functions_execution_times = handle_execution_times(alloc_functions_execution_times, initial_time);

    let execution_times = [kernel_functions_execution_times, user_functions_execution_times, alloc_functions_execution_times].concat();

    let xdp_times: maps::HashMap<_, u64, EthHeader> = maps::HashMap::try_from(ebpf.take_map("XDP_TIMES").unwrap())?;
    let xdp_times = process_xdp(xdp_times, initial_time);

    let mut throughput_stats: Queue<_, ThroughputStat> = Queue::try_from(ebpf.take_map("THROUGHPUT_STATS").unwrap())?;
    let throughput_stats = throughput::collect_queue(&mut throughput_stats, initial_time);
    let throughput = process_throughput(throughput_stats, interfaces, initial_time);

    info!("Writing results to file...");

    let json_data = JsonData {
        allocations,
        execution_times,
        xdp_times,
        throughput,
    };
    let results_json = serde_json::to_string(&json_data)?;
    let results_file_path = env::current_dir()?.join("shared").join("results.json");
    let mut result_file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(results_file_path)?;

    result_file.write_all(results_json.as_bytes())?;

    Ok(())
}