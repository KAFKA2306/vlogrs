use crate::domain::ProcessMonitor as ProcessMonitorTrait;
use std::collections::BTreeSet;
use std::process::Command;
use sysinfo::System;
use tracing::{debug, info};

pub struct ProcessMonitor {
    targets: Vec<String>,
    system: System,
    last_status: bool,
    last_match: Option<String>,
}

impl ProcessMonitor {
    pub fn new(targets: Vec<String>) -> Self {
        Self {
            targets: targets.into_iter().map(|t| t.to_lowercase()).collect(),
            system: System::new_all(),
            last_status: false,
            last_match: None,
        }
    }

    fn check_processes(&self) -> Option<String> {
        self.system.processes().values().find_map(|process| {
            let name = process.name().to_lowercase();
            if self.targets.iter().any(|target| name.contains(target)) {
                return Some(format!("linux:{} (pid={})", process.name(), process.pid()));
            }

            process.exe().and_then(|exe_path| {
                let exe = exe_path.to_string_lossy().to_lowercase();
                if self.targets.iter().any(|target| exe.contains(target)) {
                    Some(format!(
                        "linux:{} (pid={}, exe={})",
                        process.name(),
                        process.pid(),
                        exe_path.to_string_lossy()
                    ))
                } else {
                    None
                }
            })
        })
    }

    fn is_wsl() -> bool {
        std::env::var("WSL_DISTRO_NAME").is_ok()
    }

    fn normalized_windows_targets(&self) -> Vec<String> {
        let mut names = BTreeSet::new();
        for target in &self.targets {
            let base = target
                .rsplit(['\\', '/'])
                .next()
                .unwrap_or(target)
                .trim()
                .trim_end_matches(".exe")
                .to_string();
            if !base.is_empty() {
                names.insert(base);
            }
        }
        names.into_iter().collect()
    }

    fn check_windows_processes(&self) -> Option<String> {
        if !Self::is_wsl() {
            return None;
        }

        let names = self.normalized_windows_targets();
        if names.is_empty() {
            return None;
        }

        let joined = names.join(",");
        let output = Command::new("/mnt/c/Windows/System32/WindowsPowerShell/v1.0/powershell.exe")
            .args([
                "-NoLogo",
                "-NoProfile",
                "-Command",
                &format!(
                    "Get-Process -Name {} -ErrorAction SilentlyContinue | Select-Object -ExpandProperty ProcessName",
                    joined
                ),
            ])
            .output();

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let first = stdout
                    .lines()
                    .map(str::trim)
                    .find(|line| !line.is_empty())
                    .map(ToOwned::to_owned);
                if let Some(proc_name) = &first {
                    debug!("Windows process detected via powershell.exe: {}", proc_name);
                    Some(format!("windows:{}", proc_name))
                } else {
                    None
                }
            }
            Err(e) => {
                debug!("Failed to query Windows processes: {}", e);
                None
            }
        }
    }
}

impl ProcessMonitorTrait for ProcessMonitor {
    fn is_running(&mut self) -> bool {
        self.system.refresh_processes();
        let match_info = self
            .check_processes()
            .or_else(|| self.check_windows_processes());
        let current_status = match_info.is_some();

        if current_status != self.last_status {
            self.last_status = current_status;
            if current_status {
                let matched = match_info
                    .clone()
                    .unwrap_or_else(|| "unknown-target".to_string());
                self.last_match = Some(matched.clone());
                info!("Target process detected: {}", matched);
            } else {
                info!("Target process no longer detected.");
                self.last_match = None;
            }
        } else if current_status && self.last_match != match_info {
            // Surface target switches (e.g. VRChat -> Discord) while recording.
            if let Some(matched) = match_info.clone() {
                info!("Target process updated: {}", matched);
                self.last_match = Some(matched);
            }
        }
        current_status
    }
}
