use crate::domain::{Curator, ImageGenerator, Novelizer};
use crate::infrastructure::fs_utils;
use std::fs;
use std::path::Path;
use tracing::{info, warn};

pub struct BuildNovelUseCase {
    novelizer: Box<dyn Novelizer>,
    curator: Box<dyn Curator>,
    image_generator: Box<dyn ImageGenerator>,
}

impl BuildNovelUseCase {
    pub fn new(
        novelizer: Box<dyn Novelizer>,
        curator: Box<dyn Curator>,
        image_generator: Box<dyn ImageGenerator>,
    ) -> Self {
        Self {
            novelizer,
            curator,
            image_generator,
        }
    }

    pub async fn execute(&self, date: &str) {
        let summary_path = crate::domain::constants::SUMMARY_FILE_TEMPLATE.replace("{}", date);
        if !Path::new(&summary_path).exists() {
            panic!("Summary not found for {}", date);
        }

        let today_summary = fs::read_to_string(&summary_path).unwrap();
        let novel_path = crate::domain::constants::NOVEL_FILE_TEMPLATE.replace("{}", date);

        let novel_so_far = if Path::new(&novel_path).exists() {
            fs::read_to_string(&novel_path).unwrap()
        } else {
            String::new()
        };

        info!("Generating chapter for {}...", date);

        let mut chapter = String::new();
        let max_retries = 3;

        for attempt in 1..=max_retries {
            chapter = self
                .novelizer
                .generate_chapter(&today_summary, &novel_so_far)
                .await;

            let mut found_prohibited = false;
            for word in crate::domain::constants::PROHIBITED_WORDS {
                if chapter.to_lowercase().contains(&word.to_lowercase()) {
                    warn!("Prohibited word found: {}", word);
                    found_prohibited = true;
                    break;
                }
            }

            if found_prohibited {
                info!("Retry {}/{}: Prohibited words found.", attempt, max_retries);
                continue;
            }

            let eval = self.curator.evaluate(&today_summary, &chapter).await;
            info!(
                "Curator Score: Faithfulness={}, Quality={}, Reason={}",
                eval.faithfulness_score, eval.quality_score, eval.reasoning
            );

            if eval.quality_score >= 3 {
                break;
            } else {
                info!(
                    "Retry {}/{}: Quality verification failed (Score < 3).",
                    attempt, max_retries
                );
            }
        }

        let content = if novel_so_far.is_empty() {
            chapter.clone()
        } else {
            format!("{}\n\n{}", novel_so_far, chapter)
        };

        fs_utils::atomic_write(&novel_path, content);
        info!("Novel saved to {}", novel_path);

        let photo_path = crate::domain::constants::PHOTO_FILE_TEMPLATE.replace("{}", date);
        self.image_generator.generate(&chapter, &photo_path).await;


    }
}
