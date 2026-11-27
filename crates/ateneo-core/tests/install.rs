// SPDX-FileCopyrightText: 2026 James Paul Vizuete Davila
// SPDX-License-Identifier: AGPL-3.0-or-later

use ateneo_core::install::{self, DatabaseSource, Install};
use ateneo_core::settings;
use serde_json::{Value, json};
use sqlx::PgPool;

fn answers() -> Install {
    Install {
        domain: "repositorio.demo.edu.ec".into(),
        database: DatabaseSource::Docker,
        site_language: "es".into(),
        timezone: "America/Guayaquil".into(),
    }
}

#[sqlx::test(migrations = false)]
async fn save_stores_settings_and_audits(pool: PgPool) {
    ateneo_core::db::migrate(&pool).await.unwrap();

    install::save(&pool, &answers()).await.unwrap();

    assert_eq!(
        settings::get_text(&pool, settings::SITE_LANGUAGE)
            .await
            .unwrap()
            .as_deref(),
        Some("es")
    );
    assert_eq!(
        settings::get_text(&pool, settings::SITE_TIMEZONE)
            .await
            .unwrap()
            .as_deref(),
        Some("America/Guayaquil")
    );
    assert_eq!(
        settings::get_text(&pool, settings::SITE_STATE)
            .await
            .unwrap()
            .as_deref(),
        Some("install_incomplete")
    );

    let (actor, details): (String, Value) =
        sqlx::query_as("SELECT actor_kind, details FROM audit_log WHERE action = 'install.init'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(actor, "console");
    assert_eq!(
        details,
        json!({
            "domain": "repositorio.demo.edu.ec",
            "database": "docker",
            "site_language": "es",
            "timezone": "America/Guayaquil",
        })
    );
}

#[sqlx::test(migrations = false)]
async fn save_can_be_repeated_after_a_failed_install(pool: PgPool) {
    ateneo_core::db::migrate(&pool).await.unwrap();

    install::save(&pool, &answers()).await.unwrap();
    let mut second = answers();
    second.timezone = "UTC".into();
    install::save(&pool, &second).await.unwrap();

    assert_eq!(
        settings::get_text(&pool, settings::SITE_TIMEZONE)
            .await
            .unwrap()
            .as_deref(),
        Some("UTC")
    );
}

#[sqlx::test(migrations = false)]
async fn save_fails_without_migrations_and_leaves_nothing(pool: PgPool) {
    assert!(install::save(&pool, &answers()).await.is_err());

    let tables: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM pg_tables WHERE schemaname = 'public' AND tablename = 'settings'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(tables, 0);
}
