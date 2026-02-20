use crate::use_cases;
pub async fn run() {
    let use_case = use_cases::status::StatusUseCase::new();
    use_case.execute().await;
}
