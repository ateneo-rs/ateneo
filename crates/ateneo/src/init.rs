// SPDX-FileCopyrightText: 2026 James Paul Vizuete Davila
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::Path;
use std::time::Duration;

use ateneo_core::config::{self, DatabaseConfig, NewConfig};
use ateneo_core::db;
use ateneo_core::install::{self, DatabaseSource, Install};
use ateneo_core::settings;
use sqlx::PgPool;

use crate::exit::{Exit, Prompt, config_error, db_error, is_interactive, query_error};
use crate::i18n::{t, tr};

const INSTALLER_LANGUAGE: &str = "es";
const CONNECT_ATTEMPTS: u32 = 10;
const RETRY_DELAY: Duration = Duration::from_secs(3);
const INIT_CONNECTIONS: u32 = 2;
const DOCKER_MARKER: &str = "/.dockerenv";

enum Database {
    FromEnvironment { docker: bool },
    Typed(String),
}

struct Answers {
    domain: String,
    database: Database,
    site_language: String,
    timezone: String,
}

pub async fn run() -> Result<(), Exit> {
    let path = config::path();
    if path.exists() {
        return Err(Exit::Failed(tr(
            "init-exists",
            &[("path", path.display().to_string())],
        )));
    }
    if !is_interactive() {
        return Err(Exit::Failed(t("init-no-terminal")));
    }

    cliclack::intro(t("init-intro")).or_cancel()?;

    let _installer_language: &str = cliclack::select(t("init-installer-language"))
        .item(INSTALLER_LANGUAGE, t("language-es"), "")
        .interact()
        .or_cancel()?;

    let domain = ask_domain()?;
    let (pool, database) = connect().await?;
    let site_language = ask_site_language()?;
    let timezone = ask_timezone(&pool).await?;

    let answers = Answers {
        domain,
        database,
        site_language,
        timezone,
    };

    cliclack::note(t("init-summary-title"), summary(&answers, &path)).or_cancel()?;
    let confirmed = cliclack::confirm(t("init-confirm"))
        .initial_value(true)
        .interact()
        .or_cancel()?;
    if !confirmed {
        cliclack::outro_cancel(t("init-not-confirmed")).or_cancel()?;
        return Err(Exit::Cancelled);
    }

    finish(&pool, &answers, &path).await
}

fn ask_domain() -> Result<String, Exit> {
    let domain: String = cliclack::input(t("init-domain"))
        .placeholder(&t("init-domain-placeholder"))
        .validate(|value: &String| {
            config::validate_domain(value)
                .map(|_| ())
                .map_err(|err| config_error(&err))
        })
        .interact()
        .or_cancel()?;
    Ok(config::validate_domain(&domain)?)
}

async fn connect() -> Result<(PgPool, Database), Exit> {
    if let Some(url) = config::database_url_from_env(|name| std::env::var(name).ok())? {
        let docker = Path::new(DOCKER_MARKER).exists();
        let pool = connect_with_retries(&url, docker).await?;
        return Ok((pool, Database::FromEnvironment { docker }));
    }

    loop {
        let url: String = cliclack::input(t("init-db-url"))
            .placeholder(&t("init-db-url-placeholder"))
            .validate(|value: &String| {
                config::validate_database_url(value)
                    .map(|_| ())
                    .map_err(|err| config_error(&err))
            })
            .interact()
            .or_cancel()?;
        let url = config::validate_database_url(&url)?;

        let spinner = cliclack::spinner();
        spinner.start(t("init-db-testing"));
        match db::connect(&database(&url)).await {
            Ok(pool) => {
                spinner.stop(t("init-db-ok"));
                return Ok((pool, Database::Typed(url)));
            }
            Err(err) => {
                spinner.error(db_error(&err));
                cliclack::log::warning(t("init-db-try-again")).or_cancel()?;
            }
        }
    }
}

async fn connect_with_retries(url: &str, docker: bool) -> Result<PgPool, Exit> {
    let config = database(&config::validate_database_url(url)?);
    let spinner = cliclack::spinner();
    spinner.start(if docker {
        t("init-db-docker")
    } else {
        t("init-db-environment")
    });

    let mut attempt = 1;
    loop {
        match db::connect(&config).await {
            Ok(pool) => {
                spinner.stop(if docker {
                    t("init-db-docker-ok")
                } else {
                    t("init-db-environment-ok")
                });
                return Ok(pool);
            }
            Err(err) if attempt >= CONNECT_ATTEMPTS => {
                spinner.error(db_error(&err));
                cliclack::outro_cancel(t("init-db-gave-up")).or_cancel()?;
                return Err(Exit::Reported);
            }
            Err(_) => {
                spinner.set_message(tr(
                    "init-db-retrying",
                    &[
                        ("attempt", (attempt + 1).to_string()),
                        ("total", CONNECT_ATTEMPTS.to_string()),
                    ],
                ));
                tokio::time::sleep(RETRY_DELAY).await;
                attempt += 1;
            }
        }
    }
}

