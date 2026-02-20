pub mod build_novel;
pub mod doctor;
pub mod evaluate;
pub mod health;
pub mod monitor;
pub mod pending;
pub mod process;
pub mod status;
pub mod sync;
pub mod sync_activity;
pub mod synthesis;
pub mod task_runner;
pub mod transcode;
use crate::domain::Environment;
use tracing::info;
pub struct SetupUseCase {
    env: Box<dyn Environment>,
}
impl SetupUseCase {
    pub fn new(env: Box<dyn Environment>) -> Self {
        Self { env }
    }
    pub fn execute(&self) {
        self.env.ensure_directories();
        self.env.ensure_config();
        info!("Setup complete.");
    }
}
pub struct HealthMonitor;
impl HealthMonitor {
    pub async fn run() {
        let mut sys = sysinfo::System::new_all();
        loop {
            sys.refresh_cpu();
            sys.refresh_memory();
            let cpu = sys.global_cpu_info().cpu_usage();
            let total_mem = sys.total_memory();
            let used_mem = sys.used_memory();
            let mem_pct = if total_mem > 0 {
                (used_mem as f64 / total_mem as f64) * 100.0
            } else {
                0.0
            };
            if cpu >= crate::domain::constants::HEALTH_THRESHOLD_PERCENT as f32
                || mem_pct >= crate::domain::constants::HEALTH_THRESHOLD_PERCENT
            {
                tracing::warn!(
                    "health-check high usage cpu={:.1}% memory={:.1}% - Triggering self-restart",
                    cpu,
                    mem_pct
                );
                std::process::exit(1);
            } else {
                tracing::info!("health-check cpu={:.1}% memory={:.1}%", cpu, mem_pct);
            }
            tokio::time::sleep(std::time::Duration::from_secs(
                crate::domain::constants::HEALTH_CHECK_INTERVAL_SECS,
            ))
            .await;
        }
    }
}
