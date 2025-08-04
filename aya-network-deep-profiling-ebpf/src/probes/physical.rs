use crate::profile_function;

profile_function!(
    net_rx_action,
    __napi_poll,
    __netif_receive_skb,
    netif_receive_skb,
    netif_receive_skb_core,
    __netif_receive_skb_core_p_constprop_p_0,
    napi_gro_receive,
    napi_complete_done,
    skb_gro_receive_list,
);