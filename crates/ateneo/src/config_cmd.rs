// SPDX-FileCopyrightText: 2026 James Paul Vizuete Davila
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::io;
use std::path::PathBuf;

use ateneo_core::audit::{self, Entry};
use ateneo_core::config::{self, Config, ConfigError, DatabaseConfig, Edit, FileKey};
use ateneo_core::db;
use ateneo_core::settings::{self, SettingKey};
use serde_json::json;
use sqlx::PgPool;

use crate::exit::{Exit, Prompt, config_error};
use crate::i18n::{t, tr};
use crate::init::language_list;
use crate::prompt;
use crate::terminal::is_interactive;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Target {
    File(FileKey),
    Setting(SettingKey),
}

impl Target {
    fn all() -> Vec<Target> {
        FileKey::ALL
            .into_iter()
            .map(Target::File)
            .chain(SettingKey::ALL.into_iter().map(Target::Setting))
            .collect()
    }

    fn from_name(name: &str) -> Option<Target> {
        FileKey::from_name(name)
            .map(Target::File)
            .or_else(|| SettingKey::from_name(name).map(Target::Setting))
    }

    fn name(self) -> &'static str {
        match self {
            Self::File(key) => key.name(),
            Self::Setting(key) => key.name(),
        }
    }

    fn label(self) -> String {
        match self {
            Self::File(FileKey::Domain) => t("option-domain"),
            Self::File(FileKey::Port) => t("option-port"),
            Self::File(FileKey::DataDir) => t("option-data-dir"),
            Self::File(FileKey::DatabaseUrl) => t("option-database-url"),
            Self::File(FileKey::DatabaseMaxConnections) => t("option-max-connections"),
            Self::Setting(SettingKey::SiteLanguage) => t("option-site-language"),
            Self::Setting(SettingKey::SiteTimezone) => t("option-site-timezone"),
        }
    }

    fn example(self) -> String {
        match self {
            Self::File(FileKey::Domain) => t("init-domain-example"),
            Self::File(FileKey::Port) => t("example-port"),
            Self::File(FileKey::DataDir) => t("example-data-dir"),
            Self::File(FileKey::DatabaseUrl) => t("example-database-url"),
            Self::File(FileKey::DatabaseMaxConnections) => t("example-max-connections"),
            Self::Setting(SettingKey::SiteLanguage) => t("language-example"),
            Self::Setting(SettingKey::SiteTimezone) => t("init-timezone-example"),
        }
    }

    fn show(self, value: Option<&str>) -> String {
        match value {
            None => t("value-none"),
            Some(value) if self == Self::File(FileKey::DatabaseUrl) => config::redact_url(value),
            Some(value) => value.to_string(),
        }
    }
}

enum Apply {
    File(Edit),
    Setting(String),
}

struct Change {
    target: Target,
    old: Option<String>,
    new: String,
    apply: Apply,
}

impl Change {
    fn changes(&self) -> bool {
        self.old.as_deref() != Some(self.new.as_str())
    }

    fn lines(&self) -> String {
        [
            tr(
                "config-change-option",
                &[("value", self.target.name().to_string())],
            ),
            tr(
                "config-change-before",
                &[("value", self.target.show(self.old.as_deref()))],
            ),
            tr(
                "config-change-after",
                &[("value", self.target.show(Some(&self.new)))],
            ),
        ]
        .join("\n")
    }
}

struct Context {
    path: PathBuf,
    text: String,
    pool: PgPool,
}

