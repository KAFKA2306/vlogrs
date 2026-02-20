use crate::infrastructure;
use crate::infrastructure::settings::Settings;
use crate::use_cases;
use tracing::info;
pub async fn run(date: String) {
    let settings: Settings = Settings::new().unwrap();
    info!("Evaluating content for: {}", date);
    let prompts = infrastructure::prompts::Prompts::load().unwrap();
    let gemini = infrastructure::llm::GeminiClient::new(
        settings.google_api_key.clone(),
        settings.gemini_model.clone(),
        prompts,
    );
    let supabase = if !settings.supabase_url.is_empty() {
        Some(infrastructure::api::SupabaseClient::new(
            settings.supabase_url,
            settings.supabase_service_role_key,
        ))
    } else {
        None
    };
    let use_case =
        use_cases::evaluate::EvaluateDailyContentUseCase::new(Box::new(gemini), supabase);
    use_case.execute(&date).await;
}
