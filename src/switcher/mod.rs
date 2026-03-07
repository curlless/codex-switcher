use clap::{FromArgMatches, error::ErrorKind};
use std::process::Command as ProcessCommand;

use crate::switcher::cli::{Cli, Commands, ConfigCommands, command_with_examples};

pub fn run_cli() {
    let args: Vec<std::ffi::OsString> = std::env::args_os().collect();
    if let Err(message) = run_cli_with_args(args) {
        eprintln!("{message}");
        std::process::exit(1);
    }
}

fn run_cli_with_args(args: Vec<std::ffi::OsString>) -> Result<(), String> {
    if args.len() == 1 {
        let name = package_command_name();
        println!("{name} {}", env!("CARGO_PKG_VERSION"));
        println!();
        let mut cmd = command_with_examples();
        let _ = cmd.print_help();
        println!();
        return Ok(());
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
                return Ok(());
            }
            if err.kind() == ErrorKind::DisplayVersion {
                println!("{} {}", command_name(), env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            return Err(err.to_string());
        }
    };
    let cli = Cli::from_arg_matches(&matches).map_err(|err| err.to_string())?;
    set_plain(cli.plain);
    if let Err(message) = run(cli) {
        if message == CANCELLED_MESSAGE {
            let message = format_cancel(use_color_stdout());
            print_output_block(&message);
            return Ok(());
        }
        return Err(message);
    }
    Ok(())
}

fn run(cli: Cli) -> Result<(), String> {
    let paths = resolve_paths()?;
    ensure_paths(&paths)?;

    if let Commands::Config { command } = &cli.command {
        return match command {
            ConfigCommands::Edit => edit_config(&paths),
            ConfigCommands::Show => show_config(&paths),
            ConfigCommands::DetectCodexApp { write_env } => detect_codex_app(&paths, *write_env),
        };
    }

    let check_for_update_on_startup = (std::env::var_os("CODEX_SWITCHER_ENABLE_UPDATE").is_some()
        || std::env::var_os("CODEX_PROFILES_ENABLE_UPDATE").is_some())
        && std::env::var_os("CODEX_SWITCHER_SKIP_UPDATE").is_none()
        && std::env::var_os("CODEX_PROFILES_SKIP_UPDATE").is_none();
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
        Commands::Config { .. } => unreachable!("config handled before startup checks"),
        Commands::Save { label } => save_profile(&paths, label),
        Commands::Load { label } => load_profile(&paths, label),
        Commands::List => list_profiles(&paths, false, false, false, false),
        Commands::Status {
            all: _all,
            current,
            label,
        } => {
            if label.is_some() && _all {
                return Err("Error: --label cannot be combined with --all.".to_string());
            }
            if label.is_some() && current {
                return Err("Error: --label cannot be combined with --current.".to_string());
            }
            if _all && current {
                return Err("Error: --all cannot be combined with --current.".to_string());
            }
            if let Some(label) = label {
                status_label(&paths, &label)
            } else if current {
                status_profiles(&paths, false)
            } else {
                // New default: status shows all saved profiles.
                status_profiles(&paths, true)
            }
        }
        Commands::Switch {
            dry_run,
            reload_ide,
            reload_app,
        } => {
            if reload_ide && reload_app.is_some() {
                return Err("Error: --reload-ide cannot be combined with --reload-app.".to_string());
            }
            let reload_target = if reload_ide {
                Some(ReloadAppTarget::All)
            } else {
                switch_reload_target(&paths, reload_app)?
            };
            switch_best_profile(&paths, dry_run, reload_target)
        }
        Commands::ReloadApp { dry_run, target } => {
            let target = effective_reload_target(&paths, target)?;
            reload_app(&paths, dry_run, target)
        }
        Commands::Reserve { label } => reserve_profile(&paths, label),
        Commands::Unreserve { label } => unreserve_profile(&paths, label),
        Commands::Migrate { from, overwrite } => migrate_profiles(&paths, from, overwrite),
        Commands::Delete { yes, label } => delete_profile(&paths, yes, label),
        Commands::RelayLogin { url } => relay_login(url),
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
mod config;
mod ide_reload;
mod profile_store;
mod profiles;
mod relay;
mod requirements;
#[cfg(all(test, feature = "switcher-unit-tests"))]
mod test_utils;
mod ui;
mod updates;
mod usage;

pub use auth::*;
pub use common::*;
pub use config::*;
pub use ide_reload::*;
pub use profile_store::*;
pub use profiles::*;
pub use relay::*;
pub use requirements::*;
pub use ui::*;
pub use updates::*;
pub use usage::*;

#[cfg(all(test, feature = "switcher-unit-tests"))]
mod tests {
    use super::*;
    use crate::switcher::test_utils::{make_paths, set_env_guard};
    use std::ffi::OsString;
    use std::fs;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn run_cli_with_args_help() {
        let args = vec![OsString::from("codex-switcher")];
        run_cli_with_args(args).unwrap();
    }

    #[test]
    fn run_cli_with_args_display_help() {
        let args = vec![OsString::from("codex-switcher"), OsString::from("--help")];
        run_cli_with_args(args).unwrap();
    }

    #[test]
    fn run_cli_with_args_errors() {
        let args = vec![OsString::from("codex-switcher"), OsString::from("nope")];
        let err = run_cli_with_args(args).unwrap_err();
        assert!(err.contains("error"));
    }

    #[cfg(unix)]
    #[test]
    fn run_update_action_paths() {
        let dir = tempfile::tempdir().expect("tempdir");
        let bin = dir.path().join("npm");
        fs::write(&bin, "#!/bin/sh\nexit 0\n").unwrap();
        let mut perms = fs::metadata(&bin).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&bin, perms).unwrap();
        let path = dir.path().to_string_lossy().into_owned();
        {
            let _env = set_env_guard("PATH", Some(&path));
            run_update_action(UpdateAction::NpmGlobalLatest).unwrap();
        }
        {
            let _env = set_env_guard("PATH", Some(""));
            let err = run_update_action(UpdateAction::NpmGlobalLatest).unwrap_err();
            assert!(err.contains("failed to run update command"));
        }
    }

    #[test]
    fn run_cli_list_command() {
        let dir = tempfile::tempdir().expect("tempdir");
        let paths = make_paths(dir.path());
        fs::create_dir_all(&paths.profiles).unwrap();
        let home = dir.path().to_string_lossy().into_owned();
        let _home = set_env_guard("CODEX_SWITCHER_HOME", Some(&home));
        let _skip = set_env_guard("CODEX_SWITCHER_SKIP_UPDATE", Some("1"));
        let cli = Cli {
            plain: true,
            command: Commands::List,
        };
        run(cli).unwrap();
    }
}
