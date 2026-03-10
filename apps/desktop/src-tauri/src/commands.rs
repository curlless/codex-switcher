use codex_switcher::switcher::{
    ActiveProfileStatusPayload, ProfilesOverviewPayload, ReloadAppTarget,
    ReloadOutcomePayload, SwitchExecutionPayload, SwitchPreviewPayload,
    active_profile_status, ensure_paths, execute_best_switch, execute_reload_outcome,
    execute_switch, profiles_overview, resolve_paths, switch_preview, write_atomic,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwitchPreviewRequest {
    profile_label: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwitchExecuteRequest {
    profile_label: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReloadTargetInfo {
    id: String,
    label: String,
    description: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReloadTargetsPayload {
    targets: Vec<ReloadTargetInfo>,
    last_reloaded: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReloadTargetRequest {
    target: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SmokeTraceSnapshot {
    phase: String,
    view: String,
    active_profile: Option<String>,
    selected_label: Option<String>,
    profile_count: usize,
    refresh_count: usize,
    event: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopCommandError {
    code: String,
    message: String,
    retryable: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandSuccess<T> {
    ok: bool,
    data: T,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandFailure {
    ok: bool,
    error: DesktopCommandError,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum DesktopCommandResult<T> {
    Ok(CommandSuccess<T>),
    Err(CommandFailure),
}

impl<T> DesktopCommandResult<T> {
    fn ok(data: T) -> Self {
        Self::Ok(CommandSuccess { ok: true, data })
    }

    fn err(code: &str, message: &str, retryable: bool) -> Self {
        Self::Err(CommandFailure {
            ok: false,
            error: DesktopCommandError {
                code: code.to_string(),
                message: message.to_string(),
                retryable,
            },
        })
    }
}

fn switcher_paths() -> Result<codex_switcher::switcher::Paths, String> {
    let paths = resolve_paths()?;
    ensure_paths(&paths)?;
    Ok(paths)
}

fn smoke_trace_path(paths: &codex_switcher::switcher::Paths) -> PathBuf {
    paths.codex.join("desktop-smoke-trace.json")
}

#[tauri::command]
pub async fn desktop_profiles_overview() -> DesktopCommandResult<ProfilesOverviewPayload> {
    tauri::async_runtime::spawn_blocking(|| {
        match switcher_paths().and_then(|paths| profiles_overview(&paths)) {
            Ok(payload) => DesktopCommandResult::ok(payload),
            Err(message) => {
                DesktopCommandResult::err("switcher-query-failed", &message, true)
            }
        }
    })
    .await
    .unwrap_or_else(|error| {
        DesktopCommandResult::err(
            "switcher-query-failed",
            &format!("failed to join desktop profiles query: {error}"),
            true,
        )
    })
}

#[tauri::command]
pub async fn desktop_active_profile_status() -> DesktopCommandResult<ActiveProfileStatusPayload> {
    tauri::async_runtime::spawn_blocking(|| {
        match switcher_paths().and_then(|paths| active_profile_status(&paths)) {
            Ok(payload) => DesktopCommandResult::ok(payload),
            Err(message) => {
                DesktopCommandResult::err("switcher-query-failed", &message, true)
            }
        }
    })
    .await
    .unwrap_or_else(|error| {
        DesktopCommandResult::err(
            "switcher-query-failed",
            &format!("failed to join desktop active-profile query: {error}"),
            true,
        )
    })
}

#[tauri::command]
pub fn desktop_switch_preview(
    request: SwitchPreviewRequest,
) -> DesktopCommandResult<SwitchPreviewPayload> {
    if request.profile_label.trim().is_empty() {
        return DesktopCommandResult::err(
            "missing-profile-selection",
            "Choose a profile before requesting a switch preview.",
            true,
        );
    }

    match switcher_paths().and_then(|paths| switch_preview(&paths, &request.profile_label)) {
        Ok(payload) => DesktopCommandResult::ok(payload),
        Err(message) => {
            DesktopCommandResult::err("switcher-preview-failed", &message, true)
        }
    }
}

#[tauri::command]
pub fn desktop_switch_execute(
    request: SwitchExecuteRequest,
) -> DesktopCommandResult<SwitchExecutionPayload> {
    if request.profile_label.trim().is_empty() {
        return DesktopCommandResult::err(
            "missing-profile-selection",
            "Choose a profile before executing a switch.",
            true,
        );
    }

    match switcher_paths().and_then(|paths| execute_switch(&paths, &request.profile_label, None)) {
        Ok(payload) => DesktopCommandResult::ok(payload),
        Err(message) => {
            DesktopCommandResult::err("switcher-execute-failed", &message, true)
        }
    }
}

#[tauri::command]
pub fn desktop_switch_best_execute() -> DesktopCommandResult<SwitchExecutionPayload> {
    match switcher_paths().and_then(|paths| execute_best_switch(&paths, None)) {
        Ok(payload) => DesktopCommandResult::ok(payload),
        Err(message) => {
            DesktopCommandResult::err("switcher-execute-failed", &message, true)
        }
    }
}

#[tauri::command]
pub fn desktop_smoke_mode() -> bool {
    std::env::var("CODEX_SWITCHER_SMOKE_AUTOMATION")
        .map(|value| {
            let normalized = value.trim().to_ascii_lowercase();
            normalized == "1" || normalized == "true" || normalized == "yes"
        })
        .unwrap_or(false)
}

#[tauri::command]
pub fn desktop_record_smoke_trace(snapshot: SmokeTraceSnapshot) -> DesktopCommandResult<bool> {
    if !desktop_smoke_mode() {
        return DesktopCommandResult::ok(false);
    }

    match switcher_paths().and_then(|paths| {
        let trace_path = smoke_trace_path(&paths);
        let payload = serde_json::to_vec_pretty(&snapshot)
            .map_err(|error| format!("failed to serialize smoke trace: {error}"))?;
        write_atomic(&trace_path, &payload)?;
        Ok(true)
    }) {
        Ok(written) => DesktopCommandResult::ok(written),
        Err(message) => {
            DesktopCommandResult::err("smoke-trace-write-failed", &message, true)
        }
    }
}

#[tauri::command]
pub fn desktop_reload_targets() -> DesktopCommandResult<ReloadTargetsPayload> {
    DesktopCommandResult::ok(ReloadTargetsPayload {
        targets: vec![
            ReloadTargetInfo {
                id: "codex".to_string(),
                label: "Reload Codex".to_string(),
                description:
                    "Refresh the Codex desktop session after an account switch.".to_string(),
            },
            ReloadTargetInfo {
                id: "cursor".to_string(),
                label: "Reload Cursor".to_string(),
                description:
                    "Refresh Cursor when the bootstrap shell updates editor-side auth.".to_string(),
            },
        ],
        last_reloaded: "Shared Rust reload services are ready.".to_string(),
    })
}

#[tauri::command]
pub fn desktop_reload_target(
    request: ReloadTargetRequest,
) -> DesktopCommandResult<ReloadOutcomePayload> {
    let target = match request.target.as_str() {
        "codex" => ReloadAppTarget::Codex,
        "cursor" => ReloadAppTarget::Cursor,
        _ => {
            return DesktopCommandResult::err(
                "unknown-reload-target",
                "The requested reload target is not part of the approved desktop bootstrap surface.",
                false,
            );
        }
    };

    match switcher_paths().and_then(|paths| execute_reload_outcome(&paths, target)) {
        Ok(payload) => DesktopCommandResult::ok(payload),
        Err(message) => {
            DesktopCommandResult::err("reload-target-failed", &message, true)
        }
    }
}
