#![windows_subsystem = "windows"]

fn is_another_instance_running() -> bool {
    let mut sys = sysinfo::System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    let current_pid = sysinfo::Pid::from_u32(std::process::id());
    for (pid, process) in sys.processes() {
        if *pid != current_pid {
            let name = process.name().to_string_lossy().to_lowercase();
            if name == "finder.exe" || name == "finder" || name == "omni-launcher.exe" || name == "omni-launcher" {
                return true;
            }
        }
    }
    false
}

fn main() {
    if is_another_instance_running() {
        return; // Another instance is already running
    }

    finder_lib::run();
}
