use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::io::IsTerminal as _;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::Duration as StdDuration;

use crate::write_atomic;

// We use the latest version from the cask if installation is via homebrew - homebrew does not immediately pick up the latest release and can lag behind.
const HOMEBREW_CASK_URL: &str =
    "https://raw.githubusercontent.com/Homebrew/homebrew-cask/HEAD/Casks/c/codex-profiles.rb";
const LATEST_RELEASE_URL: &str =
    "https://api.github.com/repos/midhunmonachan/codex-profiles/releases/latest";
const RELEASE_NOTES_URL: &str = "https://github.com/midhunmonachan/codex-profiles/releases/latest";
const HOMEBREW_CASK_URL_OVERRIDE_ENV_VAR: &str = "CODEX_PROFILES_HOMEBREW_CASK_URL";
const LATEST_RELEASE_URL_OVERRIDE_ENV_VAR: &str = "CODEX_PROFILES_LATEST_RELEASE_URL";
const UPDATE_AVAILABLE: &str = "Update available!";
const VERSION_FILENAME: &str = "profiles/version.json";

/// Update action the CLI should perform after the prompt exits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateAction {
    /// Update via `npm install -g codex-profiles`.
    NpmGlobalLatest,
    /// Update via `bun install -g codex-profiles`.
    BunGlobalLatest,
    /// Update via `brew upgrade codex-profiles`.
    BrewUpgrade,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallSource {
    Npm,
    Bun,
    Brew,
    Unknown,
}