fn database(url: &str) -> DatabaseConfig {
    DatabaseConfig {
        url: url.to_string(),
        max_connections: INIT_CONNECTIONS,
    }
}

fn ask_site_language() -> Result<String, Exit> {
    let mut select = cliclack::select(t("init-site-language"));
    for code in settings::SITE_LANGUAGES {
        select = select.item(*code, language_name(code), "");
    }
    let code: &str = select
        .initial_value(settings::DEFAULT_SITE_LANGUAGE)
        .interact()
        .or_cancel()?;
    Ok(code.to_string())
}

pub fn language_name(code: &str) -> String {
    match code {
        "es" => t("language-es"),
        other => other.to_string(),
    }
}

async fn ask_timezone(pool: &PgPool) -> Result<String, Exit> {
    let detected =
        iana_time_zone::get_timezone().unwrap_or_else(|_| settings::DEFAULT_TIMEZONE.to_string());
    loop {
        let timezone: String = cliclack::input(t("init-timezone"))
            .placeholder(&tr("init-timezone-detected", &[("zone", detected.clone())]))
            .default_input(&detected)
            .interact()
            .or_cancel()?;
        let timezone = timezone.trim().to_string();
        if settings::is_timezone(pool, &timezone).await? {
            return Ok(timezone);
        }
        cliclack::log::error(tr("init-timezone-invalid", &[("zone", timezone)])).or_cancel()?;
    }
}

fn summary(answers: &Answers, path: &Path) -> String {
    let database = match &answers.database {
        Database::FromEnvironment { docker: true } => t("init-summary-db-docker"),
        Database::FromEnvironment { docker: false } => t("init-summary-db-environment"),
        Database::Typed(url) => config::redact_url(url),
    };
    [
        tr("init-summary-domain", &[("value", answers.domain.clone())]),
        tr("init-summary-database", &[("value", database)]),
        tr(
            "init-summary-site-language",
            &[("value", language_name(&answers.site_language))],
        ),
        tr(
            "init-summary-timezone",
            &[("value", answers.timezone.clone())],
        ),
        tr(
            "init-summary-file",
            &[("value", path.display().to_string())],
        ),
    ]
    .join("\n")
}

async fn finish(pool: &PgPool, answers: &Answers, path: &Path) -> Result<(), Exit> {
    let spinner = cliclack::spinner();
    spinner.start(t("init-migrating"));
    if let Err(err) = db::migrate(pool).await {
        spinner.error(db_error(&err));
        return failed();
    }
    spinner.stop(t("init-migrated"));

    let spinner = cliclack::spinner();
    spinner.start(t("init-saving"));
    if let Err(err) = save(pool, answers).await {
        spinner.error(query_error(&err));
        return failed();
    }

    let new_config = NewConfig {
        domain: answers.domain.clone(),
        database_url: match &answers.database {
            Database::Typed(url) => Some(url.clone()),
            Database::FromEnvironment { .. } => None,
        },
    };
    if let Err(err) = config::create_new(path, &new_config.to_toml()) {
        spinner.error(config_error(&err));
        return failed();
    }
    spinner.stop(tr("init-saved", &[("path", path.display().to_string())]));

    cliclack::outro(t("init-next-step")).or_cancel()?;
    Ok(())
}

async fn save(pool: &PgPool, answers: &Answers) -> sqlx::Result<()> {
    let database = match answers.database {
        Database::FromEnvironment { docker: true } => DatabaseSource::Docker,
        Database::FromEnvironment { docker: false } => DatabaseSource::Environment,
        Database::Typed(_) => DatabaseSource::File,
    };
    install::save(
        pool,
        &Install {
            domain: answers.domain.clone(),
            database,
            site_language: answers.site_language.clone(),
            timezone: answers.timezone.clone(),
        },
    )
    .await
}

fn failed() -> Result<(), Exit> {
    cliclack::outro_cancel(t("init-failed")).or_cancel()?;
    Err(Exit::Reported)
}
