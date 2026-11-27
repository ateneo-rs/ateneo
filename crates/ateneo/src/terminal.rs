// SPDX-FileCopyrightText: 2026 James Paul Vizuete Davila
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::io::{self, IsTerminal};

use cliclack::{Theme, ThemeState};
use console::Emoji;

use crate::i18n::t;

const BAR: Emoji = Emoji("│", "|");

pub fn is_interactive() -> bool {
    io::stdin().is_terminal() && io::stderr().is_terminal()
}

pub struct AteneoTheme;

impl Theme for AteneoTheme {
    fn format_confirm(&self, state: &ThemeState, confirm: bool) -> String {
        let yes = self.radio_item(state, confirm, &t("answer-yes"), "");
        let no = self.radio_item(state, !confirm, &t("answer-no"), "");
        let divider = match state {
            ThemeState::Active => self.placeholder_style(state).apply_to(" / ").to_string(),
            _ => String::new(),
        };
        format!(
            "{bar}  {yes}{divider}{no}\n",
            bar = self.bar_color(state).apply_to(BAR),
        )
    }
}

pub struct Setup {
    #[cfg(windows)]
    restore: Vec<(windows_sys::Win32::Foundation::HANDLE, u32)>,
}

pub fn setup() -> Setup {
    cliclack::set_theme(AteneoTheme);
    Setup {
        #[cfg(windows)]
        restore: windows::enable_virtual_terminal(),
    }
}

#[cfg(windows)]
impl Drop for Setup {
    fn drop(&mut self) {
        windows::restore(&self.restore);
    }
}

#[cfg(windows)]
mod windows {
    use windows_sys::Win32::Foundation::HANDLE;
    use windows_sys::Win32::System::Console::{
        ENABLE_VIRTUAL_TERMINAL_PROCESSING, GetConsoleMode, GetStdHandle, STD_ERROR_HANDLE,
        STD_OUTPUT_HANDLE, SetConsoleMode,
    };

    pub fn enable_virtual_terminal() -> Vec<(HANDLE, u32)> {
        let mut changed = Vec::new();
        for id in [STD_OUTPUT_HANDLE, STD_ERROR_HANDLE] {
            unsafe {
                let handle = GetStdHandle(id);
                let mut mode = 0;
                if GetConsoleMode(handle, &mut mode) == 0
                    || mode & ENABLE_VIRTUAL_TERMINAL_PROCESSING != 0
                {
                    continue;
                }
                if SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING) != 0 {
                    changed.push((handle, mode));
                }
            }
        }
        changed
    }

    pub fn restore(changed: &[(HANDLE, u32)]) {
        for (handle, mode) in changed {
            unsafe {
                SetConsoleMode(*handle, *mode);
            }
        }
    }
}
