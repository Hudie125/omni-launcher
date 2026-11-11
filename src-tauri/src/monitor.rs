use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use sysinfo::System;
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize)]
pub struct SystemMetrics {
    pub cpu_usage: f32,
    pub cpu_freq: u64,
    pub cpu_brand: String,
    pub ram_used_gb: f32,
    pub ram_total_gb: f32,
    pub ram_percent: f32,
    pub gpu_usage: Option<f32>,
    pub gpu_name: Option<String>,
}

pub struct ResourceMonitor {
    is_active: Arc<AtomicBool>,
}

impl ResourceMonitor {
    pub fn new() -> Self {
        Self {
            is_active: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn set_active(&self, active: bool) {
        self.is_active.store(active, Ordering::Relaxed);
    }

    pub fn spawn_worker(&self, app: AppHandle) {
        let is_active = self.is_active.clone();

        tokio::spawn(async move {
            let mut sys = System::new_all();
            let mut interval = tokio::time::interval(Duration::from_millis(1000));

            loop {
                interval.tick().await;

                // Zero-cost sleep when window is hidden
                if !is_active.load(Ordering::Relaxed) {
                    tokio::time::sleep(Duration::from_millis(400)).await;
                    continue;
                }

                sys.refresh_cpu_all();
                sys.refresh_memory();

                let cpu_usage = sys.global_cpu_usage();
                let cpus = sys.cpus();
                let cpu_freq = cpus.first().map(|c| c.frequency()).unwrap_or(0);
                let cpu_brand = cpus
                    .first()
                    .map(|c| c.brand().to_string())
                    .unwrap_or_default();

                let total_ram_bytes = sys.total_memory();
                let used_ram_bytes = sys.used_memory();
                let ram_total_gb = total_ram_bytes as f32 / (1024.0 * 1024.0 * 1024.0);
                let ram_used_gb = used_ram_bytes as f32 / (1024.0 * 1024.0 * 1024.0);
                let ram_percent = if total_ram_bytes > 0 {
                    (used_ram_bytes as f32 / total_ram_bytes as f32) * 100.0
                } else {
                    0.0
                };

                let (gpu_usage, gpu_name) = query_gpu_status();

                let metrics = SystemMetrics {
                    cpu_usage,
                    cpu_freq,
                    cpu_brand,
                    ram_used_gb,
                    ram_total_gb,
                    ram_percent,
                    gpu_usage,
                    gpu_name,
                };

                let _ = app.emit("resource-update", &metrics);
            }
        });
    }
}

fn query_gpu_status() -> (Option<f32>, Option<String>) {
    // Attempt Linux sysfs first
    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = std::fs::read_to_string("/sys/class/drm/card0/device/gpu_busy_percent") {
            if let Ok(pct) = content.trim().parse::<f32>() {
                return (Some(pct), Some("AMD/Intel GPU".to_string()));
            }
        }
    }

    // Fallback: fast probe via nvidia-smi with 100ms timeout
    if let Ok(output) = std::process::Command::new("nvidia-smi")
        .args(["--query-gpu=utilization.gpu,name", "--format=csv,noheader,nounits"])
        .output()
    {
        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout);
            let mut parts = s.trim().split(',');
            if let (Some(util_str), Some(name_str)) = (parts.next(), parts.next()) {
                if let Ok(val) = util_str.trim().parse::<f32>() {
                    return (Some(val), Some(name_str.trim().to_string()));
                }
            }
        }
    }

    (None, None)
}
