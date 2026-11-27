// SPDX-FileCopyrightText: 2026 James Paul Vizuete Davila
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::exit::{Exit, Prompt};
use crate::i18n::t;

pub struct Choice<T> {
    pub value: T,
    pub label: String,
    pub hint: String,
}

pub fn choose<T: Clone + Eq>(prompt: &str, choices: Vec<Choice<T>>) -> Result<Option<T>, Exit> {
    if choices.is_empty() {
        return Ok(None);
    }
    let mut select = cliclack::select(prompt);
    for choice in choices {
        select = select.item(Some(choice.value), choice.label, choice.hint);
    }
    select
        .item(None, t("choice-other"), "")
        .interact()
        .or_cancel()
}

pub fn text<V>(
    prompt: &str,
    example: &str,
    previous: Option<&str>,
    validate: V,
) -> Result<String, Exit>
where
    V: Fn(&str) -> Result<String, String> + Clone + 'static,
{
    let check = validate.clone();
    let mut input = cliclack::input(prompt).required(false);
    input = match previous {
        Some(previous) => input.placeholder(previous).default_input(previous),
        None => input.placeholder(example),
    };
    let value: String = input
        .validate(move |value: &String| match required(value) {
            Ok(()) => check(value).map(|_| ()),
            Err(message) => Err(message),
        })
        .interact()
        .or_cancel()?;
    validate(&value).map_err(Exit::Failed)
}

pub fn required(value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err(t("field-required"))
    } else {
        Ok(())
    }
}

pub fn password(prompt: &str) -> Result<String, Exit> {
    cliclack::password(prompt)
        .allow_empty()
        .interact()
        .or_cancel()
}

pub fn yes_no(prompt: &str, initial: bool) -> Result<bool, Exit> {
    cliclack::confirm(prompt)
        .initial_value(initial)
        .interact()
        .or_cancel()
}
