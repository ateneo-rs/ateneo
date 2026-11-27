// SPDX-FileCopyrightText: 2026 James Paul Vizuete Davila
// SPDX-License-Identifier: AGPL-3.0-or-later

mod error;

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde::Deserialize;

pub use error::ConfigError;

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

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        Self::load_with(|name| std::env::var(name).ok())
    }

    pub fn load_with(env: impl Fn(&str) -> Option<String>) -> Result<Self, ConfigError> {
        let (path, explicit) = match env(ENV_CONFIG) {
            Some(path) => (PathBuf::from(path), true),
            None => (PathBuf::from(DEFAULT_PATH), false),
        };
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
        let file: FileConfig = toml::from_str(text).map_err(|source| ConfigError::Parse {
            path: path.to_path_buf(),
            source: Box::new(source),
        })?;
        resolve(file, &env)
    }
}

fn resolve(file: FileConfig, env: &impl Fn(&str) -> Option<String>) -> Result<Config, ConfigError> {
    let domain = env(ENV_DOMAIN)
        .or(file.domain)
        .ok_or(ConfigError::Missing {
            key: "domain",
            var: ENV_DOMAIN,
        })?;
    let domain = validate_domain(domain)?;

    let port = match env(ENV_PORT) {
        Some(value) => parse_number("port", &value)?,
        None => file.port.unwrap_or(DEFAULT_PORT),
    };
    if port == 0 {
        return Err(ConfigError::Invalid {
            key: "port",
            reason: "debe ser un número entre 1 y 65535".into(),
        });
    }

    let data_dir = env(ENV_DATA_DIR)
        .map(PathBuf::from)
        .or(file.data_dir)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_DATA_DIR));

    let max_connections = match env(ENV_DATABASE_MAX_CONNECTIONS) {
        Some(value) => parse_number("database.max_connections", &value)?,
        None => file
            .database
            .max_connections
            .unwrap_or(DEFAULT_MAX_CONNECTIONS),
    };
    if max_connections == 0 {
        return Err(ConfigError::Invalid {
            key: "database.max_connections",
            reason: "debe ser al menos 1".into(),
        });
    }

    let url = match (env(ENV_DATABASE_URL), env(ENV_DATABASE_URL_FILE)) {
        (Some(_), Some(_)) => {
            return Err(ConfigError::Conflict {
                first: ENV_DATABASE_URL,
                second: ENV_DATABASE_URL_FILE,
            });
        }
        (Some(url), None) => url,
        (None, Some(path)) => read_secret(PathBuf::from(path))?,
        (None, None) => match (file.database.url, file.database.url_file) {
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
    let url = validate_database_url(url)?;

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

fn parse_number<T: FromStr>(key: &'static str, value: &str) -> Result<T, ConfigError> {
    value.trim().parse().map_err(|_| ConfigError::Invalid {
        key,
        reason: format!("«{value}» no es un número válido"),
    })
}

fn read_secret(path: PathBuf) -> Result<String, ConfigError> {
    match fs::read_to_string(&path) {
        Ok(text) => Ok(text.trim().to_string()),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Err(ConfigError::NotFound { path }),
        Err(source) => Err(ConfigError::Read { path, source }),
    }
}

fn validate_domain(domain: String) -> Result<String, ConfigError> {
    let domain = domain.trim().to_string();
    let reason = if domain.is_empty() {
        Some("está vacío".to_string())
    } else if domain.contains("://") {
        Some(format!(
            "«{domain}» lleva el protocolo; escribe solo el dominio, por ejemplo repositorio.ejemplo.edu"
        ))
    } else if domain.contains('/') || domain.chars().any(char::is_whitespace) {
        Some(format!(
            "«{domain}» no puede tener barras ni espacios; escribe solo el dominio"
        ))
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

fn validate_database_url(url: String) -> Result<String, ConfigError> {
    let url = url.trim().to_string();
    if url.starts_with("postgres://") || url.starts_with("postgresql://") {
        Ok(url)
    } else {
        Err(ConfigError::Invalid {
            key: "database.url",
            reason: "debe empezar con postgres:// o postgresql://".into(),
        })
    }
}
