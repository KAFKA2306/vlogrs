use anyhow::{Context, Result};
use chrono::Utc;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Stdio;
use std::sync::mpsc::channel;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::sleep;
use tracing::{error, info, warn};
#[cfg(windows)]
use windows::core::PWSTR;
#[cfg(windows)]
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowTextW};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct Activity {
    timestamp: String,
    app_name: String,
    window_title: String,
    is_discord: bool,
    is_vrchat: bool,
}

#[allow(dead_code)]
struct Constants;
impl Constants {
    const AUDIO_DIR: &'static str = "inbox/audio";
    const HEARTBEAT_INTERVAL_SECS: u64 = 60;
    const SCAN_INTERVAL_MS: u64 = 1000;
    const DISCORD_PROC: &'static str = "discord.exe";
    const VRCHAT_PROC: &'static str = "vrchat.exe";
    const RECORDER_SCRIPT: &'static str = "src/windows/audio_recorder.py";
    const PYTHON_CMD: &'static str = "python";
}

struct Agent {
    last_activity: Option<Activity>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    info!("--- VLog Windows Agent v0.1.0 ---");

    #[cfg(not(windows))]
    {
        warn!("Running on non-Windows platform. Process monitoring is DISABLED.");
        info!("[STATUS] Agent: STUB MODE");
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;
            info!("[HEARTBEAT] Agent Stalled (STUB MODE)");
        }
    }

    #[cfg(windows)]
    {
        info!("[STATUS] Process Monitor: ACTIVE (Targets: Discord, VRChat)");

        let mut agent = Agent::new();
        agent.run().await?;
        Ok(())
    }
}

#[cfg(windows)]
impl Agent {
    fn new() -> Self {
        Self {
            last_activity: None,
        }
    }

    async fn run(&mut self) -> Result<()> {
        let mut audio_process = Some(self.spawn_audio_recorder()?);

        let (tx, rx) = channel();
        let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

        let audio_dir = Path::new(Constants::AUDIO_DIR);
        if !audio_dir.exists() {
            let _ = std::fs::create_dir_all(audio_dir);
        }

        watcher.watch(audio_dir, RecursiveMode::NonRecursive)?;

        let mut last_heartbeat = std::time::Instant::now();

        loop {
            if let Some(proc) = audio_process.as_mut() {
                if let Ok(Some(status)) = proc.try_wait() {
                    warn!(
                        "Audio recorder exited with status: {}. Attempting restart in 5s...",
                        status
                    );
                    audio_process = None;
                }
            } else {
                audio_process = Some(self.spawn_audio_recorder()?);
            }

            if let Ok(Ok(event)) = rx.try_recv() {
                match event.kind {
                    notify::EventKind::Create(_) => {
                        info!("Detected new audio file: {:?}", event.paths);
                    }
                    _ => {}
                }
            }

            if let Some(mut activity) = self.get_current_activity() {
                let app_name_lower = activity.app_name.to_lowercase();
                if app_name_lower.contains(Constants::DISCORD_PROC) {
                    activity.is_discord = true;
                }
                if app_name_lower.contains(Constants::VRCHAT_PROC) {
                    activity.is_vrchat = true;
                }

                let prev_discord = self
                    .last_activity
                    .as_ref()
                    .map(|a| a.is_discord)
                    .unwrap_or(false);
                let prev_vrchat = self
                    .last_activity
                    .as_ref()
                    .map(|a| a.is_vrchat)
                    .unwrap_or(false);

                if prev_discord && !activity.is_discord {
                    info!("[STATUS] Target LOST: Discord");
                }
                if prev_vrchat && !activity.is_vrchat {
                    info!("[STATUS] Target LOST: VRChat");
                }

                if !prev_discord && activity.is_discord {
                    info!("[STATUS] Target FOUND: Discord");
                }
                if !prev_vrchat && activity.is_vrchat {
                    info!("[STATUS] Target FOUND: VRChat");
                }

                if Some(&activity) != self.last_activity.as_ref() {
                    let match_tag = if activity.is_discord {
                        "DISCORD"
                    } else if activity.is_vrchat {
                        "VRCHAT"
                    } else {
                        "NONE"
                    };

                    info!(
                        "[SCAN] Process: '{}' (Title: '{}') -> Match: {}",
                        activity.app_name, activity.window_title, match_tag
                    );

                    self.log_activity(&activity);
                    self.last_activity = Some(activity);
                }
            }

            if last_heartbeat.elapsed() >= Duration::from_secs(Constants::HEARTBEAT_INTERVAL_SECS) {
                let current_target = if let Some(a) = &self.last_activity {
                    if a.is_discord {
                        "Discord"
                    } else if a.is_vrchat {
                        "VRChat"
                    } else {
                        "None"
                    }
                } else {
                    "None"
                };
                info!(
                    "[HEARTBEAT] Agent Active. Current Target: {}",
                    current_target
                );
                last_heartbeat = std::time::Instant::now();
            }

            sleep(Duration::from_millis(Constants::SCAN_INTERVAL_MS)).await;
        }
    }

    fn spawn_audio_recorder(&self) -> Result<tokio::process::Child> {
        let child = Command::new(Constants::PYTHON_CMD)
            .arg(Constants::RECORDER_SCRIPT)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .kill_on_drop(true)
            .spawn()
            .context(format!("Failed to spawn {}", Constants::RECORDER_SCRIPT))?;
        info!("Started {} with PID: {:?}", Constants::RECORDER_SCRIPT, child.id());
        Ok(child)
    }

    fn get_current_activity(&self) -> Option<Activity> {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.0 == 0 {
                return Some(Activity {
                    timestamp: Utc::now().to_rfc3339(),
                    app_name: "Idle".to_string(),
                    window_title: "No Active Window".to_string(),
                    is_discord: false,
                    is_vrchat: false,
                });
            }

            let mut title_buf = [0u16; 512];
            let len = GetWindowTextW(hwnd, &mut title_buf);
            let window_title = String::from_utf16_lossy(&title_buf[..len as usize]);

            let mut process_id = 0u32;
            windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId(
                hwnd,
                Some(&mut process_id),
            );

            let app_name = self
                .get_app_name(process_id)
                .unwrap_or_else(|_| "Unknown".to_string());

            Some(Activity {
                timestamp: Utc::now().to_rfc3339(),
                app_name,
                window_title,
                is_discord: false,
                is_vrchat: false,
            })
        }
    }

    fn get_app_name(&self, pid: u32) -> Result<String> {
        unsafe {
            let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid)
                .unwrap_or_else(|_| panic!("OpenProcess failed for PID {}", pid));

            let mut buf = [0u16; 512];
            let mut len = buf.len() as u32;

            QueryFullProcessImageNameW(
                handle,
                PROCESS_NAME_WIN32,
                PWSTR(buf.as_mut_ptr()),
                &mut len,
            )?;

            let path = String::from_utf16_lossy(&buf[..len as usize]);

            let file_name = std::path::Path::new(&path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown")
                .to_string();
            Ok(file_name)
        }
    }

    fn log_activity(&self, activity: &Activity) {
        if let Ok(json) = serde_json::to_string(activity) {
            info!("{}", json);
        }
    }
}

#[cfg(not(windows))]
impl Agent {
    fn new() -> Self {
        Self {
            last_activity: None,
        }
    }
}
