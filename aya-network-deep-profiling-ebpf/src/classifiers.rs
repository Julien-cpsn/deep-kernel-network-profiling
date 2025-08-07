use aya_ebpf::bindings::TC_ACT_OK;
use aya_ebpf::helpers::bpf_ktime_get_ns;
use aya_ebpf::macros::classifier;
use aya_ebpf::programs::TcContext;
use aya_network_deep_profiling_common::{PacketDirection, ThroughputStat};
use crate::THROUGHPUT_STATS;

#[classifier]
pub fn tc_ingress(ctx: TcContext) -> i32 {
    try_tc_egress(ctx).unwrap_or_else(|_| TC_ACT_OK)
}

fn try_tc_ingress(ctx: TcContext) -> Result<i32, ()> {
    let time = unsafe { bpf_ktime_get_ns() };

    let stat = ThroughputStat {
        timestamp: time,
        packet_size: ctx.len(),
        direction: PacketDirection::Ingress,
        if_index: unsafe { (*ctx.skb.skb).ifindex },
    };

    THROUGHPUT_STATS.push(&stat, 0).map_err(|_| ())?;
    Ok(TC_ACT_OK)
}


#[classifier]
pub fn tc_egress(ctx: TcContext) -> i32 {
    try_tc_egress(ctx).unwrap_or_else(|_| TC_ACT_OK)
}

fn try_tc_egress(ctx: TcContext) -> Result<i32, ()> {
    let time = unsafe { bpf_ktime_get_ns() };

    let stat = ThroughputStat {
        timestamp: time,
        packet_size: ctx.len(),
        direction: PacketDirection::Egress,
        if_index: unsafe { (*ctx.skb.skb).ifindex },
    };

    THROUGHPUT_STATS.push(&stat, 0).map_err(|_| ())?;
    Ok(TC_ACT_OK)
}
