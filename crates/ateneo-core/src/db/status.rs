// SPDX-FileCopyrightText: 2026 James Paul Vizuete Davila
// SPDX-License-Identifier: AGPL-3.0-or-later

use sqlx::PgPool;

use super::MIGRATOR;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Status {
    pub server_version: String,
    pub applied: Vec<AppliedMigration>,
    pub pending: Vec<PendingMigration>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedMigration {
    pub version: i64,
    pub description: String,
    pub installed_on: String,
    pub success: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingMigration {
    pub version: i64,
    pub description: String,
}

pub async fn status(pool: &PgPool) -> sqlx::Result<Status> {
    let server_version: String = sqlx::query_scalar("SHOW server_version")
        .fetch_one(pool)
        .await?;

    let has_table: bool =
        sqlx::query_scalar("SELECT to_regclass('public._sqlx_migrations') IS NOT NULL")
            .fetch_one(pool)
            .await?;

    let applied = if has_table {
        sqlx::query_as::<_, (i64, String, String, bool)>(
            "SELECT version, description, \
             to_char(installed_on, 'YYYY-MM-DD HH24:MI:SS TZ'), success \
             FROM _sqlx_migrations ORDER BY version",
        )
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(
            |(version, description, installed_on, success)| AppliedMigration {
                version,
                description,
                installed_on,
                success,
            },
        )
        .collect()
    } else {
        Vec::new()
    };

    let pending = MIGRATOR
        .iter()
        .filter(|migration| migration.migration_type.is_up_migration())
        .filter(|migration| !applied.iter().any(|done| done.version == migration.version))
        .map(|migration| PendingMigration {
            version: migration.version,
            description: migration.description.to_string(),
        })
        .collect();

    Ok(Status {
        server_version,
        applied,
        pending,
    })
}
