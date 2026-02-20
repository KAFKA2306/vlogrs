use crate::infrastructure::settings::Settings;
use crate::use_cases;
pub async fn run() {
    let settings: Settings = Settings::new().unwrap();
    let use_case = use_cases::sync::SyncUseCase::new(settings);
    use_case.execute().await;
}
