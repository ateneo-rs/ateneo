// SPDX-FileCopyrightText: 2026 James Paul Vizuete Davila
// SPDX-License-Identifier: AGPL-3.0-or-later

mod error;
mod file;
mod url;

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde::Deserialize;

pub use error::{ConfigError, InvalidReason};
pub use file::{Edit, FileKey, NewConfig, create_new, current_value, prepare_edit, replace};
pub use url::{DEFAULT_DATABASE_PORT, DatabaseParts, redact_url, validate_host, validate_name};

pub const DEFAULT_PATH: &str = "ateneo.toml";
pub const DEFAULT_PORT: u16 = 8080;
pub const DEFAULT_DATA_DIR: &str = "datos";
pub const DEFAULT_MAX_CONNECTIONS: u32 = 10;

pub const ENV_CONFIG: &str = "ATENEO_CONFIG";
pub const ENV_DOMAIN: &str = "ATENEO_DOMAIN";
pub const ENV_PORT: &str = "ATENEO_PORT";
pub const ENV_DATA_DIR: &str = "ATENEO_DATA_DIR";
pub const ENV_DATABASE_URL: &str = "ATENEO_DATABASE_URL";
pub const ENV_DATABASE_URL_FILE: &str = "ATENEO_DATABASE_URL_FILE";
pub const ENV_DATABASE_MAX_CONNECTIONS: &str = "ATENEO_DATABASE_MAX_CONNECTIONS";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub domain: String,
    pub port: u16,
    pub data_dir: PathBuf,
    pub database: DatabaseConfig,
}

#[derive(Clone, PartialEq, Eq)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

impl fmt::Debug for DatabaseConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DatabaseConfig")
            .field("url", &"***")
            .field("max_connections", &self.max_connections)
            .finish()
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileConfig {
    domain: Option<String>,
    port: Option<u16>,
    data_dir: Option<PathBuf>,
    #[serde(default)]
    database: FileDatabase,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileDatabase {
    url: Option<String>,
    url_file: Option<PathBuf>,
    max_connections: Option<u32>,
}

pub fn path() -> PathBuf {
    path_with(|name| std::env::var(name).ok())
}

pub fn path_with(env: impl Fn(&str) -> Option<String>) -> PathBuf {
    env(ENV_CONFIG)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_PATH))
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        Self::load_with(|name| std::env::var(name).ok())
    }

    pub fn load_with(env: impl Fn(&str) -> Option<String>) -> Result<Self, ConfigError> {
        let explicit = env(ENV_CONFIG).is_some();
        let path = path_with(&env);
        match fs::read_to_string(&path) {
            Ok(text) => Self::from_toml(&path, &text, env),
            Err(err) if err.kind() == io::ErrorKind::NotFound && !explicit => {
                resolve(FileConfig::default(), &env)
            }
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                Err(ConfigError::NotFound { path })
            }
            Err(source) => Err(ConfigError::Read { path, source }),
        }
    }

    pub fn from_toml(
        path: &Path,
        text: &str,
        env: impl Fn(&str) -> Option<String>,
    ) -> Result<Self, ConfigError> {
        resolve(parse_file(path, text)?, &env)
    }
}

fn parse_file(path: &Path, text: &str) -> Result<FileConfig, ConfigError> {
    toml::from_str(text).map_err(|source| ConfigError::Parse {
        path: path.to_path_buf(),
        source: Box::new(source),
    })
}

