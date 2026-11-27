// SPDX-FileCopyrightText: 2026 James Paul Vizuete Davila
// SPDX-License-Identifier: AGPL-3.0-or-later

use ateneo_core::db::{self, Incompatible, Inspection, MIN_SERVER_MAJOR, MIN_SERVER_VERSION};
use sqlx::PgPool;

#[sqlx::test(migrations = false)]
async fn empty_database_is_accepted(pool: PgPool) {
    let inspection = db::inspect(&pool).await.unwrap();

    assert!(inspection.server_version_num >= MIN_SERVER_VERSION);
    assert!(inspection.foreign_tables.is_empty());
    assert_eq!(inspection.check(), Ok(()));
}

#[sqlx::test(migrations = false)]
async fn ateneo_database_is_accepted(pool: PgPool) {
    db::migrate(&pool).await.unwrap();

    let inspection = db::inspect(&pool).await.unwrap();

    assert_eq!(inspection.check(), Ok(()));
}

#[sqlx::test(migrations = false)]
async fn foreign_tables_are_rejected(pool: PgPool) {
    sqlx::query("CREATE TABLE clientes (id int)")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("CREATE SCHEMA ventas")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("CREATE TABLE ventas.facturas (id int)")
        .execute(&pool)
        .await
        .unwrap();

    let inspection = db::inspect(&pool).await.unwrap();

    assert_eq!(
        inspection.check(),
        Err(Incompatible::ForeignTables(vec![
            "public.clientes".to_string(),
            "ventas.facturas".to_string(),
        ]))
    );
}

#[sqlx::test(migrations = false)]
async fn migrations_of_another_app_are_rejected(pool: PgPool) {
    db::migrate(&pool).await.unwrap();
    sqlx::query(
        "INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time) \
         VALUES (99999, 'otra app', true, '\\x00', 0)",
    )
    .execute(&pool)
    .await
    .unwrap();

    let inspection = db::inspect(&pool).await.unwrap();

    assert!(
        matches!(inspection.check(), Err(Incompatible::ForeignTables(tables)) if tables.contains(&"public._sqlx_migrations".to_string()))
    );
}

#[test]
fn old_servers_are_rejected() {
    let inspection = Inspection {
        server_version: "14.12".into(),
        server_version_num: 140_012,
        foreign_tables: Vec::new(),
    };

    assert_eq!(
        inspection.check(),
        Err(Incompatible::OldServer {
            version: "14.12".into()
        })
    );
    assert_eq!(MIN_SERVER_MAJOR, 15);
}

#[test]
fn version_fifteen_is_enough() {
    let inspection = Inspection {
        server_version: "15.0".into(),
        server_version_num: 150_000,
        foreign_tables: Vec::new(),
    };

    assert_eq!(inspection.check(), Ok(()));
}
