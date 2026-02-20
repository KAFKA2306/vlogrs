use crate::domain::{Curator, Novelizer};
use crate::infrastructure::db::EventRepository;
use anyhow::Result;
use chrono::{DateTime, Utc};
#[allow(dead_code)]
pub struct SynthesisUseCase {
    event_repo: EventRepository,
    novelizer: Box<dyn Novelizer>,
    curator: Box<dyn Curator>,
}
impl SynthesisUseCase {
    pub fn new(
        event_repo: EventRepository,
        novelizer: Box<dyn Novelizer>,
        curator: Box<dyn Curator>,
    ) -> Self {
        Self {
            event_repo,
            novelizer,
            curator,
        }
    }
    pub async fn synthesize_day(&self, _date: DateTime<Utc>) -> Result<()> {
        Ok(())
    }
}
