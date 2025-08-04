use crate::XDP_TIMES;
use aya_ebpf::bindings::xdp_action::XDP_PASS;
use aya_ebpf::helpers::bpf_ktime_get_ns;
use aya_ebpf::macros::xdp;
use aya_ebpf::programs::XdpContext;
use aya_log_ebpf::info;
use aya_network_deep_profiling_common::{EthHeader};

#[xdp]
pub fn xdp_packet_log(ctx: XdpContext) -> u32 {
    match try_xdp_packet_log(ctx) {
        Ok(ret) => ret,
        Err(_) => XDP_PASS,
    }
}

#[inline(always)]
unsafe fn ptr_at<T>(ctx: &XdpContext, offset: usize) -> Result<*const T, ()> {
    let start = ctx.data();
    let end = ctx.data_end();
    let len = size_of::<T>();

    if start + offset + len > end {
        return Err(());
    }

    let ptr = (start + offset) as *const T;

    unsafe {
        Ok(&*ptr)
    }
}

fn try_xdp_packet_log(ctx: XdpContext) -> Result<u32, ()> {
    let eth_hdr: *const EthHeader = unsafe { ptr_at(&ctx, 0)? };
    let time = unsafe { bpf_ktime_get_ns() };

    unsafe {
        info!(
            &ctx,
            "New packet for {:X}:{:X}:{:X}:{:X}:{:X}:{:X}",
            (*eth_hdr).dst_addr[0],
            (*eth_hdr).dst_addr[1],
            (*eth_hdr).dst_addr[2],
            (*eth_hdr).dst_addr[3],
            (*eth_hdr).dst_addr[4],
            (*eth_hdr).dst_addr[5]
        );
    }

    XDP_TIMES.insert(&time, unsafe { &*eth_hdr }, 0).map_err(|_| ())?;

    Ok(XDP_PASS)
}