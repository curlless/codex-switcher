use clap::ValueEnum;
#[cfg(windows)]
use directories::BaseDirs;
use serde::{Deserialize, Serialize};
#[cfg(windows)]
use serde_json::Value;
#[cfg(windows)]
use std::fs;
#[cfg(windows)]
use std::os::windows::process::CommandExt;
#[cfg(windows)]
use std::path::Path;
use std::path::PathBuf;
#[cfg(windows)]
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdeReloadOutcome {
    pub attempted: bool,
    pub restarted: bool,
    pub message: String,
    pub manual_hints: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CodexAppOverride {
    pub executable_path: Option<PathBuf>,
    pub app_user_model_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodexAppDiscovery {
    pub executable_path: PathBuf,
    pub app_user_model_id: Option<String>,
    pub source: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReloadAppTarget {
    #[default]
    All,
    Codex,
    Cursor,
}

impl ReloadAppTarget {
    fn includes_codex(self) -> bool {
        matches!(self, Self::All | Self::Codex)
    }

    fn includes_cursor(self) -> bool {
        matches!(self, Self::All | Self::Cursor)
    }

    #[cfg(windows)]
    fn label(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Codex => "codex",
            Self::Cursor => "cursor",
        }
    }
}

pub fn reload_ide_best_effort() -> IdeReloadOutcome {
    reload_ide_target_best_effort(ReloadAppTarget::All)
}

pub fn reload_ide_target_best_effort(target: ReloadAppTarget) -> IdeReloadOutcome {
    reload_ide_target_best_effort_with_codex_override(target, None)
}

pub fn reload_ide_target_best_effort_with_codex_override(
    target: ReloadAppTarget,
    codex_override: Option<&CodexAppOverride>,
) -> IdeReloadOutcome {
    #[cfg(windows)]
    {
        reload_windows(false, target, codex_override)
    }
    #[cfg(not(windows))]
    {
        let _ = codex_override;
        IdeReloadOutcome {
            attempted: false,
            restarted: false,
            message: "IDE auto-reload is only implemented for Windows in this build.".to_string(),
            manual_hints: default_manual_hints(target),
        }
    }
}

pub fn inspect_ide_reload() -> IdeReloadOutcome {
    inspect_ide_reload_target(ReloadAppTarget::All)
}

pub fn inspect_ide_reload_target(target: ReloadAppTarget) -> IdeReloadOutcome {
    inspect_ide_reload_target_with_codex_override(target, None)
}

pub fn inspect_ide_reload_target_with_codex_override(
    target: ReloadAppTarget,
    codex_override: Option<&CodexAppOverride>,
) -> IdeReloadOutcome {
    #[cfg(windows)]
    {
        reload_windows(true, target, codex_override)
    }
    #[cfg(not(windows))]
    {
        let _ = codex_override;
        IdeReloadOutcome {
            attempted: false,
            restarted: false,
            message: "IDE auto-reload is only implemented for Windows in this build.".to_string(),
            manual_hints: default_manual_hints(target),
        }
    }
}

#[cfg(windows)]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct WindowsProcessInfo {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "ProcessId")]
    process_id: u32,
    #[serde(rename = "ExecutablePath")]
    executable_path: Option<String>,
}

#[cfg(windows)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct CursorProtocolAutomation {
    cursor_exe: PathBuf,
    uri: String,
}

#[cfg(windows)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct CodexAppLaunchTarget {
    executable_path: PathBuf,
    app_user_model_id: Option<String>,
    source: String,
}

