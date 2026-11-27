// SPDX-FileCopyrightText: 2026 James Paul Vizuete Davila
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::path::{Path, PathBuf};

use ateneo_core::config::{
    self, Config, ConfigError, FileKey, InvalidReason, NewConfig, create_new, current_value,
    prepare_edit, redact_url, replace,
};

const WITH_COMMENTS: &str = r#"# Configuración del servidor
domain = "viejo.edu.ec"

# Conexión
[database]
url_file = "/run/secrets/db"
"#;

fn no_env(_: &str) -> Option<String> {
    None
}

fn temp_path(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "ateneo-file-test-{}-{name}.toml",
        std::process::id()
    ));
    let _ = fs::remove_file(&path);
    path
}

#[test]
fn new_config_with_url_is_complete() {
    let text = NewConfig {
        domain: "repositorio.demo.edu.ec".into(),
        database_url: Some("postgres://ateneo:clave@localhost/ateneo".into()),
    }
    .to_toml();

    let config = Config::from_toml(Path::new("ateneo.toml"), &text, no_env).unwrap();

    assert_eq!(config.domain, "repositorio.demo.edu.ec");
    assert_eq!(config.port, config::DEFAULT_PORT);
    assert_eq!(
        config.database.url,
        "postgres://ateneo:clave@localhost/ateneo"
    );
    assert_eq!(
        config.database.max_connections,
        config::DEFAULT_MAX_CONNECTIONS
    );
}

#[test]
fn new_config_without_url_uses_the_environment() {
    let text = NewConfig {
        domain: "demo.edu".into(),
        database_url: None,
    }
    .to_toml();

    assert!(!text.contains("url"));
    let config = Config::from_toml(Path::new("ateneo.toml"), &text, |name| {
        (name == config::ENV_DATABASE_URL).then(|| "postgres://db/ateneo".to_string())
    })
    .unwrap();
    assert_eq!(config.database.url, "postgres://db/ateneo");
}

#[test]
fn new_config_escapes_quotes() {
    let text = NewConfig {
        domain: "demo.edu".into(),
        database_url: Some("postgres://u:a\"b@localhost/a".into()),
    }
    .to_toml();

    let config = Config::from_toml(Path::new("ateneo.toml"), &text, no_env).unwrap();
    assert_eq!(config.database.url, "postgres://u:a\"b@localhost/a");
}

#[test]
fn create_new_writes_the_file() {
    let path = temp_path("create");

    create_new(&path, "domain = \"demo.edu\"\n").unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "domain = \"demo.edu\"\n"
    );
    fs::remove_file(&path).unwrap();
}

#[test]
fn create_new_never_overwrites() {
    let path = temp_path("exists");
    fs::write(&path, "original").unwrap();

    let err = create_new(&path, "nuevo").unwrap_err();

    assert!(matches!(err, ConfigError::AlreadyExists { .. }));
    assert_eq!(fs::read_to_string(&path).unwrap(), "original");
    fs::remove_file(&path).unwrap();
}

#[cfg(unix)]
#[test]
fn create_new_and_replace_are_private() {
    use std::os::unix::fs::PermissionsExt;

    let path = temp_path("mode");
    create_new(&path, "domain = \"demo.edu\"\n").unwrap();
    assert_eq!(
        fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o600
    );

    replace(&path, "domain = \"otro.edu\"\n").unwrap();
    assert_eq!(
        fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    fs::remove_file(&path).unwrap();
}

#[test]
fn replace_changes_the_contents() {
    let path = temp_path("replace");
    fs::write(&path, "domain = \"viejo.edu\"\n").unwrap();

    replace(&path, "domain = \"nuevo.edu\"\n").unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "domain = \"nuevo.edu\"\n"
    );
    fs::remove_file(&path).unwrap();
}

#[test]
fn edit_keeps_comments_and_reports_old_value() {
    let edit = prepare_edit(
        Path::new("ateneo.toml"),
        WITH_COMMENTS,
        FileKey::Domain,
        " nuevo.edu.ec ",
    )
    .unwrap();

    assert_eq!(edit.old.as_deref(), Some("viejo.edu.ec"));
    assert_eq!(edit.new, "nuevo.edu.ec");
    assert!(edit.changes());
    assert!(edit.contents().contains("# Configuración del servidor"));
    assert!(edit.contents().contains("# Conexión"));
    assert!(edit.contents().contains("domain = \"nuevo.edu.ec\""));
}

