use crate::domain::LifeEvent;
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
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);

        let pool = SqlitePool::connect_with(options).await?;

        sqlx::query(include_str!("schema.sql"))
            .execute(&pool)
            .await
            .context("Failed to run migrations")?;

        Ok(Self { pool })
    }

    pub async fn save(&self, event: &LifeEvent) -> Result<()> {
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
}