impl UpdateAction {
    /// Returns the list of command-line arguments for invoking the update.
    pub fn command_args(self) -> (&'static str, &'static [&'static str]) {
        match self {
            UpdateAction::NpmGlobalLatest => ("npm", &["install", "-g", "codex-profiles"]),
            UpdateAction::BunGlobalLatest => ("bun", &["install", "-g", "codex-profiles"]),
            UpdateAction::BrewUpgrade => ("brew", &["upgrade", "codex-profiles"]),
        }
    }

    /// Returns string representation of the command-line arguments for invoking the update.
    pub fn command_str(self) -> String {
        let (command, args) = self.command_args();
        shlex::try_join(std::iter::once(command).chain(args.iter().copied()))
            .unwrap_or_else(|_| format!("{command} {}", args.join(" ")))
    }
}

pub fn detect_install_source() -> InstallSource {
    let exe = std::env::current_exe().unwrap_or_default();
    let managed_by_npm = std::env::var_os("CODEX_PROFILES_MANAGED_BY_NPM").is_some();
    let managed_by_bun = std::env::var_os("CODEX_PROFILES_MANAGED_BY_BUN").is_some();
    detect_install_source_inner(
        cfg!(target_os = "macos"),
        &exe,
        managed_by_npm,
        managed_by_bun,
    )
}

#[doc(hidden)]
pub fn detect_install_source_inner(
    is_macos: bool,
    current_exe: &std::path::Path,
    managed_by_npm: bool,
    managed_by_bun: bool,
) -> InstallSource {
    if managed_by_npm {
        InstallSource::Npm
    } else if managed_by_bun {
        InstallSource::Bun
    } else if is_macos && is_brew_install(current_exe) {
        InstallSource::Brew
    } else {
        InstallSource::Unknown
    }
}

fn is_brew_install(current_exe: &std::path::Path) -> bool {
    (current_exe.starts_with("/opt/homebrew") || current_exe.starts_with("/usr/local"))
        && current_exe.file_name().and_then(|name| name.to_str()) == Some("codex-profiles")
}

pub(crate) fn get_update_action() -> Option<UpdateAction> {
    get_update_action_with_debug(cfg!(debug_assertions), detect_install_source())
}

fn get_update_action_with_debug(
    is_debug: bool,
    install_source: InstallSource,
) -> Option<UpdateAction> {
    if is_debug {
        return None;
    }
    match install_source {
        InstallSource::Npm => Some(UpdateAction::NpmGlobalLatest),
        InstallSource::Bun => Some(UpdateAction::BunGlobalLatest),
        InstallSource::Brew => Some(UpdateAction::BrewUpgrade),
        InstallSource::Unknown => None,
    }
}

#[derive(Clone, Debug)]
pub struct UpdateConfig {
    pub codex_home: PathBuf,
    pub check_for_update_on_startup: bool,
}

#[derive(Deserialize, Debug, Clone)]
struct ReleaseInfo {
    tag_name: String,
}

pub enum UpdatePromptOutcome {
    Continue,
    RunUpdate(UpdateAction),
}

pub fn run_update_prompt_if_needed(config: &UpdateConfig) -> Result<UpdatePromptOutcome, String> {
    let mut input = io::stdin().lock();
    let mut output = io::stderr();
    run_update_prompt_if_needed_with_io(
        config,
        cfg!(debug_assertions),
        io::stdin().is_terminal(),
        &mut input,
        &mut output,
    )
}

fn run_update_prompt_if_needed_with_io(
    config: &UpdateConfig,
    is_debug: bool,
    is_tty: bool,
    input: &mut impl io::BufRead,
    output: &mut impl Write,
) -> Result<UpdatePromptOutcome, String> {
    run_update_prompt_if_needed_with_io_and_source(
        config,
        is_debug,
        is_tty,
        detect_install_source(),
        input,
        output,
    )
}

fn run_update_prompt_if_needed_with_io_and_source(
    config: &UpdateConfig,
    is_debug: bool,
    is_tty: bool,
    install_source: InstallSource,
    input: &mut impl io::BufRead,
    output: &mut impl Write,
) -> Result<UpdatePromptOutcome, String> {
    if is_debug {
        return Ok(UpdatePromptOutcome::Continue);
    }

    let Some(latest_version) = get_upgrade_version_for_popup_with_debug(config, is_debug) else {
        return Ok(UpdatePromptOutcome::Continue);
    };
    let Some(update_action) = get_update_action_with_debug(false, install_source) else {
        return Ok(UpdatePromptOutcome::Continue);
    };

    let current_version = current_version();
    if !is_tty {
        write_prompt(
            output,
            format_args!("{UPDATE_AVAILABLE} {current_version} -> {latest_version}\n"),
        )?;
        write_prompt(
            output,
            format_args!("Run `{}` to update.\n", update_action.command_str()),
        )?;
        return Ok(UpdatePromptOutcome::Continue);
    }

    write_prompt(
        output,
        format_args!("\n✨ {UPDATE_AVAILABLE} {current_version} -> {latest_version}\n"),
    )?;
    write_prompt(output, format_args!("Release notes: {RELEASE_NOTES_URL}\n"))?;
    write_prompt(output, format_args!("\n"))?;
    write_prompt(
        output,
        format_args!("1) Update now (runs `{}`)\n", update_action.command_str()),
    )?;
    write_prompt(output, format_args!("2) Skip\n"))?;
    write_prompt(output, format_args!("3) Skip until next version\n"))?;
    write_prompt(output, format_args!("Select [1-3]: "))?;
    output.flush().map_err(prompt_io_error)?;

    let mut selection = String::new();
    input
        .read_line(&mut selection)
        .map_err(|err| format!("Error: failed to read update choice: {err}"))?;

    match selection.trim() {
        "1" => Ok(UpdatePromptOutcome::RunUpdate(update_action)),
        "3" => {
            if let Err(err) = dismiss_version(config, &latest_version) {
                write_prompt(
                    output,
                    format_args!("Failed to persist update dismissal: {err}\n"),
                )?;
            }
            Ok(UpdatePromptOutcome::Continue)
        }
        _ => Ok(UpdatePromptOutcome::Continue),
    }
}

fn prompt_io_error(err: impl std::fmt::Display) -> String {
    format!("Error: failed to prompt for update: {err}")
}

fn write_prompt(output: &mut impl Write, args: std::fmt::Arguments) -> Result<(), String> {
    output.write_fmt(args).map_err(prompt_io_error)
}

fn current_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub fn get_upgrade_version(config: &UpdateConfig) -> Option<String> {
    get_upgrade_version_with_debug(config, cfg!(debug_assertions))
}

fn get_upgrade_version_with_debug(config: &UpdateConfig, is_debug: bool) -> Option<String> {
    if updates_disabled_with_debug(config, is_debug) {
        return None;
    }
    let version_file = version_filepath(config);
    let mut info = read_version_info(&version_file).ok();

    let should_check = match &info {
        None => true,
        Some(info) => info.last_checked_at < Utc::now() - Duration::hours(20),
    };
    if should_check {
        if info.is_none() {
            if let Err(err) = check_for_update(&version_file) {
                eprintln!("Failed to update version: {err}");
            }
            info = read_version_info(&version_file).ok();
        } else {
            let version_file = version_file.clone();
            std::thread::spawn(move || {
                if let Err(err) = check_for_update(&version_file) {
                    eprintln!("Failed to update version: {err}");
                }
            });
        }
    }

    info.and_then(|info| {
        if is_newer(&info.latest_version, env!("CARGO_PKG_VERSION")).unwrap_or(false) {
            Some(info.latest_version)
        } else {
            None
        }
    })
}

fn check_for_update(version_file: &Path) -> anyhow::Result<()> {
    check_for_update_with_action(version_file, get_update_action())
}

fn check_for_update_with_action(
    version_file: &Path,
    update_action: Option<UpdateAction>,
) -> anyhow::Result<()> {
    let latest_version = match update_action {
        Some(UpdateAction::BrewUpgrade) => {
            fetch_version_from_cask().or_else(fetch_version_from_release)
        }
        _ => fetch_version_from_release(),
    };

    // Preserve any previously dismissed version if present.
    let prev_info = read_version_info(version_file).ok();
    let prev_dismissed = prev_info
        .as_ref()
        .and_then(|info| info.dismissed_version.clone());
    let prev_prompted = prev_info.as_ref().and_then(|info| info.last_prompted_at);
    if latest_version.is_none() {
        let info = VersionInfo {
            latest_version: env!("CARGO_PKG_VERSION").to_string(),
            last_checked_at: Utc::now(),
            dismissed_version: prev_dismissed.clone(),
            last_prompted_at: prev_prompted,
        };
        return write_version_info(version_file, &info);
    }
    let info = VersionInfo {
        latest_version: latest_version.unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string()),
        last_checked_at: Utc::now(),
        dismissed_version: prev_dismissed,
        last_prompted_at: prev_prompted,
    };

