#![no_std]

use core::fmt;
use core::fmt::Debug;
use core::hash::Hash;

#[cfg(feature = "user")]
use serde::Serialize;

mod macros;

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "user", derive(Serialize))]
#[repr(C)]
pub struct AllocInfo {
    pub size: u64,
    pub alloc_type: AllocType,
    pub timestamp: u64,
    pub stack_id: i64,
    pub pid: u32,
}

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "user", derive(Serialize))]
#[repr(u8)]
pub enum AllocType {
    Malloc,
    Free
}

#[derive(Copy, Clone, Debug)]
pub struct FunctionWithDirection<F: Program>(pub F, pub FunctionDirection);

pub trait Program: Clone + Copy + Eq + PartialEq + Hash {
    fn to_str(self) -> &'static str;
}

enum_display! {
    #[allow(non_camel_case_types)]
    #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
    #[repr(u16)]
    pub enum Function {
        Kmalloc,
        Kfree,

        // Packet Reception
        virtnet_poll,// (driver-specific, virtio_net)
        napi_alloc_skb,
        napi_gro_receive,
        net_rx_action,
        __napi_poll,
        //handle_softirqs,
        //irq_exit_rcu,
        __common_interrupt,

        // Network Stack Processing
        netif_receive_skb_list_internal,
        __netif_receive_skb_list_core,
        ip_list_rcv,
        ip_sublist_rcv,
        ip_rcv,
        ip_rcv_core,
        ip_rcv_finish,

        // Routing Decision
        ip_route_input_noref,
        ip_route_input_slow,
        __fib_lookup,
        fib_table_lookup,

        // Forwarding Logic
        ip_forward,
        ip_forward_options,
        ip_send_check,
        //nf_hook,
        __icmp_send,
        icmp_push_reply,
        ip_append_data,
        ip_setup_cork_p_constprop_p_0,

        // Packet Transmission
        ip_output,
        ip_finish_output,
        ip_finish_output2,
        __dev_queue_xmit,
        __qdisc_run,

        //Optional/Conditional
        nf_hook_slow,
        ip_fragment_p_constprop_p_0,
        ip_options_compile,
        xdst_queue_output,
        sch_direct_xmit,
        netif_carrier_on,
        netif_carrier_off,
        netif_carrier_event,
        __alloc_skb
    }
}

enum_display! {
    #[allow(non_camel_case_types)]
    #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
    #[repr(u8)]
    pub enum Malloc {
        bpf_map_kmalloc_node,
        mempool_kmalloc,
        __traceiter_kmalloc,
        __probestub_kmalloc,
        kmalloc_size_roundup,
        free_large_kmalloc,
        ___kmalloc_large_node,
        __kmalloc_large_noprof,
        __kmalloc_large_node_noprof,
        __kmalloc_noprof,
        __kmalloc_node_track_caller_noprof,
        __kmalloc_cache_node_noprof,
        __kmalloc_node_noprof,
        __kmalloc_cache_noprof,
        bio_kmalloc,
        devm_kmalloc_match,
        devm_kmalloc_release,
        devm_kmalloc,
        sock_kmalloc,
        kmalloc_reserve,
        kmalloc_fix_flags
    }
}

enum_display! {
    #[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
    #[repr(u8)]
    pub enum FunctionDirection {
        Entry,
        Exit
    }
}

#[cfg(feature = "user")]
mod user {
    use super::*;
    use aya::Pod;

    unsafe impl Pod for AllocInfo {}
    unsafe impl Pod for Function {}
    unsafe impl Pod for FunctionDirection {}
    unsafe impl<F: Program + 'static> Pod for FunctionWithDirection<F> {}
}