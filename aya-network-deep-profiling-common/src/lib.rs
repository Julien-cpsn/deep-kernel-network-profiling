#![no_std]
#![allow(unused_variables)]
#![allow(unused_attributes)]
#![allow(unused_doc_comments)]
#![allow(unreachable_code)]
#![allow(unreachable_patterns)]

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
    Alloc,
    Free
}

#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct FunctionCall<F: Program> {
    pub function: F,
    pub direction: FunctionDirection,
    pub depth: u32,
    pub cpuid: u32,
}

enum_display! {
    #[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
    #[repr(u8)]
    pub enum FunctionDirection {
        Entry,
        Exit
    }
}

pub trait Program: Clone + Copy + Eq + PartialEq + Hash {
    fn to_str(self) -> &'static str;
}

enum_display! {
    #[allow(non_camel_case_types)]
    #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
    #[repr(u16)]
    pub enum KernelFunction {
        /* ===== Physical layer processing ===== */

        /* --- Packet Reception --- */
        /// Manages softirq processing, including network-related softirqs (e.g., NET_RX_SOFTIRQ).
        //handle_softirqs,
        /// Exits an interrupt context with RCU (Read-Copy-Update) handling.
        //irq_exit_rcu,
        //__common_interrupt,
        /// Polling function for virtual network devices (e.g., virtio-net).
        //virtnet_poll,
        /// Polling function for E1000 network devices.
        e1000_netpoll,
        /// Allocates socket buffers during NAPI polling.
        //napi_alloc_skb,
        /// Processes packets in the softirq context, invoked by the NAPI polling mechanism.
        net_rx_action,
        __napi_poll,
        /// Entry point for packets received from the network interface.
        /// It handles initial processing, such as protocol identification, and decides whether to pass the packet to the local stack or forward it.
        __netif_receive_skb,
        /// A wrapper around __netif_receive_skb, used by network drivers to deliver packets to the kernel's network stack.
        netif_receive_skb,
        /// Handle initial packet processing. Useful for fine-grained analysis of receive path bottlenecks.
        netif_receive_skb_core,
        __netif_receive_skb_core_p_constprop_p_0,
        /// Handles Generic Receive Offload (GRO), aggregating packets to improve performance before passing them to the stack.
        napi_gro_receive,
        /// Marks the completion of NAPI polling and re-enables interrupts. This is critical for understanding NAPI performance and interrupt handling.
        napi_complete_done,
        /// Handles non-aggregated GRO packets, complementing napi_gro_receive for cases where GRO doesn’t aggregate packets.
        skb_gro_receive_list,

        /* ===== Link layer processing ===== */

        /// Extracts the Ethernet protocol type from the packet and sets the skb (socket buffer) protocol field, typically called for Ethernet frames.
        eth_type_trans,
        /// Constructs Ethernet headers for outgoing packets. Useful if you’re analyzing link-layer header processing.
        eth_header,
        /// Queues packets for transmission on the output interface, used when a packet is being forwarded or sent from the local system.
        __dev_queue_xmit,
        /// Handles packet sniffing (e.g., for tcpdump) before transmission. If you’re profiling packet capture or monitoring tools, this is relevant.
        dev_queue_xmit_nit,

        /* ===== Network layer Processing (IP) ===== */

        ip_list_rcv,
        ip_sublist_rcv,
        /// Entry point for IPv4 packets. Performs initial validation (e.g., checksum, header sanity checks) and passes the packet to higher layers or forwarding logic.
        ip_rcv,
        /// Core processing for IPv4 packet reception (checksums, header validation).
        ip_rcv_core,
        /// Completes IPv4 packet reception, calling routing or local delivery.
        ip_rcv_finish,
        /// Completes the reception of IPv4 packets after initial validation in ip_rcv. It’s a key part of the receive path.
        ip_rcv_finish_core_p_isra_p_0,
        /// Handles local delivery of IP packets to the transport layer.
        //ip_input,
        /// Delivers packets to the local host after routing decisions. This is critical for profiling packets destined for the local system.
        ip_local_deliver,
        /// Handles multicast routing for IP packets. Include this if you’re profiling multicast traffic.
        ip_mr_input,
        /// Sends an IP packet after preparation, bridging higher-layer protocols to the output path.
        ip_send_skb,

        /* --- Routing Decision --- */
        ip_route_input_rcu_p_part_p_0,
        ip_route_input_noref,
        ip_route_input_slow,
        __fib_lookup,
        fib_table_lookup,
        /// Queues packets for output based on routing decisions.
        xdst_queue_output,

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
        /// Processes TCP packets after initial checks in tcp_v4_rcv. Essential for detailed TCP stack profiling.
        tcp_v4_do_rcv,
        /// Queues UDP packets to the socket, complementing udp_rcv for UDP delivery.
        udp_queue_rcv_skb,
        /// Transmits a TCP segment, handling segmentation and checksums.
        __tcp_transmit_skb,
        /// Sends a UDP datagram, preparing it for transmission.
        udp_send_skb,

        /* --- Socket Layer --- */
        /// Queues packets to a socket’s receive queue.
        skb_queue_tail,
        /// A wrapper around __sock_queue_rcv_skb for socket delivery. Including both might help isolate socket queuing overheads.
        vsock_queue_rcv_skb,
        /// Delivers packets to the appropriate socket for user-space applications.
        __sock_queue_rcv_skb,

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
        /// Handles transmission of locally generated IP packets (e.g., from sockets). This complements ip_output for locally originated traffic.
        ip_queue_xmit,
        /// Transmits a packet on the hardware
        //start_xmit,
        netpoll_start_xmit,
        /// Processes packets in the output queue of a network device, ensuring proper scheduling and traffic control (e.g., QoS policies).
        __qdisc_run,
        /// Attempts to directly transmit packets without queuing if the device is ready, optimizing performance.
        /// It’s called during the transmission path, often in conjunction with __qdisc_run
        sch_direct_xmit,
        /// Resolves neighbor (ARP) entries for packet transmission. Critical for link-layer address resolution.
        neigh_resolve_output,

        /* ===== Others ===== */
        /* --- Filtering --- */
        /// Schedules packets for processing in the softirq context (used in older drivers or specific cases)
        netif_rx,
        /// Handles packets received in non-interrupt contexts (e.g., loopback or virtual interfaces). Useful for profiling non-standard interfaces.
        netif_rx_internal,
        /// Invokes Netfilter hooks for packet filtering, NAT, or other processing (e.g., iptables rules).
        nf_hook_slow,
        /// Executes iptables rules for packet filtering. Include this if you’re profiling firewall performance.
        ipt_do_table,

        /* --- Optional/Conditional --- */
        /// parses and compiles IP options from the packet header
        ip_options_compile,

        /// Handles IP packet fragmentation.
        ip_fragment_p_constprop_p_0,
        /*
        netif_carrier_on,
        netif_carrier_off,
        netif_carrier_event,
        */

        /* --- XDP --- */
        /// Processes packets in a software-based XDP layer before passing them to the network stack.
        do_xdp_generic,
        /// Handles the XDP_REDIRECT action, redirecting the packet to another interface or CPU using an eBPF map
        xdp_do_redirect,

        /* --- BPF --- */
        bpf_xdp_redirect,
        bpf_msg_redirect_map,
        bpf_sk_redirect_map,
        bpf_xdp_redirect_map
    }
}

