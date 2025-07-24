use crate::profile_function;

profile_function!(
    tcp_v4_rcv,
    udp_rcv,
    __tcp_transmit_skb,
    udp_send_skb,
    skb_queue_tail,
    __sock_queue_rcv_skb,
    dev_hard_start_xmit,
    ip_output,
    ip_finish_output,
    ip_finish_output2,
    start_xmit,
    __qdisc_run,
    sch_direct_xmit
);