#[cfg(windows)]
fn reload_windows(
    dry_run: bool,
    target: ReloadAppTarget,
    codex_override: Option<&CodexAppOverride>,
) -> IdeReloadOutcome {
    let processes = match list_reload_targets() {
        Ok(processes) => processes,
        Err(err) => {
            return IdeReloadOutcome {
                attempted: false,
                restarted: false,
                message: format!("Reload hint: failed to inspect running IDE processes ({err})."),
                manual_hints: default_manual_hints(target),
            };
        }
    };

    let cursor_detected = target.includes_cursor() && processes.iter().any(is_cursor_process);
    let extension_detected =
        target.includes_cursor() && processes.iter().any(is_cursor_extension_process);
    let cursor_automation = if cursor_detected || extension_detected {
        resolve_cursor_protocol_automation()
    } else {
        None
    };
    let standalone_app_pids: Vec<u32> = if target.includes_codex() {
        processes
            .iter()
            .filter(|process| is_standalone_codex_app_process(process))
            .map(|process| process.process_id)
            .collect()
    } else {
        Vec::new()
    };
    let codex_launch_target = if target.includes_codex() {
        resolve_codex_app_launch_target(&processes, codex_override)
    } else {
        None
    };

    let mut killed = Vec::new();
    let mut issues = Vec::new();
    let mut cursor_reload_dispatched = false;
    let mut codex_relaunched = false;
    let should_dispatch_cursor_reload =
        should_dispatch_cursor_reload(target, !standalone_app_pids.is_empty());

    if !dry_run {
        for pid in &standalone_app_pids {
            let output = hidden_console_command("taskkill")
                .args(["/PID", &pid.to_string(), "/F"])
                .output();
            match output {
                Ok(output) if output.status.success() => killed.push(*pid),
                Ok(output) => {
                    issues.push(format!("pid {pid}: taskkill exited with {}", output.status))
                }
                Err(err) => issues.push(format!("pid {pid}: failed to run taskkill ({err})")),
            }
        }
        if !killed.is_empty() {
            if let Some(launch_target) = codex_launch_target.as_ref() {
                match dispatch_codex_app_reload(launch_target) {
                    Ok(()) => codex_relaunched = true,
                    Err(err) => issues.push(format!("codex relaunch failed ({err})")),
                }
            } else {
                issues.push("codex relaunch failed (could not resolve launch target)".to_string());
            }
        }
        if should_dispatch_cursor_reload && let Some(automation) = cursor_automation.as_ref() {
            match dispatch_cursor_protocol_reload(automation) {
                Ok(()) => cursor_reload_dispatched = true,
                Err(err) => issues.push(format!("cursor protocol reload failed ({err})")),
            }
        }
    }

    let mut manual_hints = Vec::new();
    if cursor_detected || extension_detected {
        manual_hints.push(cursor_manual_hint().to_string());
        if cursor_automation.is_none() {
            manual_hints.push(cursor_helper_hint().to_string());
        }
    }
    if !standalone_app_pids.is_empty() {
        manual_hints.push(codex_app_manual_hint().to_string());
    }
    if manual_hints.is_empty() {
        manual_hints = default_manual_hints(target);
    }

    if dry_run {
        if !standalone_app_pids.is_empty() {
            let mut message = format!(
                "Reload hint: dry run detected standalone Codex app processes (PID {}).",
                join_pids(&standalone_app_pids)
            );
            if let Some(launch_target) = codex_launch_target.as_ref() {
                message.push_str(&format!(
                    " Relaunch target is available ({})",
                    launch_target.source
                ));
                message.push('.');
            } else {
                message.push_str(" Relaunch target could not be resolved.");
            }
            if cursor_detected || extension_detected {
                if let Some(automation) = cursor_automation.as_ref() {
                    message.push_str(&format!(
                        " Cursor protocol reload is available via {}.",
                        automation.uri
                    ));
                } else {
                    message.push_str(
                        " Cursor extension would still require a separate window reload.",
                    );
                }
            }
            return IdeReloadOutcome {
                attempted: false,
                restarted: false,
                message,
                manual_hints,
            };
        }

        if cursor_detected || extension_detected {
            return IdeReloadOutcome {
                attempted: false,
                restarted: false,
                message: if let Some(automation) = cursor_automation.as_ref() {
                    format!(
                        "Reload hint: dry run detected Cursor Codex extension; protocol reload is available via {}.",
                        automation.uri
                    )
                } else {
                    "Reload hint: dry run detected Cursor Codex extension; actual reload would still be manual via Reload Window."
                        .to_string()
                },
                manual_hints,
            };
        }
    }

    if !killed.is_empty() {
        let mut message = if codex_relaunched {
            format!(
                "Reload hint: restarted standalone Codex app (terminated PID {} and launched it again).",
                join_pids(&killed)
            )
        } else {
            format!(
                "Reload hint: terminated standalone Codex app processes (PID {}).",
                join_pids(&killed)
            )
        };
        if cursor_reload_dispatched {
            message.push_str(" Cursor reload was requested via protocol command path.");
        } else if cursor_detected || extension_detected {
            message.push_str(
                " Cursor stays running because extension reload requires a window reload.",
            );
        }
        return IdeReloadOutcome {
            attempted: true,
            restarted: true,
            message,
            manual_hints,
        };
    }

    if !issues.is_empty() {
        return IdeReloadOutcome {
            attempted: !standalone_app_pids.is_empty(),
            restarted: false,
            message: format!("Reload hint: {}", issues.join("; ")),
            manual_hints,
        };
    }

    if cursor_detected || extension_detected {
        return IdeReloadOutcome {
            attempted: cursor_reload_dispatched,
            restarted: cursor_reload_dispatched,
            message: if cursor_reload_dispatched {
                "Reload hint: requested Cursor Codex extension reload via protocol command path."
                    .to_string()
            } else {
                "Reload hint: detected Cursor Codex extension; automatic extension reload is not implemented."
                    .to_string()
            },
            manual_hints,
        };
    }

    if !standalone_app_pids.is_empty() {
        return IdeReloadOutcome {
            attempted: false,
            restarted: false,
            message:
                "Reload hint: detected standalone Codex app, but no process termination was needed."
                    .to_string(),
            manual_hints,
        };
    }

    IdeReloadOutcome {
        attempted: false,
        restarted: false,
        message: format!(
            "Reload hint: no supported Cursor/Codex Windows targets were detected for target '{}'.",
            target.label()
        ),
        manual_hints,
    }
}

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[cfg(windows)]
fn hidden_console_command(program: impl AsRef<std::ffi::OsStr>) -> Command {
    let mut command = Command::new(program);
    command.creation_flags(CREATE_NO_WINDOW);
    command
}