pub async fn menu() -> Result<(), Exit> {
    if !is_interactive() {
        return Err(Exit::Failed(t("config-no-terminal")));
    }
    cliclack::intro(t("config-intro")).or_cancel()?;
    let context = match open().await {
        Ok(context) => context,
        Err(Exit::Failed(message)) => {
            cliclack::outro_cancel(message).or_cancel()?;
            return Err(Exit::Reported);
        }
        Err(other) => return Err(other),
    };

    let mut select = cliclack::select(t("config-choose"));
    for target in Target::all() {
        let current = current(&context, target).await?;
        select = select.item(target, target.label(), target.show(current.as_deref()));
    }
    let target = select.interact().or_cancel()?;
    let current = current(&context, target).await?;

    let keep = current
        .as_deref()
        .filter(|_| target != Target::File(FileKey::DatabaseUrl));
    let change = loop {
        let raw = prompt::text(
            &tr("config-new-value", &[("option", target.label())]),
            &target.example(),
            keep,
            |value| Ok(value.to_string()),
        )?;
        match prepare(&context, target, &raw).await {
            Ok(change) => break change,
            Err(Exit::Failed(message)) => cliclack::log::error(message).or_cancel()?,
            Err(other) => return Err(other),
        }
    };

    if !change.changes() {
        cliclack::outro(t("config-unchanged")).or_cancel()?;
        return Ok(());
    }
    warn_overrides(target, |message| {
        cliclack::log::warning(message).or_cancel()
    })?;
    cliclack::note(t("config-review"), change.lines()).or_cancel()?;
    if !prompt::yes_no(&t("config-confirm"), false)? {
        cliclack::outro_cancel(t("config-not-confirmed")).or_cancel()?;
        return Err(Exit::Cancelled);
    }

    match apply(&context, change).await {
        Ok(()) => {
            reminders(target, |message| cliclack::log::info(message).or_cancel())?;
            cliclack::outro(t("config-saved")).or_cancel()?;
            Ok(())
        }
        Err(Exit::Failed(message)) => {
            cliclack::outro_cancel(message).or_cancel()?;
            Err(Exit::Reported)
        }
        Err(other) => Err(other),
    }
}

pub async fn set(name: &str, raw: &str, yes: bool) -> Result<(), Exit> {
    let target = Target::from_name(name).ok_or_else(|| {
        Exit::Failed(tr(
            "config-unknown-option",
            &[
                ("option", name.to_string()),
                (
                    "options",
                    Target::all()
                        .iter()
                        .map(|target| target.name())
                        .collect::<Vec<_>>()
                        .join(", "),
                ),
            ],
        ))
    })?;
    if !yes && !is_interactive() {
        return Err(Exit::Failed(t("config-set-needs-yes")));
    }

    let context = open().await?;
    let change = prepare(&context, target, raw).await?;
    if !change.changes() {
        println!("{}", t("config-unchanged"));
        return Ok(());
    }

    println!("{}", change.lines());
    warn_overrides(target, |message| {
        eprintln!("{}", tr("warning-line", &[("message", message)]));
        Ok(())
    })?;
    if !yes && !prompt::yes_no(&t("config-confirm"), false)? {
        eprintln!("{}", t("config-not-confirmed"));
        return Err(Exit::Cancelled);
    }

    apply(&context, change).await?;
    reminders(target, |message| {
        println!("{message}");
        Ok(())
    })?;
    println!("{}", t("config-saved"));
    Ok(())
}

async fn open() -> Result<Context, Exit> {
    let path = config::path();
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            return Err(Exit::Failed(tr(
                "config-missing-file",
                &[("path", path.display().to_string())],
            )));
        }
        Err(source) => return Err(ConfigError::Read { path, source }.into()),
    };
    let loaded = Config::load()?;
    let pool = db::connect(&loaded.database).await?;
    Ok(Context { path, text, pool })
}

async fn current(context: &Context, target: Target) -> Result<Option<String>, Exit> {
    Ok(match target {
        Target::File(key) => config::current_value(&context.path, &context.text, key)?,
        Target::Setting(key) => settings::get_text(&context.pool, key.name()).await?,
    })
}

