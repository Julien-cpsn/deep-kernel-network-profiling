use crate::profile_function;

profile_function!(
    virtnet_poll,
    napi_alloc_skb,
    napi_gro_receive,
    net_rx_action,
    __napi_poll,
    //handle_softirqs,
    //irq_exit_rcu,
    __common_interrupt
);