fn cursor_manual_hint() -> &'static str {
    "Cursor Codex extension: run Command Palette -> Developer: Reload Window."
}

fn codex_app_manual_hint() -> &'static str {
    "Codex app for Windows: if relaunch fails or account state still looks stale, open the app again manually."
}

fn cursor_helper_hint() -> &'static str {
    "Cursor automation: install the Commands Executor extension (ionutvmi.vscode-commands-executor) to enable protocol-based Reload Window."
}

#[cfg(windows)]
fn should_dispatch_cursor_reload(target: ReloadAppTarget, has_standalone_codex: bool) -> bool {
    !matches!(target, ReloadAppTarget::All) || !has_standalone_codex
}

fn default_manual_hints(target: ReloadAppTarget) -> Vec<String> {
    let mut hints = Vec::new();
    if target.includes_cursor() {
        hints.push(cursor_manual_hint().to_string());
        hints.push(cursor_helper_hint().to_string());
    }
    if target.includes_codex() {
        hints.push(codex_app_manual_hint().to_string());
    }
    hints
}

#[cfg(windows)]
fn join_pids(pids: &[u32]) -> String {
    pids.iter()
        .map(|pid| pid.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(windows)]
const CURSOR_PROTOCOL_SCHEME: &str = "cursor";
#[cfg(windows)]
const COMMANDS_EXECUTOR_EXTENSION_ID: &str = "ionutvmi.vscode-commands-executor";

#[cfg(windows)]
fn resolve_cursor_protocol_automation() -> Option<CursorProtocolAutomation> {
    let cursor_exe = resolve_cursor_exe_path()?;
    if !cursor_commands_executor_installed() {
        return None;
    }
    Some(CursorProtocolAutomation {
        cursor_exe,
        uri: build_cursor_reload_uri(),
    })
}

#[cfg(windows)]
fn dispatch_cursor_protocol_reload(automation: &CursorProtocolAutomation) -> Result<(), String> {
    let status = Command::new(&automation.cursor_exe)
        .args(["--open-url", "--", &automation.uri])
        .status()
        .map_err(|err| format!("failed to launch Cursor protocol handler ({err})"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("Cursor exited with status {status}"))
    }
}

#[cfg(windows)]
fn build_cursor_reload_uri() -> String {
    let command_payload = r#"[{"id":"workbench.action.reloadWindow"}]"#;
    format!(
        "{CURSOR_PROTOCOL_SCHEME}://{COMMANDS_EXECUTOR_EXTENSION_ID}/runCommands?data={}",
        percent_encode_query_value(command_payload)
    )
}

#[cfg(windows)]
fn percent_encode_query_value(raw: &str) -> String {
    let mut encoded = String::with_capacity(raw.len() * 3);
    for byte in raw.bytes() {
        let is_unreserved = matches!(
            byte,
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~'
        );
        if is_unreserved {
            encoded.push(byte as char);
        } else {
            encoded.push('%');
            encoded.push_str(&format!("{byte:02X}"));
        }
    }
    encoded
}

#[cfg(windows)]
fn cursor_commands_executor_installed() -> bool {
    cursor_extensions_dir()
        .and_then(|dir| {
            has_extension_with_prefix(&dir, COMMANDS_EXECUTOR_EXTENSION_ID).then_some(dir)
        })
        .is_some()
}

#[cfg(windows)]
fn cursor_extensions_dir() -> Option<PathBuf> {
    let home = BaseDirs::new()?.home_dir().to_path_buf();
    Some(home.join(".cursor").join("extensions"))
}

#[cfg(windows)]
fn has_extension_with_prefix(dir: &Path, prefix: &str) -> bool {
    fs::read_dir(dir)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.filter_map(Result::ok))
        .filter_map(|entry| entry.file_name().into_string().ok())
        .any(|name| name == prefix || name.starts_with(&format!("{prefix}-")))
}

