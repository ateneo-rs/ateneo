// SPDX-FileCopyrightText: 2026 James Paul Vizuete Davila
// SPDX-License-Identifier: AGPL-3.0-or-later

use sqlx::PgPool;

#[sqlx::test(migrations = false)]
async fn migrations_create_central_tables(pool: PgPool) {
    ateneo_core::db::migrate(&pool).await.unwrap();

    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT tablename::text FROM pg_tables \
         WHERE schemaname = 'public' AND tablename IN ('settings', 'audit_log') \
         ORDER BY tablename",
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    assert_eq!(tables, ["audit_log", "settings"]);
}

#[sqlx::test(migrations = false)]
async fn migrations_can_run_twice(pool: PgPool) {
    ateneo_core::db::migrate(&pool).await.unwrap();
    ateneo_core::db::migrate(&pool).await.unwrap();
}

#[sqlx::test(migrations = false)]
async fn settings_store_json_values(pool: PgPool) {
    ateneo_core::db::migrate(&pool).await.unwrap();

    sqlx::query("INSERT INTO settings (key, value) VALUES ('site.name', '\"Repositorio\"')")
        .execute(&pool)
        .await
        .unwrap();

    let name: String =
        sqlx::query_scalar("SELECT value #>> '{}' FROM settings WHERE key = 'site.name'")
            .fetch_one(&pool)
            .await
            .unwrap();

    assert_eq!(name, "Repositorio");
}

#[sqlx::test(migrations = false)]
async fn audit_log_rejects_unknown_actor_kind(pool: PgPool) {
    ateneo_core::db::migrate(&pool).await.unwrap();

    let result =
        sqlx::query("INSERT INTO audit_log (actor_kind, action) VALUES ('robot', 'config.set')")
            .execute(&pool)
            .await;

    assert!(result.is_err());

    sqlx::query("INSERT INTO audit_log (actor_kind, action) VALUES ('console', 'config.set')")
        .execute(&pool)
        .await
        .unwrap();
}
