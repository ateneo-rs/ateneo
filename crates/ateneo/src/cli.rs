// SPDX-FileCopyrightText: 2026 James Paul Vizuete Davila
// SPDX-License-Identifier: AGPL-3.0-or-later

use clap::{Arg, ArgAction, Command};

use crate::i18n::t;

pub const INIT: &str = "init";
pub const CONFIG: &str = "config";
pub const SET: &str = "set";
pub const STATUS: &str = "status";
pub const OPTION: &str = "option";
pub const VALUE: &str = "value";
pub const YES: &str = "yes";
pub const VERBOSE: &str = "verbose";

pub fn command() -> Command {
    base(Command::new("ateneo"), Sections::Commands)
        .about(t("cli-about"))
        .version(env!("CARGO_PKG_VERSION"))
        .override_usage(t("cli-usage"))
        .disable_version_flag(true)
        .arg(
            Arg::new(VERBOSE)
                .long(VERBOSE)
                .global(true)
                .action(ArgAction::SetTrue)
                .help(t("cli-verbose-flag")),
        )
        .arg(
            Arg::new("version")
                .short('V')
                .long("version")
                .action(ArgAction::Version)
                .help(t("cli-version-flag")),
        )
        .subcommand(
            base(Command::new(INIT), Sections::Options)
                .about(t("cli-init-about"))
                .override_usage(t("cli-init-usage")),
        )
        .subcommand(
            base(Command::new(CONFIG), Sections::Commands)
                .about(t("cli-config-about"))
                .override_usage(t("cli-config-usage"))
                .subcommand(
                    base(Command::new(SET), Sections::Arguments)
                        .about(t("cli-config-set-about"))
                        .override_usage(t("cli-config-set-usage"))
                        .arg(
                            Arg::new(OPTION)
                                .required(true)
                                .value_name(t("cli-option-name"))
                                .help(t("cli-option-help")),
                        )
                        .arg(
                            Arg::new(VALUE)
                                .required(true)
                                .allow_hyphen_values(true)
                                .value_name(t("cli-value-name"))
                                .help(t("cli-value-help")),
                        )
                        .arg(
                            Arg::new(YES)
                                .short('y')
                                .long(YES)
                                .action(ArgAction::SetTrue)
                                .help(t("cli-yes-help")),
                        ),
                ),
        )
        .subcommand(
            base(Command::new(STATUS), Sections::Options)
                .about(t("cli-status-about"))
                .override_usage(t("cli-status-usage")),
        )
}

enum Sections {
    Options,
    Commands,
    Arguments,
}

fn base(command: Command, sections: Sections) -> Command {
    let usage = format!("{}: {{usage}}", t("cli-usage-heading"));
    let options = format!("{}:\n{{options}}", t("cli-options-heading"));
    let template = match sections {
        Sections::Options => format!("{{about}}\n\n{usage}\n\n{options}"),
        Sections::Commands => format!(
            "{{about}}\n\n{usage}\n\n{}:\n{{subcommands}}\n\n{options}",
            t("cli-commands-heading")
        ),
        Sections::Arguments => format!(
            "{{about}}\n\n{usage}\n\n{}:\n{{positionals}}\n\n{options}",
            t("cli-arguments-heading")
        ),
    };
    command
        .help_template(template)
        .disable_help_flag(true)
        .disable_help_subcommand(true)
        .arg(
            Arg::new("help")
                .short('h')
                .long("help")
                .action(ArgAction::Help)
                .help(t("cli-help-flag")),
        )
}