#[cfg(windows)]
fn resolve_cursor_exe_path() -> Option<PathBuf> {
    local_cursor_exe_path()
        .filter(|path| path.is_file())
        .or_else(|| {
            query_cursor_protocol_command()
                .and_then(|command| parse_windows_command_exe(&command))
                .filter(|path| path.is_file())
        })
}

#[cfg(windows)]
fn local_cursor_exe_path() -> Option<PathBuf> {
    let local_app_data = std::env::var_os("LOCALAPPDATA")?;
    Some(
        PathBuf::from(local_app_data)
            .join("Programs")
            .join("Cursor")
            .join("Cursor.exe"),
    )
}

#[cfg(windows)]
fn query_cursor_protocol_command() -> Option<String> {
    let output = hidden_console_command("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "(Get-ItemProperty 'Registry::HKEY_CLASSES_ROOT\\cursor\\shell\\open\\command').'('(default)')'",
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!value.is_empty()).then_some(value)
}

#[cfg(windows)]
fn parse_windows_command_exe(raw: &str) -> Option<PathBuf> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(rest) = trimmed.strip_prefix('"') {
        let end = rest.find('"')?;
        return Some(PathBuf::from(&rest[..end]));
    }
    trimmed.split_whitespace().next().map(PathBuf::from)
}

#[cfg(windows)]
fn resolve_codex_app_launch_target(
    processes: &[WindowsProcessInfo],
    codex_override: Option<&CodexAppOverride>,
) -> Option<CodexAppLaunchTarget> {
    if let Some(launch_target) = codex_launch_target_from_override(codex_override) {
        return Some(launch_target);
    }
    if let Some(launch_target) =
        codex_launch_target_from_override(env_codex_app_override().as_ref())
    {
        return Some(launch_target);
    }
    let executable_path = processes
        .iter()
        .find_map(standalone_codex_executable_path)?;
    Some(CodexAppLaunchTarget {
        app_user_model_id: build_codex_app_user_model_id(&executable_path),
        executable_path,
        source: "running process".to_string(),
    })
}

#[cfg(windows)]
fn codex_launch_target_from_override(
    override_: Option<&CodexAppOverride>,
) -> Option<CodexAppLaunchTarget> {
    let override_ = override_?;
    let valid_path = override_
        .executable_path
        .as_ref()
        .filter(|path| path.is_file())
        .cloned();
    let (executable_path, app_user_model_id) = if let Some(executable_path) = valid_path {
        let app_user_model_id = override_
            .app_user_model_id
            .clone()
            .or_else(|| build_codex_app_user_model_id(&executable_path));
        (executable_path, app_user_model_id)
    } else if override_.app_user_model_id.is_some() {
        let local_target = local_codex_launch_target()?;
        let app_user_model_id = local_target
            .app_user_model_id
            .or_else(|| override_.app_user_model_id.clone());
        (local_target.executable_path, app_user_model_id)
    } else {
        return None;
    };
    Some(CodexAppLaunchTarget {
        executable_path,
        app_user_model_id,
        source: "override".to_string(),
    })
}

