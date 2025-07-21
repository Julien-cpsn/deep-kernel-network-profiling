use once_cell::sync::Lazy;
use std::fs;

pub fn mean(list: &[u64]) -> u64 {
    let sum: u64 = Iterator::sum(list.iter());
    sum.div_euclid(list.len() as u64)
}

pub fn median(list: &[u64]) -> u64 {
    let len = list.len();
    let mid = len / 2;
    if len % 2 == 0 {
        mean(&list[(mid - 1)..(mid + 1)])
    } else {
        list[mid]
    }
}

pub static CPU_FREQUENCY: Lazy<f64> = Lazy::new(|| {
    let cpuinfo = fs::read_to_string("/proc/cpuinfo").unwrap();
    for line in cpuinfo.lines() {
        if line.starts_with("cpu MHz") {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() == 2 {
                match parts[1].trim().parse::<f64>() {
                    Ok(value) => return value * 1_000_000.0,
                    Err(_) => break
                }
            }
        }
    }

    // Default value
    2_000_000_000.0
});