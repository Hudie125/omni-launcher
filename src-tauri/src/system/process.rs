use serde::{Deserialize, Serialize};
use sysinfo::{Pid, ProcessesToUpdate, System};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessItem {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_mb: u64,
}

pub fn list_running_processes(filter: &str) -> Vec<ProcessItem> {
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All, true);

    let filter_lower = filter.trim().to_lowercase();
    let mut list = Vec::new();

    for (pid, proc_) in sys.processes() {
        let name = proc_.name().to_string_lossy().to_string();
        if filter_lower.is_empty() || name.to_lowercase().contains(&filter_lower) {
            list.push(ProcessItem {
                pid: pid.as_u32(),
                name,
                cpu_usage: proc_.cpu_usage(),
                memory_mb: proc_.memory() / (1024 * 1024),
            });
        }
    }

    // Sort by memory usage descending
    list.sort_by(|a, b| b.memory_mb.cmp(&a.memory_mb));
    list.into_iter().take(20).collect()
}

pub fn kill_process_by_name(name: &str) -> Result<usize, String> {
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All, true);

    let target = name.trim().to_lowercase();
    let mut killed = 0;

    for (_, proc_) in sys.processes() {
        let p_name = proc_.name().to_string_lossy().to_lowercase();
        if p_name == target || p_name == format!("{}.exe", target) {
            if proc_.kill() {
                killed += 1;
            }
        }
    }

    if killed > 0 {
        Ok(killed)
    } else {
        Err(format!("Process '{}' not found", name))
    }
}

pub fn kill_process_by_pid(pid_u32: u32) -> Result<(), String> {
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All, true);

    let pid = Pid::from_u32(pid_u32);
    if let Some(proc_) = sys.process(pid) {
        if proc_.kill() {
            Ok(())
        } else {
            Err(format!("Failed to kill PID {}", pid_u32))
        }
    } else {
        Err(format!("PID {} not found", pid_u32))
    }
}