async fn prepare(context: &Context, target: Target, raw: &str) -> Result<Change, Exit> {
    match target {
        Target::File(key) => {
            let edit = config::prepare_edit(&context.path, &context.text, key, raw)
                .map_err(|err| Exit::Failed(config_error(&err)))?;
            Ok(Change {
                target,
                old: edit.old.clone(),
                new: edit.new.clone(),
                apply: Apply::File(edit),
            })
        }
        Target::Setting(key) => {
            let value = raw.trim().to_string();
            let valid = match key {
                SettingKey::SiteLanguage => settings::is_site_language(&value),
                SettingKey::SiteTimezone => settings::is_timezone(&context.pool, &value).await?,
            };
            if !valid {
                return Err(Exit::Failed(invalid_setting(key, &value)));
            }
            Ok(Change {
                target,
                old: settings::get_text(&context.pool, key.name()).await?,
                new: value.clone(),
                apply: Apply::Setting(value),
            })
        }
    }
}

fn invalid_setting(key: SettingKey, value: &str) -> String {
    match key {
        SettingKey::SiteLanguage => tr(
            "config-language-invalid",
            &[
                ("value", value.to_string()),
                ("options", language_list(settings::SITE_LANGUAGES)),
            ],
        ),
        SettingKey::SiteTimezone => tr("init-timezone-invalid", &[("zone", value.to_string())]),
    }
}

async fn apply(context: &Context, change: Change) -> Result<(), Exit> {
    let entry = |kind| {
        Entry::console(audit::CONFIG_SET)
            .target(kind, change.target.name())
            .details(json!({
                "key": change.target.name(),
                "old": change.old.as_deref().map(|old| change.target.show(Some(old))),
                "new": change.target.show(Some(&change.new)),
            }))
    };

    match &change.apply {
        Apply::Setting(value) => {
            let mut tx = context.pool.begin().await?;
            settings::set(&mut *tx, change.target.name(), &json!(value)).await?;
            audit::record(&mut *tx, &entry("setting")).await?;
            tx.commit().await?;
            Ok(())
        }
        Apply::File(edit) => {
            let new_pool;
            let pool = if edit.key == FileKey::DatabaseUrl {
                new_pool = db::connect(&DatabaseConfig {
                    url: edit.new.clone(),
                    max_connections: 1,
                })
                .await?;
                db::migrate(&new_pool).await?;
                &new_pool
            } else {
                &context.pool
            };

            let mut tx = pool.begin().await?;
            audit::record(&mut *tx, &entry("config_file")).await?;
            config::replace(&context.path, edit.contents())?;
            if let Err(err) = tx.commit().await {
                let restored = config::replace(&context.path, &context.text);
                return Err(Exit::Failed(match restored {
                    Ok(()) => tr("config-audit-failed", &[("detail", err.to_string())]),
                    Err(restore) => tr(
                        "config-restore-failed",
                        &[
                            ("detail", err.to_string()),
                            ("restore", config_error(&restore)),
                        ],
                    ),
                }));
            }
            Ok(())
        }
    }
}

fn warn_overrides(
    target: Target,
    mut warn: impl FnMut(String) -> Result<(), Exit>,
) -> Result<(), Exit> {
    let Target::File(key) = target else {
        return Ok(());
    };
    let mut vars = vec![key.env_var()];
    if key == FileKey::DatabaseUrl {
        vars.push(config::ENV_DATABASE_URL_FILE);
    }
    for var in vars {
        if std::env::var_os(var).is_some() {
            warn(tr("config-env-override", &[("var", var.to_string())]))?;
        }
    }
    Ok(())
}

fn reminders(target: Target, mut say: impl FnMut(String) -> Result<(), Exit>) -> Result<(), Exit> {
    match target {
        Target::File(FileKey::Domain) => say(t("config-domain-reminder")),
        Target::File(
            FileKey::Port
            | FileKey::DataDir
            | FileKey::DatabaseUrl
            | FileKey::DatabaseMaxConnections,
        ) => say(t("config-restart-reminder")),
        Target::Setting(_) => Ok(()),
    }
}
