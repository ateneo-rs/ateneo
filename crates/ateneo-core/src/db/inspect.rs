// SPDX-FileCopyrightText: 2026 James Paul Vizuete Davila
// SPDX-License-Identifier: AGPL-3.0-or-later

use sqlx::PgPool;

use super::MIGRATOR;

pub const MIN_SERVER_VERSION: i32 = 150_000;
pub const MIN_SERVER_MAJOR: i32 = MIN_SERVER_VERSION / 10_000;
const MIGRATIONS_TABLE: &str = "public._sqlx_migrations";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inspection {
    pub server_version: String,
    pub server_version_num: i32,
    pub foreign_tables: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Incompatible {
    OldServer { version: String },
    ForeignTables(Vec<String>),
}

impl Inspection {
    pub fn check(&self) -> Result<(), Incompatible> {
        if self.server_version_num < MIN_SERVER_VERSION {
            return Err(Incompatible::OldServer {
                version: self.server_version.clone(),
            });
        }
        if !self.foreign_tables.is_empty() {
            return Err(Incompatible::ForeignTables(self.foreign_tables.clone()));
        }
        Ok(())
    }
}

pub async fn inspect(pool: &PgPool) -> sqlx::Result<Inspection> {
    let (server_version, server_version_num): (String, i32) = sqlx::query_as(
        "SELECT current_setting('server_version'), current_setting('server_version_num')::int",
    )
    .fetch_one(pool)
    .await?;

    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT schemaname || '.' || tablename FROM pg_tables \
         WHERE schemaname NOT IN ('pg_catalog', 'information_schema') \
         AND schemaname NOT LIKE 'pg\\_%' \
         ORDER BY 1",
    )
    .fetch_all(pool)
    .await?;

    let ours =
        tables.iter().any(|table| table == MIGRATIONS_TABLE) && applied_are_ours(pool).await?;
    let foreign_tables = if ours { Vec::new() } else { tables };

    Ok(Inspection {
        server_version,
        server_version_num,
        foreign_tables,
    })
}

async fn applied_are_ours(pool: &PgPool) -> sqlx::Result<bool> {
    let applied: Vec<(i64, Vec<u8>)> =
        sqlx::query_as("SELECT version, checksum FROM _sqlx_migrations")
            .fetch_all(pool)
            .await?;
    Ok(applied.iter().all(|(version, checksum)| {
        MIGRATOR.iter().any(|migration| {
            migration.version == *version && migration.checksum.as_ref() == checksum.as_slice()
        })
    }))
}
