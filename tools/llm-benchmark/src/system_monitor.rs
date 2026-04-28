use crate::types::ThermalSample;
use std::{
    fs,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};
use sysinfo::{Pid, ProcessRefreshKind, RefreshKind, System};

#[derive(Default, Debug, Clone)]
pub struct RunningStats {
    pub cpu_samples: Vec<f32>,
    pub thermal_samples: Vec<ThermalSample>,
    pub memory_peak_kib: u64,
}

fn read_cpu_thermal_c() -> Option<f32> {
    let thermal_dir = Path::new("/sys/class/thermal");
    let entries = fs::read_dir(thermal_dir).ok()?;
    for entry in entries.flatten() {
        let p = entry.path();
        if !p.is_dir() {
            continue;
        }
        let type_path = p.join("type");
        let temp_path = p.join("temp");
        let zone_type = fs::read_to_string(type_path).ok()?.to_lowercase();
        if !zone_type.contains("cpu") && !zone_type.contains("x86_pkg_temp") {
            continue;
        }
        let raw = fs::read_to_string(temp_path).ok()?;
        if let Ok(v) = raw.trim().parse::<f32>() {
            return Some(v / 1000.0);
        }
    }
    None
}

pub fn monitor_hardware(
    alive: Arc<AtomicBool>,
    pid: Pid,
    sample_every: Duration,
    started_at: Instant,
) -> thread::JoinHandle<RunningStats> {
    thread::spawn(move || {
        let mut stats = RunningStats::default();
        let mut system = System::new_with_specifics(
            RefreshKind::new().with_processes(ProcessRefreshKind::new().with_memory()),
        );
        while alive.load(Ordering::Relaxed) {
            system.refresh_cpu_usage();
            system.refresh_processes_specifics(ProcessRefreshKind::new().with_memory());

            let cpu = system.global_cpu_info().cpu_usage();
            stats.cpu_samples.push(cpu);
            if let Some(c) = read_cpu_thermal_c() {
                stats.thermal_samples.push(ThermalSample {
                    timestamp_ms: started_at.elapsed().as_millis(),
                    celsius: Some(c),
                });
            } else {
                stats.thermal_samples.push(ThermalSample {
                    timestamp_ms: started_at.elapsed().as_millis(),
                    celsius: None,
                });
            }
            if let Some(process) = system.process(pid) {
                stats.memory_peak_kib = stats.memory_peak_kib.max(process.memory() / 1024);
            }
            thread::sleep(sample_every);
        }
        stats
    })
}

pub fn process_memory_kib(pid: Pid) -> u64 {
    let mut system = System::new_with_specifics(
        RefreshKind::new().with_processes(ProcessRefreshKind::new().with_memory()),
    );
    system.refresh_processes_specifics(ProcessRefreshKind::new().with_memory());
    system
        .process(pid)
        .map(|p| p.memory() / 1024)
        .unwrap_or_default()
}
