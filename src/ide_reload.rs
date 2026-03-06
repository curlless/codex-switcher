use clap::ValueEnum;
#[cfg(windows)]
use directories::BaseDirs;
use serde::{Deserialize, Serialize};
#[cfg(windows)]
use serde_json::Value;
#[cfg(windows)]
use std::fs;
#[cfg(windows)]
use std::path::{Path, PathBuf};
#[cfg(windows)]
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdeReloadOutcome {
    pub attempted: bool,
    pub restarted: bool,
    pub message: String,
    pub manual_hints: Vec<String>,
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
    #[cfg(windows)]
    {
        reload_windows(false, target)
    }
    #[cfg(not(windows))]
    {
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
    #[cfg(windows)]
    {
        reload_windows(true, target)
    }
    #[cfg(not(windows))]
    {
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
fn reload_windows(dry_run: bool, target: ReloadAppTarget) -> IdeReloadOutcome {
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

    let mut killed = Vec::new();
    let mut issues = Vec::new();
    let mut cursor_reload_dispatched = false;
    let should_dispatch_cursor_reload =
        should_dispatch_cursor_reload(target, !standalone_app_pids.is_empty());

    if !dry_run {
        for pid in &standalone_app_pids {
            let output = Command::new("taskkill")
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
        if should_dispatch_cursor_reload {
            if let Some(automation) = cursor_automation.as_ref() {
                match dispatch_cursor_protocol_reload(automation) {
                    Ok(()) => cursor_reload_dispatched = true,
                    Err(err) => issues.push(format!("cursor protocol reload failed ({err})")),
                }
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
        let mut message = format!(
            "Reload hint: terminated standalone Codex app processes (PID {}).",
            join_pids(&killed)
        );
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

fn cursor_manual_hint() -> &'static str {
    "Cursor Codex extension: run Command Palette -> Developer: Reload Window."
}

fn codex_app_manual_hint() -> &'static str {
    "Codex app for Windows: if account state still looks stale, close and reopen the app after the switch."
}

fn cursor_helper_hint() -> &'static str {
    "Cursor automation: install the Commands Executor extension (ionutvmi.vscode-commands-executor) to enable protocol-based Reload Window."
}

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
    let output = Command::new("powershell")
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
fn list_reload_targets() -> Result<Vec<WindowsProcessInfo>, String> {
    let output = Command::new("powershell")
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
    normalized_path(process)
        .as_deref()
        .is_some_and(|path| path.contains("\\windowsapps\\openai.codex_"))
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
    use super::{
        ReloadAppTarget, WindowsProcessInfo, build_cursor_reload_uri, has_extension_with_prefix,
        is_cursor_extension_process, is_standalone_codex_app_process, parse_windows_command_exe,
        parse_windows_processes, should_dispatch_cursor_reload,
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

    #[test]
    fn all_target_prioritizes_codex_over_cursor_reload() {
        assert!(!should_dispatch_cursor_reload(ReloadAppTarget::All, true));
        assert!(should_dispatch_cursor_reload(ReloadAppTarget::All, false));
        assert!(should_dispatch_cursor_reload(ReloadAppTarget::Codex, true));
        assert!(should_dispatch_cursor_reload(ReloadAppTarget::Cursor, true));
    }
}