#[cfg(windows)]
fn env_codex_app_override() -> Option<CodexAppOverride> {
    let executable_path = std::env::var_os("CODEX_SWITCHER_CODEX_APP_PATH")
        .or_else(|| std::env::var_os("CODEX_PROFILES_CODEX_APP_PATH"))
        .filter(|value| !value.is_empty())
        .map(PathBuf::from);
    let app_user_model_id = std::env::var("CODEX_SWITCHER_CODEX_APP_AUMID")
        .ok()
        .or_else(|| std::env::var("CODEX_PROFILES_CODEX_APP_AUMID").ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    (executable_path.is_some() || app_user_model_id.is_some()).then_some(CodexAppOverride {
        executable_path,
        app_user_model_id,
    })
}

#[cfg(windows)]
fn standalone_codex_executable_path(process: &WindowsProcessInfo) -> Option<PathBuf> {
    if !is_standalone_codex_app_process(process) {
        return None;
    }
    process
        .executable_path
        .as_ref()
        .map(PathBuf::from)
        .filter(|path| {
            path.file_name()
                .is_some_and(|name| name.eq_ignore_ascii_case("Codex.exe"))
        })
}

#[cfg(windows)]
fn dispatch_codex_app_reload(launch_target: &CodexAppLaunchTarget) -> Result<(), String> {
    let mut errors = Vec::new();
    if let Some(app_user_model_id) = launch_target.app_user_model_id.as_deref() {
        let status = Command::new("explorer.exe")
            .arg(format!(r"shell:AppsFolder\{app_user_model_id}"))
            .status();
        match status {
            Ok(status) if status.success() => return Ok(()),
            Ok(status) => errors.push(format!("explorer.exe exited with status {status}")),
            Err(err) => errors.push(format!("failed to launch via AppsFolder ({err})")),
        }
    }

    Command::new(&launch_target.executable_path)
        .spawn()
        .map(|_| ())
        .map_err(|err| {
            errors.push(format!(
                "failed to launch executable {} ({err})",
                launch_target.executable_path.display()
            ));
            errors.join("; ")
        })
}

#[cfg(windows)]
fn build_codex_app_user_model_id(executable_path: &Path) -> Option<String> {
    let package_dir = codex_package_dir(executable_path)?;
    let package_dir_name = package_dir.file_name()?.to_str()?;
    let publisher_id = package_dir_name.split_once("__")?.1;
    let manifest_path = package_dir.join("AppxManifest.xml");
    let manifest = fs::read_to_string(manifest_path).ok()?;
    let identity_name = extract_xml_attribute(&manifest, "Identity", "Name")?;
    let application_id = extract_xml_attribute(&manifest, "Application", "Id")?;
    Some(format!("{identity_name}_{publisher_id}!{application_id}"))
}

#[cfg(windows)]
fn local_codex_launch_target() -> Option<CodexAppLaunchTarget> {
    query_codex_app_packages()
        .into_iter()
        .flatten()
        .find_map(|package| codex_launch_target_from_package(&package))
}

#[cfg(windows)]
fn codex_package_dir(executable_path: &Path) -> Option<&Path> {
    executable_path.ancestors().find(|ancestor| {
        ancestor
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("OpenAI.Codex_") && name.contains("__"))
    })
}

#[cfg(windows)]
fn extract_xml_attribute(xml: &str, tag: &str, attribute: &str) -> Option<String> {
    let open_tag = format!("<{tag} ");
    let start = xml.find(&open_tag)?;
    let fragment = &xml[start..xml[start..].find('>')? + start];
    let needle = format!(r#"{attribute}=""#);
    let attr_start = fragment.find(&needle)? + needle.len();
    let rest = &fragment[attr_start..];
    let attr_end = rest.find('"')?;
    Some(rest[..attr_end].to_string())
}

#[cfg(windows)]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct AppxPackageInfo {
    #[serde(rename = "PackageFamilyName")]
    package_family_name: Option<String>,
    #[serde(rename = "InstallLocation")]
    install_location: Option<String>,
}

#[cfg(windows)]
pub fn detect_codex_app_discovery(
    codex_override: Option<&CodexAppOverride>,
) -> Result<CodexAppDiscovery, String> {
    let processes = list_reload_targets()?;
    if let Some(target) = resolve_codex_app_launch_target(&processes, codex_override) {
        return Ok(CodexAppDiscovery {
            executable_path: target.executable_path,
            app_user_model_id: target.app_user_model_id,
            source: target.source,
        });
    }
    if let Some(target) = query_codex_app_packages()
        .map_err(|err| format!("failed to query installed Codex app ({err})"))?
        .into_iter()
        .find_map(|package| codex_launch_target_from_package(&package))
    {
        return Ok(CodexAppDiscovery {
            executable_path: target.executable_path,
            app_user_model_id: target.app_user_model_id,
            source: target.source,
        });
    }
    Err("could not locate Codex app installation".to_string())
}

