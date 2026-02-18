use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use crate::domain::LifeEvent;
use anyhow::{Result, Context};
use std::str::FromStr;

pub struct EventRepository {
    pool: SqlitePool,
}

impl EventRepository {
    pub async fn new(db_url: &str) -> Result<Self> {
        let options = SqliteConnectOptions::from_str(db_url)?
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal); // Milestone 54: WAL mode

        let pool = SqlitePool::connect_with(options).await?;
        
        // Milestone 53: Compile-time check (sqlx doesn't do it easily for raw SQL here, but we can use macro later)
        sqlx::query(include_str!("schema.sql"))
            .execute(&pool)
            .await
            .context("Failed to run migrations")?;

        Ok(Self { pool })
    }

    pub async fn save(&self, event: &LifeEvent) -> Result<()> {
        let payload = serde_json::to_string(&event.payload)?;
        
        sqlx::query(
            "INSERT INTO life_events (id, timestamp, source_type, metadata) VALUES (?, ?, ?, ?)"
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
