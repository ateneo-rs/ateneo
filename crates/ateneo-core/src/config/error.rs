// SPDX-FileCopyrightText: 2026 James Paul Vizuete Davila
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fmt;
use std::io;
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("no existe el archivo de configuración {}", path.display())]
    NotFound { path: PathBuf },

    #[error("no se pudo leer {}: {source}", path.display())]
    Read {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("{} tiene un error: {source}", path.display())]
    Parse {
        path: PathBuf,
        #[source]
        source: Box<toml::de::Error>,
    },

    #[error("{} tiene un error de sintaxis: {source}", path.display())]
    Syntax {
        path: PathBuf,
        #[source]
        source: Box<toml_edit::TomlError>,
    },

    #[error("falta `{key}`: agrégalo en ateneo.toml o usa la variable {var}")]
    Missing {
        key: &'static str,
        var: &'static str,
    },

    #[error("`{key}` no es válido: {reason}")]
    Invalid {
        key: &'static str,
        reason: InvalidReason,
    },

    #[error("usa `{first}` o `{second}`, no los dos a la vez")]
    Conflict {
        first: &'static str,
        second: &'static str,
    },

    #[error("ya existe {}", path.display())]
    AlreadyExists { path: PathBuf },

    #[error("no se pudo escribir {}: {source}", path.display())]
    Write {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("`{key}` no es una opción de ateneo.toml")]
    UnknownKey { key: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvalidReason {
    Empty,
    HasProtocol(String),
    HasSlashOrSpace(String),
    NotANumber(String),
    PortOutOfRange,
    NoConnections,
    NotPostgres,
}

impl fmt::Display for InvalidReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "está vacío"),
            Self::HasProtocol(value) => write!(
                f,
                "«{value}» lleva el protocolo; escribe solo el dominio, por ejemplo repositorio.ejemplo.edu"
            ),
            Self::HasSlashOrSpace(value) => write!(
                f,
                "«{value}» no puede tener barras ni espacios; escribe solo el dominio"
            ),
            Self::NotANumber(value) => write!(f, "«{value}» no es un número válido"),
            Self::PortOutOfRange => write!(f, "debe ser un número entre 1 y 65535"),
            Self::NoConnections => write!(f, "debe ser al menos 1"),
            Self::NotPostgres => write!(f, "debe empezar con postgres:// o postgresql://"),
        }
    }
}