#[cfg(not(windows))]
pub fn detect_codex_app_discovery(
    _codex_override: Option<&CodexAppOverride>,
) -> Result<CodexAppDiscovery, String> {
    Err("Codex app detection is only implemented for Windows in this build.".to_string())
}

#[cfg(windows)]
fn query_codex_app_packages() -> Result<Vec<AppxPackageInfo>, String> {
    let output = hidden_console_command("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "Get-AppxPackage -Name 'OpenAI.Codex' | Select-Object PackageFamilyName, InstallLocation | ConvertTo-Json -Compress",
        ])
        .output()
        .map_err(|err| format!("failed to run Get-AppxPackage ({err})"))?;
    if !output.status.success() {
        return Err(format!("PowerShell exited with {}", output.status));
    }
    parse_appx_packages(&String::from_utf8_lossy(&output.stdout))
}

#[cfg(windows)]
fn parse_appx_packages(raw: &str) -> Result<Vec<AppxPackageInfo>, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    let value: Value = serde_json::from_str(trimmed)
        .map_err(|err| format!("failed to parse appx JSON ({err})"))?;
    match value {
        Value::Null => Ok(Vec::new()),
        Value::Array(_) => serde_json::from_value(value)
            .map_err(|err| format!("failed to decode appx package list ({err})")),
        Value::Object(_) => serde_json::from_value(value)
            .map(|package| vec![package])
            .map_err(|err| format!("failed to decode appx package entry ({err})")),
        _ => Err("unexpected JSON shape from appx query".to_string()),
    }
}

#[cfg(windows)]
fn codex_launch_target_from_package(package: &AppxPackageInfo) -> Option<CodexAppLaunchTarget> {
    let install_location = package
        .install_location
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)?;
    let executable_path = install_location.join("app").join("Codex.exe");
    let app_user_model_id = package
        .package_family_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|family| format!("{family}!App"))
        .or_else(|| build_codex_app_user_model_id(&executable_path));
    Some(CodexAppLaunchTarget {
        executable_path,
        app_user_model_id,
        source: "Get-AppxPackage".to_string(),
    })
}

#[cfg(windows)]
fn list_reload_targets() -> Result<Vec<WindowsProcessInfo>, String> {
    let output = hidden_console_command("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            windows_process_query(),
        ])
        .output()
        .map_err(|err| format!("failed to run PowerShell process query ({err})"))?;
    if !output.status.success() {
        return Err(format!("PowerShell exited with {}", output.status));
    }
    parse_windows_processes(&String::from_utf8_lossy(&output.stdout))
}

#[cfg(windows)]
fn windows_process_query() -> &'static str {
    "$ErrorActionPreference = 'Stop'; \
     Get-CimInstance Win32_Process | \
     Where-Object { \
         $_.Name -match '^(?i:cursor\\.exe|codex\\.exe|codex-app-server\\.exe)$' -or \
         $_.ExecutablePath -like 'C:\\Program Files\\WindowsApps\\OpenAI.Codex_*' -or \
         $_.ExecutablePath -like '*openai.chatgpt-*' \
     } | \
     Select-Object Name, ProcessId, ExecutablePath | \
     ConvertTo-Json -Compress"
}

#[cfg(windows)]
fn parse_windows_processes(raw: &str) -> Result<Vec<WindowsProcessInfo>, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    let value: Value = serde_json::from_str(trimmed)
        .map_err(|err| format!("failed to parse process JSON ({err})"))?;

    match value {
        Value::Null => Ok(Vec::new()),
        Value::Array(_) => serde_json::from_value(value)
            .map_err(|err| format!("failed to decode process list ({err})")),
        Value::Object(_) => serde_json::from_value(value)
            .map(|process| vec![process])
            .map_err(|err| format!("failed to decode process entry ({err})")),
        _ => Err("unexpected JSON shape from process query".to_string()),
    }
}

#[cfg(windows)]
fn is_cursor_process(process: &WindowsProcessInfo) -> bool {
    process.name.eq_ignore_ascii_case("cursor.exe")
}

#[cfg(windows)]
fn is_cursor_extension_process(process: &WindowsProcessInfo) -> bool {
    normalized_path(process)
        .as_deref()
        .is_some_and(|path| path.contains("\\openai.chatgpt-"))
}

