use crate::profile_function;

profile_function!(Kernel, k,
    netif_rx,
    netif_rx_internal,
    nf_hook_slow,
    ipt_do_table,
    ip_options_compile,
    ip_fragment_p_constprop_p_0,
    xdp_do_redirect,
    do_xdp_generic,
    bpf_xdp_redirect,
    bpf_msg_redirect_map,
    bpf_sk_redirect_map,
    bpf_xdp_redirect_map
);