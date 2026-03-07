use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileCard {
    label: String,
    plan: String,
    reserved: bool,
    status: String,
    seven_day_remaining: String,
    five_hour_remaining: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfilesOverviewPayload {
    workspace_label: String,
    profiles: Vec<ProfileCard>,
    events: Vec<String>,
    last_refresh: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveProfileStatusPayload {
    active_profile: String,
    summary: String,
    reserved_profiles: usize,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwitchPreviewRequest {
    profile_label: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionNotice {
    title: String,
    detail: String,
    status: String,
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

fn sample_profiles() -> Vec<ProfileCard> {
    vec![
        ProfileCard {
            label: "work-pro".to_string(),
            plan: "ChatGPT Pro".to_string(),
            reserved: false,
            status: "active".to_string(),
            seven_day_remaining: "74%".to_string(),
            five_hour_remaining: "61%".to_string(),
        },
        ProfileCard {
            label: "openclaw-raymond".to_string(),
            plan: "Team".to_string(),
            reserved: true,
            status: "reserved".to_string(),
            seven_day_remaining: "93%".to_string(),
            five_hour_remaining: "80%".to_string(),
        },
        ProfileCard {
            label: "night-shift".to_string(),
            plan: "Plus".to_string(),
            reserved: false,
            status: "available".to_string(),
            seven_day_remaining: "41%".to_string(),
            five_hour_remaining: "34%".to_string(),
        },
    ]
}

#[tauri::command]
pub fn desktop_profiles_overview() -> DesktopCommandResult<ProfilesOverviewPayload> {
    DesktopCommandResult::ok(ProfilesOverviewPayload {
        workspace_label: "Desktop shell scaffold".to_string(),
        profiles: sample_profiles(),
        events: vec![
            "Desktop shell scaffold ready".to_string(),
            "Native command bridge returns typed placeholder data".to_string(),
            "Next step is extracting shared Rust services from CLI-shaped flows".to_string(),
        ],
        last_refresh: "2026-03-07 12:25 +05".to_string(),
    })
}

#[tauri::command]
pub fn desktop_active_profile_status() -> DesktopCommandResult<ActiveProfileStatusPayload> {
    DesktopCommandResult::ok(ActiveProfileStatusPayload {
        active_profile: "work-pro".to_string(),
        summary: "Pro profile is active and the desktop shell is only consuming typed placeholder data."
            .to_string(),
        reserved_profiles: 1,
    })
}

#[tauri::command]
pub fn desktop_switch_preview(
    request: SwitchPreviewRequest,
) -> DesktopCommandResult<ActionNotice> {
    if request.profile_label.trim().is_empty() {
        return DesktopCommandResult::err(
            "missing-profile-selection",
            "Choose a profile before requesting a switch preview.",
            true,
        );
    }

    DesktopCommandResult::ok(ActionNotice {
        title: "Preview switch".to_string(),
        detail: format!(
            "The desktop bridge would validate a switch from work-pro to {} without leaking terminal strings into the UI.",
            request.profile_label
        ),
        status: "placeholder".to_string(),
    })
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
        last_reloaded: "No reload issued in this scaffold".to_string(),
    })
}

#[tauri::command]
pub fn desktop_reload_target(
    request: ReloadTargetRequest,
) -> DesktopCommandResult<ActionNotice> {
    let detail = match request.target.as_str() {
        "codex" => "Codex reload stays behind a narrow desktop command boundary for now.",
        "cursor" => "Cursor reload stays behind the Rust bridge until service extraction lands.",
        _ => {
            return DesktopCommandResult::err(
                "unknown-reload-target",
                "The requested reload target is not part of the approved desktop bootstrap surface.",
                false,
            )
        }
    };

    DesktopCommandResult::ok(ActionNotice {
        title: format!("Reload {}", request.target),
        detail: detail.to_string(),
        status: "placeholder".to_string(),
    })
}
