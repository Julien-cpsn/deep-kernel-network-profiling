use crate::profile_function;

profile_function!(
    ip_route_input_noref,
    ip_route_input_slow,
    __fib_lookup,
    fib_table_lookup
);