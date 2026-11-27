// SPDX-FileCopyrightText: 2026 James Paul Vizuete Davila
// SPDX-License-Identifier: AGPL-3.0-or-later

use serde_json::Value;
use sqlx::PgExecutor;

pub const SITE_LANGUAGE: &str = "site.language";
pub const SITE_TIMEZONE: &str = "site.timezone";
pub const SITE_STATE: &str = "site.state";

pub const SITE_LANGUAGES: &[&str] = &["es"];
pub const DEFAULT_SITE_LANGUAGE: &str = "es";
pub const DEFAULT_TIMEZONE: &str = "UTC";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SiteState {
    InstallIncomplete,
    Fresh,
    Published,
    UnderConstruction,
}

impl SiteState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InstallIncomplete => "install_incomplete",
            Self::Fresh => "fresh",
            Self::Published => "published",
            Self::UnderConstruction => "under_construction",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingKey {
    SiteLanguage,
    SiteTimezone,
}

impl SettingKey {
    pub const ALL: [SettingKey; 2] = [SettingKey::SiteLanguage, SettingKey::SiteTimezone];

    pub fn name(self) -> &'static str {
        match self {
            Self::SiteLanguage => SITE_LANGUAGE,
            Self::SiteTimezone => SITE_TIMEZONE,
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|key| key.name() == name)
    }
}

pub fn is_site_language(code: &str) -> bool {
    SITE_LANGUAGES.contains(&code)
}

pub async fn is_timezone<'e>(executor: impl PgExecutor<'e>, name: &str) -> sqlx::Result<bool> {
    sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM pg_timezone_names WHERE name = $1)")
        .bind(name)
        .fetch_one(executor)
        .await
}

pub async fn get<'e>(executor: impl PgExecutor<'e>, key: &str) -> sqlx::Result<Option<Value>> {
    sqlx::query_scalar("SELECT value FROM settings WHERE key = $1")
        .bind(key)
        .fetch_optional(executor)
        .await
}

pub async fn get_text<'e>(
    executor: impl PgExecutor<'e>,
    key: &str,
) -> sqlx::Result<Option<String>> {
    sqlx::query_scalar("SELECT value #>> '{}' FROM settings WHERE key = $1")
        .bind(key)
        .fetch_optional(executor)
        .await
}

pub async fn set<'e>(executor: impl PgExecutor<'e>, key: &str, value: &Value) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT INTO settings (key, value) VALUES ($1, $2) \
         ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = now()",
    )
    .bind(key)
    .bind(value)
    .execute(executor)
    .await?;
    Ok(())
}
