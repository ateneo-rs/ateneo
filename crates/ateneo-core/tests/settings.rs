// SPDX-FileCopyrightText: 2026 James Paul Vizuete Davila
// SPDX-License-Identifier: AGPL-3.0-or-later

use ateneo_core::settings::{self, SettingKey, SiteState};
use serde_json::json;
use sqlx::PgPool;

#[sqlx::test(migrations = false)]
async fn set_inserts_and_updates(pool: PgPool) {
    ateneo_core::db::migrate(&pool).await.unwrap();

    assert_eq!(
        settings::get(&pool, settings::SITE_STATE).await.unwrap(),
        None
    );

    settings::set(
        &pool,
        settings::SITE_STATE,
        &json!(SiteState::InstallIncomplete.as_str()),
    )
    .await
    .unwrap();
    settings::set(
        &pool,
        settings::SITE_STATE,
        &json!(SiteState::Fresh.as_str()),
    )
    .await
    .unwrap();

    assert_eq!(
        settings::get_text(&pool, settings::SITE_STATE)
            .await
            .unwrap()
            .as_deref(),
        Some("fresh")
    );
    assert_eq!(
        settings::get(&pool, settings::SITE_STATE).await.unwrap(),
        Some(json!("fresh"))
    );
}

#[sqlx::test(migrations = false)]
async fn knows_timezones(pool: PgPool) {
    assert!(
        settings::is_timezone(&pool, "America/Guayaquil")
            .await
            .unwrap()
    );
    assert!(settings::is_timezone(&pool, "UTC").await.unwrap());
    assert!(!settings::is_timezone(&pool, "Marte/Olympus").await.unwrap());
}

#[test]
fn site_languages() {
    assert!(settings::is_site_language("es"));
    assert!(!settings::is_site_language("xx"));
    assert!(settings::is_site_language(settings::DEFAULT_SITE_LANGUAGE));
}

#[test]
fn setting_keys_round_trip() {
    for key in SettingKey::ALL {
        assert_eq!(SettingKey::from_name(key.name()), Some(key));
    }
    assert_eq!(SettingKey::from_name("site.state"), None);
}
