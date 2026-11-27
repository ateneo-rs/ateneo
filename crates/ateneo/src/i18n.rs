// SPDX-FileCopyrightText: 2026 James Paul Vizuete Davila
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::borrow::Cow;
use std::collections::HashMap;

use fluent_templates::fluent_bundle::FluentValue;
use fluent_templates::{LanguageIdentifier, Loader, langid, static_loader};

static_loader! {
    static LOCALES = {
        locales: "./locales",
        fallback_language: "es",
        customise: |bundle| bundle.set_use_isolating(false),
    };
}

const LANGUAGE: LanguageIdentifier = langid!("es");

pub fn t(id: &str) -> String {
    LOCALES.lookup(&LANGUAGE, id)
}

pub fn tr(id: &str, args: &[(&'static str, String)]) -> String {
    let args: HashMap<Cow<'static, str>, FluentValue<'static>> = args
        .iter()
        .map(|(name, value)| (Cow::Borrowed(*name), FluentValue::from(value.clone())))
        .collect();
    LOCALES.lookup_with_args(&LANGUAGE, id, &args)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use super::*;

    fn ids_in(source: &str) -> Vec<String> {
        let mut ids = Vec::new();
        for marker in ["t(\"", "tr(\""] {
            let mut rest = source;
            while let Some(start) = rest.find(marker) {
                let standalone = rest[..start]
                    .chars()
                    .next_back()
                    .is_none_or(|c| !(c.is_alphanumeric() || c == '_' || c == ':' || c == '.'));
                let after = &rest[start + marker.len()..];
                let end = after.find('"').unwrap_or(after.len());
                if standalone {
                    ids.push(after[..end].to_string());
                }
                rest = &after[end..];
            }
        }
        ids
    }

    fn defined_ids() -> Vec<String> {
        let locales = Path::new(env!("CARGO_MANIFEST_DIR")).join("locales/es");
        let mut ids = Vec::new();
        for entry in fs::read_dir(locales).unwrap() {
            let text = fs::read_to_string(entry.unwrap().path()).unwrap();
            for line in text.lines() {
                if let Some((id, _)) = line.split_once(" =")
                    && !id.is_empty()
                    && id.chars().all(|c| c.is_ascii_lowercase() || c == '-')
                {
                    ids.push(id.to_string());
                }
            }
        }
        ids
    }

    #[test]
    fn every_message_resolves() {
        for id in defined_ids() {
            assert!(!t(&id).is_empty(), "{id}");
        }
    }

    #[test]
    fn every_message_used_in_the_code_exists() {
        let defined = defined_ids();
        let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut missing = Vec::new();
        for entry in fs::read_dir(&src).unwrap() {
            let path = entry.unwrap().path();
            if path.file_name().is_some_and(|name| name == "i18n.rs") {
                continue;
            }
            let source = fs::read_to_string(&path).unwrap();
            for id in ids_in(&source) {
                if !defined.contains(&id) {
                    missing.push(format!("{}: {id}", path.display()));
                }
            }
        }
        assert!(missing.is_empty(), "{missing:#?}");
    }
}
