use codex_switcher::switcher::{
    ActiveProfileStatusPayload, ProfilesOverviewPayload, ReloadAppTarget,
    ReloadOutcomePayload, SwitchPreviewPayload, active_profile_status, ensure_paths,
    execute_reload_outcome, profiles_overview, resolve_paths, switch_preview,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwitchPreviewRequest {
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

#[tauri::command]
pub fn desktop_profiles_overview() -> DesktopCommandResult<ProfilesOverviewPayload> {
    match switcher_paths().and_then(|paths| profiles_overview(&paths)) {
        Ok(payload) => DesktopCommandResult::ok(payload),
        Err(message) => {
            DesktopCommandResult::err("switcher-query-failed", &message, true)
        }
    }
}

#[tauri::command]
pub fn desktop_active_profile_status() -> DesktopCommandResult<ActiveProfileStatusPayload> {
    match switcher_paths().and_then(|paths| active_profile_status(&paths)) {
        Ok(payload) => DesktopCommandResult::ok(payload),
        Err(message) => {
            DesktopCommandResult::err("switcher-query-failed", &message, true)
        }
    }
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
