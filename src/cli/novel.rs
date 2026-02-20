use crate::infrastructure;
use crate::infrastructure::settings::Settings;
use crate::use_cases;
use tracing::info;
pub async fn run(date: String) {
    let settings: Settings = Settings::new().unwrap();
    info!("Building novel for: {}", date);
    let prompts = infrastructure::prompts::Prompts::load().unwrap();
    let gemini = infrastructure::llm::GeminiClient::new(
        settings.google_api_key.clone(),
        settings.gemini_model.clone(),
        prompts,
    );
    let image_generator = infrastructure::PythonImageGenerator::new();
    let use_case = use_cases::build_novel::BuildNovelUseCase::new(
        Box::new(gemini.clone()),
        Box::new(gemini),
        Box::new(image_generator),
    );
    use_case.execute(&date).await;
}
