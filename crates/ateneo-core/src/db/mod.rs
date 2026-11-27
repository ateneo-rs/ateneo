// SPDX-FileCopyrightText: 2026 James Paul Vizuete Davila
// SPDX-License-Identifier: AGPL-3.0-or-later

mod status;

use std::time::Duration;

use sqlx::migrate::{MigrateError, Migrator};
use sqlx::postgres::{PgPool, PgPoolOptions};

use crate::config::DatabaseConfig;

pub use status::{AppliedMigration, PendingMigration, Status, status};

pub static MIGRATOR: Migrator = sqlx::migrate!();

const ACQUIRE_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("no se pudo conectar a PostgreSQL: {0}")]
    Connect(#[source] sqlx::Error),

    #[error("no se pudieron aplicar las migraciones: {0}")]
    Migrate(#[from] MigrateError),
}

pub async fn connect(config: &DatabaseConfig) -> Result<PgPool, DbError> {
    PgPoolOptions::new()
        .max_connections(config.max_connections)
        .acquire_timeout(ACQUIRE_TIMEOUT)
        .connect(&config.url)
        .await
        .map_err(DbError::Connect)
}

pub async fn migrate(pool: &PgPool) -> Result<(), DbError> {
    MIGRATOR.run(pool).await?;
    Ok(())
}
