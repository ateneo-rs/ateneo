// SPDX-FileCopyrightText: 2026 James Paul Vizuete Davila
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::io;
use std::time::Duration;

use ateneo_core::config::{DatabaseConfig, DatabaseParts, redact_url, validate_host};
use ateneo_core::db::{self, ConnectProblem, DbError};
use sqlx::postgres::PgConnectOptions;

fn parts_from_env() -> DatabaseParts {
    let url = std::env::var("DATABASE_URL").unwrap();
    let rest = url.split_once("://").unwrap().1;
    let (credentials, location) = rest.rsplit_once('@').unwrap();
    let (user, password) = credentials.split_once(':').unwrap();
    let (address, database) = location.split_once('/').unwrap();
    let (host, port) = address.rsplit_once(':').unwrap();
    DatabaseParts {
        host: host.to_string(),
        port: port.parse().unwrap(),
        user: user.to_string(),
        password: password.to_string(),
        database: database.to_string(),
    }
}

async fn problem(parts: &DatabaseParts) -> Option<ConnectProblem> {
    let config = DatabaseConfig {
        url: parts.to_url(),
        max_connections: 1,
    };
    db::connect(&config)
        .await
        .err()
        .and_then(|err| err.problem())
}

#[test]
fn builds_urls_from_parts() {
    let parts = DatabaseParts {
        host: "db.ejemplo.edu".into(),
        port: 5433,
        user: "ateneo".into(),
        password: "clave".into(),
        database: "repositorio".into(),
    };

    assert_eq!(
        parts.to_url(),
        "postgres://ateneo:clave@db.ejemplo.edu:5433/repositorio"
    );
}

#[test]
fn encodes_special_characters() {
    let parts = DatabaseParts {
        host: "localhost".into(),
        port: 5432,
        user: "ana pérez".into(),
        password: "p@ss:w/rd#?%".into(),
        database: "mi base".into(),
    };
    let url = parts.to_url();

    let options: PgConnectOptions = url.parse().unwrap();
    assert_eq!(options.get_username(), "ana pérez");
    assert_eq!(options.get_database(), Some("mi base"));
    assert_eq!(options.get_host(), "localhost");
    assert_eq!(
        redact_url(&url),
        "postgres://ana%20p%C3%A9rez:***@localhost:5432/mi%20base"
    );
}

#[test]
fn empty_password_is_left_out() {
    let parts = DatabaseParts {
        host: "localhost".into(),
        port: 5432,
        user: "ateneo".into(),
        password: String::new(),
        database: "ateneo".into(),
    };

    assert_eq!(parts.to_url(), "postgres://ateneo@localhost:5432/ateneo");
}

#[test]
fn wraps_ipv6_hosts() {
    let parts = DatabaseParts {
        host: "::1".into(),
        port: 5432,
        user: "a".into(),
        password: String::new(),
        database: "b".into(),
    };

    assert_eq!(parts.to_url(), "postgres://a@[::1]:5432/b");
}

#[test]
fn debug_hides_the_password() {
    let mut parts = parts_from_env();
    parts.password = "muy-secreta".into();

    assert!(!format!("{parts:?}").contains("muy-secreta"));
}

#[test]
fn validates_hosts() {
    assert_eq!(validate_host(" localhost ").unwrap(), "localhost");
    assert!(validate_host("").is_err());
    assert!(validate_host("postgres://localhost").is_err());
    assert!(validate_host("usuario@localhost").is_err());
    assert!(validate_host("mi servidor").is_err());
}

#[tokio::test]
async fn connects_with_parts() {
    let parts = parts_from_env();

    assert_eq!(problem(&parts).await, None);
}

#[tokio::test]
async fn reports_wrong_password() {
    let mut parts = parts_from_env();
    parts.password = "incorrecta".into();

    assert_eq!(problem(&parts).await, Some(ConnectProblem::BadCredentials));
}

#[tokio::test]
async fn reports_missing_database() {
    let mut parts = parts_from_env();
    parts.database = "ateneo_no_existe".into();

    assert_eq!(problem(&parts).await, Some(ConnectProblem::DatabaseMissing));
}

#[tokio::test]
async fn reports_no_server() {
    let mut parts = parts_from_env();
    parts.host = "127.0.0.1".into();
    parts.port = 1;

    assert_eq!(problem(&parts).await, Some(ConnectProblem::NoServer));
}

#[test]
fn classifies_timeouts() {
    assert_eq!(
        db::connect_problem(&sqlx::Error::PoolTimedOut),
        ConnectProblem::Timeout
    );
    assert_eq!(
        db::connect_problem(&sqlx::Error::Io(io::Error::from(io::ErrorKind::TimedOut))),
        ConnectProblem::Timeout
    );
    assert_eq!(
        DbError::ConnectTimeout.problem(),
        Some(ConnectProblem::Timeout)
    );
}

#[test]
fn detects_listening_servers() {
    let parts = parts_from_env();

    assert!(db::is_listening(
        &parts.host,
        parts.port,
        Duration::from_secs(1)
    ));
    assert!(!db::is_listening(
        "127.0.0.1",
        1,
        Duration::from_millis(300)
    ));
}
