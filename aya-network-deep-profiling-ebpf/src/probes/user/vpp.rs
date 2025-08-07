use crate::profile_function;

profile_function!(User, u,
    dpdk_input_node_fn_hsw,
    dpdk_input_node_fn_icl,
    dpdk_input_node_fn_skx,
    crypto_dispatch_node_fn,
    crypto_dispatch_node_fn_hsw,
    crypto_dispatch_node_fn_icl,
    crypto_dispatch_node_fn_skx,
    bier_disp_dispatch_node_fn,
    bier_disp_dispatch_node_fn_hsw,
    bier_disp_dispatch_node_fn_icl,
    bier_disp_dispatch_node_fn_skx,
    punt_dispatch_node_fn,
    punt_dispatch_node_fn_hsw,
    punt_dispatch_node_fn_icl,
    punt_dispatch_node_fn_skx
);