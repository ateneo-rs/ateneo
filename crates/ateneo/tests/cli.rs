// SPDX-FileCopyrightText: 2026 James Paul Vizuete Davila
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use serde_json::{Value, json};
use sqlx::PgPool;

const ATENEO_VARS: &[&str] = &[
    "ATENEO_CONFIG",
    "ATENEO_DOMAIN",
    "ATENEO_PORT",
    "ATENEO_DATA_DIR",
    "ATENEO_DATABASE_URL",
    "ATENEO_DATABASE_URL_FILE",
    "ATENEO_DATABASE_MAX_CONNECTIONS",
];

fn ateneo(config: Option<&Path>, args: &[&str]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_ateneo"));
    for var in ATENEO_VARS {
        command.env_remove(var);
    }
    if let Some(config) = config {
        command.env("ATENEO_CONFIG", config);
    }
    command.args(args).stdin(Stdio::null()).output().unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn temp_path(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "ateneo-cli-test-{}-{name}.toml",
        std::process::id()
    ));
    let _ = fs::remove_file(&path);
    path
}

async fn database_url(pool: &PgPool) -> String {
    let base = std::env::var("DATABASE_URL").unwrap();
    let name: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(pool)
        .await
        .unwrap();
    let cut = base.rfind('/').unwrap();
    format!("{}/{name}", &base[..cut])
}

async fn installed(pool: &PgPool, name: &str) -> PathBuf {
    ateneo_core::db::migrate(pool).await.unwrap();
    let path = temp_path(name);
    let text = ateneo_core::config::NewConfig {
        domain: "viejo.edu.ec".into(),
        database_url: Some(database_url(pool).await),
    }
    .to_toml();
    fs::write(&path, text).unwrap();
    path
}

#[test]
fn without_subcommand_shows_help() {
    let output = ateneo(None, &[]);

    assert!(output.status.success());
    let text = stdout(&output);
    assert!(text.contains("Comandos:"));
    assert!(text.contains("init"));
    assert!(text.contains("config"));
    assert!(text.contains("status"));
    assert!(!text.contains("Usage"));
}

#[test]
fn init_does_not_overwrite_an_existing_file() {
    let path = temp_path("init-exists");
    fs::write(&path, "domain = \"mio.edu\"\n").unwrap();

    let output = ateneo(Some(&path), &["init"]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("Ya existe"));
    assert_eq!(fs::read_to_string(&path).unwrap(), "domain = \"mio.edu\"\n");
    fs::remove_file(&path).unwrap();
}

#[test]
fn init_without_terminal_explains_what_to_do() {
    let path = temp_path("init-no-tty");

    let output = ateneo(Some(&path), &["init"]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("docker compose exec -it"));
    assert!(!path.exists());
}

#[test]
fn config_menu_without_terminal_points_to_set() {
    let output = ateneo(None, &["config"]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("config set"));
}

#[test]
fn config_set_rejects_unknown_options() {
    let output = ateneo(None, &["config", "set", "color", "rojo", "--yes"]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("site.timezone"));
}

#[test]
fn config_set_without_terminal_needs_yes() {
    let output = ateneo(None, &["config", "set", "port", "9000"]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("--yes"));
}

