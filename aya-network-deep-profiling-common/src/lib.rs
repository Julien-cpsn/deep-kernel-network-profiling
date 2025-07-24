#![no_std]

use core::fmt;
use core::fmt::Debug;
use core::hash::Hash;
#[cfg(feature = "user")]
use serde::Serialize;

mod macros;

pub const STRING_AS_BYTES_MAX_LEN: usize = 50;

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "user", derive(Serialize))]
#[repr(C)]
pub struct AllocInfo {
    pub alloc_type: AllocType,
    pub alloc_direction: AllocDirection,
    pub size: u64,
    pub timestamp: u64,
    pub stack_id: i64,
    pub pid: u32,
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "user", derive(Serialize))]
#[repr(u8)]
pub enum AllocType {
    kmalloc,
    kmem_cache
}

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "user", derive(Serialize))]
#[repr(u8)]
pub enum AllocDirection {
    Malloc,
    Free
}

#[derive(Copy, Clone, Debug)]
pub struct FunctionCall<F: Program> {
    pub function: F,
    pub direction: FunctionDirection,
    pub depth: u32,
    pub cpuid: u32,
}

pub trait Program: Clone + Copy + Eq + PartialEq + Hash {
    fn to_str(self) -> &'static str;
}

enum_display! {
    #[allow(non_camel_case_types)]
    #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
    #[repr(u8)]
    pub enum Function {
        /* ===== Physical layer processing ===== */

        /* --- Packet Reception --- */
        /// Processes packets in the softirq context, invoked by the NAPI polling mechanism.
        net_rx_action,
        __napi_poll,
        //handle_softirqs,
        //irq_exit_rcu,
        //__common_interrupt,
        //virtnet_poll,
        //napi_alloc_skb,
        /// Entry point for packets received from the network interface.
        /// It handles initial processing, such as protocol identification, and decides whether to pass the packet to the local stack or forward it.
        __netif_receive_skb,
        /// A wrapper around __netif_receive_skb, used by network drivers to deliver packets to the kernel's network stack.
        netif_receive_skb,
        /// Handles Generic Receive Offload (GRO), aggregating packets to improve performance before passing them to the stack.
        napi_gro_receive,

        /* ===== Link layer processing ===== */

        /// Extracts the Ethernet protocol type from the packet and sets the skb (socket buffer) protocol field, typically called for Ethernet frames.
        eth_type_trans,
        /// Queues packets for transmission on the output interface, used when a packet is being forwarded or sent from the local system.
        __dev_queue_xmit,

        /* ===== Network layer Processing (IP) ===== */

        ip_list_rcv,
        ip_sublist_rcv,
        /// Entry point for IPv4 packets. Performs initial validation (e.g., checksum, header sanity checks) and passes the packet to higher layers or forwarding logic.
        ip_rcv,
        //ip_rcv_core,
        //ip_rcv_finish,
        /// Handles local delivery of IP packets destined for the host.
        //ip_input,

        /* --- Routing Decision --- */
        ip_route_input_rcu_p_part_p_0,
        ip_route_input_noref,
        ip_route_input_slow,
        __fib_lookup,
        fib_table_lookup,

        /* --- Forwarding Logic --- */
        /// Manages forwarding of packets not destined for the local host, applying routing decisions.
        ip_forward,
        /// Processes IP header options (e.g., source routing) when a packet is being forwarded to another host
        ip_forward_options,
        /// Computes and sets the IP checksum for outgoing packets
        ip_send_check,
        /// Sends ICMP messages, such as error responses (e.g., Destination Unreachable, Time Exceeded) or replies (e.g., for ping).
        __icmp_send,
        /// Handles the preparation and queuing of ICMP replies for transmission, typically called after processing an incoming ICMP request. It’s part of the ICMP protocol handling in the kernel.
        icmp_push_reply,
        /// Works with the socket buffer (skb) to prepare the packet payload.
        ip_append_data,
        /// Delays packet transmission to allow multiple writes to be combined into a single packet, improving efficiency (e.g., for UDP or raw sockets)
        ip_setup_cork_p_constprop_p_0,

        /* ===== Transport layer processing ===== */

        /// Entry points for TCP and UDP packets, respectively.
        /// These functions handle protocol-specific processing, such as TCP state machine updates or UDP socket delivery.
        tcp_v4_rcv,
        udp_rcv,
        /// Transmits a TCP segment, handling segmentation and checksums.
        __tcp_transmit_skb,
        /// Sends a UDP datagram, preparing it for transmission.
        udp_send_skb,

        /* --- Socket Layer Delivery --- */
        /// Queues packets to a socket’s receive queue.
        skb_queue_tail,
        /// Delivers packets to the appropriate socket for user-space applications.
        __sock_queue_rcv_skb,
        /*
        /// Creates copies or clones of packets for processing (e.g., for forwarding or multiple listeners).
        skb_clone,
        skb_copy,
        kfree_skb,
        */

        /* --- Packet Transmission --- */
        /// Sends the packet out through the network interface, called after queuing via dev_queue_xmit
        dev_hard_start_xmit,
        /// Manages the transmit queue state of a network device.
        //netif_start_queue,
        //netif_stop_queue,

        /// Handles the transmission of IP packets, whether locally generated or forwarded.
        /// It performs routing decisions and passes the packet to lower layers (e.g., ip_finish_output).
        ip_output,
        /// Handles tasks like fragmentation (if needed) and passing the packet to the link layer for transmission
        ip_finish_output,
        /// Deals with the final steps of packet transmission, such as resolving the next-hop device and invoking dev_queue_xmit to send the packet to the network device.
        ip_finish_output2,
        /// Transmits a packet on the hardware
        start_xmit,
        /// Processes packets in the output queue of a network device, ensuring proper scheduling and traffic control (e.g., QoS policies).
        __qdisc_run,
        /// Attempts to directly transmit packets without queuing if the device is ready, optimizing performance.
        /// It’s called during the transmission path, often in conjunction with __qdisc_run
        sch_direct_xmit,

        /* --- Filtering --- */
        /// Schedules packets for processing in the softirq context (used in older drivers or specific cases)
        netif_rx,
        /// Invokes Netfilter hooks for packet filtering, NAT, or other processing (e.g., iptables rules).
        nf_hook_slow,

        /* --- Optional/Conditional --- */
        /// parses and compiles IP options from the packet header
        ip_options_compile,
        /*
        ip_fragment_p_constprop_p_0,
        xdst_queue_output,
        netif_carrier_on,
        netif_carrier_off,
        netif_carrier_event,
        __alloc_skb*/
    }
}

enum_display! {
    #[allow(non_camel_case_types)]
    #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
    #[repr(u8)]
    pub enum Alloc {
        // Kmalloc
        bpf_map_kmalloc_node,
        mempool_kmalloc,
        //__traceiter_kmalloc,
        //__probestub_kmalloc,
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

        // KFree
        kfree,
        //__traceiter_kfree,
        //__probestub_kfree,
        kfree_skbmem,
        kfree_skb_list_reason,
        __kfree_skb,
        kfree_skb_partial,
        __napi_kfree_skb,
        dev_kfree_skb_irq_reason,
        dev_kfree_skb_any_reason,
        //__traceiter_kfree_skb,
        //__probestub_kfree_skb
    }
}

enum_display! {
    #[allow(non_camel_case_types)]
    #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
    #[repr(u8)]
    pub enum Tracepoint {
        kmalloc,
        kfree,
        kmem_cache_alloc,
        kmem_cache_free
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
    unsafe impl<F: Program + 'static> Pod for FunctionCall<F> {}
}