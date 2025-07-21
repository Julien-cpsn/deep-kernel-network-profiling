use crate::profile_function;

profile_function!(
    netif_receive_skb_list_internal,
    __netif_receive_skb_list_core,
    ip_list_rcv,
    ip_sublist_rcv,
    ip_rcv,
    ip_rcv_core,
    ip_rcv_finish
);