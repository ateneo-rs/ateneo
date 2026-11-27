// SPDX-FileCopyrightText: 2026 James Paul Vizuete Davila
// SPDX-License-Identifier: AGPL-3.0-or-later

use ateneo_core::config::{self, Config};
use ateneo_core::db;
use ateneo_core::settings;

use crate::exit::{Exit, db_error, query_error};
use crate::i18n::{t, tr};

pub async fn run() -> Result<(), Exit> {
    let config = Config::load()?;

    println!(
        "{}",
        tr(
            "status-title",
            &[("version", env!("CARGO_PKG_VERSION").to_string())]
        )
    );
    println!(
        "{}",
        tr(
            "status-config-file",
            &[("path", config::path().display().to_string())]
        )
    );
    println!(
        "{}",
        tr("status-domain", &[("value", config.domain.clone())])
    );
    println!(
        "{}",
        tr(
            "status-database",
            &[("value", config::redact_url(&config.database.url))]
        )
    );

    let pool = match db::connect(&config.database).await {
        Ok(pool) => pool,
        Err(err) => {
            println!(
                "{}",
                tr("status-connection-failed", &[("detail", db_error(&err))])
            );
            return Err(Exit::Reported);
        }
    };

    let status = match db::status(&pool).await {
        Ok(status) => status,
        Err(err) => {
            println!(
                "{}",
                tr("status-connection-failed", &[("detail", query_error(&err))])
            );
            return Err(Exit::Reported);
        }
    };
    println!(
        "{}",
        tr(
            "status-connection-ok",
            &[("version", status.server_version.clone())]
        )
    );

    println!();
    println!(
        "{}",
        tr(
            "status-migrations-applied",
            &[("count", status.applied.len().to_string())]
        )
    );
    for migration in &status.applied {
        let id = if migration.success {
            "status-migration-ok"
        } else {
            "status-migration-failed"
        };
        println!(
            "{}",
            tr(
                id,
                &[
                    ("version", format!("{:04}", migration.version)),
                    ("description", migration.description.clone()),
                    ("date", migration.installed_on.clone()),
                ]
            )
        );
    }

    if status.pending.is_empty() {
        println!("{}", t("status-migrations-none-pending"));
    } else {
        println!(
            "{}",
            tr(
                "status-migrations-pending",
                &[("count", status.pending.len().to_string())]
            )
        );
        for migration in &status.pending {
            println!(
                "{}",
                tr(
                    "status-migration-pending",
                    &[
                        ("version", format!("{:04}", migration.version)),
                        ("description", migration.description.clone()),
                    ]
                )
            );
        }
        println!("{}", t("status-migrations-hint"));
        return Ok(());
    }

    let state = settings::get_text(&pool, settings::SITE_STATE).await?;
    println!();
    println!(
        "{}",
        tr("status-site-state", &[("value", site_state(state))])
    );
    Ok(())
}

fn site_state(state: Option<String>) -> String {
    match state.as_deref() {
        Some("install_incomplete") => t("site-state-install-incomplete"),
        Some("fresh") => t("site-state-fresh"),
        Some("published") => t("site-state-published"),
        Some("under_construction") => t("site-state-under-construction"),
        Some(other) => other.to_string(),
        None => t("value-none"),
    }
}
