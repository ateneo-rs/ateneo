// SPDX-FileCopyrightText: 2026 James Paul Vizuete Davila
// SPDX-License-Identifier: AGPL-3.0-or-later

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

    #[error("falta `{key}`: agrégalo en ateneo.toml o usa la variable {var}")]
    Missing {
        key: &'static str,
        var: &'static str,
    },

    #[error("`{key}` no es válido: {reason}")]
    Invalid { key: &'static str, reason: String },

    #[error("usa `{first}` o `{second}`, no los dos a la vez")]
    Conflict {
        first: &'static str,
        second: &'static str,
    },
}