enum_display! {
    #[allow(non_camel_case_types)]
    #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
    #[cfg_attr(any(feature = "dpdk", feature = "vpp"), repr(u16))]
    pub enum UserFunction {
        /* ===== DPDK ===== */
        /// Receives a burst of packets from a specified RX queue into an array of mbufs.
        #[cfg(feature = "dpdk")]
        rte_eth_rx_burst_mode_get => "/usr/lib/x86_64-linux-gnu/librte_ethdev.so",
        #[cfg(feature = "dpdk")]
        rte_malloc => "/usr/lib/x86_64-linux-gnu/librte_eal.so",

        /* ===== VPP ===== */

        #[cfg(feature = "vpp")]
        dpdk_input_node_fn_hsw => "/usr/lib/x86_64-linux-gnu/vpp_plugins/dpdk_plugin.so",
        #[cfg(feature = "vpp")]
        dpdk_input_node_fn_icl => "/usr/lib/x86_64-linux-gnu/vpp_plugins/dpdk_plugin.so",
        #[cfg(feature = "vpp")]
        dpdk_input_node_fn_skx => "/usr/lib/x86_64-linux-gnu/vpp_plugins/dpdk_plugin.so",

        #[cfg(feature = "vpp")]
        crypto_dispatch_node_fn => "/usr/lib/x86_64-linux-gnu/libvnet.so.25.06",
        #[cfg(feature = "vpp")]
        crypto_dispatch_node_fn_hsw => "/usr/lib/x86_64-linux-gnu/libvnet.so.25.06",
        #[cfg(feature = "vpp")]
        crypto_dispatch_node_fn_icl => "/usr/lib/x86_64-linux-gnu/libvnet.so.25.06",
        #[cfg(feature = "vpp")]
        crypto_dispatch_node_fn_skx => "/usr/lib/x86_64-linux-gnu/libvnet.so.25.06",

        #[cfg(feature = "vpp")]
        bier_disp_dispatch_node_fn => "/usr/lib/x86_64-linux-gnu/libvnet.so.25.06",
        #[cfg(feature = "vpp")]
        bier_disp_dispatch_node_fn_hsw => "/usr/lib/x86_64-linux-gnu/libvnet.so.25.06",
        #[cfg(feature = "vpp")]
        bier_disp_dispatch_node_fn_icl => "/usr/lib/x86_64-linux-gnu/libvnet.so.25.06",
        #[cfg(feature = "vpp")]
        bier_disp_dispatch_node_fn_skx => "/usr/lib/x86_64-linux-gnu/libvnet.so.25.06",

        #[cfg(feature = "vpp")]
        punt_dispatch_node_fn => "/usr/lib/x86_64-linux-gnu/libvlib.so.25.06",
        #[cfg(feature = "vpp")]
        punt_dispatch_node_fn_hsw => "/usr/lib/x86_64-linux-gnu/libvlib.so.25.06",
        #[cfg(feature = "vpp")]
        punt_dispatch_node_fn_icl => "/usr/lib/x86_64-linux-gnu/libvlib.so.25.06",
        #[cfg(feature = "vpp")]
        punt_dispatch_node_fn_skx => "/usr/lib/x86_64-linux-gnu/libvlib.so.25.06",
    }
}