#[sqlx::test(migrations = false)]
async fn config_set_changes_the_file_and_audits(pool: PgPool) {
    let path = installed(&pool, "set-domain").await;

    let output = ateneo(
        Some(&path),
        &["config", "set", "domain", "nuevo.edu.ec", "--yes"],
    );

    assert!(output.status.success(), "{}", stderr(&output));
    let shown = stdout(&output);
    assert!(shown.contains("Antes: viejo.edu.ec"));
    assert!(shown.contains("Después: nuevo.edu.ec"));
    assert!(
        fs::read_to_string(&path)
            .unwrap()
            .contains("domain = \"nuevo.edu.ec\"")
    );

    let (actor, target, details): (String, Option<String>, Value) = sqlx::query_as(
        "SELECT actor_kind, target_id, details FROM audit_log WHERE action = 'config.set'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(actor, "console");
    assert_eq!(target.as_deref(), Some("domain"));
    assert_eq!(
        details,
        json!({"key": "domain", "old": "viejo.edu.ec", "new": "nuevo.edu.ec"})
    );
    fs::remove_file(&path).unwrap();
}

#[sqlx::test(migrations = false)]
async fn config_set_hides_the_database_password(pool: PgPool) {
    let path = installed(&pool, "set-url").await;
    let url = database_url(&pool)
        .await
        .replacen("postgres://", "postgresql://", 1);

    let output = ateneo(
        Some(&path),
        &["config", "set", "database.url", &url, "--yes"],
    );

    assert!(output.status.success(), "{}", stderr(&output));
    let shown = stdout(&output);
    assert!(shown.contains("Antes: postgres://ateneo:***@"));
    assert!(shown.contains("Después: postgresql://ateneo:***@"));
    assert!(!shown.contains(":ateneo@"));
    assert!(fs::read_to_string(&path).unwrap().contains(&url));

    let details: Value =
        sqlx::query_scalar("SELECT details FROM audit_log WHERE action = 'config.set'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(details.to_string().contains("***"));
    assert!(!details.to_string().contains(":ateneo@"));
    fs::remove_file(&path).unwrap();
}

#[sqlx::test(migrations = false)]
async fn config_set_with_same_value_changes_nothing(pool: PgPool) {
    let path = installed(&pool, "set-same").await;

    let output = ateneo(
        Some(&path),
        &["config", "set", "domain", "viejo.edu.ec", "--yes"],
    );

    assert!(output.status.success());
    assert!(stdout(&output).contains("No se cambió nada"));
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM audit_log")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
    fs::remove_file(&path).unwrap();
}

#[sqlx::test(migrations = false)]
async fn config_set_changes_settings_and_audits(pool: PgPool) {
    let path = installed(&pool, "set-timezone").await;

    let output = ateneo(
        Some(&path),
        &[
            "config",
            "set",
            "site.timezone",
            "America/Guayaquil",
            "--yes",
        ],
    );

    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(
        ateneo_core::settings::get_text(&pool, "site.timezone")
            .await
            .unwrap()
            .as_deref(),
        Some("America/Guayaquil")
    );
    let target: String =
        sqlx::query_scalar("SELECT target_type FROM audit_log WHERE action = 'config.set'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(target, "setting");

    let output = ateneo(
        Some(&path),
        &["config", "set", "site.timezone", "Marte/Olympus", "--yes"],
    );
    assert!(!output.status.success());
    fs::remove_file(&path).unwrap();
}

#[sqlx::test(migrations = false)]
async fn config_set_rejects_invalid_values_without_writing(pool: PgPool) {
    let path = installed(&pool, "set-invalid").await;
    let before = fs::read_to_string(&path).unwrap();

    let output = ateneo(Some(&path), &["config", "set", "port", "0", "--yes"]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("65535"));
    assert_eq!(fs::read_to_string(&path).unwrap(), before);
    fs::remove_file(&path).unwrap();
}

#[sqlx::test(migrations = false)]
async fn status_shows_connection_and_migrations(pool: PgPool) {
    let path = installed(&pool, "status").await;
    ateneo_core::settings::set(&pool, "site.state", &json!("install_incomplete"))
        .await
        .unwrap();

    let output = ateneo(Some(&path), &["status"]);

    assert!(output.status.success(), "{}", stderr(&output));
    let text = stdout(&output);
    assert!(text.contains("Conexión: correcta"));
    assert!(text.contains("0001 settings audit log"));
    assert!(text.contains("Migraciones pendientes: ninguna"));
    assert!(text.contains("instalación incompleta"));
    assert!(!text.contains(":ateneo@"));
    fs::remove_file(&path).unwrap();
}

#[test]
fn status_reports_a_failed_connection() {
    let path = temp_path("status-down");
    fs::write(
        &path,
        "domain = \"demo.edu\"\n[database]\nurl = \"postgres://nadie:clave@127.0.0.1:1/ninguna\"\n",
    )
    .unwrap();

    let output = ateneo(Some(&path), &["status"]);

    assert!(!output.status.success());
    let text = stdout(&output);
    assert!(text.contains("Conexión: falló"));
    assert!(text.contains("No hay un servidor PostgreSQL en esa dirección."));
    assert!(!text.contains("Detalle técnico"));
    assert!(!text.contains("clave"));

    let verbose = ateneo(Some(&path), &["status", "--verbose"]);
    assert!(stdout(&verbose).contains("Detalle técnico"));

    let before = ateneo(Some(&path), &["--verbose", "status"]);
    assert!(stdout(&before).contains("Detalle técnico"));
    fs::remove_file(&path).unwrap();
}

#[sqlx::test(migrations = false)]
async fn status_explains_wrong_credentials(pool: PgPool) {
    let path = installed(&pool, "status-password").await;
    let text = fs::read_to_string(&path).unwrap();
    fs::write(&path, text.replace(":ateneo@", ":incorrecta@")).unwrap();

    let output = ateneo(Some(&path), &["status"]);

    assert!(!output.status.success());
    let text = stdout(&output);
    assert!(text.contains("El usuario o la contraseña de PostgreSQL no son correctos."));
    assert!(!text.contains("28P01"));
    assert!(!text.contains("incorrecta"));
    fs::remove_file(&path).unwrap();
}

#[test]
fn help_mentions_verbose() {
    let output = ateneo(None, &["--help"]);

    assert!(stdout(&output).contains("--verbose"));
}