    write_version_info(version_file, &info)
}

#[doc(hidden)]
pub fn is_newer(latest: &str, current: &str) -> Option<bool> {
    match (parse_version(latest), parse_version(current)) {
        (Some(l), Some(c)) => Some(l > c),
        _ => None,
    }
}

#[doc(hidden)]
pub fn extract_version_from_cask(cask_contents: &str) -> anyhow::Result<String> {
    cask_contents
        .lines()
        .find_map(|line| {
            let line = line.trim();
            line.strip_prefix("version \"")
                .and_then(|rest| rest.strip_suffix('"'))
                .map(ToString::to_string)
        })
        .ok_or_else(|| anyhow::anyhow!("Failed to find version in Homebrew cask file"))
}

#[doc(hidden)]
pub fn extract_version_from_latest_tag(latest_tag_name: &str) -> anyhow::Result<String> {
    for prefix in ["v", "rust-v"] {
        if let Some(version) = latest_tag_name.strip_prefix(prefix) {
            return Ok(version.to_string());
        }
    }
    Err(anyhow::anyhow!(
        "Failed to parse latest tag name '{latest_tag_name}'"
    ))
}

fn fetch_version_from_cask() -> Option<String> {
    let response = update_agent()
        .get(&homebrew_cask_url())
        .header("User-Agent", "codex-profiles")
        .call();
    match response {
        Ok(mut resp) => {
            let contents = resp.body_mut().read_to_string().ok()?;
            extract_version_from_cask(&contents).ok()
        }
        Err(ureq::Error::StatusCode(404)) => None,
        Err(_) => None,
    }
}

