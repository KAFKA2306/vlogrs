use crate::infrastructure::api::GeminiClient;
use serde_json::Value;

pub struct Novelizer {
    gemini: GeminiClient,
}

impl Novelizer {
    pub fn new(gemini: GeminiClient) -> Self {
        Self { gemini }
    }

    pub async fn generate_chapter(
        &self,
        today_summary: &str,
        novel_so_far: &str,
    ) -> anyhow::Result<String> {
        let prompt = format!(
            "これまでのあらすじ:\n{}\n\n今日の出来事要約:\n{}\n\nこれらを元に、日常系VLogの一部として小説の1章を執筆してください。読者が情景を思い浮かべられるよう、情緒豊かな表現を心がけてください。",
            novel_so_far, today_summary
        );
        self.gemini.generate_content(&prompt).await
    }
}

pub struct Curator {
    gemini: GeminiClient,
}

impl Curator {
    pub fn new(gemini: GeminiClient) -> Self {
        Self { gemini }
    }

    pub async fn evaluate(&self, summary: &str, novel: &str) -> anyhow::Result<Value> {
        let prompt = format!(
            "以下の要約と小説の内容を比較し、評価してください。JSON形式で出力してください。\n\n要約:\n{}\n\n小説:\n{}\n\n期待する形式:\n{{ \"faithfulness_score\": 1-5, \"quality_score\": 1-5, \"reasoning\": \"評価理由\" }}",
            summary, novel
        );
        let response = self.gemini.generate_content(&prompt).await?;

        let cleaned = response
            .trim_start_matches("```json")
            .trim_end_matches("```")
            .trim();
        let result: Value = serde_json::from_str(cleaned)?;
        Ok(result)
    }
}

pub struct ImageGenerator;

impl Default for ImageGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageGenerator {
    pub fn new() -> Self {
        Self
    }

    pub async fn generate(&self, _prompt: &str, _output_path: &str) -> anyhow::Result<()> {
        log::info!(
            "Image generation placeholder called for prompt: {}",
            _prompt
        );
        Ok(())
    }
}
