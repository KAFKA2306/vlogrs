use log::info;
use sysinfo::System;

pub struct ProcessMonitor {
    targets: Vec<String>,
    system: System,
    last_status: bool,
}

impl ProcessMonitor {
    pub fn new(targets: Vec<String>) -> Self {
        Self {
            targets: targets.into_iter().map(|t: String| t.to_lowercase()).collect(),
            system: System::new_all(),
            last_status: false,
        }
    }

    pub fn is_running(&mut self) -> bool {
        self.system.refresh_processes();
        let current_status: bool = self.check_processes();

        if current_status != self.last_status {
            self.last_status = current_status;
            if current_status {
                info!("Target process detected.");
            } else {
                info!("Target process no longer detected.");
            }
        }
        current_status
    }

    fn check_processes(&self) -> bool {
        for process in self.system.processes().values() {
            let name: String = process.name().to_lowercase();

            if self.targets.iter().any(|target: &String| name.contains(target)) {
                return true;
            }

            let exe_path = process.exe().unwrap();
            let exe: String = exe_path.to_string_lossy().to_lowercase();
            if self.targets.iter().any(|target: &String| exe.contains(target)) {
                return true;
            }
        }
        false
    }
}
