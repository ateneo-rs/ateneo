// SPDX-FileCopyrightText: 2026 James Paul Vizuete Davila
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::io;
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};

use ateneo_core::config::{ConfigError, InvalidReason};
use ateneo_core::db::{ConnectProblem, DbError};

use crate::i18n::{t, tr};

#[derive(Debug)]
pub enum Exit {
    Cancelled,
    Failed(String),
    Reported,
}

impl Exit {
    pub fn report(self) -> ExitCode {
        match self {
            Self::Cancelled => ExitCode::from(130),
            Self::Reported => ExitCode::FAILURE,
            Self::Failed(message) => {
                eprintln!("{}", tr("error-line", &[("message", message)]));
                ExitCode::FAILURE
            }
        }
    }
}

impl From<ConfigError> for Exit {
    fn from(err: ConfigError) -> Self {
        Self::Failed(config_error(&err))
    }
}

impl From<DbError> for Exit {
    fn from(err: DbError) -> Self {
        Self::Failed(db_error(&err))
    }
}

impl From<sqlx::Error> for Exit {
    fn from(err: sqlx::Error) -> Self {
        Self::Failed(query_error(&err))
    }
}

pub trait Prompt<T> {
    fn or_cancel(self) -> Result<T, Exit>;
}

impl<T> Prompt<T> for io::Result<T> {
    fn or_cancel(self) -> Result<T, Exit> {
        self.map_err(|err| {
            if err.kind() == io::ErrorKind::Interrupted {
                let _ = cliclack::outro_cancel(t("cancelled"));
                Exit::Cancelled
            } else {
                Exit::Failed(terminal_error(&err))
            }
        })
    }
}

static VERBOSE: AtomicBool = AtomicBool::new(false);

pub fn set_verbose(verbose: bool) {
    VERBOSE.store(verbose, Ordering::Relaxed);
}

fn with_detail(message: String, detail: impl ToString) -> String {
    if VERBOSE.load(Ordering::Relaxed) {
        format!(
            "{message}\n{}",
            tr("error-detail", &[("detail", detail.to_string())])
        )
    } else {
        message
    }
}

pub fn config_error(err: &ConfigError) -> String {
    match err {
        ConfigError::NotFound { path } => tr(
            "config-error-not-found",
            &[("path", path.display().to_string())],
        ),
        ConfigError::Read { path, source } => tr(
            "config-error-read",
            &[
                ("path", path.display().to_string()),
                ("detail", source.to_string()),
            ],
        ),
        ConfigError::Parse { path, source } => tr(
            "config-error-parse",
            &[
                ("path", path.display().to_string()),
                ("detail", source.to_string()),
            ],
        ),
        ConfigError::Syntax { path, source } => tr(
            "config-error-parse",
            &[
                ("path", path.display().to_string()),
                ("detail", source.to_string()),
            ],
        ),
        ConfigError::Missing { key, var } => tr(
            "config-error-missing",
            &[("key", key.to_string()), ("var", var.to_string())],
        ),
        ConfigError::Invalid { key, reason } => tr(
            "config-error-invalid",
            &[("key", key.to_string()), ("reason", invalid_reason(reason))],
        ),
        ConfigError::Conflict { first, second } => tr(
            "config-error-conflict",
            &[("first", first.to_string()), ("second", second.to_string())],
        ),
        ConfigError::AlreadyExists { path } => tr(
            "config-error-exists",
            &[("path", path.display().to_string())],
        ),
        ConfigError::Write { path, source } => tr(
            "config-error-write",
            &[
                ("path", path.display().to_string()),
                ("detail", source.to_string()),
            ],
        ),
        ConfigError::UnknownKey { key } => {
            tr("config-error-unknown-key", &[("key", key.to_string())])
        }
    }
}

pub fn invalid_reason(reason: &InvalidReason) -> String {
    match reason {
        InvalidReason::Empty => t("reason-empty"),
        InvalidReason::HasProtocol(value) => tr("reason-protocol", &[("value", value.to_string())]),
        InvalidReason::HasSlashOrSpace(value) => {
            tr("reason-slash-space", &[("value", value.to_string())])
        }
        InvalidReason::NotANumber(value) => {
            tr("reason-not-a-number", &[("value", value.to_string())])
        }
        InvalidReason::PortOutOfRange => t("reason-port-range"),
        InvalidReason::NoConnections => t("reason-no-connections"),
        InvalidReason::NotPostgres => t("reason-not-postgres"),
    }
}

pub fn db_error(err: &DbError) -> String {
    let message = match err.problem() {
        Some(problem) => connect_problem(problem),
        None => t("db-error-migrate"),
    };
    let detail = match err {
        DbError::Connect(source) => source.to_string(),
        DbError::ConnectTimeout => err.to_string(),
        DbError::Migrate(source) => source.to_string(),
    };
    with_detail(message, detail)
}

pub fn connect_problem(problem: ConnectProblem) -> String {
    match problem {
        ConnectProblem::BadCredentials => t("db-error-credentials"),
        ConnectProblem::NoServer => t("db-error-no-server"),
        ConnectProblem::DatabaseMissing => t("db-error-database-missing"),
        ConnectProblem::Timeout => t("db-error-timeout"),
        ConnectProblem::Other => t("db-error-connect"),
    }
}

pub fn query_error(err: &sqlx::Error) -> String {
    with_detail(t("db-error-query"), err)
}

pub fn terminal_error(err: &io::Error) -> String {
    with_detail(t("error-terminal"), err)
}
