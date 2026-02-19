use crate::domain::constants::{HEALTH_CHECK_INTERVAL_SECS, HEALTH_THRESHOLD_PERCENT};
use anyhow::Result;
use sysinfo::System;
use tokio::time::{sleep, Duration};
use tracing::{info, warn};

pub struct HealthMonitor;

impl HealthMonitor {
    pub async fn run() -> Result<()> {
        let mut sys = System::new_all();
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

            if cpu >= HEALTH_THRESHOLD_PERCENT as f32 || mem_pct >= HEALTH_THRESHOLD_PERCENT {
                warn!(
                    "health-check high usage cpu={:.1}% memory={:.1}% - Triggering self-restart",
                    cpu, mem_pct
                );
                std::process::exit(1);
            } else {
                info!("health-check cpu={:.1}% memory={:.1}%", cpu, mem_pct);
            }

            sleep(Duration::from_secs(HEALTH_CHECK_INTERVAL_SECS)).await;
        }
    }
}
