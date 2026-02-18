use std::time::Duration;
use tokio::time::sleep;
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowTextW};
use windows::Win32::System::ProcessStatus::GetModuleFileNameExW;
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
use windows::Win32::Foundation::{HWND, HANDLE};
use serde::{Serialize, Deserialize};
use chrono::Utc;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct Activity {
    timestamp: String,
    app_name: String,
    window_title: String,
}

struct Agent {
    last_activity: Option<Activity>,
}

impl Agent {
    fn new() -> Self {
        Self { last_activity: None }
    }

    async fn run(&mut self) -> anyhow::Result<()> {
        loop {
            if let Some(activity) = self.get_current_activity() {
                if Some(&activity) != self.last_activity.as_ref() {
                    self.log_activity(&activity)?;
                    self.last_activity = Some(activity);
                }
            }
            sleep(Duration::from_millis(1000)).await;
        }
    }

    fn get_current_activity(&self) -> Option<Activity> {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.0 == 0 { return None; }

            let mut title_buf = [0u16; 512];
            let len = GetWindowTextW(hwnd, &mut title_buf);
            let window_title = String::from_utf16_lossy(&title_buf[..len as usize]);

            let mut process_id = 0u32;
            windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId(hwnd, Some(&mut process_id));
            
            let app_name = self.get_app_name(process_id).unwrap_or_else(|_| "Unknown".to_string());

            Some(Activity {
                timestamp: Utc::now().to_rfc3339(),
                app_name,
                window_title,
            })
        }
    }

    fn get_app_name(&self, pid: u32) -> anyhow::Result<String> {
        unsafe {
            let handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid)?;
            let mut buf = [0u16; 512];
            let len = GetModuleFileNameExW(handle, None, &mut buf);
            let path = String::from_utf16_lossy(&buf[..len as usize]);
            let name = std::path::Path::new(&path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown")
                .to_string();
            Ok(name)
        }
    }

    fn log_activity(&self, activity: &Activity) -> anyhow::Result<()> {
        let json = serde_json::to_string(activity)?;
        println!("{}", json); // Forward to parent process or log to file
        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut agent = Agent::new();
    agent.run().await
}
