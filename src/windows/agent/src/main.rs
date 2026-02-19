use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;
use tracing::info;
use windows::core::PWSTR;

use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowTextW};

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
        Self {
            last_activity: None,
        }
    }

    async fn run(&mut self) {
        loop {
            if let Some(activity) = self.get_current_activity() {
                if Some(&activity) != self.last_activity.as_ref() {
                    self.log_activity(&activity);
                    self.last_activity = Some(activity);
                }
            }
            sleep(Duration::from_millis(1000)).await;
        }
    }

    fn get_current_activity(&self) -> Option<Activity> {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.0 == 0 {
                return None;
            }

            let mut title_buf = [0u16; 512];
            let len = GetWindowTextW(hwnd, &mut title_buf);
            let window_title = String::from_utf16_lossy(&title_buf[..len as usize]);

            let mut process_id = 0u32;
            windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId(
                hwnd,
                Some(&mut process_id),
            );

            let app_name = self.get_app_name(process_id);

            Some(Activity {
                timestamp: Utc::now().to_rfc3339(),
                app_name,
                window_title,
            })
        }
    }

    fn get_app_name(&self, pid: u32) -> String {
        unsafe {
            let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).unwrap();
            let mut buf = [0u16; 512];
            let mut len = buf.len() as u32;
            QueryFullProcessImageNameW(
                handle,
                PROCESS_NAME_WIN32,
                PWSTR(buf.as_mut_ptr()),
                &mut len,
            )
            .unwrap();
            let path = String::from_utf16_lossy(&buf[..len as usize]);
            std::path::Path::new(&path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap()
                .to_string()
        }
    }

    fn log_activity(&self, activity: &Activity) {
        let json = serde_json::to_string(activity).unwrap();
        info!("{}", json);
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let mut agent = Agent::new();
    agent.run().await;
}
