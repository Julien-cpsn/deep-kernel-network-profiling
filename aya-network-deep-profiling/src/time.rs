use crate::utils::{mean, median, CPU_FREQUENCY};
use aya::maps::{MapData, PerCpuHashMap};
use aya_network_deep_profiling_common::{FunctionCall, FunctionDirection, Program};
use serde::Serialize;
use std::collections::HashMap;
use rayon::prelude::*;
use crate::ARGS;

#[derive(Debug, Clone, Serialize)]
pub struct ExecutionTimeRow {
    pub function_name: String,
    pub start_time: u64,
    pub end_time: u64,
    pub duration: u64,
    pub inner_duration: u64,
    pub depth: u32,
    pub cpuid: u32
}

const TIMEOUT: u64 = 60_000_000_000;

pub fn filter_times<F: Program + 'static>(times: PerCpuHashMap<MapData, u64, FunctionCall<F>>, initial_time: u64) -> Vec<(u64, FunctionCall<F>)> {
    let mut filtered_times: Vec<(u64, FunctionCall<F>)> = vec![];

    for (time, function_calls) in times.iter().filter_map(|t| t.ok()) {
        if time < initial_time {
            continue;
        }

        for function_call in function_calls.iter() {
            filtered_times.push((time, *function_call));
        }
    }

    filtered_times.sort_by(|(a, _), (b, _)| a.cmp(b));

    filtered_times
}

pub fn handle_execution_times<F: Program>(times: Vec<(u64, FunctionCall<F>)>, initial_time: u64) -> Vec<ExecutionTimeRow> {
    let mut arranged_times: HashMap<String, Vec<u64>> = HashMap::new();
    let mut execution_times: Vec<ExecutionTimeRow> = Vec::new();

    for (time, function_call) in times {
        let function = function_call.function.to_str().replace("_p_", ".");
        let direction = function_call.direction;
        let depth = function_call.depth;
        let cpuid = function_call.cpuid;

        match arranged_times.get_mut(&function) {
            Some(arranged_time) => match direction {
                FunctionDirection::Entry => arranged_time.push(time),
                FunctionDirection::Exit => {
                    let len = arranged_time.len() - 1;
                    let start_time = arranged_time[len];
                    let duration = time - arranged_time[len];

                    if duration > ARGS.max_time {
                        arranged_times.remove(&function);
                        continue;
                    }

                    execution_times.push(ExecutionTimeRow {
                        function_name: function.to_string(),
                        start_time: start_time.saturating_sub(initial_time),
                        end_time: time.saturating_sub(initial_time),
                        duration,
                        inner_duration: duration,
                        depth,
                        cpuid
                    });
                    arranged_time[len] = duration;
                }
            }
            None => match direction {
                FunctionDirection::Entry => {
                    arranged_times.insert(function, vec![time]);
                },
                FunctionDirection::Exit => {}
            }
        };
    }

    arranged_times.par_iter_mut().for_each(|(_, a)| a.retain(|e| *e < TIMEOUT));
    execution_times.retain(|e| e.duration < TIMEOUT);

    execution_times.sort_by_key(|row| (row.start_time, row.depth));

    for i in 0..execution_times.len() {
        let mut child_duration_sum = 0;
        let parent = &execution_times[i];
        let parent_start = parent.start_time + initial_time;
        let parent_end = parent.end_time + initial_time;
        let parent_depth = parent.depth;

        // Look for child calls (higher depth, within parent's time window)
        for (j, _) in execution_times.iter().enumerate() {
            if i == j {
                continue;
            }
            let candidate = &execution_times[j];
            if candidate.depth > parent_depth && candidate.start_time + initial_time >= parent_start && candidate.end_time + initial_time <= parent_end {
                child_duration_sum += candidate.duration;
            }
        }

        // Update inner_duration (ensure non-negative)
        execution_times[i].inner_duration = parent.duration.saturating_sub(child_duration_sum);
    }

    println!("============================================== Execution Time Statistics ==============================================");
    println!(
        "{: <35} {:>18} {:>18} {:>22} {:>22}",
        "Name", "Mean time", "Median time", "Mean cycles", "Median cycles"
    );
    println!("-----------------------------------------------------------------------------------------------------------------------");

    for (function, time) in arranged_times {
        print!("{function: <35} ");

        if time.is_empty() {
            println!();
            continue;
        }

        let mean_time = mean(&time);
        let median_time = median(&time);

        let mean_cycles = (mean_time as f64 * *CPU_FREQUENCY / 1_000_000_000.0) as u64;
        let median_cycles = (median_time as f64 * *CPU_FREQUENCY / 1_000_000_000.0) as u64;
        println!(
            "{: >15} ns {: >15} ns {: >15} cycles {: >15} cycles",
            mean_time.to_string(),
            median_time.to_string(),
            mean_cycles.to_string(),
            median_cycles.to_string()
        );
    }

    execution_times
}