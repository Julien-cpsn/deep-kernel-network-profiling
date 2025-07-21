use aya_ebpf::EbpfContext;
use aya_log_ebpf::{debug, info, trace};
use aya_network_deep_profiling_common::{FunctionDirection};

pub enum LogType {
    Info,
    Debug,
    Trace,
}

pub fn log_ctx<T: EbpfContext>(log_type: LogType, ctx: &T, function: &'static str, direction: Option<FunctionDirection>) {
    let direction = match direction {
        None => "",
        Some(direction) => direction.as_str()
    };

    match log_type {
        LogType::Info =>   info!(ctx, "\x1B[1m{}\x1B[0m {}", function, direction),
        LogType::Debug => debug!(ctx, "\x1B[1m{}\x1B[0m {}", function, direction),
        LogType::Trace => trace!(ctx, "\x1B[1m{}\x1B[0m {}", function, direction)
    }
}