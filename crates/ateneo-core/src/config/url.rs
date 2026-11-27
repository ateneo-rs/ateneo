// SPDX-FileCopyrightText: 2026 James Paul Vizuete Davila
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::{ConfigError, InvalidReason};

pub const DEFAULT_DATABASE_PORT: u16 = 5432;

#[derive(Clone, PartialEq, Eq)]
pub struct DatabaseParts {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
}

impl std::fmt::Debug for DatabaseParts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DatabaseParts")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("user", &self.user)
            .field("password", &"***")
            .field("database", &self.database)
            .finish()
    }
}

impl DatabaseParts {
    pub fn to_url(&self) -> String {
        let host = if self.host.contains(':') && !self.host.starts_with('[') {
            format!("[{}]", self.host)
        } else {
            self.host.clone()
        };
        let credentials = if self.password.is_empty() {
            encode(&self.user)
        } else {
            format!("{}:{}", encode(&self.user), encode(&self.password))
        };
        format!(
            "postgres://{credentials}@{host}:{}/{}",
            self.port,
            encode(&self.database)
        )
    }
}

pub fn validate_host(host: &str) -> Result<String, ConfigError> {
    let host = host.trim();
    let reason = if host.is_empty() {
        Some(InvalidReason::Empty)
    } else if host.contains(['/', '@', '?', '#']) || host.chars().any(char::is_whitespace) {
        Some(InvalidReason::HasSlashOrSpace(host.to_string()))
    } else {
        None
    };
    match reason {
        Some(reason) => Err(ConfigError::Invalid {
            key: "database.host",
            reason,
        }),
        None => Ok(host.to_string()),
    }
}

pub fn validate_name(key: &'static str, value: &str) -> Result<String, ConfigError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(ConfigError::Invalid {
            key,
            reason: InvalidReason::Empty,
        });
    }
    Ok(value.to_string())
}

fn encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            out.push(char::from(byte));
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

pub fn redact_url(url: &str) -> String {
    let Some(scheme_end) = url.find("://") else {
        return url.to_string();
    };
    let rest_start = scheme_end + 3;
    let rest = &url[rest_start..];
    let authority_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let authority = &rest[..authority_end];
    let Some(at) = authority.rfind('@') else {
        return url.to_string();
    };
    let userinfo = &authority[..at];
    let Some(colon) = userinfo.find(':') else {
        return url.to_string();
    };
    format!(
        "{}{}:***{}",
        &url[..rest_start],
        &userinfo[..colon],
        &url[rest_start + at..]
    )
}
