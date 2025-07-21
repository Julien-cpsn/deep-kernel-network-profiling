use crate::profile_function;

profile_function!(
    ip_forward,
    ip_forward_options,
    ip_send_check,
    //nf_hook,
    __icmp_send,
    icmp_push_reply,
    ip_append_data,
    ip_setup_cork_p_constprop_p_0
);