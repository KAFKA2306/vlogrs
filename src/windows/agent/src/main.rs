#[allow(unused_imports, dead_code)]
use anyhow::{Context, Result};
#[allow(unused_imports, dead_code)]
use chrono::Utc;
#[allow(unused_imports, dead_code)]
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
#[allow(unused_imports, dead_code)]
use serde::{Deserialize, Serialize};
#[allow(unused_imports, dead_code)]
use std::path::Path;
#[allow(unused_imports, dead_code)]
use std::process::Stdio;
#[allow(unused_imports, dead_code)]
use std::sync::mpsc::channel;
#[allow(unused_imports, dead_code)]
use std::time::Duration;
#[allow(unused_imports, dead_code)]
use tokio::process::Command;
#[allow(unused_imports, dead_code)]
use tokio::time::sleep;
#[allow(unused_imports, dead_code)]
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
struct Agent {
    last_activity: Option<Activity>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    // Startup Banner
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
        if let Err(e) = agent.run().await {
            error!("Agent crashed: {:?}", e);
            return Err(e);
        }
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
        // Start Audio Recorder Subprocess
        let mut audio_process = match self.spawn_audio_recorder() {
            Ok(p) => Some(p),
            Err(e) => {
                warn!("Initial audio recorder start failed: {}", e);
                None
            }
        };

        // File Watcher Setup
        let (tx, rx) = channel();
        let watcher_result = RecommendedWatcher::new(tx, Config::default());
        let mut watcher = match watcher_result {
            Ok(w) => Some(w),
            Err(e) => {
                warn!("Failed to create file watcher: {}", e);
                None
            }
        };
        
        let audio_dir = Path::new("inbox/audio");
        if !audio_dir.exists() {
            let _ = std::fs::create_dir_all(audio_dir);
        }
        
        if audio_dir.exists() {
             info!("[STATUS] Audio Ingest: READY ({:?})", audio_dir);
        }

        if let Some(w) = watcher.as_mut() {
             if let Err(e) = w.watch(audio_dir, RecursiveMode::NonRecursive) {
                warn!("Failed to watch audio directory: {}", e);
                info!("[STATUS] File Watcher: FAILED");
             } else {
                info!("[STATUS] File Watcher: ACTIVE");
             }
        }

        let mut last_heartbeat = std::time::Instant::now();

        loop {
            // 1. Check Subprocess (Resilient)
            if let Some(proc) = audio_process.as_mut() {
                if let Ok(Some(status)) = proc.try_wait() {
                    warn!("Audio recorder exited with status: {}. Attempting restart in 5s...", status);
                    audio_process = None; // clear it
                }
            } else {
                // Try to restart
                 match self.spawn_audio_recorder() {
                     Ok(p) => audio_process = Some(p),
                     Err(e) => warn!("Failed to restart audio recorder: {}", e),
                 }
            }

            // 2. Check File Events
            if let Ok(Ok(event)) = rx.try_recv() {
                match event.kind {
                    notify::EventKind::Create(_) => {
                        info!("Detected new audio file: {:?}", event.paths);
                    }
                    _ => {}
                }
            }

            // 3. Monitor specific apps
            if let Some(mut activity) = self.get_current_activity() {
                // Annotate specific apps
                let app_name_lower = activity.app_name.to_lowercase();
                if app_name_lower.contains("discord.exe") {
                    activity.is_discord = true;
                }
                if app_name_lower.contains("vrchat.exe") {
                    activity.is_vrchat = true;
                }

                // State Transition Logic
                let prev_discord = self.last_activity.as_ref().map(|a| a.is_discord).unwrap_or(false);
                let prev_vrchat = self.last_activity.as_ref().map(|a| a.is_vrchat).unwrap_or(false);

                // Check for Loss
                if prev_discord && !activity.is_discord {
                    info!("[STATUS] Target LOST: Discord");
                }
                if prev_vrchat && !activity.is_vrchat {
                    info!("[STATUS] Target LOST: VRChat");
                }

                // Check for Gain (already handled by general change log, but we can make it explicit)
                if !prev_discord && activity.is_discord {
                     info!("[STATUS] Target FOUND: Discord");
                }
                if !prev_vrchat && activity.is_vrchat {
                     info!("[STATUS] Target FOUND: VRChat");
                }

                if Some(&activity) != self.last_activity.as_ref() {
                    // Match Status Logging
                    let match_tag = if activity.is_discord {
                        "DISCORD"
                    } else if activity.is_vrchat {
                        "VRCHAT"
                    } else {
                        "NONE"
                    };

                    info!("[SCAN] Process: '{}' (Title: '{}') -> Match: {}", activity.app_name, activity.window_title, match_tag);
                    
                    self.log_activity(&activity);
                    self.last_activity = Some(activity);
                }
            }

            // Heartbeat every 60 seconds
            if last_heartbeat.elapsed() >= Duration::from_secs(60) {
                let current_target = if let Some(a) = &self.last_activity {
                    if a.is_discord { "Discord" } else if a.is_vrchat { "VRChat" } else { "None" }
                } else {
                    "None"
                };
                info!("[HEARTBEAT] Agent Active. Current Target: {}", current_target);
                last_heartbeat = std::time::Instant::now();
            }

            sleep(Duration::from_millis(1000)).await;
        }
    }

    fn spawn_audio_recorder(&self) -> Result<tokio::process::Child> {
        let child = Command::new("python")
            .arg("src/windows/audio_recorder.py")
            .stdout(Stdio::inherit()) // Pipe to main stdout
            .stderr(Stdio::inherit())
            .kill_on_drop(true)
            .spawn()
            .context("Failed to spawn audio_recorder.py")?;
        info!("Started audio_recorder.py with PID: {:?}", child.id());
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

            let app_name = self.get_app_name(process_id).unwrap_or_else(|_| "Unknown".to_string());

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
            // Use PROCESS_QUERY_LIMITED_INFORMATION for better compatibility (e.g. Anti-Cheat)
            let handle = match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
                Ok(h) => h,
                Err(e) => {
                    // Log the specific error (e.g. Access Denied = 5)
                    warn!("[ERROR] OpenProcess failed for PID {}: {:?}", pid, e);
                    return Ok("Unknown".to_string());
                }
            };

            let mut buf = [0u16; 512];
            let mut len = buf.len() as u32;
            
            if let Err(e) = QueryFullProcessImageNameW(
                handle,
                PROCESS_NAME_WIN32,
                PWSTR(buf.as_mut_ptr()),
                &mut len,
            ) {
                 warn!("[ERROR] QueryFullProcessImageNameW failed for PID {}: {:?}", pid, e);
                 return Ok("Unknown".to_string());
            }

            let path = String::from_utf16_lossy(&buf[..len as usize]);
            
            // INVESTIGATION: Log the full raw path
            info!("[INVESTIGATION] PID: {} -> Full Path: '{}'", pid, path);

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
#[allow(dead_code, unused_imports)]
impl Agent {
    fn new() -> Self {
        Self {
            last_activity: None,
        }
    }
}
