// SPDX-FileCopyrightText: 2026 James Paul Vizuete Davila
// SPDX-License-Identifier: AGPL-3.0-or-later

pub mod detect;
mod inspect;
mod status;

use std::io;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::Duration;

use sqlx::migrate::{MigrateError, Migrator};
use sqlx::postgres::{PgConnectOptions, PgConnection, PgPool, PgPoolOptions};
use sqlx::{ConnectOptions, Connection};

use crate::config::DatabaseConfig;

pub use inspect::{Incompatible, Inspection, MIN_SERVER_MAJOR, MIN_SERVER_VERSION, inspect};
pub use status::{AppliedMigration, PendingMigration, Status, status};

pub static MIGRATOR: Migrator = sqlx::migrate!();

const ACQUIRE_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("no se pudo conectar a PostgreSQL: {0}")]
    Connect(#[source] sqlx::Error),

    #[error("PostgreSQL no respondió a tiempo")]
    ConnectTimeout,

    #[error("no se pudieron aplicar las migraciones: {0}")]
    Migrate(#[from] MigrateError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectProblem {
    BadCredentials,
    NoServer,
    DatabaseMissing,
    Timeout,
    Other,
}

impl DbError {
    pub fn problem(&self) -> Option<ConnectProblem> {
        match self {
            Self::Connect(err) => Some(connect_problem(err)),
            Self::ConnectTimeout => Some(ConnectProblem::Timeout),
            Self::Migrate(_) => None,
        }
    }
}

pub fn connect_problem(err: &sqlx::Error) -> ConnectProblem {
    match err {
        sqlx::Error::Database(db) => match db.code().as_deref() {
            Some("28P01" | "28000") => ConnectProblem::BadCredentials,
            Some("3D000") => ConnectProblem::DatabaseMissing,
            _ => ConnectProblem::Other,
        },
        sqlx::Error::PoolTimedOut => ConnectProblem::Timeout,
        sqlx::Error::Io(io) if io.kind() == io::ErrorKind::TimedOut => ConnectProblem::Timeout,
        sqlx::Error::Io(_) => ConnectProblem::NoServer,
        _ => ConnectProblem::Other,
    }
}

pub async fn connect(config: &DatabaseConfig) -> Result<PgPool, DbError> {
    let options: PgConnectOptions = config.url.parse().map_err(DbError::Connect)?;
    let options = options.disable_statement_logging();

    match tokio::time::timeout(ACQUIRE_TIMEOUT, PgConnection::connect_with(&options)).await {
        Err(_) => return Err(DbError::ConnectTimeout),
        Ok(Err(err)) => return Err(DbError::Connect(err)),
        Ok(Ok(connection)) => {
            let _ = connection.close().await;
        }
    }

    PgPoolOptions::new()
        .max_connections(config.max_connections)
        .acquire_timeout(ACQUIRE_TIMEOUT)
        .connect_with(options)
        .await
        .map_err(DbError::Connect)
}

pub async fn migrate(pool: &PgPool) -> Result<(), DbError> {
    MIGRATOR.run(pool).await?;
    Ok(())
}

pub fn is_listening(host: &str, port: u16, timeout: Duration) -> bool {
    let Ok(addresses) = (host, port).to_socket_addrs() else {
        return false;
    };
    addresses
        .collect::<Vec<SocketAddr>>()
        .iter()
        .any(|address| TcpStream::connect_timeout(address, timeout).is_ok())
}