fn fetch_version_from_release() -> Option<String> {
    let response = update_agent()
        .get(&latest_release_url())
        .header("User-Agent", "codex-profiles")
        .call();
    match response {
        Ok(mut resp) => {
            let ReleaseInfo {
                tag_name: latest_tag_name,
            } = resp.body_mut().read_json().ok()?;
            extract_version_from_latest_tag(&latest_tag_name).ok()
        }
        Err(ureq::Error::StatusCode(404)) => None,
        Err(_) => None,
    }
}

/// Returns the latest version to show in a popup, if it should be shown.
/// This respects the user's dismissal choice for the current latest version.
pub fn get_upgrade_version_for_popup(config: &UpdateConfig) -> Option<String> {
    get_upgrade_version_for_popup_with_debug(config, cfg!(debug_assertions))
}

fn get_upgrade_version_for_popup_with_debug(
    config: &UpdateConfig,
    is_debug: bool,
) -> Option<String> {
    if updates_disabled_with_debug(config, is_debug) {
        return None;
    }

    let version_file = version_filepath(config);
    let latest = get_upgrade_version_with_debug(config, is_debug)?;
    let info = read_version_info(&version_file).ok();
    if info
        .as_ref()
        .and_then(|info| info.last_prompted_at)
        .is_some_and(|last| last > Utc::now() - Duration::hours(24))
    {
        return None;
    }
    // If the user dismissed this exact version previously, do not show the popup.
    if info
        .as_ref()
        .and_then(|info| info.dismissed_version.as_deref())
        == Some(latest.as_str())
    {
        return None;
    }
    if let Some(mut info) = info {
        info.last_prompted_at = Some(Utc::now());
        let _ = write_version_info(&version_file, &info);
    }
    Some(latest)
}

/// Persist a dismissal for the current latest version so we don't show
/// the update popup again for this version.
pub fn dismiss_version(config: &UpdateConfig, version: &str) -> anyhow::Result<()> {
    if updates_disabled(config) {
        return Ok(());
    }
    let version_file = version_filepath(config);
    let mut info = match read_version_info(&version_file) {
        Ok(info) => info,
        Err(_) => return Ok(()),
    };
    info.dismissed_version = Some(version.to_string());
    info.last_prompted_at = Some(Utc::now());
    write_version_info(&version_file, &info)
}

fn parse_version(v: &str) -> Option<(u64, u64, u64)> {
    let mut iter = v.trim().split('.');
    let maj = iter.next()?.parse::<u64>().ok()?;
    let min = iter.next()?.parse::<u64>().ok()?;
    let pat = iter.next()?.parse::<u64>().ok()?;
    Some((maj, min, pat))
}

fn updates_disabled(config: &UpdateConfig) -> bool {
    updates_disabled_with_debug(config, cfg!(debug_assertions))
}

