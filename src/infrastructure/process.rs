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
            targets: targets.into_iter().map(|t| t.to_lowercase()).collect(),
            system: System::new_all(),
            last_status: false,
        }
    }

    pub fn is_running(&mut self) -> bool {
        self.system.refresh_processes();
        let current_status = self.check_processes();

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
        self.system.processes().values().any(|process| {
            let name = process.name().to_lowercase();
            if self.targets.iter().any(|target| name.contains(target)) {
                return true;
            }

            process.exe().map_or(false, |exe_path| {
                let exe = exe_path.to_string_lossy().to_lowercase();
                self.targets.iter().any(|target| exe.contains(target))
            })
        })
    }
}
