// SPDX-FileCopyrightText: 2026 James Paul Vizuete Davila
// SPDX-License-Identifier: AGPL-3.0-or-later

use serde_json::json;
use sqlx::PgPool;

use crate::audit::{self, Entry};
use crate::settings::{self, SiteState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseSource {
    Docker,
    Environment,
    File,
}

impl DatabaseSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Docker => "docker",
            Self::Environment => "environment",
            Self::File => "file",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Install {
    pub domain: String,
    pub database: DatabaseSource,
    pub site_language: String,
    pub timezone: String,
}

pub async fn save(pool: &PgPool, install: &Install) -> sqlx::Result<()> {
    let mut tx = pool.begin().await?;
    settings::set(
        &mut *tx,
        settings::SITE_LANGUAGE,
        &json!(install.site_language),
    )
    .await?;
    settings::set(&mut *tx, settings::SITE_TIMEZONE, &json!(install.timezone)).await?;
    settings::set(
        &mut *tx,
        settings::SITE_STATE,
        &json!(SiteState::InstallIncomplete.as_str()),
    )
    .await?;
    audit::record(
        &mut *tx,
        &Entry::console(audit::INSTALL_INIT).details(json!({
            "domain": install.domain,
            "database": install.database.as_str(),
            "site_language": install.site_language,
            "timezone": install.timezone,
        })),
    )
    .await?;
    tx.commit().await
}