enum_display! {
    #[allow(non_camel_case_types)]
    #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
    #[repr(u8)]
    pub enum Alloc {
        // kmalloc
        bio_kmalloc,
        devm_kmalloc,
        devm_kmalloc_match,
        devm_kmalloc_release,
        free_large_kmalloc,
        kmalloc_fix_flags,
        kmalloc_reserve,
        kmalloc_size_roundup,
        mempool_kmalloc,
        sock_kmalloc,
        ___kmalloc_large_node,
        __kmalloc_cache_node_noprof,
        __kmalloc_cache_noprof,
        __kmalloc_large_node_noprof,
        __kmalloc_large_noprof,
        __kmalloc_node_noprof,
        __kmalloc_node_track_caller_noprof,
        __kmalloc_noprof,

        // kfree
        dev_kfree_skb_any_reason,
        dev_kfree_skb_irq_reason,
        kfree,
        kfree_skb_list_reason,
        kfree_skb_partial,
        kfree_skbmem,
        __kfree_skb,
        __napi_kfree_skb,

        // kmem_cache
        do_kmem_cache_create,
        kmem_cache_alloc_bulk_noprof,
        kmem_cache_alloc_lru_noprof,
        kmem_cache_alloc_node_noprof,
        kmem_cache_alloc_noprof,
        kmem_cache_charge,
        kmem_cache_destroy,
        kmem_cache_flags,
        kmem_cache_free,
        kmem_cache_free_bulk,
        kmem_cache_free_bulk_p_part_p_0,
        kmem_cache_release,
        kmem_cache_shrink,
        kmem_cache_size,
        __kmem_cache_do_shrink,
        __kmem_cache_empty,
        __kmem_cache_release,
        __kmem_cache_shrink,
        __kmem_cache_shutdown,
        /*
        /// Allocates socket buffers for packet processing.
        __alloc_skb,
        /// Allocates socket buffers during NAPI polling.
        napi_alloc_skb,
        /// Allocates socket buffers for packet reception. Complements __alloc_skb for memory profiling.
        dev_alloc_skb,
        /// Creates copies or clones of packets for processing (e.g., for forwarding or multiple listeners).
        skb_clone,
        skb_copy,
        kfree_skb,
        /// Frees socket buffers for datagram protocols (e.g., UDP). Useful for memory management profiling.
        skb_free_datagram,
        /// Copies packet data to user space (e.g., for socket reads). Relevant for socket performance.
        skb_copy_datagram_iter
        /// Frees socket buffers after processing. Complements kfree_skb for memory profiling.
        consume_skb
         */
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

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "user", derive(Serialize))]
#[repr(C, packed)]
pub struct ThroughputStat {
    pub timestamp: u64,
    pub packet_size: u32,
    pub direction: PacketDirection,
    pub if_index: u32
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "user", derive(Serialize))]
#[repr(u8)]
pub enum PacketDirection {
    Ingress,
    Egress,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C, packed)]
