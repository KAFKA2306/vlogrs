use crate::domain::{ImageGenerator, Novelizer};
use log::info;
use std::fs;
use std::path::Path;

pub struct BuildNovelUseCase {
    novelizer: Box<dyn Novelizer>,
    image_generator: Box<dyn ImageGenerator>,
}

impl BuildNovelUseCase {
    pub fn new(novelizer: Box<dyn Novelizer>, image_generator: Box<dyn ImageGenerator>) -> Self {
        Self {
            novelizer,
            image_generator,
        }
    }

    pub async fn execute(&self, date: &str) -> String {
        let summary_path: String = format!("data/summaries/{}_summary.txt", date);
        if !Path::new(&summary_path).exists() {
            panic!("Summary not found for {}", date);
        }

        let today_summary: String = fs::read_to_string(&summary_path).unwrap();
        let novel_path: String = format!("data/novels/{}.md", date);

        let novel_so_far: String = if Path::new(&novel_path).exists() {
            fs::read_to_string(&novel_path).unwrap()
        } else {
            String::new()
        };

        info!("Generating chapter for {}...", date);
        let chapter: String = self
            .novelizer
            .generate_chapter(&today_summary, &novel_so_far)
            .await;

        let content: String = if novel_so_far.is_empty() {
            chapter.clone()
        } else {
            format!("{}\n\n{}", novel_so_far, chapter)
        };

        fs::create_dir_all("data/novels").unwrap();
        fs::write(&novel_path, content).unwrap();
        info!("Novel saved to {}", novel_path);

        let photo_path: String = format!("data/photos/{}.png", date);
        fs::create_dir_all("data/photos").unwrap();
        self.image_generator.generate(&chapter, &photo_path).await;

        novel_path
    }
}
