use crate::domain::ProcessMonitor as ProcessMonitorTrait;
use std::collections::BTreeSet;
use std::process::Command;
use sysinfo::System;
use tracing::{debug, info};
#[cfg(windows)]
use windows::core::PWSTR;
#[cfg(windows)]
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::GetWindowTextW;
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

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

    #[cfg(windows)]
    fn check_native_windows_processes(&self) -> Option<String> {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.0 == 0 {
                return None;
            }

            let mut process_id = 0u32;
            GetWindowThreadProcessId(hwnd, Some(&mut process_id));

            let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id).ok()?;
            let mut buf = [0u16; 512];
            let mut len = buf.len() as u32;
            
            if QueryFullProcessImageNameW(handle, PROCESS_NAME_WIN32, PWSTR(buf.as_mut_ptr()), &mut len).is_err() {
                 return None;
            }
            
            let path = String::from_utf16_lossy(&buf[..len as usize]);
            let path_obj = std::path::Path::new(&path);
            let exe_name = path_obj
                .file_name()
                .and_then(|n| n.to_str())?
                .to_lowercase();

            if self.targets.iter().any(|target| exe_name.contains(target)) {
                 info!("Target process detected (Native Windows): {}", exe_name);
                 return Some(format!("windows-native:{}", exe_name));
            }
            None
        }
    }

    #[cfg(not(windows))]
    fn check_native_windows_processes(&self) -> Option<String> {
        None
    }
}

impl ProcessMonitorTrait for ProcessMonitor {
    fn is_running(&mut self) -> bool {
        self.system.refresh_processes();
        let match_info = self
            .check_processes()
            .or_else(|| self.check_windows_processes())
            .or_else(|| self.check_native_windows_processes());
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