pub struct EthHeader {
    pub dst_addr: [u8; 6],
    pub src_addr: [u8; 6],
    pub ether_type: EtherHeaderType,
}

#[repr(u16)]
#[derive(Hash, PartialEq, Eq, Debug, Copy, Clone)]
pub enum EtherHeaderType {
    Loop = 0x0060_u16.to_be(),
    Ipv4 = 0x0800_u16.to_be(),
    Arp = 0x0806_u16.to_be(),
    Ipv6 = 0x86DD_u16.to_be(),
    FibreChannel = 0x8906_u16.to_be(),
    Infiniband = 0x8915_u16.to_be(),
    LoopbackIeee8023 = 0x9000_u16.to_be(),
}

#[cfg(feature = "user")]
mod user {
    use super::*;
    use aya::Pod;

    unsafe impl Pod for AllocInfo {}
    unsafe impl Pod for KernelFunction {}
    unsafe impl Pod for UserFunction {}
    unsafe impl Pod for Alloc {}
    unsafe impl Pod for FunctionDirection {}
    unsafe impl<F: Program + 'static> Pod for FunctionCall<F> {}
    unsafe impl Pod for ThroughputStat {}
    unsafe impl Pod for PacketDirection {}
    unsafe impl Pod for EthHeader {}


    unsafe impl Send for AllocInfo {}
    unsafe impl Sync for AllocInfo {}

    unsafe impl Send for KernelFunction {}
    unsafe impl Sync for KernelFunction {}

    unsafe impl Send for UserFunction {}
    unsafe impl Sync for UserFunction {}

    unsafe impl Send for Alloc {}
    unsafe impl Sync for Alloc {}

    unsafe impl Send for FunctionDirection {}
    unsafe impl Sync for FunctionDirection {}

    unsafe impl<F: Program + Send + Sync> Send for FunctionCall<F> {}
    unsafe impl<F: Program + Send + Sync> Sync for FunctionCall<F> {}

    unsafe impl Send for ThroughputStat {}
    unsafe impl Sync for ThroughputStat {}

    unsafe impl Send for PacketDirection {}
    unsafe impl Sync for PacketDirection {}

    unsafe impl Send for EthHeader {}
    unsafe impl Sync for EthHeader {}

    unsafe impl Send for EtherHeaderType {}
    unsafe impl Sync for EtherHeaderType {}

}