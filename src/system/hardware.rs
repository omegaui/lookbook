//! Reads basic hardware facts from /proc. Inside Flatpak, /proc shows the host.

use std::fs;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Hardware {
    /// For example "6.17.2-arch1-1".
    pub kernel: String,
    pub cpu_model: String,
    pub cpu_threads: usize,
    pub memory_bytes: u64,
}

impl Hardware {
    pub fn detect() -> Self {
        let cpuinfo = read_or_empty("/proc/cpuinfo");
        let meminfo = read_or_empty("/proc/meminfo");

        Self {
            kernel: read_or_empty("/proc/sys/kernel/osrelease").trim().to_owned(),
            cpu_model: parse_cpu_model(&cpuinfo),
            cpu_threads: count_cpu_threads(&cpuinfo),
            memory_bytes: parse_memory_bytes(&meminfo),
        }
    }
}

fn read_or_empty(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_default()
}

/// The first "model name" line of /proc/cpuinfo.
fn parse_cpu_model(cpuinfo: &str) -> String {
    cpuinfo
        .lines()
        .find(|line| line.starts_with("model name"))
        .and_then(|line| line.split_once(':'))
        .map(|(_, model)| model.trim().to_owned())
        .unwrap_or_default()
}

/// /proc/cpuinfo has one "processor" entry per hardware thread.
fn count_cpu_threads(cpuinfo: &str) -> usize {
    cpuinfo
        .lines()
        .filter(|line| line.starts_with("processor"))
        .count()
}

/// /proc/meminfo gives "MemTotal:  32559348 kB".
fn parse_memory_bytes(meminfo: &str) -> u64 {
    let kilobytes = meminfo
        .lines()
        .find(|line| line.starts_with("MemTotal:"))
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or_default();

    kilobytes * 1024
}

#[cfg(test)]
mod tests {
    use super::*;

    const CPUINFO: &str = "processor\t: 0\nmodel name\t: AMD Ryzen 7 7840U\n\nprocessor\t: 1\nmodel name\t: AMD Ryzen 7 7840U\n";

    #[test]
    fn reads_cpu_model_and_threads() {
        assert_eq!(parse_cpu_model(CPUINFO), "AMD Ryzen 7 7840U");
        assert_eq!(count_cpu_threads(CPUINFO), 2);
    }

    #[test]
    fn reads_total_memory() {
        let meminfo = "MemTotal:       32559348 kB\nMemFree:         1000 kB\n";
        assert_eq!(parse_memory_bytes(meminfo), 32_559_348 * 1024);
    }
}
