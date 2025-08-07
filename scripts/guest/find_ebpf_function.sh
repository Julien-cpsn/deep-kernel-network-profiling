cat /sys/kernel/debug/tracing/available_filter_functions | grep "$1" | sort

# ex: ./find_ebpf_function.sh malloc