#[test]
fn edit_uses_defaults_as_old_value() {
    let edit = prepare_edit(
        Path::new("ateneo.toml"),
        WITH_COMMENTS,
        FileKey::Port,
        "9000",
    )
    .unwrap();

    assert_eq!(edit.old.as_deref(), Some("8080"));
    assert!(edit.contents().contains("port = 9000"));
}

#[test]
fn edit_of_database_url_removes_url_file() {
    let edit = prepare_edit(
        Path::new("ateneo.toml"),
        WITH_COMMENTS,
        FileKey::DatabaseUrl,
        "postgres://localhost/ateneo",
    )
    .unwrap();

    assert_eq!(edit.old, None);
    assert!(!edit.contents().contains("url_file"));
    let config = Config::from_toml(Path::new("ateneo.toml"), edit.contents(), no_env).unwrap();
    assert_eq!(config.database.url, "postgres://localhost/ateneo");
}

#[test]
fn edit_creates_the_database_table() {
    let edit = prepare_edit(
        Path::new("ateneo.toml"),
        "domain = \"demo.edu\"\n",
        FileKey::DatabaseMaxConnections,
        "4",
    )
    .unwrap();

    assert!(edit.contents().contains("[database]"));
    assert!(edit.contents().contains("max_connections = 4"));
}

#[test]
fn edit_with_same_value_has_no_changes() {
    let edit = prepare_edit(
        Path::new("ateneo.toml"),
        WITH_COMMENTS,
        FileKey::Domain,
        "viejo.edu.ec",
    )
    .unwrap();

    assert!(!edit.changes());
}

#[test]
fn edit_rejects_invalid_values() {
    let err =
        prepare_edit(Path::new("ateneo.toml"), WITH_COMMENTS, FileKey::Port, "0").unwrap_err();
    assert!(matches!(
        err,
        ConfigError::Invalid {
            key: "port",
            reason: InvalidReason::PortOutOfRange
        }
    ));

    let err = prepare_edit(
        Path::new("ateneo.toml"),
        WITH_COMMENTS,
        FileKey::Domain,
        "https://demo.edu",
    )
    .unwrap_err();
    assert!(matches!(
        err,
        ConfigError::Invalid {
            reason: InvalidReason::HasProtocol(_),
            ..
        }
    ));
}

#[test]
fn edit_rejects_broken_files() {
    let err = prepare_edit(
        Path::new("ateneo.toml"),
        "domain = ",
        FileKey::Domain,
        "a.edu",
    )
    .unwrap_err();

    assert!(matches!(err, ConfigError::Parse { .. }));
}

#[test]
fn current_value_reads_file_or_default() {
    let path = Path::new("ateneo.toml");

    assert_eq!(
        current_value(path, WITH_COMMENTS, FileKey::Domain)
            .unwrap()
            .as_deref(),
        Some("viejo.edu.ec")
    );
    assert_eq!(
        current_value(path, WITH_COMMENTS, FileKey::DataDir)
            .unwrap()
            .as_deref(),
        Some(config::DEFAULT_DATA_DIR)
    );
    assert_eq!(
        current_value(path, WITH_COMMENTS, FileKey::DatabaseUrl).unwrap(),
        None
    );
}

#[test]
fn keys_round_trip_by_name() {
    for key in FileKey::ALL {
        assert_eq!(FileKey::from_name(key.name()), Some(key));
    }
    assert_eq!(FileKey::from_name("database.url_file"), None);
}

#[test]
fn redact_url_hides_only_the_password() {
    assert_eq!(
        redact_url("postgres://ateneo:secreto@localhost:5432/ateneo"),
        "postgres://ateneo:***@localhost:5432/ateneo"
    );
    assert_eq!(
        redact_url("postgres://ateneo@localhost/ateneo"),
        "postgres://ateneo@localhost/ateneo"
    );
    assert_eq!(
        redact_url("postgres://localhost/a@b"),
        "postgres://localhost/a@b"
    );
    assert_eq!(
        redact_url("postgresql://u:p@ss:w@db/x"),
        "postgresql://u:***@db/x"
    );
}

#[test]
fn path_follows_the_environment() {
    assert_eq!(
        config::path_with(no_env),
        PathBuf::from(config::DEFAULT_PATH)
    );
    assert_eq!(
        config::path_with(|name| (name == config::ENV_CONFIG).then(|| "/etc/ateneo.toml".into())),
        PathBuf::from("/etc/ateneo.toml")
    );
}

#[test]
fn database_url_from_env_reads_files_and_conflicts() {
    assert_eq!(config::database_url_from_env(no_env).unwrap(), None);

    let err = config::database_url_from_env(|_| Some("x".into())).unwrap_err();
    assert!(matches!(err, ConfigError::Conflict { .. }));
}