fn resolve(file: FileConfig, env: &impl Fn(&str) -> Option<String>) -> Result<Config, ConfigError> {
    let domain = env(ENV_DOMAIN)
        .or(file.domain)
        .ok_or(ConfigError::Missing {
            key: "domain",
            var: ENV_DOMAIN,
        })?;
    let domain = validate_domain(&domain)?;

    let port = match env(ENV_PORT) {
        Some(value) => validate_port(&value)?,
        None => check_port(file.port.unwrap_or(DEFAULT_PORT))?,
    };

    let data_dir = env(ENV_DATA_DIR)
        .map(PathBuf::from)
        .or(file.data_dir)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_DATA_DIR));

    let max_connections = match env(ENV_DATABASE_MAX_CONNECTIONS) {
        Some(value) => validate_max_connections(&value)?,
        None => check_max_connections(
            file.database
                .max_connections
                .unwrap_or(DEFAULT_MAX_CONNECTIONS),
        )?,
    };

    let url = match database_url_from_env(env)? {
        Some(url) => url,
        None => match (file.database.url, file.database.url_file) {
            (Some(_), Some(_)) => {
                return Err(ConfigError::Conflict {
                    first: "database.url",
                    second: "database.url_file",
                });
            }
            (Some(url), None) => url,
            (None, Some(path)) => read_secret(path)?,
            (None, None) => {
                return Err(ConfigError::Missing {
                    key: "database.url",
                    var: ENV_DATABASE_URL,
                });
            }
        },
    };
    let url = validate_database_url(&url)?;

    Ok(Config {
        domain,
        port,
        data_dir,
        database: DatabaseConfig {
            url,
            max_connections,
        },
    })
}

pub fn database_url_from_env(
    env: impl Fn(&str) -> Option<String>,
) -> Result<Option<String>, ConfigError> {
    match (env(ENV_DATABASE_URL), env(ENV_DATABASE_URL_FILE)) {
        (Some(_), Some(_)) => Err(ConfigError::Conflict {
            first: ENV_DATABASE_URL,
            second: ENV_DATABASE_URL_FILE,
        }),
        (Some(url), None) => Ok(Some(url)),
        (None, Some(path)) => read_secret(PathBuf::from(path)).map(Some),
        (None, None) => Ok(None),
    }
}

fn parse_number<T: FromStr>(key: &'static str, value: &str) -> Result<T, ConfigError> {
    value.trim().parse().map_err(|_| ConfigError::Invalid {
        key,
        reason: InvalidReason::NotANumber(value.to_string()),
    })
}

fn read_secret(path: PathBuf) -> Result<String, ConfigError> {
    match fs::read_to_string(&path) {
        Ok(text) => Ok(text.trim().to_string()),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Err(ConfigError::NotFound { path }),
        Err(source) => Err(ConfigError::Read { path, source }),
    }
}

pub fn validate_domain(domain: &str) -> Result<String, ConfigError> {
    let domain = domain.trim().to_string();
    let reason = if domain.is_empty() {
        Some(InvalidReason::Empty)
    } else if domain.contains("://") {
        Some(InvalidReason::HasProtocol(domain.clone()))
    } else if domain.contains('/') || domain.chars().any(char::is_whitespace) {
        Some(InvalidReason::HasSlashOrSpace(domain.clone()))
    } else {
        None
    };
    match reason {
        Some(reason) => Err(ConfigError::Invalid {
            key: "domain",
            reason,
        }),
        None => Ok(domain),
    }
}

pub fn validate_port(value: &str) -> Result<u16, ConfigError> {
    check_port(parse_number("port", value)?)
}

fn check_port(port: u16) -> Result<u16, ConfigError> {
    if port == 0 {
        return Err(ConfigError::Invalid {
            key: "port",
            reason: InvalidReason::PortOutOfRange,
        });
    }
    Ok(port)
}

pub fn validate_max_connections(value: &str) -> Result<u32, ConfigError> {
    check_max_connections(parse_number("database.max_connections", value)?)
}

fn check_max_connections(max_connections: u32) -> Result<u32, ConfigError> {
    if max_connections == 0 {
        return Err(ConfigError::Invalid {
            key: "database.max_connections",
            reason: InvalidReason::NoConnections,
        });
    }
    Ok(max_connections)
}

pub fn validate_data_dir(value: &str) -> Result<PathBuf, ConfigError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(ConfigError::Invalid {
            key: "data_dir",
            reason: InvalidReason::Empty,
        });
    }
    Ok(PathBuf::from(value))
}

pub fn validate_database_url(url: &str) -> Result<String, ConfigError> {
    let url = url.trim().to_string();
    if url.starts_with("postgres://") || url.starts_with("postgresql://") {
        Ok(url)
    } else {
        Err(ConfigError::Invalid {
            key: "database.url",
            reason: InvalidReason::NotPostgres,
        })
    }
}
