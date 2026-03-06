#[cfg(windows)]
use serde::Deserialize;
#[cfg(windows)]
use serde_json::Value;
#[cfg(windows)]
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdeReloadOutcome {
    pub attempted: bool,
    pub restarted: bool,
    pub message: String,
    pub manual_hints: Vec<String>,
}

pub fn reload_ide_best_effort() -> IdeReloadOutcome {
    #[cfg(windows)]
    {
        reload_windows(false)
    }
    #[cfg(not(windows))]
    {
        IdeReloadOutcome {
            attempted: false,
            restarted: false,
            message: "IDE auto-reload is only implemented for Windows in this build.".to_string(),
            manual_hints: vec![
                "Cursor Codex extension: run Command Palette -> Developer: Reload Window."
                    .to_string(),
            ],
        }
    }
}

pub fn inspect_ide_reload() -> IdeReloadOutcome {
    #[cfg(windows)]
    {
        reload_windows(true)
    }
    #[cfg(not(windows))]
    {
        IdeReloadOutcome {
            attempted: false,
            restarted: false,
            message: "IDE auto-reload is only implemented for Windows in this build.".to_string(),
            manual_hints: vec![
                "Cursor Codex extension: run Command Palette -> Developer: Reload Window."
                    .to_string(),
            ],
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
fn reload_windows(dry_run: bool) -> IdeReloadOutcome {
    let processes = match list_reload_targets() {
        Ok(processes) => processes,
        Err(err) => {
            return IdeReloadOutcome {
                attempted: false,
                restarted: false,
                message: format!("Reload hint: failed to inspect running IDE processes ({err})."),
                manual_hints: default_manual_hints(),
            };
        }
    };

    let cursor_detected = processes.iter().any(is_cursor_process);
    let extension_detected = processes.iter().any(is_cursor_extension_process);
    let standalone_app_pids: Vec<u32> = processes
        .iter()
        .filter(|process| is_standalone_codex_app_process(process))
        .map(|process| process.process_id)
        .collect();

    let mut killed = Vec::new();
    let mut issues = Vec::new();

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
    }

    let mut manual_hints = Vec::new();
    if cursor_detected || extension_detected {
        manual_hints.push(cursor_manual_hint().to_string());
    }
    if !standalone_app_pids.is_empty() {
        manual_hints.push(codex_app_manual_hint().to_string());
    }
    if manual_hints.is_empty() {
        manual_hints = default_manual_hints();
    }

    if dry_run {
        if !standalone_app_pids.is_empty() {
            let mut message = format!(
                "Reload hint: dry run detected standalone Codex app processes (PID {}).",
                join_pids(&standalone_app_pids)
            );
            if cursor_detected || extension_detected {
                message.push_str(" Cursor extension would still require a separate window reload.");
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
                message: "Reload hint: dry run detected Cursor Codex extension; actual reload would still be manual via Reload Window."
                    .to_string(),
                manual_hints,
            };
        }
    }

    if !killed.is_empty() {
        let mut message = format!(
            "Reload hint: terminated standalone Codex app processes (PID {}).",
            join_pids(&killed)
        );
        if cursor_detected || extension_detected {
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
            attempted: false,
            restarted: false,
            message: "Reload hint: detected Cursor Codex extension; automatic extension reload is not implemented."
                .to_string(),
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
        message: "Reload hint: no supported Cursor/Codex Windows targets were detected."
            .to_string(),
        manual_hints,
    }
}

#[cfg(windows)]
fn cursor_manual_hint() -> &'static str {
    "Cursor Codex extension: run Command Palette -> Developer: Reload Window."
}

#[cfg(windows)]
fn codex_app_manual_hint() -> &'static str {
    "Codex app for Windows: if account state still looks stale, close and reopen the app after the switch."
}

#[cfg(windows)]
fn default_manual_hints() -> Vec<String> {
    vec![
        cursor_manual_hint().to_string(),
        codex_app_manual_hint().to_string(),
    ]
}

#[cfg(windows)]
fn join_pids(pids: &[u32]) -> String {
    pids.iter()
        .map(|pid| pid.to_string())
        .collect::<Vec<_>>()
        .join(", ")
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
        WindowsProcessInfo, is_cursor_extension_process, is_standalone_codex_app_process,
        parse_windows_processes,
    };

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
}
