use crate::domain::{EventRepository as EventRepositoryTrait, LifeEvent};
use anyhow::{Context, Result};
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use std::str::FromStr;

pub struct EventRepository {
    pool: SqlitePool,
}

impl EventRepository {
    pub async fn new(db_url: &str) -> Result<Self> {
        let options = SqliteConnectOptions::from_str(db_url)?
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .busy_timeout(std::time::Duration::from_secs(10));

        let pool = SqlitePool::connect_with(options).await?;

        sqlx::query(include_str!("schema.sql"))
            .execute(&pool)
            .await
            .context("Failed to run migrations")?;

        Ok(Self { pool })
    }
}

#[async_trait::async_trait]
impl EventRepositoryTrait for EventRepository {
    async fn save(&self, event: &LifeEvent) -> Result<()> {
        let payload = serde_json::to_string(&event.payload)?;

        sqlx::query(
            "INSERT INTO life_events (id, timestamp, source_type, metadata) VALUES (?, ?, ?, ?)",
        )
        .bind(event.id.to_string())
        .bind(event.timestamp)
        .bind(format!("{:?}", event.source))
        .bind(payload)
        .execute(&self.pool)
        .await
        .context("Failed to insert life event")?;

        Ok(())
    }

    async fn find_by_timerange(
        &self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<LifeEvent>> {
        let rows = sqlx::query(
            "SELECT id, timestamp, source_type, metadata FROM life_events WHERE timestamp >= ? AND timestamp <= ? ORDER BY timestamp ASC",
        )
        .bind(start)
        .bind(end)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch life events")?;

        let mut events = Vec::new();
        for row in rows {
            let id: String = sqlx::Row::get(&row, "id");
            let timestamp: chrono::DateTime<chrono::Utc> = sqlx::Row::get(&row, "timestamp");
            let source_type_str: String = sqlx::Row::get(&row, "source_type");
            let metadata_str: String = sqlx::Row::get(&row, "metadata");

            let source = match source_type_str.as_str() {
                "WindowsAudio" => crate::domain::SourceType::WindowsAudio,
                "WindowsActivity" => crate::domain::SourceType::WindowsActivity,
                "UbuntuMonitor" => crate::domain::SourceType::UbuntuMonitor,
                _ => crate::domain::SourceType::System,
            };

            let payload: serde_json::Value = serde_json::from_str(&metadata_str)?;

            events.push(LifeEvent {
                id: uuid::Uuid::parse_str(&id)?,
                timestamp,
                source,
                payload,
            });
        }

        Ok(events)
    }
}
