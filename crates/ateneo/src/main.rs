// SPDX-FileCopyrightText: 2026 James Paul Vizuete Davila
// SPDX-License-Identifier: AGPL-3.0-or-later

mod cli;
mod config_cmd;
mod exit;
mod i18n;
mod init;
mod prompt;
mod status;
mod terminal;

use std::process::ExitCode;

use clap::ArgMatches;

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let _terminal = terminal::setup();
    let mut command = cli::command();
    let matches = match command.try_get_matches_from_mut(std::env::args_os()) {
        Ok(matches) => matches,
        Err(err) => err.exit(),
    };
    exit::set_verbose(verbose(&matches));

    let result = match matches.subcommand() {
        Some((cli::INIT, _)) => init::run().await,
        Some((cli::CONFIG, config)) => match config.subcommand() {
            Some((cli::SET, args)) => {
                let option = args
                    .get_one::<String>(cli::OPTION)
                    .map_or("", String::as_str);
                let value = args
                    .get_one::<String>(cli::VALUE)
                    .map_or("", String::as_str);
                config_cmd::set(option, value, args.get_flag(cli::YES)).await
            }
            _ => config_cmd::menu().await,
        },
        Some((cli::STATUS, _)) => status::run().await,
        _ => {
            let _ = command.print_help();
            Ok(())
        }
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(exit) => exit.report(),
    }
}

fn verbose(matches: &ArgMatches) -> bool {
    let mut current = Some(matches);
    while let Some(level) = current {
        if level.get_flag(cli::VERBOSE) {
            return true;
        }
        current = level.subcommand().map(|(_, sub)| sub);
    }
    false
}