#[cfg(windows)]
fn is_standalone_codex_app_process(process: &WindowsProcessInfo) -> bool {
    normalized_path(process).as_deref().is_some_and(|path| {
        path.contains("\\windowsapps\\openai.codex_") && path.ends_with("\\app\\codex.exe")
    })
}

#[cfg(windows)]
fn normalized_path(process: &WindowsProcessInfo) -> Option<String> {
    process
        .executable_path
        .as_ref()
        .map(|path| path.replace('/', "\\").to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    #[cfg(windows)]
    use super::ReloadAppTarget;
    #[cfg(windows)]
    use super::{
        AppxPackageInfo, CodexAppOverride, WindowsProcessInfo, build_codex_app_user_model_id,
        build_cursor_reload_uri, codex_launch_target_from_package, extract_xml_attribute,
        has_extension_with_prefix, is_cursor_extension_process, is_standalone_codex_app_process,
        parse_appx_packages, parse_windows_command_exe, parse_windows_processes,
        should_dispatch_cursor_reload,
    };
    #[cfg(windows)]
    use std::fs;
    #[cfg(windows)]
    use std::path::PathBuf;

    #[cfg(windows)]
    #[test]
    fn parse_windows_processes_accepts_array() {
        let raw = r#"[{"Name":"Cursor.exe","ProcessId":101,"ExecutablePath":"C:\\Users\\tompski\\AppData\\Local\\Programs\\Cursor\\Cursor.exe"},{"Name":"Codex.exe","ProcessId":202,"ExecutablePath":"C:\\Program Files\\WindowsApps\\OpenAI.Codex_1\\app\\Codex.exe"}]"#;
        let processes = parse_windows_processes(raw).expect("parse array");
        assert_eq!(processes.len(), 2);
        assert_eq!(processes[0].name, "Cursor.exe");
        assert_eq!(processes[1].process_id, 202);
    }

    #[cfg(windows)]
    #[test]
    fn parse_windows_processes_accepts_single_object() {
        let raw = r#"{"Name":"codex.exe","ProcessId":303,"ExecutablePath":"C:\\Users\\tompski\\.cursor\\extensions\\openai.chatgpt-1\\bin\\windows-x86_64\\codex.exe"}"#;
        let processes = parse_windows_processes(raw).expect("parse object");
        assert_eq!(processes.len(), 1);
        assert_eq!(processes[0].name, "codex.exe");
        assert_eq!(processes[0].process_id, 303);
    }

    #[cfg(windows)]
    #[test]
    fn classification_distinguishes_extension_and_standalone_app() {
        let extension = WindowsProcessInfo {
            name: "codex.exe".to_string(),
            process_id: 1,
            executable_path: Some(
                "C:\\Users\\tompski\\.cursor\\extensions\\openai.chatgpt-1\\bin\\windows-x86_64\\codex.exe"
                    .to_string(),
            ),
        };
        let app = WindowsProcessInfo {
            name: "Codex.exe".to_string(),
            process_id: 2,
            executable_path: Some(
                "C:\\Program Files\\WindowsApps\\OpenAI.Codex_1\\app\\Codex.exe".to_string(),
            ),
        };

        assert!(is_cursor_extension_process(&extension));
        assert!(!is_standalone_codex_app_process(&extension));
        assert!(is_standalone_codex_app_process(&app));
        assert!(!is_cursor_extension_process(&app));
    }

    #[cfg(windows)]
    #[test]
    fn build_codex_app_user_model_id_reads_manifest() {
        let dir = tempfile::tempdir().expect("tempdir");
        let package_dir = dir
            .path()
            .join("OpenAI.Codex_26.304.1528.0_x64__2p2nqsd0c76g0");
        fs::create_dir_all(package_dir.join("app")).expect("package dir");
        fs::write(
            package_dir.join("AppxManifest.xml"),
            r#"<?xml version="1.0"?><Package><Identity Name="OpenAI.Codex" /><Applications><Application Id="App" Executable="app\Codex.exe" /></Applications></Package>"#,
        )
        .expect("manifest");
        fs::write(package_dir.join("app").join("Codex.exe"), "").expect("exe");
        let aumid = build_codex_app_user_model_id(&package_dir.join("app").join("Codex.exe"))
            .expect("aumid");
        assert_eq!(aumid, "OpenAI.Codex_2p2nqsd0c76g0!App");
    }

    #[cfg(windows)]
    #[test]
    fn extract_xml_attribute_reads_expected_value() {
        let xml = r#"<Package><Identity Name="OpenAI.Codex" /><Applications><Application Id="App" /></Applications></Package>"#;
        assert_eq!(
            extract_xml_attribute(xml, "Identity", "Name").as_deref(),
            Some("OpenAI.Codex")
        );
        assert_eq!(
            extract_xml_attribute(xml, "Application", "Id").as_deref(),
            Some("App")
        );
    }

    #[cfg(windows)]
    #[test]
    fn parse_appx_packages_accepts_single_object() {
        let raw = r#"{"PackageFamilyName":"OpenAI.Codex_2p2nqsd0c76g0","InstallLocation":"C:\\Program Files\\WindowsApps\\OpenAI.Codex_26.304.1528.0_x64__2p2nqsd0c76g0"}"#;
        let packages = parse_appx_packages(raw).expect("packages");
        assert_eq!(packages.len(), 1);
        assert_eq!(
            packages[0].package_family_name.as_deref(),
            Some("OpenAI.Codex_2p2nqsd0c76g0")
        );
    }

    #[cfg(windows)]
    #[test]
    fn codex_launch_target_from_package_uses_package_family_name() {
        let package = AppxPackageInfo {
            package_family_name: Some("OpenAI.Codex_2p2nqsd0c76g0".to_string()),
            install_location: Some(
                "C:\\Program Files\\WindowsApps\\OpenAI.Codex_26.304.1528.0_x64__2p2nqsd0c76g0"
                    .to_string(),
            ),
        };
        let target = codex_launch_target_from_package(&package).expect("target");
        assert_eq!(
            target.app_user_model_id.as_deref(),
            Some("OpenAI.Codex_2p2nqsd0c76g0!App")
        );
        assert!(
            target
                .executable_path
                .to_string_lossy()
                .ends_with("app\\Codex.exe")
        );
    }

    #[cfg(windows)]
    #[test]
    fn override_can_supply_codex_path_without_running_process() {
        let dir = tempfile::tempdir().expect("tempdir");
        let exe = dir.path().join("Codex.exe");
        fs::write(&exe, "").expect("exe");
        let override_ = CodexAppOverride {
            executable_path: Some(exe.clone()),
            app_user_model_id: Some("OpenAI.Codex_test!App".to_string()),
        };
        let target = super::codex_launch_target_from_override(Some(&override_)).expect("target");
        assert_eq!(target.executable_path, exe);
        assert_eq!(
            target.app_user_model_id.as_deref(),
            Some("OpenAI.Codex_test!App")
        );
    }

    #[cfg(windows)]
    #[test]
    fn build_cursor_reload_uri_targets_reload_window_command() {
        let uri = build_cursor_reload_uri();
        assert!(uri.starts_with("cursor://ionutvmi.vscode-commands-executor/runCommands?data="));
        assert!(uri.contains("workbench.action.reloadWindow"));
        assert!(uri.contains("%5B%7B"));
    }

    #[cfg(windows)]
    #[test]
    fn has_extension_with_prefix_matches_versioned_directory() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::create_dir_all(dir.path().join("ionutvmi.vscode-commands-executor-1.0.0"))
            .expect("create extension dir");
        assert!(has_extension_with_prefix(
            dir.path(),
            "ionutvmi.vscode-commands-executor"
        ));
    }

    #[cfg(windows)]
    #[test]
    fn parse_windows_command_exe_reads_quoted_command() {
        let path = parse_windows_command_exe(
            r#""C:\Users\tompski\AppData\Local\Programs\Cursor\Cursor.exe" "--open-url" "--" "%1""#,
        )
        .expect("parse exe");
        assert_eq!(
            path,
            PathBuf::from(r"C:\Users\tompski\AppData\Local\Programs\Cursor\Cursor.exe")
        );
    }

    #[cfg(windows)]
    #[test]
    fn all_target_prioritizes_codex_over_cursor_reload() {
        assert!(!should_dispatch_cursor_reload(ReloadAppTarget::All, true));
        assert!(should_dispatch_cursor_reload(ReloadAppTarget::All, false));
        assert!(should_dispatch_cursor_reload(ReloadAppTarget::Codex, true));
        assert!(should_dispatch_cursor_reload(ReloadAppTarget::Cursor, true));
    }
}
