#[derive(Copy, Clone, Debug)]
pub struct MemStat {
    pub total_allocated: u64,
    pub total_freed: u64,
    pub current_usage: i64,
    pub peak_usage: u64,
    pub alloc_count: u32,
    pub free_count: u32,
}
