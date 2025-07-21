use aya_ebpf::EbpfContext;
use aya_ebpf::helpers::bpf_get_smp_processor_id;
use crate::STACK_TRACES;

pub struct FullContext<T: EbpfContext> {
    pub uid: u32,
    pub gid: u32,
    pub tgid: u32,
    pub pid: u32,
    pub cpuid: u32,
    pub stack_id: i64,
    pub ctx: T,
}


pub fn get_full_ctx<T: EbpfContext>(ctx: T) -> Result<FullContext<T>, u32> {
    let uid = ctx.uid();
    let gid = ctx.gid();
    let tgid = ctx.tgid();
    let pid = ctx.pid();
    let cpuid = unsafe { bpf_get_smp_processor_id() } as u32;
    let stack_id = match unsafe { STACK_TRACES.get_stackid(&ctx, 0) } {
        Ok(stack_id) => stack_id,
        _ => return Err(0),
    };

    Ok(FullContext {
        uid,
        gid,
        tgid,
        pid,
        cpuid,
        stack_id,
        ctx,
    })
}