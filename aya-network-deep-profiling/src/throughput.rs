use aya::maps::{MapData, Queue};
use getifaddrs::Interface;
use serde::Serialize;
use rayon::prelude::*;
use aya_network_deep_profiling_common::{PacketDirection, ThroughputStat};

#[derive(Serialize)]
pub struct ThroughputRow(pub u64, pub u32, pub PacketDirection, pub String);

pub fn collect_queue(throughput_stats: &mut Queue<MapData, ThroughputStat>, initial_time: u64) -> Vec<ThroughputStat> {
    let mut throughput: Vec<ThroughputStat> = Vec::new();

    while let Ok(throughput_stat) = throughput_stats.pop(0) {
        if throughput_stat.timestamp < initial_time {
            continue;
        }

        throughput.push(throughput_stat);
    }

    throughput
}


pub fn process_throughput(throughput_stats: Vec<ThroughputStat>, interfaces: Vec<Interface>, initial_time: u64) -> Vec<ThroughputRow> {
    throughput_stats
        .par_iter()
        .map(|throughput_stat| {
            let interface_name = interfaces
                .par_iter()
                .filter(|i| i.index.is_some())
                .find_first(|i| i.index.unwrap() == throughput_stat.if_index)
                .map(|interface| interface.name.clone())
                .unwrap_or_else(|| String::from("Unknown"));

            ThroughputRow(
                throughput_stat.timestamp.saturating_sub(initial_time),
                throughput_stat.packet_size,
                throughput_stat.direction,
                interface_name,
            )
        })
        .collect()
}