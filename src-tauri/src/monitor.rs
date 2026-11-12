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
    pub gpu_temp: Option<f32>,
    pub gpu_name: Option<String>,
    pub battery: Option<String>,
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

        std::thread::spawn(move || {
            let mut sys = System::new_all();

            loop {
                // Zero-cost sleep when window is hidden
                if !is_active.load(Ordering::Relaxed) {
                    std::thread::sleep(Duration::from_millis(400));
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
                    .unwrap_or_else(get_cpu_brand_name);

                let total_ram_bytes = sys.total_memory();
                let used_ram_bytes = sys.used_memory();
                let ram_total_gb = total_ram_bytes as f32 / (1024.0 * 1024.0 * 1024.0);
                let ram_used_gb = used_ram_bytes as f32 / (1024.0 * 1024.0 * 1024.0);
                let ram_percent = if total_ram_bytes > 0 {
                    (used_ram_bytes as f32 / total_ram_bytes as f32) * 100.0
                } else {
                    0.0
                };

                let (gpu_usage, gpu_temp, gpu_name) = query_gpu_status();
                let battery = query_battery_status();

                let metrics = SystemMetrics {
                    cpu_usage,
                    cpu_freq,
                    cpu_brand,
                    ram_used_gb,
                    ram_total_gb,
                    ram_percent,
                    gpu_usage,
                    gpu_temp,
                    gpu_name,
                    battery,
                };

                let _ = app.emit("resource-update", &metrics);
                std::thread::sleep(Duration::from_millis(1000));
            }
        });
    }
}

fn get_cpu_brand_name() -> String {
    #[cfg(target_os = "windows")]
    {
        use winreg::enums::HKEY_LOCAL_MACHINE;
        use winreg::RegKey;
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        if let Ok(key) = hklm.open_subkey(r"HARDWARE\DESCRIPTION\System\CentralProcessor\0") {
            if let Ok(name) = key.get_value::<String, _>("ProcessorNameString") {
                return name.trim().to_string();
            }
        }
    }
    "CPU".into()
}

fn query_gpu_status() -> (Option<f32>, Option<f32>, Option<String>) {
    // Attempt Linux sysfs first
    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = std::fs::read_to_string("/sys/class/drm/card0/device/gpu_busy_percent") {
            if let Ok(pct) = content.trim().parse::<f32>() {
                return (Some(pct), None, Some("AMD/Intel GPU".to_string()));
            }
        }
    }

    // Fallback: probe via nvidia-smi with CREATE_NO_WINDOW
    let mut cmd = std::process::Command::new("nvidia-smi");
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW completely stops console flashing
    }

    if let Ok(output) = cmd
        .args(["--query-gpu=utilization.gpu,temperature.gpu,name", "--format=csv,noheader,nounits"])
        .output()
    {
        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout);
            let mut parts = s.trim().split(',');
            let util = parts.next().and_then(|u| u.trim().parse::<f32>().ok());
            let temp = parts.next().and_then(|t| t.trim().parse::<f32>().ok());
            let name = parts.next().map(|n| n.trim().to_string());
            return (util, temp, name);
        }
    }

    (None, None, None)
}

fn query_battery_status() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        #[repr(C)]
        struct SystemPowerStatus {
            ac_status: u8,
            battery_flag: u8,
            battery_life_percent: u8,
            system_status_flag: u8,
            battery_life_time: u32,
            battery_full_life_time: u32,
        }

        extern "system" {
            fn GetSystemPowerStatus(status: *mut SystemPowerStatus) -> i32;
        }

        let mut status: SystemPowerStatus = unsafe { std::mem::zeroed() };
        let ok = unsafe { GetSystemPowerStatus(&mut status) };
        if ok != 0 && status.battery_life_percent <= 100 {
            let mode = if status.ac_status == 1 { "AC" } else { "BAT" };
            return Some(format!("{}: {}%", mode, status.battery_life_percent));
        }
    }

    None
}
