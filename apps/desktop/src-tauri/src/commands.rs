use codex_switcher::switcher::{
    ActiveProfileStatusPayload, AvailabilityPayload, ProfileCard, ProfilesOverviewPayload,
    ReloadAppTarget, ReloadOutcomePayload, SwitchExecutionPayload, SwitchPreviewPayload,
    SwitchProfilePayload,
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

fn env_flag(name: &str) -> bool {
    std::env::var(name)
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

fn gui_demo_mode() -> bool {
    env_flag("CODEX_SWITCHER_GUI_DEMO") || env_flag("CODEX_SWITCHER_SCREENSHOT_DEMO")
}

#[derive(Clone)]
struct DemoProfile {
    label: &'static str,
    plan: &'static str,
    reserved: bool,
    status: &'static str,
    seven_day_remaining: &'static str,
    five_hour_remaining: &'static str,
    availability: Option<AvailabilityPayload>,
}

fn demo_availability(
    tag: &str,
    label: &str,
    reason: &str,
    retryable: bool,
) -> AvailabilityPayload {
    AvailabilityPayload {
        tag: tag.to_string(),
        label: label.to_string(),
        reason: reason.to_string(),
        retryable,
    }
}

fn demo_profiles() -> Vec<DemoProfile> {
    vec![
        DemoProfile {
            label: "demo-main",
            plan: "Pro",
            reserved: false,
            status: "active",
            seven_day_remaining: "83%",
            five_hour_remaining: "61%",
            availability: None,
        },
        DemoProfile {
            label: "work-team",
            plan: "Team",
            reserved: false,
            status: "available",
            seven_day_remaining: "72%",
            five_hour_remaining: "100%",
            availability: None,
        },
        DemoProfile {
            label: "personal-pro",
            plan: "Pro",
            reserved: false,
            status: "available",
            seven_day_remaining: "48%",
            five_hour_remaining: "37%",
            availability: None,
        },
        DemoProfile {
            label: "reserved-batch",
            plan: "Enterprise",
            reserved: true,
            status: "reserved",
            seven_day_remaining: "64%",
            five_hour_remaining: "52%",
            availability: None,
        },
        DemoProfile {
            label: "vps-agent-1",
            plan: "Key",
            reserved: false,
            status: "available",
            seven_day_remaining: "--",
            five_hour_remaining: "--",
            availability: Some(demo_availability(
                "apiKeyUnsupported",
                "API key only",
                "Usage unavailable for API key login",
                false,
            )),
        },
        DemoProfile {
            label: "usage-404",
            plan: "Pro",
            reserved: false,
            status: "available",
            seven_day_remaining: "--",
            five_hour_remaining: "--",
            availability: Some(demo_availability(
                "usageFetchError",
                "Usage fetch error",
                "failed to fetch usage: http status: 404",
                true,
            )),
        },
        DemoProfile {
            label: "stale-session",
            plan: "Team",
            reserved: false,
            status: "available",
            seven_day_remaining: "--",
            five_hour_remaining: "--",
            availability: Some(demo_availability(
                "missingAccessToken",
                "Auth token missing",
                "Missing access token",
                true,
            )),
        },
        DemoProfile {
            label: "broken-account",
            plan: "Pro",
            reserved: false,
            status: "available",
            seven_day_remaining: "--",
            five_hour_remaining: "--",
            availability: Some(demo_availability(
                "missingAccountId",
                "Account id missing",
                "Missing account id",
                true,
            )),
        },
        DemoProfile {
            label: "free-trial",
            plan: "Free",
            reserved: false,
            status: "available",
            seven_day_remaining: "--",
            five_hour_remaining: "--",
            availability: Some(demo_availability(
                "freePlanUnsupported",
                "Free plan",
                "Usage unavailable for free plan",
                false,
            )),
        },
        DemoProfile {
            label: "window-partial",
            plan: "Plus",
            reserved: false,
            status: "available",
            seven_day_remaining: "--",
            five_hour_remaining: "--",
            availability: Some(demo_availability(
                "missingFiveHourWindow",
                "5h window missing",
                "Missing 5h usage window",
                false,
            )),
        },
        DemoProfile {
            label: "weekly-gap",
            plan: "Plus",
            reserved: false,
            status: "available",
            seven_day_remaining: "--",
            five_hour_remaining: "--",
            availability: Some(demo_availability(
                "missingSevenDayWindow",
                "7d window missing",
                "Missing 7d usage window",
                false,
            )),
        },
    ]
}

fn demo_profile_cards() -> Vec<ProfileCard> {
    demo_profiles()
        .into_iter()
        .map(|profile| ProfileCard {
            label: profile.label.to_string(),
            plan: profile.plan.to_string(),
            reserved: profile.reserved,
            status: profile.status.to_string(),
            seven_day_remaining: profile.seven_day_remaining.to_string(),
            five_hour_remaining: profile.five_hour_remaining.to_string(),
            availability: profile.availability,
        })
        .collect()
}

fn demo_switch_profiles(requested_profile: &str) -> Vec<SwitchProfilePayload> {
    let recommended = "work-team";
    let mut next_rank = 1usize;
    demo_profiles()
        .into_iter()
        .map(|profile| {
            let can_rank = !profile.reserved && profile.availability.is_none();
            let rank = if can_rank {
                let current_rank = Some(next_rank);
                next_rank += 1;
                current_rank
            } else {
                None
            };
            SwitchProfilePayload {
                label: profile.label.to_string(),
                plan: profile.plan.to_string(),
                reserved: profile.reserved,
                status: profile.status.to_string(),
                current: profile.label == "demo-main",
                recommended: profile.label == recommended,
                rank,
                seven_day_remaining: profile.seven_day_remaining.to_string(),
                five_hour_remaining: profile.five_hour_remaining.to_string(),
                availability: profile.availability,
            }
        })
        .map(|profile| {
            if profile.label == requested_profile {
                profile
            } else {
                profile
            }
        })
        .collect()
}

fn demo_switch_preview_payload(requested_profile: &str) -> SwitchPreviewPayload {
    let profiles = demo_switch_profiles(requested_profile);
    let selected = profiles.iter().find(|profile| profile.label == requested_profile);
    let can_switch = selected
        .map(|profile| !profile.current && !profile.reserved && profile.availability.is_none())
        .unwrap_or(false);
    let summary = match selected {
        Some(profile) if profile.current => {
            "demo-main is already the active profile in the shared Rust runtime.".to_string()
        }
        Some(profile) if profile.availability.is_some() => format!(
            "{} is not currently switchable [{}]: {}.",
            profile.label,
            profile
                .availability
                .as_ref()
                .map(|availability| availability.label.to_uppercase().replace(' ', "_"))
                .unwrap_or_else(|| "UNAVAILABLE".to_string()),
            profile
                .availability
                .as_ref()
                .map(|availability| availability.reason.as_str())
                .unwrap_or("Unavailable")
        ),
        Some(profile) if profile.reserved => {
            format!("{} is reserved and excluded from automatic switching.", profile.label)
        }
        Some(profile) if profile.label == "work-team" => {
            "work-team is the current best switch candidate from the shared Rust runtime."
                .to_string()
        }
        Some(profile) => format!(
            "{} is available, but work-team is currently the best switch candidate.",
            profile.label
        ),
        None => format!(
            "Profile '{requested_profile}' is not available in the shared switcher runtime."
        ),
    };

    SwitchPreviewPayload {
        requested_profile: requested_profile.to_string(),
        active_profile: Some("demo-main".to_string()),
        recommended_profile: Some("work-team".to_string()),
        can_switch,
        summary,
        profiles,
        manual_hints: vec![
            "Demo mode is active: all profile data in this window is fake and safe for screenshots."
                .to_string(),
            "Use the Profiles view to capture availability tags, reserved rows, and mixed usage states."
                .to_string(),
        ],
    }
}

fn demo_switch_execution_payload(profile_label: &str) -> SwitchExecutionPayload {
    SwitchExecutionPayload {
        switched_to: profile_label.to_string(),
        previous_profile: Some("demo-main".to_string()),
        success: true,
        summary: format!(
            "Loaded {profile_label} via the shared Rust switch service."
        ),
        manual_hints: vec![
            "Demo mode is active, so this switch does not touch any real credentials.".to_string(),
        ],
    }
}

fn demo_profiles_overview_payload() -> ProfilesOverviewPayload {
    ProfilesOverviewPayload {
        workspace_label: "Shared runtime: 11 profiles".to_string(),
        profiles: demo_profile_cards(),
        events: vec![
            "Demo mode is active: this window uses safe fake accounts for screenshots."
                .to_string(),
            "Mixed states are included on purpose: reserved, API key only, missing token, missing account id, usage fetch error, and partial usage windows."
                .to_string(),
            "The best switch candidate keeps realistic non-100% headroom so screenshots look believable."
                .to_string(),
        ],
        last_refresh: "just now".to_string(),
    }
}

fn demo_active_profile_status_payload() -> ActiveProfileStatusPayload {
    ActiveProfileStatusPayload {
        active_profile: "demo-main".to_string(),
        summary: "demo-main is the current active profile exposed by the shared Rust query service."
            .to_string(),
        reserved_profiles: 1,
    }
}

fn demo_reload_targets_payload() -> ReloadTargetsPayload {
    ReloadTargetsPayload {
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
        last_reloaded: "Demo reload services ready for screenshots.".to_string(),
    }
}

fn demo_reload_outcome_payload(target: &str) -> ReloadOutcomePayload {
    ReloadOutcomePayload {
        target: target.to_string(),
        attempted: true,
        restarted: true,
        message: format!("Demo reload of {target} completed successfully."),
        manual_hints: vec![
            "Demo mode is active, so reload actions are safe no-op confirmations.".to_string(),
        ],
    }
}

fn smoke_trace_path(paths: &codex_switcher::switcher::Paths) -> PathBuf {
    paths.codex.join("desktop-smoke-trace.json")
}

#[tauri::command]
pub fn desktop_demo_mode() -> bool {
    gui_demo_mode()
}

#[tauri::command]
pub async fn desktop_profiles_overview() -> DesktopCommandResult<ProfilesOverviewPayload> {
    if gui_demo_mode() {
        return DesktopCommandResult::ok(demo_profiles_overview_payload());
    }

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
    if gui_demo_mode() {
        return DesktopCommandResult::ok(demo_active_profile_status_payload());
    }

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

    if gui_demo_mode() {
        return DesktopCommandResult::ok(demo_switch_preview_payload(&request.profile_label));
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

    if gui_demo_mode() {
        return DesktopCommandResult::ok(demo_switch_execution_payload(&request.profile_label));
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
    if gui_demo_mode() {
        return DesktopCommandResult::ok(demo_switch_execution_payload("work-team"));
    }

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
    if gui_demo_mode() {
        return DesktopCommandResult::ok(demo_reload_targets_payload());
    }

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
    if gui_demo_mode() {
        return match request.target.as_str() {
            "codex" | "cursor" => DesktopCommandResult::ok(demo_reload_outcome_payload(
                &request.target,
            )),
            _ => DesktopCommandResult::err(
                "unknown-reload-target",
                "The requested reload target is not part of the approved desktop bootstrap surface.",
                false,
            ),
        };
    }

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
