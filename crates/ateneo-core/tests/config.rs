// SPDX-FileCopyrightText: 2026 James Paul Vizuete Davila
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use ateneo_core::config::{Config, ConfigError};

const FULL: &str = r#"
domain = "repositorio.demo.edu.ec"
port = 9000
data_dir = "/opt/ateneo/datos"

[database]
url = "postgres://ateneo:secreto@localhost/ateneo"
max_connections = 4
"#;

fn env(vars: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
    let map: HashMap<String, String> = vars
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    move |name| map.get(name).cloned()
}

fn parse(text: &str, vars: &[(&str, &str)]) -> Result<Config, ConfigError> {
    Config::from_toml(Path::new("ateneo.toml"), text, env(vars))
}

fn temp_file(name: &str, contents: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("ateneo-test-{}-{name}", std::process::id()));
    fs::write(&path, contents).unwrap();
    path
}

#[test]
fn reads_every_field_from_the_file() {
    let config = parse(FULL, &[]).unwrap();

    assert_eq!(config.domain, "repositorio.demo.edu.ec");
    assert_eq!(config.port, 9000);
    assert_eq!(config.data_dir, PathBuf::from("/opt/ateneo/datos"));
    assert_eq!(
        config.database.url,
        "postgres://ateneo:secreto@localhost/ateneo"
    );
    assert_eq!(config.database.max_connections, 4);
}

#[test]
fn uses_defaults_for_optional_fields() {
    let config = parse(
        "domain = \"demo.edu\"\n[database]\nurl = \"postgres://localhost/ateneo\"\n",
        &[],
    )
    .unwrap();

    assert_eq!(config.port, 8080);
    assert_eq!(config.data_dir, PathBuf::from("datos"));
    assert_eq!(config.database.max_connections, 10);
}

#[test]
fn environment_overrides_the_file() {
    let config = parse(
        FULL,
        &[
            ("ATENEO_DOMAIN", "otro.edu"),
            ("ATENEO_PORT", "3000"),
            ("ATENEO_DATABASE_URL", "postgresql://db/ateneo"),
        ],
    )
    .unwrap();

    assert_eq!(config.domain, "otro.edu");
    assert_eq!(config.port, 3000);
    assert_eq!(config.database.url, "postgresql://db/ateneo");
}

#[test]
fn works_with_environment_only() {
    let config = Config::load_with(env(&[
        ("ATENEO_DOMAIN", "demo.edu"),
        ("ATENEO_DATABASE_URL", "postgres://localhost/ateneo"),
    ]))
    .unwrap();

    assert_eq!(config.domain, "demo.edu");
    assert_eq!(config.port, 8080);
}

#[test]
fn explicit_config_path_must_exist() {
    let missing = std::env::temp_dir().join("ateneo-no-existe.toml");
    let err = Config::load_with(env(&[("ATENEO_CONFIG", missing.to_str().unwrap())])).unwrap_err();

    assert!(matches!(err, ConfigError::NotFound { ref path } if *path == missing));
    assert!(err.to_string().contains("no existe el archivo"));
}

#[test]
fn loads_from_explicit_path() {
    let path = temp_file("explicit.toml", FULL);
    let config = Config::load_with(env(&[("ATENEO_CONFIG", path.to_str().unwrap())])).unwrap();
    fs::remove_file(&path).unwrap();

    assert_eq!(config.domain, "repositorio.demo.edu.ec");
}

#[test]
fn reports_missing_domain() {
    let err = parse("[database]\nurl = \"postgres://localhost/ateneo\"\n", &[]).unwrap_err();

    assert!(matches!(
        err,
        ConfigError::Missing {
            key: "domain",
            var: "ATENEO_DOMAIN"
        }
    ));
    assert_eq!(
        err.to_string(),
        "falta `domain`: agrégalo en ateneo.toml o usa la variable ATENEO_DOMAIN"
    );
}

#[test]
fn reports_missing_database_url() {
    let err = parse("domain = \"demo.edu\"\n", &[]).unwrap_err();

    assert!(matches!(
        err,
        ConfigError::Missing {
            key: "database.url",
            var: "ATENEO_DATABASE_URL"
        }
    ));
}

#[test]
fn reports_toml_syntax_errors_with_the_path() {
    let err = parse("domain = \n", &[]).unwrap_err();

    assert!(matches!(err, ConfigError::Parse { .. }));
    assert!(err.to_string().starts_with("ateneo.toml tiene un error"));
}

#[test]
fn rejects_unknown_fields() {
    let err = parse(&format!("{FULL}\nextra = 1\n"), &[]).unwrap_err();

    assert!(matches!(err, ConfigError::Parse { .. }));
}

#[test]
fn rejects_domain_with_protocol() {
    let err = parse(
        "domain = \"https://demo.edu\"\n[database]\nurl = \"postgres://localhost/a\"\n",
        &[],
    )
    .unwrap_err();

    assert!(matches!(err, ConfigError::Invalid { key: "domain", .. }));
}

#[test]
fn rejects_invalid_port_from_environment() {
    let err = parse(FULL, &[("ATENEO_PORT", "ochenta")]).unwrap_err();

    assert!(matches!(err, ConfigError::Invalid { key: "port", .. }));
    assert!(err.to_string().contains("ochenta"));
}

#[test]
fn rejects_port_zero() {
    let err = parse(FULL, &[("ATENEO_PORT", "0")]).unwrap_err();

    assert!(matches!(err, ConfigError::Invalid { key: "port", .. }));
}

#[test]
fn rejects_non_postgres_url_without_showing_it() {
    let err = parse(FULL, &[("ATENEO_DATABASE_URL", "mysql://root:clave@db/a")]).unwrap_err();

    assert!(matches!(
        err,
        ConfigError::Invalid {
            key: "database.url",
            ..
        }
    ));
    assert!(!err.to_string().contains("clave"));
}

#[test]
fn reads_database_url_from_a_file() {
    let secret = temp_file("url", "postgres://ateneo:desde-archivo@localhost/ateneo\n");
    let config = parse(
        "domain = \"demo.edu\"\n",
        &[("ATENEO_DATABASE_URL_FILE", secret.to_str().unwrap())],
    )
    .unwrap();
    fs::remove_file(&secret).unwrap();

    assert_eq!(
        config.database.url,
        "postgres://ateneo:desde-archivo@localhost/ateneo"
    );
}

#[test]
fn rejects_url_and_url_file_together() {
    let err = parse(
        FULL,
        &[
            ("ATENEO_DATABASE_URL", "postgres://localhost/a"),
            ("ATENEO_DATABASE_URL_FILE", "/run/secrets/db"),
        ],
    )
    .unwrap_err();

    assert!(matches!(err, ConfigError::Conflict { .. }));
}

#[test]
fn debug_output_hides_the_database_url() {
    let config = parse(FULL, &[]).unwrap();

    assert!(!format!("{config:?}").contains("secreto"));
}
