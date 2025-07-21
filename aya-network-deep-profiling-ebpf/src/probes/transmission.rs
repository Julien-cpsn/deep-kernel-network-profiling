use crate::profile_function;

profile_function!(
    ip_output,
    ip_finish_output,
    ip_finish_output2,
    __dev_queue_xmit,
    __qdisc_run
);