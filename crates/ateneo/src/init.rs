// SPDX-FileCopyrightText: 2026 James Paul Vizuete Davila
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::Path;
use std::time::Duration;

use ateneo_core::config::{self, DEFAULT_DATABASE_PORT, DatabaseConfig, DatabaseParts, NewConfig};
use ateneo_core::db;
use ateneo_core::install::{self, DatabaseSource, Install};
use ateneo_core::settings;
use sqlx::PgPool;

use crate::exit::{Exit, Prompt, config_error, db_error, query_error};
use crate::i18n::{t, tr};
use crate::prompt::{self, Choice};
use crate::terminal::is_interactive;

pub const INSTALLER_LANGUAGES: &[&str] = &["es"];
const CONNECT_ATTEMPTS: u32 = 10;
const RETRY_DELAY: Duration = Duration::from_secs(3);
const INIT_CONNECTIONS: u32 = 2;
const DOCKER_MARKER: &str = "/.dockerenv";
const LOCAL_HOST: &str = "localhost";
const PROBE_TIMEOUT: Duration = Duration::from_millis(300);

enum Database {
    FromEnvironment { docker: bool },
    Typed(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DatabaseChoice {
    Environment,
    Local,
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

    let _installer_language = ask_installer_language()?;
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
    if !prompt::yes_no(&t("init-confirm"), true)? {
        cliclack::outro_cancel(t("init-not-confirmed")).or_cancel()?;
        return Err(Exit::Cancelled);
    }

    finish(&pool, &answers, &path).await
}

pub fn supported_language(locale: &str, available: &[&'static str]) -> Option<&'static str> {
    let code = locale
        .split(['-', '_', '.', '@'])
        .next()?
        .to_ascii_lowercase();
    available
        .iter()
        .copied()
        .find(|candidate| *candidate == code)
}

pub fn language_name(code: &str) -> String {
    match code {
        "es" => t("language-es"),
        other => other.to_string(),
    }
}

fn language_choices(available: &[&'static str], detected: Option<&str>) -> Vec<Choice<String>> {
    let mut codes: Vec<&str> = available.to_vec();
    if let Some(detected) = detected {
        codes.retain(|code| *code != detected);
        codes.insert(0, detected);
    }
    codes
        .into_iter()
        .map(|code| Choice {
            value: code.to_string(),
            label: language_name(code),
            hint: if Some(code) == detected {
                t("hint-detected")
            } else {
                String::new()
            },
        })
        .collect()
}

fn ask_language(
    prompt_id: &str,
    available: &'static [&'static str],
    detected: Option<&str>,
) -> Result<String, Exit> {
    if let Some(code) = prompt::choose(&t(prompt_id), language_choices(available, detected))? {
        return Ok(code);
    }
    prompt::text(&t(prompt_id), &t("language-example"), None, move |value| {
        supported_language(value.trim(), available)
            .map(str::to_string)
            .ok_or_else(|| {
                tr(
                    "language-unavailable",
                    &[
                        ("value", value.trim().to_string()),
                        ("options", language_list(available)),
                    ],
                )
            })
    })
}

pub fn language_list(available: &[&str]) -> String {
    available
        .iter()
        .map(|code| format!("{code} ({})", language_name(code)))
        .collect::<Vec<_>>()
        .join(", ")
}

fn ask_installer_language() -> Result<String, Exit> {
    let detected = sys_locale::get_locale()
        .and_then(|locale| supported_language(&locale, INSTALLER_LANGUAGES));
    ask_language("init-installer-language", INSTALLER_LANGUAGES, detected)
}

fn ask_site_language() -> Result<String, Exit> {
    ask_language(
        "init-site-language",
        settings::SITE_LANGUAGES,
        Some(settings::DEFAULT_SITE_LANGUAGE),
    )
}

fn ask_domain() -> Result<String, Exit> {
    prompt::text(
        &t("init-domain"),
        &t("init-domain-example"),
        None,
        |value| config::validate_domain(value).map_err(|err| config_error(&err)),
    )
}

async fn connect() -> Result<(PgPool, Database), Exit> {
    let environment = config::database_url_from_env(|name| std::env::var(name).ok())?;
    let docker = Path::new(DOCKER_MARKER).exists();
    let local = db::is_listening(LOCAL_HOST, DEFAULT_DATABASE_PORT, PROBE_TIMEOUT);
    let mut previous: Option<DatabaseParts> = None;

    loop {
        let mut choices = Vec::new();
        if environment.is_some() {
            choices.push(Choice {
                value: DatabaseChoice::Environment,
                label: if docker {
                    t("init-db-choice-docker")
                } else {
                    t("init-db-choice-environment")
                },
                hint: t("hint-detected"),
            });
        }
        if local {
            choices.push(Choice {
                value: DatabaseChoice::Local,
                label: tr(
                    "init-db-choice-local",
                    &[
                        ("host", LOCAL_HOST.to_string()),
                        ("port", DEFAULT_DATABASE_PORT.to_string()),
                    ],
                ),
                hint: t("hint-detected"),
            });
        }

        let picked = prompt::choose(&t("init-db"), choices)?;
        let parts = match (picked, &environment) {
            (Some(DatabaseChoice::Environment), Some(url)) => {
                if let Some(pool) = connect_with_retries(url, docker).await? {
                    return Ok((pool, Database::FromEnvironment { docker }));
                }
                continue;
            }
            (Some(DatabaseChoice::Local), _) => {
                ask_parts(Some((LOCAL_HOST, DEFAULT_DATABASE_PORT)), previous.as_ref())?
            }
            _ => ask_parts(None, previous.as_ref())?,
        };

        let url = parts.to_url();
        if let Some(pool) = try_connect(&url).await {
            return Ok((pool, Database::Typed(url)));
        }
        previous = Some(parts);
    }
}

fn ask_parts(
    fixed: Option<(&str, u16)>,
    previous: Option<&DatabaseParts>,
) -> Result<DatabaseParts, Exit> {
    let (host, port) = match fixed {
        Some((host, port)) => (host.to_string(), port),
        None => {
            let host = prompt::text(
                &t("init-db-host"),
                &t("init-db-host-example"),
                previous.map(|parts| parts.host.as_str()),
                |value| config::validate_host(value).map_err(|err| config_error(&err)),
            )?;
            let previous_port = previous.map(|parts| parts.port.to_string());
            let port = prompt::text(
                &t("init-db-port"),
                &t("init-db-port-example"),
                previous_port.as_deref(),
                |value| {
                    config::validate_port(value)
                        .map(|port| port.to_string())
                        .map_err(|err| config_error(&err))
                },
            )?;
            (host, config::validate_port(&port)?)
        }
    };
    let user = prompt::text(
        &t("init-db-user"),
        &t("init-db-user-example"),
        previous.map(|parts| parts.user.as_str()),
        |value| config::validate_name("database.user", value).map_err(|err| config_error(&err)),
    )?;
    let password = prompt::password(&t("init-db-password"))?;
    let database = prompt::text(
        &t("init-db-name"),
        &t("init-db-name-example"),
        previous.map(|parts| parts.database.as_str()),
        |value| config::validate_name("database.name", value).map_err(|err| config_error(&err)),
    )?;
    Ok(DatabaseParts {
        host,
        port,
        user,
        password,
        database,
    })
}

async fn try_connect(url: &str) -> Option<PgPool> {
    let spinner = cliclack::spinner();
    spinner.start(t("init-db-testing"));
    match db::connect(&database(url)).await {
        Ok(pool) => {
            spinner.stop(t("init-db-ok"));
            Some(pool)
        }
        Err(err) => {
            spinner.error(db_error(&err));
            None
        }
    }
}

async fn connect_with_retries(url: &str, docker: bool) -> Result<Option<PgPool>, Exit> {
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
                spinner.stop(t("init-db-ok"));
                return Ok(Some(pool));
            }
            Err(err) if attempt >= CONNECT_ATTEMPTS => {
                spinner.error(db_error(&err));
                return Ok(None);
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

async fn ask_timezone(pool: &PgPool) -> Result<String, Exit> {
    let mut choices = Vec::new();
    if let Ok(detected) = iana_time_zone::get_timezone()
        && settings::is_timezone(pool, &detected).await?
    {
        choices.push(Choice {
            value: detected.clone(),
            label: detected,
            hint: t("hint-detected"),
        });
    }
    if let Some(zone) = prompt::choose(&t("init-timezone"), choices)? {
        return Ok(zone);
    }

    loop {
        let zone = prompt::text(
            &t("init-timezone"),
            &t("init-timezone-example"),
            None,
            |value| Ok(value.trim().to_string()),
        )?;
        if settings::is_timezone(pool, &zone).await? {
            return Ok(zone);
        }
        cliclack::log::error(tr("init-timezone-invalid", &[("zone", zone)])).or_cancel()?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_supported_languages_from_system_locales() {
        assert_eq!(supported_language("es-EC", &["es"]), Some("es"));
        assert_eq!(supported_language("es_EC.UTF-8", &["es"]), Some("es"));
        assert_eq!(supported_language("ES", &["es"]), Some("es"));
        assert_eq!(supported_language("en-US", &["es"]), None);
        assert_eq!(supported_language("", &["es"]), None);
    }

    #[test]
    fn detected_language_comes_first_and_is_only_a_hint() {
        let choices = language_choices(&["en", "es"], Some("es"));

        assert_eq!(choices[0].value, "es");
        assert_eq!(choices[0].label, language_name("es"));
        assert!(!choices[0].hint.is_empty());
        assert!(!choices[0].value.contains(&choices[0].hint));
        assert_eq!(choices[1].value, "en");
        assert!(choices[1].hint.is_empty());
    }

    #[test]
    fn without_detection_lists_available_languages_without_hints() {
        let choices = language_choices(&["es"], None);

        assert_eq!(choices.len(), 1);
        assert!(choices[0].hint.is_empty());
    }
}
