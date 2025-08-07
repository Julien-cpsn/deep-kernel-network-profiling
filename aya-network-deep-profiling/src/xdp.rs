use aya::maps;
use aya::maps::MapData;
use rayon::prelude::*;
use aya_network_deep_profiling_common::{EthHeader, EtherHeaderType};

pub fn process_xdp(xdp_times: maps::HashMap<MapData, u64, EthHeader>, initial_time: u64) -> Vec<(u64, String)> {
    xdp_times.iter()
        .par_bridge()
        .filter_map(|x| x.ok())
        .map(|(time, eth_header)| {
            let new_time = time.saturating_sub(initial_time);

            let eth_type = match eth_header.ether_type {
                EtherHeaderType::Loop => "Loop",
                EtherHeaderType::Ipv4 => "IPv4",
                EtherHeaderType::Arp => "ARP",
                EtherHeaderType::Ipv6 => "IPv6",
                EtherHeaderType::FibreChannel => "FibreChannel",
                EtherHeaderType::Infiniband => "Infiniband",
                EtherHeaderType::LoopbackIeee8023 => "LoopbackIeee8023",
            };

            let src = format!("{:0>2X}:{:0>2X}:{:0>2X}:{:0>2X}:{:0>2X}:{:0>2X}", eth_header.src_addr[0], eth_header.src_addr[1], eth_header.src_addr[2], eth_header.src_addr[3], eth_header.src_addr[4], eth_header.src_addr[5]);
            let dst = format!("{:0>2X}:{:0>2X}:{:0>2X}:{:0>2X}:{:0>2X}:{:0>2X}", eth_header.dst_addr[0], eth_header.dst_addr[1], eth_header.dst_addr[2], eth_header.dst_addr[3], eth_header.dst_addr[4], eth_header.dst_addr[5]);
            let info = format!("{eth_type}, SRC: {src}, DST: {dst}");

            (new_time, info)
        })
        .collect()
}