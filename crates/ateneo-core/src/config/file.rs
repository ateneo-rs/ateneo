// SPDX-FileCopyrightText: 2026 James Paul Vizuete Davila
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use toml_edit::{DocumentMut, Item, Table, Value, value};

use super::{
    ConfigError, DEFAULT_DATA_DIR, DEFAULT_MAX_CONNECTIONS, DEFAULT_PORT, ENV_DATA_DIR,
    ENV_DATABASE_MAX_CONNECTIONS, ENV_DATABASE_URL, ENV_DOMAIN, ENV_PORT, parse_file,
    validate_data_dir, validate_database_url, validate_domain, validate_max_connections,
    validate_port,
};

const DATABASE: &str = "database";
const URL_FILE: &str = "url_file";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileKey {
    Domain,
    Port,
    DataDir,
    DatabaseUrl,
    DatabaseMaxConnections,
}

impl FileKey {
    pub const ALL: [FileKey; 5] = [
        FileKey::Domain,
        FileKey::Port,
        FileKey::DataDir,
        FileKey::DatabaseUrl,
        FileKey::DatabaseMaxConnections,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Domain => "domain",
            Self::Port => "port",
            Self::DataDir => "data_dir",
            Self::DatabaseUrl => "database.url",
            Self::DatabaseMaxConnections => "database.max_connections",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|key| key.name() == name)
    }

    pub fn env_var(self) -> &'static str {
        match self {
            Self::Domain => ENV_DOMAIN,
            Self::Port => ENV_PORT,
            Self::DataDir => ENV_DATA_DIR,
            Self::DatabaseUrl => ENV_DATABASE_URL,
            Self::DatabaseMaxConnections => ENV_DATABASE_MAX_CONNECTIONS,
        }
    }

    pub fn is_secret(self) -> bool {
        matches!(self, Self::DatabaseUrl)
    }

    fn table(self) -> Option<&'static str> {
        match self {
            Self::DatabaseUrl | Self::DatabaseMaxConnections => Some(DATABASE),
            _ => None,
        }
    }

    fn field(self) -> &'static str {
        match self {
            Self::Domain => "domain",
            Self::Port => "port",
            Self::DataDir => "data_dir",
            Self::DatabaseUrl => "url",
            Self::DatabaseMaxConnections => "max_connections",
        }
    }

    fn default_value(self) -> Option<String> {
        match self {
            Self::Port => Some(DEFAULT_PORT.to_string()),
            Self::DataDir => Some(DEFAULT_DATA_DIR.to_string()),
            Self::DatabaseMaxConnections => Some(DEFAULT_MAX_CONNECTIONS.to_string()),
            Self::Domain | Self::DatabaseUrl => None,
        }
    }

    fn parse(self, raw: &str) -> Result<(Value, String), ConfigError> {
        Ok(match self {
            Self::Domain => {
                let domain = validate_domain(raw)?;
                (Value::from(domain.as_str()), domain)
            }
            Self::Port => {
                let port = validate_port(raw)?;
                (Value::from(i64::from(port)), port.to_string())
            }
            Self::DataDir => {
                let dir = validate_data_dir(raw)?.to_string_lossy().into_owned();
                (Value::from(dir.as_str()), dir)
            }
            Self::DatabaseUrl => {
                let url = validate_database_url(raw)?;
                (Value::from(url.as_str()), url)
            }
            Self::DatabaseMaxConnections => {
                let max = validate_max_connections(raw)?;
                (Value::from(i64::from(max)), max.to_string())
            }
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewConfig {
    pub domain: String,
    pub database_url: Option<String>,
}

impl NewConfig {
    pub fn to_toml(&self) -> String {
        let mut doc = DocumentMut::new();
        doc["domain"] = value(self.domain.as_str());
        doc["port"] = value(i64::from(DEFAULT_PORT));
        doc["data_dir"] = value(DEFAULT_DATA_DIR);
        let mut database = Table::new();
        if let Some(url) = &self.database_url {
            database["url"] = value(url.as_str());
        }
        database["max_connections"] = value(i64::from(DEFAULT_MAX_CONNECTIONS));
        doc[DATABASE] = Item::Table(database);
        doc.to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edit {
    pub key: FileKey,
    pub old: Option<String>,
    pub new: String,
    contents: String,
}

impl Edit {
    pub fn contents(&self) -> &str {
        &self.contents
    }

    pub fn changes(&self) -> bool {
        self.old.as_deref() != Some(self.new.as_str())
    }
}

pub fn current_value(path: &Path, text: &str, key: FileKey) -> Result<Option<String>, ConfigError> {
    let doc = parse_document(path, text)?;
    Ok(current(&doc, key).or_else(|| key.default_value()))
}

pub fn prepare_edit(path: &Path, text: &str, key: FileKey, raw: &str) -> Result<Edit, ConfigError> {
    let mut doc = parse_document(path, text)?;

    let old = current(&doc, key).or_else(|| key.default_value());
    let (item, new) = key.parse(raw)?;

    match key.table() {
        Some(table) => {
            let entry = doc
                .entry(table)
                .or_insert_with(|| Item::Table(Table::new()));
            entry[key.field()] = Item::Value(item);
            if key == FileKey::DatabaseUrl
                && let Some(table) = entry.as_table_like_mut()
            {
                table.remove(URL_FILE);
            }
        }
        None => doc[key.field()] = Item::Value(item),
    }

    let contents = doc.to_string();
    parse_file(path, &contents)?;
    Ok(Edit {
        key,
        old,
        new,
        contents,
    })
}

fn parse_document(path: &Path, text: &str) -> Result<DocumentMut, ConfigError> {
    parse_file(path, text)?;
    text.parse().map_err(|source| ConfigError::Syntax {
        path: path.to_path_buf(),
        source: Box::new(source),
    })
}

fn current(doc: &DocumentMut, key: FileKey) -> Option<String> {
    let item = match key.table() {
        Some(table) => doc.get(table)?.get(key.field())?,
        None => doc.get(key.field())?,
    };
    let value = item.as_value()?;
    value
        .as_str()
        .map(str::to_string)
        .or_else(|| value.as_integer().map(|n| n.to_string()))
}

pub fn create_new(path: &Path, contents: &str) -> Result<(), ConfigError> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    private(&mut options);
    let file = match options.open(path) {
        Ok(file) => file,
        Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {
            return Err(ConfigError::AlreadyExists {
                path: path.to_path_buf(),
            });
        }
        Err(source) => return Err(write_error(path, source)),
    };
    if let Err(source) = fill(file, contents) {
        let _ = fs::remove_file(path);
        return Err(write_error(path, source));
    }
    Ok(())
}

pub fn replace(path: &Path, contents: &str) -> Result<(), ConfigError> {
    let temp = temp_path(path);
    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(true);
    private(&mut options);
    let result = options
        .open(&temp)
        .and_then(|file| fill(file, contents))
        .and_then(|()| fs::rename(&temp, path));
    if let Err(source) = result {
        let _ = fs::remove_file(&temp);
        return Err(write_error(path, source));
    }
    Ok(())
}

fn fill(mut file: File, contents: &str) -> io::Result<()> {
    file.write_all(contents.as_bytes())?;
    file.sync_all()
}

fn temp_path(path: &Path) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".tmp");
    path.with_file_name(name)
}

fn write_error(path: &Path, source: io::Error) -> ConfigError {
    ConfigError::Write {
        path: path.to_path_buf(),
        source,
    }
}

#[cfg(unix)]
fn private(options: &mut OpenOptions) {
    use std::os::unix::fs::OpenOptionsExt;
    options.mode(0o600);
}

#[cfg(not(unix))]
fn private(_options: &mut OpenOptions) {}
