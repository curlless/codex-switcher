use super::*;
use crate::switcher::{
    IdeReloadOutcome, format_plan, inspect_ide_reload_target_with_codex_override,
    reload_ide_target_best_effort_with_codex_override,
};
use serde::Serialize;

const CURSOR_PROTOCOL_HELPER_HINT: &str = "Cursor automation: install the Commands Executor extension (ionutvmi.vscode-commands-executor) to enable protocol-based Reload Window.";

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileCard {
    pub label: String,
    pub plan: String,
    pub reserved: bool,
    pub status: String,
    pub seven_day_remaining: String,
    pub five_hour_remaining: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfilesOverviewPayload {
    pub workspace_label: String,
    pub profiles: Vec<ProfileCard>,
    pub events: Vec<String>,
    pub last_refresh: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveProfileStatusPayload {
    pub active_profile: String,
    pub summary: String,
    pub reserved_profiles: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SwitchProfilePayload {
    pub label: String,
    pub plan: String,
    pub reserved: bool,
    pub status: String,
    pub current: bool,
    pub recommended: bool,
    pub rank: Option<usize>,
    pub seven_day_remaining: String,
    pub five_hour_remaining: String,
    pub unavailable_reason: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SwitchPreviewPayload {
    pub requested_profile: String,
    pub active_profile: Option<String>,
    pub recommended_profile: Option<String>,
    pub can_switch: bool,
    pub summary: String,
    pub profiles: Vec<SwitchProfilePayload>,
    pub manual_hints: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReloadOutcomePayload {
    pub target: String,
    pub attempted: bool,
    pub restarted: bool,
    pub message: String,
    pub manual_hints: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SwitchExecutionPayload {
    pub switched_to: String,
    pub previous_profile: Option<String>,
    pub success: bool,
    pub summary: String,
    pub manual_hints: Vec<String>,
}

struct ProfileQueryContext {
    snapshot: Snapshot,
    rows: Vec<profile_priority::PriorityRow>,
}

pub(super) struct SwitchPlan {
    pub preview: SwitchPreviewPayload,
    pub rows: Vec<profile_priority::PriorityRow>,
    pub selected_id: Option<String>,
    pub selected_display: Option<String>,
}

pub(super) fn profiles_overview(paths: &Paths) -> Result<ProfilesOverviewPayload, String> {
    let context = load_profile_query_context(paths)?;
    let reserved_profiles = reserved_profile_count(&context.snapshot);
    let profiles = context
        .rows
        .iter()
        .map(|row| profile_card(paths, &context.snapshot, row))
        .collect::<Vec<_>>();
    let active_profile = context.rows.iter().find(|row| row.is_current);
    let mut events = vec![format!(
        "Canonical Rust runtime returned {} profile{} for the desktop shell.",
        profiles.len(),
        if profiles.len() == 1 { "" } else { "s" }
    )];
    if let Some(active_profile) = active_profile {
        events.push(format!(
            "Active profile: {}.",
            profile_selection_label(active_profile)
        ));
    }
    if reserved_profiles > 0 {
        events.push(format!(
            "{reserved_profiles} reserved profile{} stay out of auto-switch candidacy.",
            if reserved_profiles == 1 { "" } else { "s" }
        ));
    }
    if active_profile.is_some_and(|row| row.id == "__current__") {
        events.push(
            "Current auth session is not saved yet; the shared service still exposes it as the active profile."
                .to_string(),
        );
    }

    Ok(ProfilesOverviewPayload {
        workspace_label: format!(
            "Shared runtime: {} profile{}",
            profiles.len(),
            if profiles.len() == 1 { "" } else { "s" }
        ),
        profiles,
        events,
        last_refresh: Local::now().format("%Y-%m-%d %H:%M %:z").to_string(),
    })
}

pub(super) fn active_profile_status(paths: &Paths) -> Result<ActiveProfileStatusPayload, String> {
    let context = load_profile_query_context(paths)?;
    let reserved_profiles = reserved_profile_count(&context.snapshot);
    let active_profile = context.rows.iter().find(|row| row.is_current);
    let active_profile_label = active_profile
        .map(profile_selection_label)
        .unwrap_or_else(|| "No active profile".to_string());
    let summary = match active_profile {
        Some(row) if row.id == "__current__" => format!(
            "{active_profile_label} is active in auth.json but has not been saved as a reusable profile yet."
        ),
        Some(row) => {
            let profile_state = profile_status_label(row);
            format!(
                "{active_profile_label} is the current {profile_state} profile exposed by the shared Rust query service."
            )
        }
        None => "No active profile could be derived from the current auth state.".to_string(),
    };

    Ok(ActiveProfileStatusPayload {
        active_profile: active_profile_label,
        summary,
        reserved_profiles,
    })
}

pub(super) fn best_switch_plan(paths: &Paths) -> Result<SwitchPlan, String> {
    build_switch_plan(paths, None)
}

pub(super) fn switch_preview(
    paths: &Paths,
    requested_profile: &str,
) -> Result<SwitchPreviewPayload, String> {
    build_switch_plan(paths, Some(requested_profile)).map(|plan| plan.preview)
}

pub(super) fn execute_best_switch(
    paths: &Paths,
    reload_target: Option<ReloadAppTarget>,
) -> Result<SwitchExecutionPayload, String> {
    let plan = best_switch_plan(paths)?;
    execute_switch_plan(paths, plan, reload_target)
}

pub(super) fn execute_switch(
    paths: &Paths,
    requested_profile: &str,
    reload_target: Option<ReloadAppTarget>,
) -> Result<SwitchExecutionPayload, String> {
    let plan = build_switch_plan(paths, Some(requested_profile))?;
    execute_switch_plan(paths, plan, reload_target)
}

fn execute_switch_plan(
    paths: &Paths,
    plan: SwitchPlan,
    reload_target: Option<ReloadAppTarget>,
) -> Result<SwitchExecutionPayload, String> {
    let previous_profile = plan.preview.active_profile.clone();
    let selected_id = plan
        .selected_id
        .as_deref()
        .ok_or_else(no_eligible_profile_error)?;
    let selected_display = plan
        .selected_display
        .as_deref()
        .ok_or_else(no_eligible_profile_error)?;
    if !plan.preview.can_switch && previous_profile.as_deref() != Some(selected_display) {
        return Err(format!(
            "Profile '{selected_display}' is not currently switchable from the shared Rust runtime."
        ));
    }
    profile_load::load_profile_by_id(paths, selected_id, selected_display)?;
    let reload = reload_target
        .map(|target| reload_target_outcome(paths, false, target))
        .transpose()?;
    let mut manual_hints = plan.preview.manual_hints;
    let summary = if let Some(reload) = reload.as_ref() {
        manual_hints.extend(reload.manual_hints.iter().cloned());
        format!(
            "Loaded {selected_display} via the shared Rust switch service and processed {} reload guidance.",
            reload_target_display(&reload.target)
        )
    } else {
        format!("Loaded {selected_display} via the shared Rust switch service.")
    };

    Ok(SwitchExecutionPayload {
        switched_to: selected_display.to_string(),
        previous_profile,
        success: true,
        summary,
        manual_hints,
    })
}

pub(super) fn inspect_reload_outcome(
    paths: &Paths,
    target: ReloadAppTarget,
) -> Result<ReloadOutcomePayload, String> {
    reload_target_outcome(paths, true, target)
}

pub(super) fn execute_reload_outcome(
    paths: &Paths,
    target: ReloadAppTarget,
) -> Result<ReloadOutcomePayload, String> {
    reload_target_outcome(paths, false, target)
}

fn load_profile_query_context(paths: &Paths) -> Result<ProfileQueryContext, String> {
    let snapshot = load_snapshot(paths, false)?;
    let current_saved = current_saved_id(paths, &snapshot.usage_map, &snapshot.tokens);
    let rows = priority_rows(paths, &snapshot, current_saved.as_deref(), true);
    Ok(ProfileQueryContext { snapshot, rows })
}

fn build_switch_plan(paths: &Paths, requested_profile: Option<&str>) -> Result<SwitchPlan, String> {
    let context = load_profile_query_context(paths)?;
    let recommended = best_ready_row(&context.rows);
    let active_profile = context.rows.iter().find(|row| row.is_current);
    let selected = match requested_profile {
        Some(requested_profile) => find_requested_profile(&context.rows, requested_profile)?,
        None => recommended,
    };
    let selected_id = selected.map(|row| row.id.clone());
    let selected_display = selected.map(profile_selection_label);
    let profiles = switch_profiles(
        paths,
        &context.snapshot,
        &context.rows,
        recommended.map(|row| row.id.as_str()),
    );
    let preview = SwitchPreviewPayload {
        requested_profile: requested_profile
            .map(str::to_string)
            .or_else(|| selected.map(profile_selection_label))
            .unwrap_or_else(|| "No eligible profile".to_string()),
        active_profile: active_profile.map(profile_selection_label),
        recommended_profile: recommended.map(profile_selection_label),
        can_switch: selected.is_some_and(|row| {
            row.candidate && matches!(row.state, profile_priority::PriorityState::Ready(_))
        }),
        summary: switch_summary(selected, recommended),
        profiles,
        manual_hints: switch_manual_hints(selected, active_profile, recommended),
    };

    Ok(SwitchPlan {
        preview,
        rows: context.rows,
        selected_id,
        selected_display,
    })
}

fn switch_profiles(
    paths: &Paths,
    snapshot: &Snapshot,
    rows: &[profile_priority::PriorityRow],
    recommended_id: Option<&str>,
) -> Vec<SwitchProfilePayload> {
    let mut next_rank = 1usize;
    rows.iter()
        .map(|row| {
            let rank = if row.candidate
                && matches!(row.state, profile_priority::PriorityState::Ready(_))
            {
                let rank = Some(next_rank);
                next_rank += 1;
                rank
            } else {
                None
            };
            let (seven_day_remaining, five_hour_remaining, unavailable_reason) = usage_strings(row);
            SwitchProfilePayload {
                label: profile_selection_label(row),
                plan: profile_plan(paths, snapshot, row),
                reserved: !row.candidate,
                status: profile_status_label(row),
                current: row.is_current,
                recommended: recommended_id == Some(row.id.as_str()),
                rank,
                seven_day_remaining,
                five_hour_remaining,
                unavailable_reason,
            }
        })
        .collect()
}

fn usage_strings(row: &profile_priority::PriorityRow) -> (String, String, Option<String>) {
    match &row.state {
        profile_priority::PriorityState::Ready(usage) => (
            format!("{}%", usage.seven_day_left),
            format!("{}%", usage.five_hour_left),
            None,
        ),
        profile_priority::PriorityState::Unavailable(reason) => (
            "UNAVAILABLE".to_string(),
            "UNAVAILABLE".to_string(),
            Some(reason.clone()),
        ),
    }
}

fn find_requested_profile<'a>(
    rows: &'a [profile_priority::PriorityRow],
    requested_profile: &str,
) -> Result<Option<&'a profile_priority::PriorityRow>, String> {
    if rows.is_empty() {
        return Err("No saved profiles are available for switch preview.".to_string());
    }
    rows.iter()
        .find(|row| {
            let selection_label = profile_selection_label(row);
            selection_label.eq_ignore_ascii_case(requested_profile)
                || row.profile_name.eq_ignore_ascii_case(requested_profile)
                || row
                    .label
                    .as_deref()
                    .is_some_and(|label| label.eq_ignore_ascii_case(requested_profile))
        })
        .ok_or_else(|| {
            format!(
                "Profile '{requested_profile}' is not available in the shared switcher runtime."
            )
        })
        .map(Some)
}

fn switch_summary(
    selected: Option<&profile_priority::PriorityRow>,
    recommended: Option<&profile_priority::PriorityRow>,
) -> String {
    let Some(selected) = selected else {
        return "No eligible profile is available for automatic switching.".to_string();
    };
    let label = profile_selection_label(selected);
    if selected.is_current {
        return format!("{label} is already the active profile in the shared Rust runtime.");
    }
    if !selected.candidate {
        return format!("{label} is reserved and excluded from automatic switching.");
    }
    if let profile_priority::PriorityState::Unavailable(reason) = &selected.state {
        return format!("{label} is not currently switchable: {reason}.");
    }
    if recommended.is_some_and(|row| row.id == selected.id) {
        return format!(
            "{label} is the current best switch candidate from the shared Rust runtime."
        );
    }
    let recommended = recommended.map(profile_selection_label);
    match recommended {
        Some(recommended) => {
            format!(
                "{label} is available, but {recommended} is currently the best switch candidate."
            )
        }
        None => format!(
            "{label} is available, but no recommended auto-switch candidate could be derived."
        ),
    }
}

fn switch_manual_hints(
    selected: Option<&profile_priority::PriorityRow>,
    active_profile: Option<&profile_priority::PriorityRow>,
    recommended: Option<&profile_priority::PriorityRow>,
) -> Vec<String> {
    let mut hints = Vec::new();
    if active_profile.is_some_and(|row| row.id == "__current__") {
        hints.push(
            "The current auth session is unsaved; save it first if you want label-based switching to stay stable."
                .to_string(),
        );
    }
    if selected.is_some_and(|row| !row.candidate) {
        hints.push(
            "Reserved profiles stay out of automatic-switch candidacy until they are unreserved."
                .to_string(),
        );
    }
    if selected
        .is_some_and(|row| matches!(row.state, profile_priority::PriorityState::Unavailable(_)))
    {
        hints.push(
            "Refresh usage data or repair the saved profile before attempting an automatic switch."
                .to_string(),
        );
    }
    if recommended.is_none() {
        hints.push(
            "At least one non-reserved profile with readable usage data is required for automatic switching."
                .to_string(),
        );
    }
    hints
}

fn reload_target_outcome(
    paths: &Paths,
    dry_run: bool,
    target: ReloadAppTarget,
) -> Result<ReloadOutcomePayload, String> {
    let codex_override = profile_switch::codex_override_for_reload_target(paths, target)?;
    let outcome = if dry_run {
        inspect_ide_reload_target_with_codex_override(target, codex_override.as_ref())
    } else {
        reload_ide_target_best_effort_with_codex_override(target, codex_override.as_ref())
    };
    Ok(normalize_reload_outcome(target, outcome))
}

fn normalize_reload_outcome(
    target: ReloadAppTarget,
    outcome: IdeReloadOutcome,
) -> ReloadOutcomePayload {
    let mut manual_hints = outcome.manual_hints;
    if matches!(target, ReloadAppTarget::All | ReloadAppTarget::Cursor)
        && !outcome.message.contains("protocol reload is available")
        && !manual_hints
            .iter()
            .any(|hint| hint.contains("ionutvmi.vscode-commands-executor"))
    {
        manual_hints.push(CURSOR_PROTOCOL_HELPER_HINT.to_string());
    }

    ReloadOutcomePayload {
        target: reload_target_id(target).to_string(),
        attempted: outcome.attempted,
        restarted: outcome.restarted,
        message: outcome.message,
        manual_hints,
    }
}

fn reload_target_id(target: ReloadAppTarget) -> &'static str {
    match target {
        ReloadAppTarget::All => "all",
        ReloadAppTarget::Codex => "codex",
        ReloadAppTarget::Cursor => "cursor",
    }
}

fn reload_target_display(target: &str) -> &str {
    match target {
        "all" => "all-target",
        "codex" => "Codex",
        "cursor" => "Cursor",
        _ => "requested",
    }
}

fn no_eligible_profile_error() -> String {
    "Error: no eligible profile found for auto-switch.".to_string()
}

fn profile_card(
    paths: &Paths,
    snapshot: &Snapshot,
    row: &profile_priority::PriorityRow,
) -> ProfileCard {
    let (seven_day_remaining, five_hour_remaining, _) = usage_strings(row);

    ProfileCard {
        label: profile_selection_label(row),
        plan: profile_plan(paths, snapshot, row),
        reserved: !row.candidate,
        status: profile_status_label(row),
        seven_day_remaining,
        five_hour_remaining,
    }
}

fn profile_plan(paths: &Paths, snapshot: &Snapshot, row: &profile_priority::PriorityRow) -> String {
    let tokens = if row.id == "__current__" {
        read_tokens(&paths.auth).ok()
    } else {
        snapshot
            .tokens
            .get(&row.id)
            .and_then(|result| result.clone().ok())
    };
    let Some(tokens) = tokens else {
        return "Unknown plan".to_string();
    };
    let (_, plan) = extract_email_and_plan(&tokens);
    plan.map(|plan| format_plan(&plan))
        .unwrap_or_else(|| "Unknown plan".to_string())
}

fn profile_selection_label(row: &profile_priority::PriorityRow) -> String {
    row.label
        .clone()
        .unwrap_or_else(|| display_without_reserved_marker(&row.profile_name))
}

fn profile_status_label(row: &profile_priority::PriorityRow) -> String {
    if row.is_current {
        "active".to_string()
    } else if row.candidate {
        "available".to_string()
    } else {
        "reserved".to_string()
    }
}

fn reserved_profile_count(snapshot: &Snapshot) -> usize {
    snapshot
        .index
        .profiles
        .keys()
        .filter(|id| profile_is_reserved(id, snapshot))
        .count()
}

#[cfg(test)]
mod service_tests {
    use super::*;

    #[test]
    fn normalize_reload_outcome_adds_cursor_helper_hint() {
        let payload = normalize_reload_outcome(
            ReloadAppTarget::Cursor,
            IdeReloadOutcome {
                attempted: false,
                restarted: false,
                message: "Reload hint: detected Cursor Codex extension; automatic extension reload is not implemented."
                    .to_string(),
                manual_hints: Vec::new(),
            },
        );

        assert_eq!(payload.target, "cursor");
        assert!(
            payload
                .manual_hints
                .iter()
                .any(|hint| hint.contains("Commands Executor"))
        );
    }
}
