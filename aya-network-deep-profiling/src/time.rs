use crate::utils::{mean, median, CPU_FREQUENCY};
use aya_network_deep_profiling_common::{FunctionDirection, FunctionWithDirection, Program};
use std::collections::HashMap;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ExecutionTimeRow {
    pub function_name: String,
    pub start_time: u64,
    pub end_time: u64,
    pub duration: u64
}


pub fn handle_execution_times<F: Program>(times: Vec<(u64, FunctionWithDirection<F>)>, initial_time: u64) -> Vec<ExecutionTimeRow> {
    let mut arranged_times: HashMap<&str, Vec<u64>> = HashMap::new();
    let mut execution_times: Vec<ExecutionTimeRow> = Vec::new();

    for (time, function_with_direction) in times {
        let function = function_with_direction.0.to_str();
        let direction = function_with_direction.1;

        match arranged_times.get_mut(function) {
            Some(arranged_time) => match direction {
                FunctionDirection::Entry => arranged_time.push(time),
                FunctionDirection::Exit => {
                    let len = arranged_time.len() - 1;
                    let start_time = arranged_time[len];
                    let duration = time - arranged_time[len];

                    if duration > 1_000_000_000 {
                        arranged_times.remove(function);
                        continue;
                    }

                    execution_times.push(ExecutionTimeRow {
                        function_name: function.to_string(),
                        start_time: start_time.saturating_sub(initial_time),
                        end_time: time.saturating_sub(initial_time),
                        duration
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

    println!("============================================== Execution Time Statistics ==============================================");
    println!(
        "{: <35} {:>18} {:>18} {:>22} {:>22}",
        "Name", "Mean time", "Median time", "Mean cycles", "Median cycles"
    );
    println!("-----------------------------------------------------------------------------------------------------------------------");

    for (function, time) in arranged_times {
        let mean_time = mean(&time);
        let median_time = median(&time);

        let mean_cycles = (mean_time as f64 * *CPU_FREQUENCY / 1_000_000_000.0) as u64;
        let median_cycles = (median_time as f64 * *CPU_FREQUENCY / 1_000_000_000.0) as u64;
        println!(
            "{: <35} {: >15} ns {: >15} ns {: >15} cycles {: >15} cycles",
            function,
            mean_time.to_string(),
            median_time.to_string(),
            mean_cycles.to_string(),
            median_cycles.to_string()
        );
    }

    return execution_times;
}