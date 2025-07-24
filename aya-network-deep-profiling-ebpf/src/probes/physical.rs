use crate::profile_function;

profile_function!(
    net_rx_action,
    __napi_poll,
    __netif_receive_skb,
    netif_receive_skb,
    napi_gro_receive
);