// SPDX-FileCopyrightText: 2026 James Paul Vizuete Davila
// SPDX-License-Identifier: AGPL-3.0-or-later

mod cli;
mod config_cmd;
mod exit;
mod i18n;
mod init;
mod status;

use std::process::ExitCode;

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let mut command = cli::command();
    let matches = match command.try_get_matches_from_mut(std::env::args_os()) {
        Ok(matches) => matches,
        Err(err) => err.exit(),
    };

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
