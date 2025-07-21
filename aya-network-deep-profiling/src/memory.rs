use std::collections::HashMap;
use aya::maps::{Queue, StackTraceMap, HashMap as EHashMap, MapData};
use aya::util::kernel_symbols;
use aya_network_deep_profiling::MemStat;
use aya_network_deep_profiling_common::{AllocInfo, AllocType, Function, FUNCTIONS};
use crate::ARGS;

pub fn handle_memory_usage(allocations: &mut Queue<MapData, AllocInfo>, registered_functions: &EHashMap<MapData, i64, Function>, stack_traces: &StackTraceMap<MapData>, initial_time: u64) -> anyhow::Result<Vec<AllocInfo>> {
    let ksyms = kernel_symbols()?;

    let mut all_allocations: Vec<AllocInfo> = Vec::new();
    let mut memory_stats: HashMap<i64, MemStat> = HashMap::new();

    while let Ok(mut alloc_info) = allocations.pop(0) {
        alloc_info.timestamp = alloc_info.timestamp.saturating_sub(initial_time);
        all_allocations.push(alloc_info);

        match memory_stats.get_mut(&alloc_info.stack_id) {
            None => {
                let mem_stat = MemStat {
                    total_allocated: alloc_info.size,
                    total_freed: 0,
                    current_usage: alloc_info.size as i64,
                    peak_usage: alloc_info.size,
                    alloc_count: 1,
                    free_count: 0,
                };

                memory_stats.insert(alloc_info.stack_id, mem_stat);
            }
            Some(mem_stat) => {
                match alloc_info.alloc_type {
                    AllocType::Malloc => {
                        mem_stat.alloc_count += 1;
                        mem_stat.total_allocated += alloc_info.size;
                        mem_stat.current_usage += alloc_info.size as i64;
                    },
                    AllocType::Free => {
                        mem_stat.free_count += 1;
                        mem_stat.total_freed += alloc_info.size;
                        mem_stat.current_usage -= alloc_info.size as i64;
                    },
                };

                if mem_stat.current_usage > mem_stat.peak_usage as i64 {
                    mem_stat.peak_usage = mem_stat.current_usage as u64;
                }
            }
        }
    }

    println!("================================== Memory Usage Statistics ==================================");
    println!(
        "{: <15} {:<10} {:<12} {:<12} {:<12} {:<12} {:<8} {:<8}",
        "Name", "StackID", "Total_Alloc", "Total_Freed", "Current", "Peak", "Allocs", "Frees");
    println!("---------------------------------------------------------------------------------------------");

    for (stack_id, mem_stat) in memory_stats.iter() {
        let function_name = match registered_functions.get(stack_id, 0) {
            Ok(function_name) => function_name.as_str(),
            Err(_) => "Unknown"
        };

        println!(
            "{: <15} {:>10} {:>12} {:>12} {:>12} {:>12} {:>8} {:>8}",
            function_name,
            stack_id,
            mem_stat.total_allocated,
            mem_stat.total_freed,
            mem_stat.current_usage,
            mem_stat.peak_usage,
            mem_stat.alloc_count,
            mem_stat.free_count
        );

        if ARGS.trace {
            match stack_traces.get(&(*stack_id as u32), 0) {
                Ok(stack_trace) => {
                    let mut symbols = vec![];
                    for frame in stack_trace.frames() {
                        if let Some(sym) = ksyms.range(..=frame.ip).next_back().map(|(_, s)| s) {
                            symbols.push((frame.ip, Some(sym)));
                        } else {
                            symbols.push((frame.ip, None));
                        }
                    }

                    let target = symbols
                        .iter()
                        .filter_map(|symbol| symbol.1)
                        .find_map(|symbol_name| FUNCTIONS.iter().find_map(|name| match *name == symbol_name {
                            true => Some(symbol_name.as_str()),
                            false => None
                        }))
                        .unwrap_or("Unknown");

                    println!("  Target: {target}");
                    println!("  Stack trace:");
                    for symbol in symbols {
                        match symbol.1 {
                            Some(symbol_name) => println!("\t{:#X} {}", symbol.0, symbol_name),
                            None => println!("\t{:#X?}", symbol.0)
                        }

                    }
                }
                Err(e) => {
                    println!("\t[Unable to retrieve stack trace: {e}]");
                }
            }
        }
    }

    Ok(all_allocations)
}