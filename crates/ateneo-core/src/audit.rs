// SPDX-FileCopyrightText: 2026 James Paul Vizuete Davila
// SPDX-License-Identifier: AGPL-3.0-or-later

use serde_json::Value;
use sqlx::PgExecutor;

pub const INSTALL_INIT: &str = "install.init";
pub const CONFIG_SET: &str = "config.set";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Actor {
    Console,
    System,
}

impl Actor {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Console => "console",
            Self::System => "system",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Entry<'a> {
    pub actor: Actor,
    pub action: &'a str,
    pub target_type: Option<&'a str>,
    pub target_id: Option<&'a str>,
    pub details: Value,
}

impl<'a> Entry<'a> {
    pub fn console(action: &'a str) -> Self {
        Self {
            actor: Actor::Console,
            action,
            target_type: None,
            target_id: None,
            details: Value::Object(Default::default()),
        }
    }

    pub fn target(mut self, kind: &'a str, id: &'a str) -> Self {
        self.target_type = Some(kind);
        self.target_id = Some(id);
        self
    }

    pub fn details(mut self, details: Value) -> Self {
        self.details = details;
        self
    }
}

pub async fn record<'e>(executor: impl PgExecutor<'e>, entry: &Entry<'_>) -> sqlx::Result<i64> {
    sqlx::query_scalar(
        "INSERT INTO audit_log (actor_kind, action, target_type, target_id, details) \
         VALUES ($1, $2, $3, $4, $5) RETURNING id",
    )
    .bind(entry.actor.as_str())
    .bind(entry.action)
    .bind(entry.target_type)
    .bind(entry.target_id)
    .bind(&entry.details)
    .fetch_one(executor)
    .await
}
