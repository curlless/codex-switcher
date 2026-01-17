use clap::{FromArgMatches, error::ErrorKind};
use std::process::Command as ProcessCommand;

use crate::cli::{Cli, Commands, command_with_examples};

pub fn run_cli() {
    let args: Vec<std::ffi::OsString> = std::env::args_os().collect();
    if args.len() == 1 {
        let name = package_command_name();
        println!("{name} {}", env!("CARGO_PKG_VERSION"));
        println!();
        let mut cmd = command_with_examples();
        let _ = cmd.print_help();
        println!();
        return;
    }
    let cmd = command_with_examples();
    let matches = match cmd.clone().try_get_matches_from(args) {
        Ok(matches) => matches,
        Err(err) => {
            if err.kind() == ErrorKind::DisplayHelp {
                let name = package_command_name();
                println!("{name} {}", env!("CARGO_PKG_VERSION"));
                println!();
                let _ = err.print();
                println!();
                return;
            }
            err.exit();
        }
    };
    let cli = Cli::from_arg_matches(&matches).unwrap_or_else(|err| err.exit());
    set_plain(cli.plain);
    if let Err(message) = run(cli) {
        if message == CANCELLED_MESSAGE {
            let message = format_cancel(use_color_stdout());
            print_output_block(&message);
            return;
        }
        eprintln!("{message}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), String> {
    let paths = resolve_paths()?;
    ensure_paths(&paths)?;

    ensure_codex_cli(detect_install_source())?;

    let check_for_update_on_startup = std::env::var_os("CODEX_PROFILES_SKIP_UPDATE").is_none();
    let update_config = UpdateConfig {
        codex_home: paths.codex.clone(),
        check_for_update_on_startup,
    };
    match run_update_prompt_if_needed(&update_config)? {
        UpdatePromptOutcome::Continue => {}
        UpdatePromptOutcome::RunUpdate(action) => {
            return run_update_action(action);
        }
    }

    let _ = sync_current_readonly(&paths);

    match cli.command {
        Commands::Save { label } => save_profile(&paths, label),
        Commands::Load { label } => load_profile(&paths, label),
        Commands::List => list_profiles(&paths, false, false, false, false),
        Commands::Status { all, label } => {
            if label.is_some() && all {
                return Err("Error: --label cannot be combined with --all.".to_string());
            }
            if let Some(label) = label {
                status_label(&paths, &label)
            } else {
                status_profiles(&paths, all)
            }
        }
        Commands::Delete { yes, label } => delete_profile(&paths, yes, label),
        Commands::Export {
            every_code,
            code_home,
            overwrite,
        } => {
            if !every_code {
                return Err("Error: export target required (use --every-code).".to_string());
            }
            export_every_code(&paths, code_home, overwrite)
        }
        Commands::Import {
            every_code,
            code_home,
        } => {
            if !every_code {
                return Err("Error: import source required (use --every-code).".to_string());
            }
            import_every_code(&paths, code_home)
        }
    }
}

fn run_update_action(action: UpdateAction) -> Result<(), String> {
    let (command, args) = action.command_args();
    let status = ProcessCommand::new(command)
        .args(args)
        .status()
        .map_err(|err| format!("Error: failed to run update command: {err}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "Error: update command failed: {}",
            action.command_str()
        ))
    }
}
mod auth;
mod cli;
mod common;
mod everycode;
mod profiles;
mod requirements;
mod ui;
mod updates;
mod usage;

pub use auth::*;
pub use common::*;
pub use everycode::*;
pub use profiles::*;
pub use requirements::*;
pub use ui::*;
pub use updates::*;
pub use usage::*;