fn updates_disabled_with_debug(config: &UpdateConfig, is_debug: bool) -> bool {
    is_debug || !config.check_for_update_on_startup
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct VersionInfo {
    latest_version: String,
    // ISO-8601 timestamp (RFC3339)
    last_checked_at: DateTime<Utc>,
    #[serde(default)]
    dismissed_version: Option<String>,
    #[serde(default)]
    last_prompted_at: Option<DateTime<Utc>>,
}

fn version_filepath(config: &UpdateConfig) -> PathBuf {
    config.codex_home.join(VERSION_FILENAME)
}

fn read_version_info(version_file: &Path) -> anyhow::Result<VersionInfo> {
    let contents = std::fs::read_to_string(version_file)?;
    Ok(serde_json::from_str(&contents)?)
}

fn write_version_info(version_file: &Path, info: &VersionInfo) -> anyhow::Result<()> {
    let json_line = format!("{}\n", serde_json::to_string(info)?);
    write_atomic(version_file, json_line.as_bytes()).map_err(|err| anyhow::anyhow!(err))?;
    Ok(())
}

fn update_agent() -> ureq::Agent {
    let config = ureq::Agent::config_builder()
        .timeout_global(Some(StdDuration::from_secs(5)))
        .build();
    config.into()
}

fn latest_release_url() -> String {
    std::env::var(LATEST_RELEASE_URL_OVERRIDE_ENV_VAR)
        .unwrap_or_else(|_| LATEST_RELEASE_URL.to_string())
}

fn homebrew_cask_url() -> String {
    std::env::var(HOMEBREW_CASK_URL_OVERRIDE_ENV_VAR)
        .unwrap_or_else(|_| HOMEBREW_CASK_URL.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{ENV_MUTEX, http_ok_response, set_env_guard, spawn_server};
    use std::fs;
    use std::path::PathBuf;

    fn seed_version_info(config: &UpdateConfig, version: &str) {
        let version_file = version_filepath(config);
        let info = VersionInfo {
            latest_version: version.to_string(),
            last_checked_at: Utc::now(),
            dismissed_version: None,
            last_prompted_at: None,
        };
        write_version_info(&version_file, &info).unwrap();
    }

    #[test]
    fn update_action_commands() {
        let (cmd, args) = UpdateAction::NpmGlobalLatest.command_args();
        assert_eq!(cmd, "npm");
        assert!(args.contains(&"install"));
        assert!(UpdateAction::BunGlobalLatest.command_str().contains("bun"));
    }

    #[test]
    fn detect_install_source_inner_variants() {
        let exe = PathBuf::from("/usr/local/bin/codex-profiles");
        assert_eq!(
            detect_install_source_inner(true, &exe, false, false),
            InstallSource::Brew
        );
        assert_eq!(
            detect_install_source_inner(false, &exe, true, false),
            InstallSource::Npm
        );
        assert_eq!(
            detect_install_source_inner(false, &exe, false, true),
            InstallSource::Bun
        );
    }

    #[test]
    fn get_update_action_debug() {
        assert!(get_update_action_with_debug(true, InstallSource::Npm).is_none());
        assert!(get_update_action_with_debug(false, InstallSource::Npm).is_some());
    }

    #[test]
    fn extract_version_helpers() {
        assert_eq!(extract_version_from_latest_tag("v1.2.3").unwrap(), "1.2.3");
        assert_eq!(
            extract_version_from_latest_tag("rust-v2.0.0").unwrap(),
            "2.0.0"
        );
        assert!(extract_version_from_latest_tag("bad").is_err());
        let cask = "version \"1.2.3\"";
        assert_eq!(extract_version_from_cask(cask).unwrap(), "1.2.3");
        assert!(extract_version_from_cask("nope").is_err());
    }

    #[test]
    fn parse_version_and_compare() {
        assert_eq!(parse_version("1.2.3"), Some((1, 2, 3)));
        assert!(is_newer("2.0.0", "1.9.9").unwrap());
        assert!(is_newer("bad", "1.0.0").is_none());
    }

    #[test]
    fn url_overrides_work() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let _env = set_env_guard(
            LATEST_RELEASE_URL_OVERRIDE_ENV_VAR,
            Some("http://example.com"),
        );
        assert_eq!(latest_release_url(), "http://example.com");
    }

    #[test]
    fn fetch_versions_from_servers() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let release_body = "{\"tag_name\":\"v9.9.9\"}";
        let release_resp = http_ok_response(release_body, "application/json");
        let release_url = spawn_server(release_resp);
        {
            let _env = set_env_guard(LATEST_RELEASE_URL_OVERRIDE_ENV_VAR, Some(&release_url));
            assert_eq!(fetch_version_from_release().unwrap(), "9.9.9");
        }

        let cask_body = "version \"9.9.9\"";
        let cask_resp = http_ok_response(cask_body, "text/plain");
        let cask_url = spawn_server(cask_resp);
        {
            let _env = set_env_guard(HOMEBREW_CASK_URL_OVERRIDE_ENV_VAR, Some(&cask_url));
            assert_eq!(fetch_version_from_cask().unwrap(), "9.9.9");
        }
    }

    #[test]
    fn fetch_versions_handle_404() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let resp = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n".to_string();
        let url = spawn_server(resp);
        let _env = set_env_guard(LATEST_RELEASE_URL_OVERRIDE_ENV_VAR, Some(&url));
        assert!(fetch_version_from_release().is_none());
    }

    #[test]
    fn check_for_update_writes_version() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let release_body = "{\"tag_name\":\"v9.9.9\"}";
        let release_resp = http_ok_response(release_body, "application/json");
        let release_url = spawn_server(release_resp);
        let _env = set_env_guard(LATEST_RELEASE_URL_OVERRIDE_ENV_VAR, Some(&release_url));

        let dir = tempfile::tempdir().expect("tempdir");
        let version_file = dir.path().join("version.json");
        check_for_update_with_action(&version_file, None).unwrap();
        let contents = fs::read_to_string(&version_file).unwrap();
        assert!(contents.contains("9.9.9"));
    }

    #[test]
    fn updates_disabled_variants() {
        let config = UpdateConfig {
            codex_home: PathBuf::new(),
            check_for_update_on_startup: false,
        };
        assert!(updates_disabled_with_debug(&config, false));
        let config = UpdateConfig {
            codex_home: PathBuf::new(),
            check_for_update_on_startup: true,
        };
        assert!(updates_disabled_with_debug(&config, true));
    }

    #[test]
    fn run_update_prompt_paths() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let release_body = format!("{{\"tag_name\":\"v{}\"}}", "99.0.0");
        let release_resp = http_ok_response(&release_body, "application/json");
        let release_url = spawn_server(release_resp);
        let _env = set_env_guard(LATEST_RELEASE_URL_OVERRIDE_ENV_VAR, Some(&release_url));

        let dir = tempfile::tempdir().expect("tempdir");
        let config = UpdateConfig {
            codex_home: dir.path().to_path_buf(),
            check_for_update_on_startup: true,
        };
        seed_version_info(&config, "99.0.0");
        let mut input = std::io::Cursor::new("2\n");
        let mut output = Vec::new();
        let result = run_update_prompt_if_needed_with_io_and_source(
            &config,
            false,
            false,
            InstallSource::Npm,
            &mut input,
            &mut output,
        )
        .unwrap();
        assert!(matches!(result, UpdatePromptOutcome::Continue));

        let dir = tempfile::tempdir().expect("tempdir");
        let config = UpdateConfig {
            codex_home: dir.path().to_path_buf(),
            check_for_update_on_startup: true,
        };
        seed_version_info(&config, "99.0.0");
        let mut input = std::io::Cursor::new("1\n");
        let mut output = Vec::new();
        let result = run_update_prompt_if_needed_with_io_and_source(
            &config,
            false,
            true,
            InstallSource::Npm,
            &mut input,
            &mut output,
        )
        .unwrap();
        assert!(matches!(result, UpdatePromptOutcome::RunUpdate(_)));

        let dir = tempfile::tempdir().expect("tempdir");
        let config = UpdateConfig {
            codex_home: dir.path().to_path_buf(),
            check_for_update_on_startup: true,
        };
        seed_version_info(&config, "99.0.0");
        let mut input = std::io::Cursor::new("3\n");
        let mut output = Vec::new();
        let result = run_update_prompt_if_needed_with_io_and_source(
            &config,
            false,
            true,
            InstallSource::Npm,
            &mut input,
            &mut output,
        )
        .unwrap();
        assert!(matches!(result, UpdatePromptOutcome::Continue));
    }
}
