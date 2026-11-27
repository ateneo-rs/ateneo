// SPDX-FileCopyrightText: 2026 James Paul Vizuete Davila
// SPDX-License-Identifier: AGPL-3.0-or-later

use ateneo_core::audit::{self, Entry};
use serde_json::{Value, json};
use sqlx::PgPool;

#[sqlx::test(migrations = false)]
async fn records_console_entries(pool: PgPool) {
    ateneo_core::db::migrate(&pool).await.unwrap();

    let id = audit::record(
        &pool,
        &Entry::console(audit::CONFIG_SET)
            .target("config_file", "domain")
            .details(json!({"old": "a.edu", "new": "b.edu"})),
    )
    .await
    .unwrap();

    let (actor, action, target_type, target_id, details): (
        String,
        String,
        Option<String>,
        Option<String>,
        Value,
    ) = sqlx::query_as(
        "SELECT actor_kind, action, target_type, target_id, details FROM audit_log WHERE id = $1",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(actor, "console");
    assert_eq!(action, "config.set");
    assert_eq!(target_type.as_deref(), Some("config_file"));
    assert_eq!(target_id.as_deref(), Some("domain"));
    assert_eq!(details, json!({"old": "a.edu", "new": "b.edu"}));
}

#[sqlx::test(migrations = false)]
async fn entries_without_target_have_empty_details(pool: PgPool) {
    ateneo_core::db::migrate(&pool).await.unwrap();

    let id = audit::record(&pool, &Entry::console(audit::INSTALL_INIT))
        .await
        .unwrap();

    let details: Value = sqlx::query_scalar("SELECT details FROM audit_log WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(details, json!({}));
}
