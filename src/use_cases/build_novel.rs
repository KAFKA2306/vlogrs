use crate::infrastructure::ai::{ImageGenerator, Novelizer};
use log::info;
use std::fs;
use std::path::Path;

pub struct BuildNovelUseCase {
    novelizer: Novelizer,
    image_generator: ImageGenerator,
}

impl BuildNovelUseCase {
    pub fn new(novelizer: Novelizer, image_generator: ImageGenerator) -> Self {
        Self {
            novelizer,
            image_generator,
        }
    }

    pub async fn execute(&self, date: &str) -> anyhow::Result<String> {
        let summary_path = format!("data/summaries/{}_summary.txt", date);
        if !Path::new(&summary_path).exists() {
            anyhow::bail!("Summary not found for {}", date);
        }

        let today_summary = fs::read_to_string(&summary_path)?;
        let novel_path = format!("data/novels/{}.md", date);

        let novel_so_far = if Path::new(&novel_path).exists() {
            fs::read_to_string(&novel_path)?
        } else {
            String::new()
        };

        info!("Generating chapter for {}...", date);
        let chapter = self
            .novelizer
            .generate_chapter(&today_summary, &novel_so_far)
            .await?;

        let content = if novel_so_far.is_empty() {
            chapter.clone()
        } else {
            format!("{}\n\n{}", novel_so_far, chapter)
        };

        fs::create_dir_all("data/novels")?;
        fs::write(&novel_path, content)?;
        info!("Novel saved to {}", novel_path);

        let photo_path = format!("data/photos/{}.png", date);
        fs::create_dir_all("data/photos")?;
        self.image_generator.generate(&chapter, &photo_path).await?;

        Ok(novel_path)
    }
}
