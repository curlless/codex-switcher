use chrono::{DateTime, Local, Utc};
use colored::Colorize;
use directories::BaseDirs;
use inquire::{Confirm, MultiSelect, Select};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt;
use std::fs;
use std::io::{self, IsTerminal as _};
use std::path::{Path, PathBuf};

use crate::{
    CANCELLED_MESSAGE, format_action, format_entry_header, format_error, format_hint,
    format_list_hint, format_no_profiles, format_save_before_load, format_unsaved_warning,
    format_warning, inquire_select_render_config, is_inquire_cancel, is_plain, normalize_error,
    print_output_block, print_output_block_with_frame, style_text, terminal_width,
    use_color_stderr, use_color_stdout,
};
use crate::{
    Paths, codex_app_override, command_name, copy_atomic, ensure_codex_app_override, write_atomic,
};
use crate::{
    ReloadAppTarget, inspect_ide_reload_target_with_codex_override,
    reload_ide_target_best_effort_with_codex_override,
};
use crate::{
    Tokens, extract_email_and_plan, is_api_key_profile, is_free_plan, is_profile_ready,
    profile_error, read_tokens, read_tokens_opt, refresh_profile_tokens, require_identity,
    token_account_id,
};
use crate::{
    UsageLock, UsageWindow, fetch_usage_details, fetch_usage_limits, format_last_used,
    format_usage_unavailable, lock_usage, now_seconds, ordered_profiles, read_base_url,
    start_usage_spinner, stop_usage_spinner, usage_unavailable,
};

const MAX_USAGE_CONCURRENCY: usize = 4;
const SCORE_7D_WEIGHT: i64 = 70;
const SCORE_5H_WEIGHT: i64 = 30;
const CURSOR_PROTOCOL_HELPER_HINT: &str = "Cursor automation: install the Commands Executor extension (ionutvmi.vscode-commands-executor) to enable protocol-based Reload Window.";

#[derive(Clone, Debug)]
struct PriorityUsage {
    seven_day_left: i64,
    seven_day_reset: Option<String>,
    five_hour_left: i64,
    five_hour_reset: Option<String>,
    tier: u8,
    score: i64,
}

#[derive(Clone, Debug)]
enum PriorityState {
    Ready(PriorityUsage),
    Unavailable(String),
}

#[derive(Clone, Debug)]
struct PriorityRow {
    id: String,
    profile_name: String,
    label: Option<String>,
    is_current: bool,
    candidate: bool,
    state: PriorityState,
}

#[derive(Clone, Copy, Default)]
struct UsageSortKey {
    five_hour_left: Option<i64>,
    secondary_left: Option<i64>,
    reset_at: Option<i64>,
    usable: bool,
}

fn ordered_profiles_by_usage(
    snapshot: &Snapshot,
    ctx: &ListCtx,
    current_saved_id: Option<&str>,
) -> Vec<(String, u64)> {
    let mut ordered = snapshot
        .usage_map
        .iter()
        .map(|(id, ts)| (id.clone(), *ts))
        .collect::<Vec<_>>();
    let usage_scores = usage_sort_scores(snapshot, ctx, current_saved_id);
    ordered.sort_by(|(left_id, left_ts), (right_id, right_ts)| {
        let left_score = usage_scores.get(left_id).copied().unwrap_or_default();
        let right_score = usage_scores.get(right_id).copied().unwrap_or_default();
        let left_has_primary = left_score.five_hour_left.is_some();
        let right_has_primary = right_score.five_hour_left.is_some();
        let mut ordering = right_has_primary.cmp(&left_has_primary);
        if ordering != Ordering::Equal {
            return ordering;
        }
        ordering = right_score.usable.cmp(&left_score.usable);
        if ordering != Ordering::Equal {
            return ordering;
        }
        if left_score.usable && right_score.usable {
            ordering = right_score
                .five_hour_left
                .unwrap_or(-1)
                .cmp(&left_score.five_hour_left.unwrap_or(-1));
            if ordering != Ordering::Equal {
                return ordering;
            }
            ordering = right_score
                .secondary_left
                .unwrap_or(-1)
                .cmp(&left_score.secondary_left.unwrap_or(-1));
            if ordering != Ordering::Equal {
                return ordering;
            }
        } else if !left_score.usable && !right_score.usable {
            let left_reset = left_score.reset_at.unwrap_or(i64::MAX);
            let right_reset = right_score.reset_at.unwrap_or(i64::MAX);
            ordering = left_reset.cmp(&right_reset);
            if ordering != Ordering::Equal {
                return ordering;
            }
        }
        ordering = right_ts.cmp(left_ts);
        if ordering != Ordering::Equal {
            return ordering;
        }
        left_id.cmp(right_id)
    });
    ordered
}

fn usage_sort_scores(
    snapshot: &Snapshot,
    ctx: &ListCtx,
    current_saved_id: Option<&str>,
) -> HashMap<String, UsageSortKey> {
    let Some(base_url) = ctx.base_url.as_deref() else {
        return HashMap::new();
    };
    let now = ctx.now;
    let ids: Vec<String> = snapshot.usage_map.keys().cloned().collect();
    let build = |id: &String| {
        if current_saved_id == Some(id.as_str()) {
            return (id.clone(), UsageSortKey::default());
        }
        let key = usage_sort_key_for_profile(id, snapshot, base_url, now).unwrap_or_default();
        (id.clone(), key)
    };
    let mut scores = HashMap::with_capacity(ids.len());
    if ids.len() > MAX_USAGE_CONCURRENCY {
        for chunk in ids.chunks(MAX_USAGE_CONCURRENCY) {
            let chunk_scores: Vec<(String, UsageSortKey)> = chunk.par_iter().map(build).collect();
            for (id, key) in chunk_scores {
                scores.insert(id, key);
            }
        }
        return scores;
    }
    let entries: Vec<(String, UsageSortKey)> = ids.par_iter().map(build).collect();
    for (id, key) in entries {
        scores.insert(id, key);
    }
    scores
}

fn usage_sort_key_for_profile(
    id: &str,
    snapshot: &Snapshot,
    base_url: &str,
    now: DateTime<Local>,
) -> Option<UsageSortKey> {
    if profile_is_api_key(id, snapshot) || profile_is_free(id, snapshot) {
        return None;
    }
    let tokens = snapshot
        .tokens
        .get(id)
        .and_then(|result| result.as_ref().ok())?;
    let access_token = tokens.access_token.as_deref()?;
    let account_id = token_account_id(tokens)?;
    let limits = fetch_usage_limits(base_url, access_token, account_id, now).ok()?;
    let five_hour_left = usage_left_percent(limits.five_hour.as_ref())?;
    let secondary_left = usage_left_percent(limits.weekly.as_ref());
    let primary_left = five_hour_left;
    let secondary_left_value = secondary_left.unwrap_or(0);
    let primary_reset = usage_reset_at(limits.five_hour.as_ref());
    let secondary_reset = usage_reset_at(limits.weekly.as_ref());
    let reset_at = if primary_left <= 0 && secondary_left_value <= 0 {
        match (primary_reset, secondary_reset) {
            (Some(primary), Some(secondary)) => Some(primary.max(secondary)),
            (Some(primary), None) => Some(primary),
            (None, Some(secondary)) => Some(secondary),
            (None, None) => None,
        }
    } else if primary_left <= 0 {
        primary_reset
    } else if secondary_left_value <= 0 {
        secondary_reset
    } else {
        None
    };
    let usable = primary_left > 0 && secondary_left_value > 0;
    Some(UsageSortKey {
        five_hour_left: Some(five_hour_left),
        secondary_left,
        reset_at,
        usable,
    })
}

fn usage_left_percent(window: Option<&UsageWindow>) -> Option<i64> {
    window.map(|value| value.left_percent.round() as i64)
}

fn usage_reset_at(window: Option<&UsageWindow>) -> Option<i64> {
    window.map(|value| value.reset_at)
}

fn usage_reset_remaining(window: Option<&UsageWindow>) -> Option<String> {
    window
        .and_then(|value| value.reset_at_relative.as_deref())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn profile_is_api_key(id: &str, snapshot: &Snapshot) -> bool {
    snapshot
        .tokens
        .get(id)
        .and_then(|result| result.as_ref().ok())
        .map(is_api_key_profile)
        .or_else(|| {
            snapshot
                .index
                .profiles
                .get(id)
                .map(|entry| entry.is_api_key)
        })
        .unwrap_or(false)
}

fn profile_is_free(id: &str, snapshot: &Snapshot) -> bool {
    let plan = profile_plan_for_sort(id, snapshot);
    is_free_plan(plan.as_deref())
}

fn profile_plan_for_sort(id: &str, snapshot: &Snapshot) -> Option<String> {
    if let Some(tokens) = snapshot
        .tokens
        .get(id)
        .and_then(|result| result.as_ref().ok())
    {
        let (_, plan) = extract_email_and_plan(tokens);
        if plan.is_some() {
            return plan;
        }
    }
    snapshot
        .index
        .profiles
        .get(id)
        .and_then(|entry| entry.plan.clone())
}

fn priority_state_for_profile(
    id: &str,
    snapshot: &Snapshot,
    base_url: &str,
    now: DateTime<Local>,
) -> PriorityState {
    let Some(tokens) = snapshot
        .tokens
        .get(id)
        .and_then(|result| result.as_ref().ok())
    else {
        return PriorityState::Unavailable("Profile tokens are unreadable".to_string());
    };
    priority_state_for_tokens(tokens, base_url, now)
}

fn priority_state_for_tokens(
    tokens: &Tokens,
    base_url: &str,
    now: DateTime<Local>,
) -> PriorityState {
    if is_api_key_profile(tokens) {
        return PriorityState::Unavailable("Usage unavailable for API key login".to_string());
    }
    let (_, plan) = extract_email_and_plan(tokens);
    if is_free_plan(plan.as_deref()) {
        return PriorityState::Unavailable("Usage unavailable for free plan".to_string());
    }
    let Some(access_token) = tokens.access_token.as_deref() else {
        return PriorityState::Unavailable("Missing access token".to_string());
    };
    let Some(account_id) = token_account_id(tokens) else {
        return PriorityState::Unavailable("Missing account id".to_string());
    };
    let limits = match fetch_usage_limits(base_url, access_token, account_id, now) {
        Ok(limits) => limits,
        Err(err) => return PriorityState::Unavailable(normalize_error(&err.message())),
    };
    let Some(five_hour_left) = usage_left_percent(limits.five_hour.as_ref()) else {
        return PriorityState::Unavailable("Missing 5h usage window".to_string());
    };
    let Some(seven_day_left) = usage_left_percent(limits.weekly.as_ref()) else {
        return PriorityState::Unavailable("Missing 7d usage window".to_string());
    };
    let five_hour_reset = usage_reset_remaining(limits.five_hour.as_ref());
    let seven_day_reset = usage_reset_remaining(limits.weekly.as_ref());
    let tier = if seven_day_left <= 0 {
        2
    } else if five_hour_left <= 0 {
        1
    } else {
        0
    };
    let score = if tier == 0 {
        seven_day_left * SCORE_7D_WEIGHT + five_hour_left * SCORE_5H_WEIGHT
    } else {
        0
    };
    PriorityState::Ready(PriorityUsage {
        seven_day_left,
        seven_day_reset,
        five_hour_left,
        five_hour_reset,
        tier,
        score,
    })
}

fn priority_profile_email(id: &str, snapshot: &Snapshot) -> String {
    snapshot
        .tokens
        .get(id)
        .and_then(|result| result.as_ref().ok())
        .and_then(|tokens| extract_email_and_plan(tokens).0)
        .or_else(|| {
            snapshot
                .index
                .profiles
                .get(id)
                .and_then(|entry| entry.email.clone())
        })
        .unwrap_or_else(|| id.to_string())
}

fn profile_name_for_priority_row(id: &str, snapshot: &Snapshot, label: Option<&str>) -> String {
    let email = priority_profile_email(id, snapshot);
    match label {
        Some(label) => format!("{email} [{label}]"),
        None => email,
    }
}

fn priority_identity_key(row: &PriorityRow, snapshot: &Snapshot) -> String {
    if row.id == "__current__" {
        return "id:__current__".to_string();
    }
    if let Some(tokens) = snapshot
        .tokens
        .get(&row.id)
        .and_then(|result| result.as_ref().ok())
    {
        if let Some(account_id) = token_account_id(tokens) {
            return format!("account:{account_id}");
        }
        if let Some(email) = extract_email_and_plan(tokens).0 {
            return format!("email:{}", email.to_ascii_lowercase());
        }
    }
    if let Some(entry) = snapshot.index.profiles.get(&row.id) {
        if let Some(account_id) = entry.account_id.as_deref() {
            return format!("account:{account_id}");
        }
        if let Some(email) = entry.email.as_deref() {
            return format!("email:{}", email.to_ascii_lowercase());
        }
    }
    format!("id:{}", row.id)
}

fn priority_state_cmp(left: &PriorityState, right: &PriorityState) -> Ordering {
    match (left, right) {
        (PriorityState::Ready(left_usage), PriorityState::Ready(right_usage)) => left_usage
            .tier
            .cmp(&right_usage.tier)
            .then_with(|| right_usage.score.cmp(&left_usage.score)),
        (PriorityState::Ready(_), PriorityState::Unavailable(_)) => Ordering::Less,
        (PriorityState::Unavailable(_), PriorityState::Ready(_)) => Ordering::Greater,
        (PriorityState::Unavailable(_), PriorityState::Unavailable(_)) => Ordering::Equal,
    }
}

fn priority_row_has_readable_tokens(row: &PriorityRow, snapshot: &Snapshot) -> bool {
    if row.id == "__current__" {
        return true;
    }
    snapshot
        .tokens
        .get(&row.id)
        .is_some_and(|result| result.is_ok())
}

fn priority_group_target_cmp(
    left: &PriorityRow,
    right: &PriorityRow,
    snapshot: &Snapshot,
) -> Ordering {
    let ordering = right.is_current.cmp(&left.is_current);
    if ordering != Ordering::Equal {
        return ordering;
    }
    let ordering = right.label.is_some().cmp(&left.label.is_some());
    if ordering != Ordering::Equal {
        return ordering;
    }
    let ordering = right.candidate.cmp(&left.candidate);
    if ordering != Ordering::Equal {
        return ordering;
    }
    let left_readable = priority_row_has_readable_tokens(left, snapshot);
    let right_readable = priority_row_has_readable_tokens(right, snapshot);
    let ordering = right_readable.cmp(&left_readable);
    if ordering != Ordering::Equal {
        return ordering;
    }
    let ordering = priority_state_cmp(&left.state, &right.state);
    if ordering != Ordering::Equal {
        return ordering;
    }
    let left_last_used = snapshot
        .usage_map
        .get(&left.id)
        .copied()
        .unwrap_or_default();
    let right_last_used = snapshot
        .usage_map
        .get(&right.id)
        .copied()
        .unwrap_or_default();
    right_last_used
        .cmp(&left_last_used)
        .then_with(|| left.id.cmp(&right.id))
}

fn priority_display_label(rows: &[PriorityRow], target: &PriorityRow) -> Option<String> {
    target
        .label
        .clone()
        .or_else(|| {
            rows.iter()
                .find(|row| row.is_current)
                .and_then(|row| row.label.clone())
        })
        .or_else(|| {
            let mut labels = rows
                .iter()
                .filter_map(|row| row.label.clone())
                .collect::<Vec<_>>();
            labels.sort_by_key(|label| label.to_ascii_lowercase());
            labels.into_iter().next()
        })
}

fn merge_priority_rows(rows: Vec<PriorityRow>, snapshot: &Snapshot) -> PriorityRow {
    let target = rows
        .iter()
        .min_by(|left, right| priority_group_target_cmp(left, right, snapshot))
        .expect("priority group is not empty");
    let state = rows
        .iter()
        .min_by(|left, right| {
            priority_state_cmp(&left.state, &right.state)
                .then_with(|| priority_group_target_cmp(left, right, snapshot))
        })
        .map(|row| row.state.clone())
        .expect("priority group is not empty");
    let label = priority_display_label(&rows, target);
    let profile_name = if target.id == "__current__" {
        target.profile_name.clone()
    } else {
        profile_name_for_priority_row(&target.id, snapshot, label.as_deref())
    };
    PriorityRow {
        id: target.id.clone(),
        profile_name,
        label,
        is_current: rows.iter().any(|row| row.is_current),
        candidate: rows.iter().any(|row| row.candidate),
        state,
    }
}

fn dedupe_priority_rows(rows: Vec<PriorityRow>, snapshot: &Snapshot) -> Vec<PriorityRow> {
    let mut grouped: HashMap<String, Vec<PriorityRow>> = HashMap::new();
    for row in rows {
        grouped
            .entry(priority_identity_key(&row, snapshot))
            .or_default()
            .push(row);
    }
    let mut merged = grouped
        .into_values()
        .map(|group| merge_priority_rows(group, snapshot))
        .collect::<Vec<_>>();
    merged.sort_by(priority_row_cmp);
    merged
}

fn priority_rows(
    paths: &Paths,
    snapshot: &Snapshot,
    current_saved_id: Option<&str>,
    include_unsaved_current: bool,
) -> Vec<PriorityRow> {
    let base_url = read_base_url(paths);
    let now = Local::now();
    let ids: Vec<String> = snapshot.usage_map.keys().cloned().collect();
    let build = |id: &String| {
        let label = label_for_id(&snapshot.labels, id);
        let profile_name = profile_name_for_priority_row(id, snapshot, label.as_deref());
        PriorityRow {
            id: id.clone(),
            profile_name,
            label,
            is_current: current_saved_id == Some(id.as_str()),
            candidate: true,
            state: priority_state_for_profile(id, snapshot, &base_url, now),
        }
    };
    let mut rows = if ids.len() > MAX_USAGE_CONCURRENCY {
        let mut out = Vec::with_capacity(ids.len());
        for chunk in ids.chunks(MAX_USAGE_CONCURRENCY) {
            let mut chunk_rows: Vec<PriorityRow> = chunk.par_iter().map(build).collect();
            out.append(&mut chunk_rows);
        }
        out
    } else {
        ids.par_iter().map(build).collect::<Vec<_>>()
    };
    if include_unsaved_current && current_saved_id.is_none() {
        if let Ok(tokens) = read_tokens(&paths.auth) {
            let (email, _) = extract_email_and_plan(&tokens);
            let profile_name = email.unwrap_or_else(|| "Current profile".to_string());
            rows.push(PriorityRow {
                id: "__current__".to_string(),
                profile_name,
                label: None,
                is_current: true,
                candidate: false,
                state: priority_state_for_tokens(&tokens, &base_url, now),
            });
        }
    }
    dedupe_priority_rows(rows, snapshot)
}

fn priority_sort_label(row: &PriorityRow) -> String {
    row.label.as_deref().unwrap_or(&row.id).to_ascii_lowercase()
}

fn priority_row_cmp(left: &PriorityRow, right: &PriorityRow) -> Ordering {
    match (&left.state, &right.state) {
        (PriorityState::Ready(left_usage), PriorityState::Ready(right_usage)) => left_usage
            .tier
            .cmp(&right_usage.tier)
            .then_with(|| right_usage.score.cmp(&left_usage.score))
            .then_with(|| priority_sort_label(left).cmp(&priority_sort_label(right)))
            .then_with(|| left.id.cmp(&right.id)),
        (PriorityState::Ready(_), PriorityState::Unavailable(_)) => Ordering::Less,
        (PriorityState::Unavailable(_), PriorityState::Ready(_)) => Ordering::Greater,
        (PriorityState::Unavailable(_), PriorityState::Unavailable(_)) => priority_sort_label(left)
            .cmp(&priority_sort_label(right))
            .then_with(|| left.id.cmp(&right.id)),
    }
}

fn best_ready_row(rows: &[PriorityRow]) -> Option<&PriorityRow> {
    rows.iter()
        .find(|row| row.candidate && matches!(row.state, PriorityState::Ready(_)))
}

fn render_priority_table(rows: &[PriorityRow], use_color: bool) -> String {
    let headers = [
        "#", "CUR", "PROFILE", "7D", "7D RESET", "5H", "5H RESET", "TIER", "SCORE", "STATE",
    ];
    let mut data = Vec::with_capacity(rows.len());
    let mut rank = 0usize;
    for row in rows {
        let mut rank_text = "-".to_string();
        let current = if row.is_current { "*" } else { "" }.to_string();
        let mut seven = "--".to_string();
        let mut seven_reset = "--".to_string();
        let mut five = "--".to_string();
        let mut five_reset = "--".to_string();
        let mut tier = "--".to_string();
        let mut score = "--".to_string();
        let state = match &row.state {
            PriorityState::Ready(usage) => {
                if row.candidate {
                    rank += 1;
                    rank_text = rank.to_string();
                }
                seven = format!("{}%", usage.seven_day_left);
                seven_reset = usage.seven_day_reset.as_deref().unwrap_or("--").to_string();
                five = format!("{}%", usage.five_hour_left);
                five_reset = usage.five_hour_reset.as_deref().unwrap_or("--").to_string();
                tier = format!("T{}", usage.tier);
                score = if usage.tier == 0 {
                    format!("{:.1}", usage.score as f64 / 100.0)
                } else {
                    "MIN".to_string()
                };
                if row.is_current && !row.candidate {
                    "CURRENT".to_string()
                } else {
                    match usage.tier {
                        0 => "READY".to_string(),
                        1 => "5H=0".to_string(),
                        _ => "7D=0".to_string(),
                    }
                }
            }
            PriorityState::Unavailable(_) => "UNAVAILABLE".to_string(),
        };
        data.push(vec![
            rank_text,
            current,
            row.profile_name.clone(),
            seven,
            seven_reset,
            five,
            five_reset,
            tier,
            score,
            state,
        ]);
    }

    let mut widths: Vec<usize> = headers.iter().map(|value| value.len()).collect();
    for row in &data {
        for (idx, value) in row.iter().enumerate() {
            widths[idx] = widths[idx].max(value.len());
        }
    }

    let mut lines = Vec::new();
    lines.push("Priority ranking (best first)".to_string());
    lines.push(format_row(&headers, &widths, None, use_color));
    let separator = widths.iter().map(|w| "-".repeat(*w)).collect::<Vec<_>>();
    lines.push(format_row_ref(&separator, &widths, None, use_color));

    for row in data {
        let state_style = row.last().cloned().unwrap_or_default();
        lines.push(format_row_ref(&row, &widths, Some(&state_style), use_color));
    }

    let unavailable_rows: Vec<String> = rows
        .iter()
        .filter_map(|row| match &row.state {
            PriorityState::Unavailable(reason) => Some(format!("{}: {}", row.profile_name, reason)),
            PriorityState::Ready(_) => None,
        })
        .collect();
    if !unavailable_rows.is_empty() {
        lines.push(String::new());
        lines.push("Unavailable profiles".to_string());
        for line in unavailable_rows {
            lines.push(format!("- {line}"));
        }
    }

    lines.join("\n")
}

fn format_row(row: &[&str], widths: &[usize], state: Option<&str>, use_color: bool) -> String {
    let parts = row
        .iter()
        .enumerate()
        .map(|(idx, value)| format!("{value:<width$}", width = widths[idx]))
        .collect::<Vec<_>>();
    format_row_ref(&parts, widths, state, use_color)
}

fn format_row_ref(
    row: &[String],
    widths: &[usize],
    state: Option<&str>,
    use_color: bool,
) -> String {
    let mut parts = Vec::with_capacity(row.len());
    for (idx, value) in row.iter().enumerate() {
        let padded = format!("{value:<width$}", width = widths[idx]);
        let styled = if idx + 1 == row.len() {
            style_priority_state_cell(&padded, state, use_color)
        } else if idx == 1 && value == "*" {
            style_text(&padded, use_color, |text| text.cyan().bold())
        } else {
            padded
        };
        parts.push(styled);
    }
    parts.join("  ")
}

fn style_priority_state_cell(cell: &str, state: Option<&str>, use_color: bool) -> String {
    let Some(state) = state else {
        return cell.to_string();
    };
    match state {
        "READY" => style_text(cell, use_color, |text| text.green().bold()),
        "CURRENT" => style_text(cell, use_color, |text| text.cyan().bold()),
        "5H=0" => style_text(cell, use_color, |text| text.yellow().bold()),
        "7D=0" => style_text(cell, use_color, |text| text.red().bold()),
        "UNAVAILABLE" => style_text(cell, use_color, |text| text.dimmed()),
        _ => cell.to_string(),
    }
}

pub fn save_profile(paths: &Paths, label: Option<String>) -> Result<(), String> {
    let use_color = use_color_stdout();
    let mut store = ProfileStore::load(paths)?;
    let tokens = read_tokens(&paths.auth)?;
    let id = resolve_save_id(
        paths,
        &mut store.usage_map,
        &mut store.labels,
        &mut store.profiles_index,
        &tokens,
    )?;

    if let Some(label) = label.as_deref() {
        assign_label(&mut store.labels, label, &id)?;
    }

    let target = profile_path_for_id(&paths.profiles, &id);
    copy_profile(&paths.auth, &target, "save profile to")?;

    let now = now_seconds();
    store.usage_map.insert(id.clone(), now);
    let label_display = label_for_id(&store.labels, &id);
    update_profiles_index_entry(
        &mut store.profiles_index,
        &id,
        Some(&tokens),
        label_display.clone(),
        now,
        true,
    );
    store.save(paths)?;

    let info = profile_info(Some(&tokens), label_display, true, use_color);
    let message = if info.email.is_some() {
        format!("Saved profile {}", info.display)
    } else {
        "Saved profile".to_string()
    };
    let message = format_action(&message, use_color);
    print_output_block(&message);
    Ok(())
}

pub fn load_profile(paths: &Paths, label: Option<String>) -> Result<(), String> {
    let use_color_err = use_color_stderr();
    let no_profiles = format_no_profiles(paths, use_color_err);
    let (mut snapshot, mut ordered) = load_snapshot_ordered(paths, true, &no_profiles)?;

    if let Some(reason) = unsaved_reason(paths, &snapshot.tokens)? {
        match prompt_unsaved_load(paths, &reason)? {
            LoadChoice::SaveAndContinue => {
                save_profile(paths, None)?;
                let no_profiles = format_no_profiles(paths, use_color_err);
                let result = load_snapshot_ordered(paths, true, &no_profiles)?;
                snapshot = result.0;
                ordered = result.1;
            }
            LoadChoice::ContinueWithoutSaving => {}
            LoadChoice::Cancel => {
                return Err(CANCELLED_MESSAGE.to_string());
            }
        }
    }

    let candidates = make_candidates(paths, &snapshot, &ordered);
    let selected = pick_one("load", label.as_deref(), &snapshot, &candidates)?;
    let selected_id = selected.id.clone();
    let selected_display = selected.display.clone();

    match snapshot.tokens.get(&selected_id) {
        Some(Ok(_)) => {}
        Some(Err(err)) => {
            let message = err.strip_prefix("Error: ").unwrap_or(err);
            return Err(format!("Error: selected profile is invalid. {message}"));
        }
        None => {
            return Err(profile_not_found(use_color_err));
        }
    }
    load_profile_by_id(paths, &selected_id, &selected_display)
}

fn load_profile_by_id(
    paths: &Paths,
    selected_id: &str,
    selected_display: &str,
) -> Result<(), String> {
    let use_color_err = use_color_stderr();
    let use_color_out = use_color_stdout();
    let mut store = ProfileStore::load(paths)?;

    if let Err(err) = sync_current(
        paths,
        &mut store.usage_map,
        &mut store.labels,
        &mut store.profiles_index,
    ) {
        let warning = format_warning(&err, use_color_err);
        eprintln!("{warning}");
    }

    let source = profile_path_for_id(&paths.profiles, selected_id);
    if !source.is_file() {
        return Err(profile_not_found(use_color_err));
    }

    copy_profile(&source, &paths.auth, "load selected profile to")?;

    let now = now_seconds();
    store.usage_map.insert(selected_id.to_string(), now);
    let label = label_for_id(&store.labels, selected_id);
    let tokens = read_tokens(&source).ok();
    update_profiles_index_entry(
        &mut store.profiles_index,
        selected_id,
        tokens.as_ref(),
        label,
        now,
        true,
    );
    store.save(paths)?;

    let message = format_action(&format!("Loaded profile {selected_display}"), use_color_out);
    print_output_block(&message);
    Ok(())
}

pub fn switch_best_profile(
    paths: &Paths,
    dry_run: bool,
    reload_target: Option<ReloadAppTarget>,
) -> Result<(), String> {
    let use_color = use_color_stdout();
    let no_profiles = format_no_profiles(paths, use_color);
    let snapshot = load_snapshot(paths, false)?;
    if snapshot.usage_map.is_empty() {
        print_output_block(&no_profiles);
        return Ok(());
    }
    let current_saved = current_saved_id(paths, &snapshot.usage_map, &snapshot.tokens);
    let rows = priority_rows(paths, &snapshot, current_saved.as_deref(), false);
    if rows.is_empty() {
        print_output_block(&no_profiles);
        return Ok(());
    }
    let table = render_priority_table(&rows, use_color);
    print_output_block(&table);

    let Some(best) = best_ready_row(&rows) else {
        let hint = format_hint(
            "No switch performed because usage data is unavailable for all profiles.",
            use_color,
        );
        return Err(format!(
            "Error: no eligible profile found for auto-switch.{hint}"
        ));
    };

    if dry_run {
        let message = format_action(
            &format!("Dry run: best profile is {}", best.profile_name),
            use_color,
        );
        print_output_block(&message);
        return Ok(());
    }

    load_profile_by_id(paths, &best.id, &best.profile_name)?;

    if let Some(reload_target) = reload_target {
        let codex_override = codex_override_for_reload_target(paths, reload_target)?;
        let outcome = reload_ide_target_best_effort_with_codex_override(
            reload_target,
            codex_override.as_ref(),
        );
        let mut lines = Vec::new();
        let mut manual_hints = outcome.manual_hints;
        if matches!(
            reload_target,
            ReloadAppTarget::All | ReloadAppTarget::Cursor
        ) && !outcome.message.contains("protocol reload is available")
            && !manual_hints
                .iter()
                .any(|hint| hint.contains("ionutvmi.vscode-commands-executor"))
        {
            manual_hints.push(CURSOR_PROTOCOL_HELPER_HINT.to_string());
        }
        if outcome.restarted {
            lines.push(format_action(&outcome.message, use_color));
        } else {
            lines.push(format_warning(&outcome.message, use_color));
        }
        for hint in manual_hints {
            lines.push(format_hint(&hint, use_color));
        }
        print_output_block(&lines.join("\n"));
    }

    Ok(())
}

pub fn reload_app(paths: &Paths, dry_run: bool, target: ReloadAppTarget) -> Result<(), String> {
    let use_color = use_color_stdout();
    let codex_override = codex_override_for_reload_target(paths, target)?;
    let outcome = if dry_run {
        inspect_ide_reload_target_with_codex_override(target, codex_override.as_ref())
    } else {
        reload_ide_target_best_effort_with_codex_override(target, codex_override.as_ref())
    };
    let mut lines = Vec::new();
    let mut manual_hints = outcome.manual_hints;
    if matches!(target, ReloadAppTarget::All | ReloadAppTarget::Cursor)
        && !outcome.message.contains("protocol reload is available")
        && !manual_hints
            .iter()
            .any(|hint| hint.contains("ionutvmi.vscode-commands-executor"))
    {
        manual_hints.push(CURSOR_PROTOCOL_HELPER_HINT.to_string());
    }
    if outcome.restarted {
        lines.push(format_action(&outcome.message, use_color));
    } else {
        lines.push(format_warning(&outcome.message, use_color));
    }
    for hint in manual_hints {
        lines.push(format_hint(&hint, use_color));
    }
    print_output_block(&lines.join("\n"));
    Ok(())
}

fn codex_override_for_reload_target(
    paths: &Paths,
    target: ReloadAppTarget,
) -> Result<Option<crate::CodexAppOverride>, String> {
    if matches!(target, ReloadAppTarget::All | ReloadAppTarget::Codex) {
        ensure_codex_app_override(paths)
    } else {
        codex_app_override(paths)
    }
}

pub fn migrate_profiles(
    paths: &Paths,
    from: Option<String>,
    overwrite: bool,
) -> Result<(), String> {
    let from_provided = from.as_ref().is_some_and(|value| !value.trim().is_empty());
    let source_codex = resolve_migration_source_codex(paths, from)?;
    let source_profiles = source_codex.join("profiles");
    let source_index_path = source_profiles.join("profiles.json");

    if source_profiles == paths.profiles {
        if !from_provided {
            let use_color = use_color_stdout();
            let message = format_action(
                "Migration skipped: source and destination are already the same profile storage.",
                use_color,
            );
            let hint = format_hint(
                "To keep separate switcher storage, set CODEX_PROFILES_HOME to another directory and run migrate again.",
                use_color,
            );
            print_output_block(&format!("{message}\n{hint}"));
            return Ok(());
        }
        return Err(
            "Error: source and destination profile directories are the same; nothing to migrate."
                .to_string(),
        );
    }
    if !source_profiles.is_dir() {
        return Err(format!(
            "Error: source profiles directory not found: {}",
            source_profiles.display()
        ));
    }

    let source_paths = Paths {
        codex: source_codex.clone(),
        auth_codex: source_codex.clone(),
        auth: source_codex.join("auth.json"),
        profiles: source_profiles.clone(),
        profiles_index: source_index_path,
        profiles_lock: source_profiles.join("profiles.lock"),
        switcher_config: source_profiles.join("config.toml"),
    };

    let mut source_tokens = BTreeMap::new();
    for path in profile_files(&source_paths.profiles)? {
        let Some(id) = profile_id_from_path(&path) else {
            continue;
        };
        match read_tokens(&path) {
            Ok(tokens) => {
                source_tokens.insert(id, tokens);
            }
            Err(err) => {
                let warning = format_warning(
                    &format!(
                        "Skipping invalid source profile {} ({})",
                        path.display(),
                        normalize_error(&err)
                    ),
                    use_color_stderr(),
                );
                eprintln!("{warning}");
            }
        }
    }
    let source_index = read_profiles_index_relaxed(&source_paths);
    let mut store = ProfileStore::load(paths)?;

    let mut copied = 0usize;
    let mut overwritten = 0usize;
    let mut skipped = 0usize;
    let mut imported_labels = 0usize;
    for id in source_tokens.keys() {
        let source_file = profile_path_for_id(&source_paths.profiles, id);
        if !source_file.is_file() {
            continue;
        }
        let dest_file = profile_path_for_id(&paths.profiles, id);
        let existed_before = dest_file.is_file();
        if dest_file.is_file() && !overwrite {
            skipped += 1;
            continue;
        }
        copy_profile(&source_file, &dest_file, "migrate profile to")?;
        if existed_before && overwrite {
            overwritten += 1;
        } else {
            copied += 1;
        }

        if let Some(src_entry) = source_index.profiles.get(id) {
            let mut entry = src_entry.clone();
            if let Some(label) = src_entry.label.as_deref() {
                let final_label = assign_migrated_label(&mut store.labels, label, id);
                if final_label.is_some() {
                    imported_labels += 1;
                }
                entry.label = final_label;
            }
            let source_last_used = entry.last_used.unwrap_or(0);
            let dest_last_used = store.usage_map.get(id).copied().unwrap_or(0);
            if source_last_used > dest_last_used {
                store.usage_map.insert(id.clone(), source_last_used);
            } else if !store.usage_map.contains_key(id) {
                store.usage_map.insert(id.clone(), source_last_used);
            }
            store.profiles_index.profiles.insert(id.clone(), entry);
        } else {
            store.usage_map.entry(id.clone()).or_insert(0);
            store.profiles_index.profiles.entry(id.clone()).or_default();
        }
    }
    store.save(paths)?;

    let use_color = use_color_stdout();
    let summary = format_action(
        &format!(
            "Migration complete: copied={copied}, overwritten={overwritten}, skipped={skipped}, labels={imported_labels}"
        ),
        use_color,
    );
    let source_line = format!("Source preserved: {}", source_paths.profiles.display());
    let target_line = format!("Destination: {}", paths.profiles.display());
    print_output_block(&format!("{summary}\n{source_line}\n{target_line}"));
    Ok(())
}

fn normalize_source_codex_dir(path: PathBuf) -> PathBuf {
    if path.join("profiles").is_dir() {
        return path;
    }
    let nested = path.join(".codex");
    if nested.join("profiles").is_dir() {
        return nested;
    }
    path
}

fn resolve_migration_source_codex(paths: &Paths, from: Option<String>) -> Result<PathBuf, String> {
    if let Some(source_raw) = from
        .map(PathBuf::from)
        .filter(|path| !path.as_os_str().is_empty())
    {
        return Ok(normalize_source_codex_dir(source_raw));
    }

    let mut candidates = Vec::new();
    push_unique_path(&mut candidates, paths.auth_codex.clone());
    if let Some(path) = default_home_codex_dir() {
        push_unique_path(&mut candidates, path);
    }

    if let Some(source_codex) = pick_migration_source_codex(&candidates, &paths.profiles) {
        return Ok(source_codex);
    }

    Err(format!(
        "Error: could not auto-detect source profiles directory. Use `{} migrate --from <path>`.",
        command_name()
    ))
}

fn pick_migration_source_codex(
    candidates: &[PathBuf],
    destination_profiles: &Path,
) -> Option<PathBuf> {
    for candidate in candidates {
        let normalized = normalize_source_codex_dir(candidate.clone());
        let source_profiles = normalized.join("profiles");
        if source_profiles == destination_profiles {
            continue;
        }
        if source_profiles.is_dir() {
            return Some(normalized);
        }
    }
    None
}

fn push_unique_path(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if path.as_os_str().is_empty() {
        return;
    }
    if paths.iter().any(|existing| existing == &path) {
        return;
    }
    paths.push(path);
}

fn default_home_codex_dir() -> Option<PathBuf> {
    if let Some(base_dirs) = BaseDirs::new() {
        return Some(base_dirs.home_dir().join(".codex"));
    }

    std::env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .filter(|path| !path.as_os_str().is_empty())
        .map(|path| path.join(".codex"))
        .or_else(|| {
            let home = std::env::var_os("HOME")
                .map(PathBuf::from)
                .filter(|path| !path.as_os_str().is_empty())?;
            Some(home.join(".codex"))
        })
}

fn assign_migrated_label(labels: &mut Labels, desired: &str, id: &str) -> Option<String> {
    let trimmed = desired.trim();
    if trimmed.is_empty() {
        return None;
    }
    if labels.get(trimmed).is_some_and(|existing| existing == id) {
        return Some(trimmed.to_string());
    }
    if !labels.contains_key(trimmed) {
        labels.insert(trimmed.to_string(), id.to_string());
        return Some(trimmed.to_string());
    }
    for suffix in 2..10_000 {
        let candidate = format!("{trimmed}-{suffix}");
        if !labels.contains_key(&candidate) {
            labels.insert(candidate.clone(), id.to_string());
            return Some(candidate);
        }
    }
    None
}

pub fn delete_profile(paths: &Paths, yes: bool, label: Option<String>) -> Result<(), String> {
    let use_color_out = use_color_stdout();
    let use_color_err = use_color_stderr();
    let no_profiles = format_no_profiles(paths, use_color_out);
    let (snapshot, ordered) = match load_snapshot_ordered(paths, true, &no_profiles) {
        Ok(result) => result,
        Err(message) => {
            print_output_block(&message);
            return Ok(());
        }
    };

    let candidates = make_candidates(paths, &snapshot, &ordered);
    let selections = pick_many("delete", label.as_deref(), &snapshot, &candidates)?;
    let (selected_ids, displays): (Vec<String>, Vec<String>) = selections
        .iter()
        .map(|item| (item.id.clone(), item.display.clone()))
        .unzip();

    if selected_ids.is_empty() {
        return Ok(());
    }

    let mut store = ProfileStore::load(paths)?;
    if !yes && !confirm_delete_profiles(&displays)? {
        return Err(CANCELLED_MESSAGE.to_string());
    }

    for selected in &selected_ids {
        let target = profile_path_for_id(&paths.profiles, selected);
        if !target.is_file() {
            return Err(profile_not_found(use_color_err));
        }
        fs::remove_file(&target)
            .map_err(|err| format!("Error: failed to delete profile: {err}"))?;
        store.usage_map.remove(selected);
        remove_labels_for_id(&mut store.labels, selected);
        store.profiles_index.profiles.remove(selected);
        if store
            .profiles_index
            .active_profile_id
            .as_deref()
            .is_some_and(|id| id == selected)
        {
            store.profiles_index.active_profile_id = None;
        }
    }
    store.save(paths)?;

    let message = if selected_ids.len() == 1 {
        format!("Deleted profile {}", displays[0])
    } else {
        format!("Deleted {} profiles.", selected_ids.len())
    };
    let message = format_action(&message, use_color_out);
    print_output_block(&message);
    Ok(())
}

pub fn list_profiles(
    paths: &Paths,
    show_usage: bool,
    show_last_used: bool,
    allow_plain_spacing: bool,
    frame_with_separator: bool,
) -> Result<(), String> {
    let snapshot = load_snapshot(paths, false)?;
    let usage_map = &snapshot.usage_map;
    let current_saved_id = current_saved_id(paths, usage_map, &snapshot.tokens);
    let mut ctx = ListCtx::new(paths, show_usage);
    let mut spinner = None;
    if show_usage {
        spinner = Some(start_usage_spinner("Loading profiles"));
        ctx.show_spinner = false;
    }

    let ordered = if show_usage {
        ordered_profiles_by_usage(&snapshot, &ctx, current_saved_id.as_deref())
    } else {
        ordered_profiles(usage_map)
    };
    let current_entry = make_current(
        paths,
        current_saved_id.as_deref(),
        &snapshot.labels,
        &snapshot.tokens,
        &snapshot.usage_map,
        &ctx,
    );
    let separator = separator_line(2);
    let frame_separator = if frame_with_separator {
        separator_line(0)
    } else {
        None
    };
    let has_saved = !ordered.is_empty();
    if !has_saved {
        if let Some(spinner) = spinner {
            stop_usage_spinner(spinner);
        }
        if let Some(entry) = current_entry {
            let lines = render_entries(&[entry], show_last_used, &ctx, None, false);
            print_output_block(&lines.join("\n"));
        } else {
            let message = format_no_profiles(paths, ctx.use_color);
            print_output_block(&message);
        }
        return Ok(());
    }

    let filtered: Vec<(String, u64)> = ordered
        .into_iter()
        .filter(|(id, _)| current_saved_id.as_deref() != Some(id.as_str()))
        .collect();
    let list_entries = make_entries(&filtered, &snapshot, None, &ctx);

    if let Some(spinner) = spinner {
        stop_usage_spinner(spinner);
    }

    let mut lines = Vec::new();
    if let Some(entry) = current_entry {
        lines.extend(render_entries(
            &[entry],
            show_last_used,
            &ctx,
            separator.as_deref(),
            allow_plain_spacing,
        ));
        if !list_entries.is_empty() {
            push_separator(&mut lines, separator.as_deref(), allow_plain_spacing);
        }
    }
    lines.extend(render_entries(
        &list_entries,
        show_last_used,
        &ctx,
        separator.as_deref(),
        allow_plain_spacing,
    ));
    let output = lines.join("\n");
    if frame_with_separator
        && !is_plain()
        && let Some(frame_separator) = frame_separator.as_ref()
    {
        print_output_block_with_frame(&output, frame_separator);
        return Ok(());
    }
    print_output_block(&output);
    Ok(())
}

pub fn status_profiles(paths: &Paths, all: bool) -> Result<(), String> {
    if all {
        let use_color = use_color_stdout();
        let no_profiles = format_no_profiles(paths, use_color);
        let snapshot = load_snapshot(paths, false)?;
        if snapshot.usage_map.is_empty() {
            print_output_block(&no_profiles);
            return Ok(());
        }
        let current_saved = current_saved_id(paths, &snapshot.usage_map, &snapshot.tokens);
        let rows = priority_rows(paths, &snapshot, current_saved.as_deref(), true);
        let table = render_priority_table(&rows, use_color);
        print_output_block(&table);
        return Ok(());
    }
    let snapshot = load_snapshot(paths, false).ok();
    let current_saved_id = snapshot
        .as_ref()
        .and_then(|snap| current_saved_id(paths, &snap.usage_map, &snap.tokens));
    let mut ctx = ListCtx::new(paths, true);
    let spinner = start_usage_spinner("Loading profile");
    ctx.show_spinner = false;
    let empty_labels = Labels::new();
    let labels = snapshot
        .as_ref()
        .map(|snap| &snap.labels)
        .unwrap_or(&empty_labels);
    let empty_tokens = BTreeMap::new();
    let empty_usage = BTreeMap::new();
    let tokens_map = snapshot
        .as_ref()
        .map(|snap| &snap.tokens)
        .unwrap_or(&empty_tokens);
    let usage_map = snapshot
        .as_ref()
        .map(|snap| &snap.usage_map)
        .unwrap_or(&empty_usage);
    let current_entry = make_current(
        paths,
        current_saved_id.as_deref(),
        labels,
        tokens_map,
        usage_map,
        &ctx,
    );
    stop_usage_spinner(spinner);
    if let Some(entry) = current_entry {
        let lines = render_entries(&[entry], true, &ctx, None, false);
        print_output_block(&lines.join("\n"));
    } else {
        let message = format_no_profiles(paths, ctx.use_color);
        print_output_block(&message);
    }
    Ok(())
}

pub fn status_label(paths: &Paths, label: &str) -> Result<(), String> {
    let snapshot = load_snapshot(paths, false)?;
    let id = resolve_label_id(&snapshot.labels, label)?;
    let current_saved_id = current_saved_id(paths, &snapshot.usage_map, &snapshot.tokens);
    let mut ctx = ListCtx::new(paths, true);
    let spinner = start_usage_spinner("Loading profile");
    ctx.show_spinner = false;
    let separator = separator_line(2);
    let is_current = current_saved_id.as_deref() == Some(id.as_str());
    let last_used = if is_current {
        String::new()
    } else {
        snapshot
            .usage_map
            .get(&id)
            .copied()
            .map(format_last_used)
            .unwrap_or_default()
    };
    let label = label_for_id(&snapshot.labels, &id);
    let profile_path = ctx.profiles_dir.join(format!("{id}.json"));
    let entry = make_entry(
        last_used,
        label,
        snapshot.tokens.get(&id),
        snapshot.index.profiles.get(&id),
        &profile_path,
        &ctx,
        is_current,
    );
    stop_usage_spinner(spinner);
    let lines = render_entries(&[entry], true, &ctx, separator.as_deref(), true);
    print_output_block(&lines.join("\n"));
    Ok(())
}

pub fn sync_current_readonly(paths: &Paths) -> Result<(), String> {
    if !paths.auth.is_file() {
        return Ok(());
    }
    let snapshot = match load_snapshot(paths, false) {
        Ok(snapshot) => snapshot,
        Err(_) => return Ok(()),
    };
    let Some(id) = current_saved_id(paths, &snapshot.usage_map, &snapshot.tokens) else {
        return Ok(());
    };
    let target = profile_path_for_id(&paths.profiles, &id);
    if !target.is_file() {
        return Ok(());
    }
    sync_profile(paths, &target)?;
    Ok(())
}

pub type Labels = BTreeMap<String, String>;

const PROFILES_INDEX_VERSION: u8 = 1;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ProfilesIndex {
    #[serde(default = "profiles_index_version")]
    version: u8,
    #[serde(default)]
    active_profile_id: Option<String>,
    #[serde(default)]
    profiles: BTreeMap<String, ProfileIndexEntry>,
    #[serde(default)]
    pub(crate) update_cache: Option<UpdateCache>,
}

impl Default for ProfilesIndex {
    fn default() -> Self {
        Self {
            version: PROFILES_INDEX_VERSION,
            active_profile_id: None,
            profiles: BTreeMap::new(),
            update_cache: None,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct ProfileIndexEntry {
    #[serde(default)]
    account_id: Option<String>,
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    plan: Option<String>,
    #[serde(default)]
    label: Option<String>,
    #[serde(default)]
    added_at: u64,
    #[serde(default)]
    last_used: Option<u64>,
    #[serde(default)]
    is_api_key: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct UpdateCache {
    #[serde(default)]
    pub(crate) latest_version: String,
    #[serde(default = "update_cache_checked_default")]
    pub(crate) last_checked_at: DateTime<Utc>,
    #[serde(default)]
    pub(crate) dismissed_version: Option<String>,
    #[serde(default)]
    pub(crate) last_prompted_at: Option<DateTime<Utc>>,
}

fn update_cache_checked_default() -> DateTime<Utc> {
    DateTime::<Utc>::from_timestamp(0, 0).unwrap_or_else(Utc::now)
}

fn profiles_index_version() -> u8 {
    PROFILES_INDEX_VERSION
}

pub(crate) fn read_profiles_index(paths: &Paths) -> Result<ProfilesIndex, String> {
    if !paths.profiles_index.exists() {
        return Ok(ProfilesIndex::default());
    }
    let contents = fs::read_to_string(&paths.profiles_index).map_err(|err| {
        format!(
            "Error: cannot read profiles index file {}: {err}",
            paths.profiles_index.display()
        )
    })?;
    let index: ProfilesIndex = serde_json::from_str(&contents).map_err(|_| {
        format!(
            "Error: profiles index file {} is invalid JSON",
            paths.profiles_index.display()
        )
    })?;
    Ok(index)
}

pub(crate) fn read_profiles_index_relaxed(paths: &Paths) -> ProfilesIndex {
    match read_profiles_index(paths) {
        Ok(index) => index,
        Err(err) => {
            let normalized = normalize_error(&err);
            let warning = format_warning(&normalized, use_color_stderr());
            eprintln!("{warning}");
            ProfilesIndex::default()
        }
    }
}

pub(crate) fn write_profiles_index(paths: &Paths, index: &ProfilesIndex) -> Result<(), String> {
    let json = serde_json::to_string_pretty(index)
        .map_err(|err| format!("Error: failed to serialize profiles index: {err}"))?;
    write_atomic(&paths.profiles_index, format!("{json}\n").as_bytes())
        .map_err(|err| format!("Error: failed to write profiles index file: {err}"))
}

fn prune_profiles_index(index: &mut ProfilesIndex, profiles_dir: &Path) -> Result<(), String> {
    let ids = collect_profile_ids(profiles_dir)?;
    index.profiles.retain(|id, _| ids.contains(id));
    if index
        .active_profile_id
        .as_deref()
        .is_some_and(|id| !ids.contains(id))
    {
        index.active_profile_id = None;
    }
    Ok(())
}

fn sync_profiles_index(
    index: &mut ProfilesIndex,
    usage_map: &BTreeMap<String, u64>,
    labels: &Labels,
) {
    for (id, entry) in index.profiles.iter_mut() {
        entry.last_used = usage_map.get(id).copied();
        entry.label = label_for_id(labels, id);
    }
}

fn labels_from_index(index: &ProfilesIndex) -> Labels {
    let mut labels = Labels::new();
    for (id, entry) in &index.profiles {
        let Some(label) = entry.label.as_deref() else {
            continue;
        };
        let trimmed = label.trim();
        if trimmed.is_empty() || labels.contains_key(trimmed) {
            continue;
        }
        labels.insert(trimmed.to_string(), id.clone());
    }
    labels
}

fn usage_map_from_index(index: &ProfilesIndex, ids: &HashSet<String>) -> BTreeMap<String, u64> {
    let mut usage_map = BTreeMap::new();
    for id in ids {
        usage_map.insert(id.clone(), 0);
    }
    for (id, entry) in &index.profiles {
        if !ids.contains(id) {
            continue;
        }
        let Some(last_used) = entry.last_used else {
            continue;
        };
        let current = usage_map.entry(id.clone()).or_insert(0);
        if last_used > *current {
            *current = last_used;
        }
    }
    usage_map
}

fn update_profiles_index_entry(
    index: &mut ProfilesIndex,
    id: &str,
    tokens: Option<&Tokens>,
    label: Option<String>,
    now: u64,
    set_active: bool,
) {
    let entry = index.profiles.entry(id.to_string()).or_default();
    if entry.added_at == 0 {
        entry.added_at = now;
    }
    if let Some(tokens) = tokens {
        let (email, plan) = extract_email_and_plan(tokens);
        entry.email = email;
        entry.plan = plan;
        entry.account_id = token_account_id(tokens).map(str::to_string);
        entry.is_api_key = is_api_key_profile(tokens);
    }
    if let Some(label) = label {
        entry.label = Some(label);
    }
    entry.last_used = Some(now);
    if set_active {
        index.active_profile_id = Some(id.to_string());
    }
}

pub fn read_labels(paths: &Paths) -> Result<Labels, String> {
    let index = read_profiles_index(paths)?;
    Ok(labels_from_index(&index))
}

pub fn write_labels(paths: &Paths, labels: &Labels) -> Result<(), String> {
    let normalized = normalize_labels(labels);
    let mut index = read_profiles_index_relaxed(paths);
    for (id, entry) in index.profiles.iter_mut() {
        entry.label = label_for_id(&normalized, id);
    }
    for (label, id) in &normalized {
        index.profiles.entry(id.clone()).or_default().label = Some(label.clone());
    }
    write_profiles_index(paths, &index)
}

pub fn prune_labels(labels: &mut Labels, profiles_dir: &Path) {
    labels.retain(|_, id| profile_path_for_id(profiles_dir, id).is_file());
}

pub fn assign_label(labels: &mut Labels, label: &str, id: &str) -> Result<(), String> {
    let trimmed = trim_label(label)?;
    if let Some(existing) = labels.get(trimmed) {
        if existing == id {
            return Ok(());
        }
        return Err(format!(
            "Error: label '{trimmed}' already exists. {}",
            format_list_hint(use_color_stderr())
        ));
    }
    labels.insert(trimmed.to_string(), id.to_string());
    Ok(())
}

pub fn remove_labels_for_id(labels: &mut Labels, id: &str) {
    labels.retain(|_, value| value != id);
}

pub fn label_for_id(labels: &Labels, id: &str) -> Option<String> {
    labels.iter().find_map(|(label, value)| {
        if value == id {
            Some(label.clone())
        } else {
            None
        }
    })
}

pub fn resolve_label_id(labels: &Labels, label: &str) -> Result<String, String> {
    let trimmed = trim_label(label)?;
    labels.get(trimmed).cloned().ok_or_else(|| {
        format!(
            "Error: label '{trimmed}' was not found. {}",
            format_list_hint(use_color_stderr())
        )
    })
}

pub fn profile_files(profiles_dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    if !profiles_dir.exists() {
        return Ok(files);
    }
    let entries = fs::read_dir(profiles_dir)
        .map_err(|err| format!("Error: cannot read profiles directory: {err}"))?;
    for entry in entries {
        let entry = entry.map_err(|err| format!("Error: cannot read profiles directory: {err}"))?;
        let path = entry.path();
        if !is_profile_file(&path) {
            continue;
        }
        files.push(path);
    }
    Ok(files)
}

pub fn profile_id_from_path(path: &Path) -> Option<String> {
    path.file_stem()
        .and_then(|value| value.to_str())
        .filter(|stem| !stem.is_empty())
        .map(|stem| stem.to_string())
}

pub fn profile_path_for_id(profiles_dir: &Path, id: &str) -> PathBuf {
    profiles_dir.join(format!("{id}.json"))
}

pub fn collect_profile_ids(profiles_dir: &Path) -> Result<HashSet<String>, String> {
    let mut ids = HashSet::new();
    for path in profile_files(profiles_dir)? {
        if let Some(stem) = profile_id_from_path(&path) {
            ids.insert(stem);
        }
    }
    Ok(ids)
}

pub fn load_profile_tokens_map(
    paths: &Paths,
) -> Result<BTreeMap<String, Result<Tokens, String>>, String> {
    let mut map = BTreeMap::new();
    let mut removed_ids: Vec<String> = Vec::new();
    for path in profile_files(&paths.profiles)? {
        let Some(stem) = profile_id_from_path(&path) else {
            continue;
        };
        match read_tokens(&path) {
            Ok(tokens) => {
                map.insert(stem, Ok(tokens));
            }
            Err(err) => {
                let id = stem.clone();
                if let Err(remove_err) = fs::remove_file(&path) {
                    let message = format!(
                        "Error: failed to remove invalid profile {}: {remove_err}",
                        path.display()
                    );
                    map.insert(id, Err(message));
                } else {
                    removed_ids.push(id);
                    let summary = normalize_error(&err);
                    eprintln!(
                        "{}",
                        format_warning(
                            &format!("Removed invalid profile {} ({summary})", path.display()),
                            use_color_stderr()
                        )
                    );
                }
            }
        }
    }
    if !removed_ids.is_empty() {
        let mut index = read_profiles_index_relaxed(paths);
        for id in &removed_ids {
            index.profiles.remove(id);
            if index
                .active_profile_id
                .as_deref()
                .is_some_and(|active| active == id)
            {
                index.active_profile_id = None;
            }
        }
        let _ = write_profiles_index(paths, &index);
    }
    Ok(map)
}

pub(crate) fn resolve_save_id(
    paths: &Paths,
    map: &mut BTreeMap<String, u64>,
    labels: &mut Labels,
    profiles_index: &mut ProfilesIndex,
    tokens: &Tokens,
) -> Result<String, String> {
    let (account_id, email, plan) = require_identity(tokens)?;
    let (desired_base, desired, candidates) =
        desired_candidates(paths, &account_id, &email, &plan)?;
    if has_usage_signal(&candidates, map)
        && let Some(primary) = pick_primary(&candidates, map).filter(|primary| primary != &desired)
    {
        return rename_profile_id(
            paths,
            map,
            labels,
            profiles_index,
            &primary,
            &desired_base,
            &account_id,
        );
    }
    Ok(desired)
}

pub(crate) fn resolve_sync_id(
    paths: &Paths,
    map: &mut BTreeMap<String, u64>,
    labels: &mut Labels,
    profiles_index: &mut ProfilesIndex,
    tokens: &Tokens,
) -> Result<Option<String>, String> {
    let Ok((account_id, email, plan)) = require_identity(tokens) else {
        return Ok(None);
    };
    let (desired_base, desired, candidates) =
        desired_candidates(paths, &account_id, &email, &plan)?;
    if !has_usage_signal(&candidates, map) {
        if candidates.len() == 1 {
            return Ok(candidates.first().cloned());
        }
        if candidates.iter().any(|id| id == &desired) {
            return Ok(Some(desired));
        }
        return Ok(None);
    }
    let Some(primary) = pick_primary(&candidates, map) else {
        return Ok(None);
    };
    if primary != desired {
        let renamed = rename_profile_id(
            paths,
            map,
            labels,
            profiles_index,
            &primary,
            &desired_base,
            &account_id,
        )?;
        return Ok(Some(renamed));
    }
    Ok(Some(primary))
}

pub(crate) fn cached_profile_ids(
    tokens_map: &BTreeMap<String, Result<Tokens, String>>,
    account_id: &str,
    email: Option<&str>,
) -> Vec<String> {
    tokens_map
        .iter()
        .filter_map(|(id, result)| {
            result
                .as_ref()
                .ok()
                .filter(|tokens| matches_account(tokens, account_id, email))
                .map(|_| id.clone())
        })
        .collect()
}

pub(crate) fn pick_primary(
    candidates: &[String],
    usage_map: &BTreeMap<String, u64>,
) -> Option<String> {
    let mut best: Option<(String, u64)> = None;
    for candidate in candidates {
        if let Some(ts) = usage_map.get(candidate).filter(|ts| {
            best.as_ref()
                .map(|(_, best_ts)| *ts > best_ts)
                .unwrap_or(true)
        }) {
            best = Some((candidate.clone(), *ts));
        }
    }
    best.map(|(id, _)| id)
}

fn has_usage_signal(candidates: &[String], usage_map: &BTreeMap<String, u64>) -> bool {
    candidates
        .iter()
        .any(|id| usage_map.get(id).copied().unwrap_or(0) > 0)
}

fn desired_candidates(
    paths: &Paths,
    account_id: &str,
    email: &str,
    plan: &str,
) -> Result<(String, String, Vec<String>), String> {
    let (desired_base, desired) = desired_id(paths, account_id, email, plan);
    let candidates = scan_profile_ids(&paths.profiles, account_id, Some(email))?;
    Ok((desired_base, desired, candidates))
}

fn desired_id(paths: &Paths, account_id: &str, email: &str, plan: &str) -> (String, String) {
    let desired_base = profile_base(email, plan);
    let desired = unique_id(&desired_base, account_id, &paths.profiles);
    (desired_base, desired)
}

fn profile_base(email: &str, plan_label: &str) -> String {
    let email = sanitize_part(email);
    let plan = sanitize_part(plan_label);
    let email = if email.is_empty() {
        "unknown".to_string()
    } else {
        email
    };
    let plan = if plan.is_empty() {
        "unknown".to_string()
    } else {
        plan
    };
    format!("{email}-{plan}")
}

fn sanitize_part(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut last_dash = false;
    for ch in value.chars() {
        let next = if ch.is_ascii_alphanumeric() {
            Some(ch.to_ascii_lowercase())
        } else if matches!(ch, '@' | '.' | '-' | '_' | '+') {
            Some(ch)
        } else {
            Some('-')
        };
        if let Some(next) = next {
            if next == '-' {
                if last_dash {
                    continue;
                }
                last_dash = true;
            } else {
                last_dash = false;
            }
            out.push(next);
        }
    }
    out.trim_matches('-').to_string()
}

fn unique_id(base: &str, account_id: &str, profiles_dir: &Path) -> String {
    let mut candidate = base.to_string();
    let suffix = short_account_suffix(account_id);
    let mut attempts = 0usize;
    loop {
        let path = profile_path_for_id(profiles_dir, &candidate);
        if !path.is_file() {
            return candidate;
        }
        if read_tokens(&path)
            .ok()
            .is_some_and(|tokens| token_account_id(&tokens) == Some(account_id))
        {
            return candidate;
        }
        attempts += 1;
        if attempts == 1 {
            candidate = format!("{base}-{suffix}");
        } else {
            candidate = format!("{base}-{suffix}-{attempts}");
        }
    }
}

fn short_account_suffix(account_id: &str) -> String {
    account_id.chars().take(6).collect()
}

fn scan_profile_ids(
    profiles_dir: &Path,
    account_id: &str,
    email: Option<&str>,
) -> Result<Vec<String>, String> {
    let mut matches = Vec::new();
    for path in profile_files(profiles_dir)? {
        let Ok(tokens) = read_tokens(&path) else {
            continue;
        };
        if !matches_account(&tokens, account_id, email) {
            continue;
        }
        if let Some(stem) = profile_id_from_path(&path) {
            matches.push(stem);
        }
    }
    Ok(matches)
}

fn matches_account(tokens: &Tokens, account_id: &str, email: Option<&str>) -> bool {
    if token_account_id(tokens) != Some(account_id) {
        return false;
    }
    if let Some(expected) = email {
        let token_email = extract_email_and_plan(tokens).0;
        if token_email.as_deref() != Some(expected) {
            return false;
        }
    }
    true
}

fn rename_profile_id(
    paths: &Paths,
    map: &mut BTreeMap<String, u64>,
    labels: &mut Labels,
    profiles_index: &mut ProfilesIndex,
    from: &str,
    target_base: &str,
    account_id: &str,
) -> Result<String, String> {
    let desired = unique_id(target_base, account_id, &paths.profiles);
    if from == desired {
        return Ok(desired);
    }
    let from_path = profile_path_for_id(&paths.profiles, from);
    let to_path = profile_path_for_id(&paths.profiles, &desired);
    if !from_path.is_file() {
        return Err(format!("Profile {from} not found"));
    }
    fs::rename(&from_path, &to_path)
        .map_err(|err| format!("Error: failed to rename profile {from}: {err}"))?;
    if let Some(ts) = map.remove(from) {
        map.insert(desired.clone(), ts);
    }
    labels.retain(|_, value| value != from);
    if let Some(entry) = profiles_index.profiles.remove(from) {
        profiles_index.profiles.insert(desired.clone(), entry);
    }
    if profiles_index
        .active_profile_id
        .as_deref()
        .is_some_and(|id| id == from)
    {
        profiles_index.active_profile_id = Some(desired.clone());
    }
    Ok(desired)
}

pub(crate) struct Snapshot {
    pub(crate) usage_map: BTreeMap<String, u64>,
    pub(crate) labels: Labels,
    pub(crate) tokens: BTreeMap<String, Result<Tokens, String>>,
    pub(crate) index: ProfilesIndex,
}

pub(crate) fn sync_current(
    paths: &Paths,
    map: &mut BTreeMap<String, u64>,
    labels: &mut Labels,
    index: &mut ProfilesIndex,
) -> Result<(), String> {
    let Some(tokens) = read_tokens_opt(&paths.auth) else {
        return Ok(());
    };
    let id = match resolve_sync_id(paths, map, labels, index, &tokens)? {
        Some(id) => id,
        None => return Ok(()),
    };
    let target = profile_path_for_id(&paths.profiles, &id);
    sync_profile(paths, &target)?;
    let now = now_seconds();
    map.insert(id.clone(), now);
    let label = label_for_id(labels, &id);
    update_profiles_index_entry(index, &id, Some(&tokens), label, now, true);
    Ok(())
}

fn sync_profile(paths: &Paths, target: &Path) -> Result<(), String> {
    copy_atomic(&paths.auth, target)
        .map_err(|err| format!("Error: failed to sync current profile: {err}"))?;
    Ok(())
}

pub(crate) fn load_snapshot(paths: &Paths, strict_labels: bool) -> Result<Snapshot, String> {
    let _lock = lock_usage(paths)?;
    let tokens = load_profile_tokens_map(paths)?;
    let ids: HashSet<String> = tokens.keys().cloned().collect();
    let mut index = if strict_labels {
        read_profiles_index(paths)?
    } else {
        read_profiles_index_relaxed(paths)
    };
    let _ = prune_profiles_index(&mut index, &paths.profiles);
    for id in &ids {
        index.profiles.entry(id.clone()).or_default();
    }
    let usage_map = usage_map_from_index(&index, &ids);
    let labels = labels_from_index(&index);

    Ok(Snapshot {
        usage_map,
        labels,
        tokens,
        index,
    })
}

pub(crate) fn unsaved_reason(
    paths: &Paths,
    tokens_map: &BTreeMap<String, Result<Tokens, String>>,
) -> Result<Option<String>, String> {
    let Some(tokens) = read_tokens_opt(&paths.auth) else {
        return Ok(None);
    };
    let Some(account_id) = token_account_id(&tokens) else {
        return Ok(None);
    };
    let (email, _) = extract_email_and_plan(&tokens);
    let Some(email) = email else {
        return Ok(None);
    };

    let candidates = cached_profile_ids(tokens_map, account_id, Some(&email));
    if candidates.is_empty() {
        return Ok(Some("no saved profile matches auth.json".to_string()));
    }
    Ok(None)
}

pub(crate) fn current_saved_id(
    paths: &Paths,
    usage_map: &BTreeMap<String, u64>,
    tokens_map: &BTreeMap<String, Result<Tokens, String>>,
) -> Option<String> {
    let tokens = read_tokens_opt(&paths.auth)?;
    let account_id = token_account_id(&tokens)?;
    let (email, _) = extract_email_and_plan(&tokens);
    let email = email.as_deref()?;
    let candidates = cached_profile_ids(tokens_map, account_id, Some(email));
    pick_primary(&candidates, usage_map)
}

pub(crate) struct ProfileStore {
    _lock: UsageLock,
    pub(crate) usage_map: BTreeMap<String, u64>,
    pub(crate) labels: Labels,
    pub(crate) profiles_index: ProfilesIndex,
}

impl ProfileStore {
    pub(crate) fn load(paths: &Paths) -> Result<Self, String> {
        let lock = lock_usage(paths)?;
        let mut profiles_index = read_profiles_index_relaxed(paths);
        let _ = prune_profiles_index(&mut profiles_index, &paths.profiles);
        let ids = collect_profile_ids(&paths.profiles)?;
        for id in &ids {
            profiles_index.profiles.entry(id.clone()).or_default();
        }
        let usage_map = usage_map_from_index(&profiles_index, &ids);
        let labels = labels_from_index(&profiles_index);
        Ok(Self {
            _lock: lock,
            usage_map,
            labels,
            profiles_index,
        })
    }

    pub(crate) fn save(&mut self, paths: &Paths) -> Result<(), String> {
        prune_labels(&mut self.labels, &paths.profiles);
        prune_profiles_index(&mut self.profiles_index, &paths.profiles)?;
        sync_profiles_index(&mut self.profiles_index, &self.usage_map, &self.labels);
        write_profiles_index(paths, &self.profiles_index)?;
        Ok(())
    }
}

fn profile_not_found(use_color: bool) -> String {
    format!(
        "Selected profile not found. {}",
        format_list_hint(use_color)
    )
}

fn load_snapshot_ordered(
    paths: &Paths,
    strict_labels: bool,
    no_profiles_message: &str,
) -> Result<(Snapshot, Vec<(String, u64)>), String> {
    let snapshot = load_snapshot(paths, strict_labels)?;
    let ordered = ordered_profiles(&snapshot.usage_map);
    if ordered.is_empty() {
        return Err(no_profiles_message.to_string());
    }
    Ok((snapshot, ordered))
}

fn copy_profile(source: &Path, dest: &Path, context: &str) -> Result<(), String> {
    copy_atomic(source, dest)
        .map_err(|err| format!("Error: failed to {context} {}: {err}", dest.display()))?;
    Ok(())
}

fn make_candidates(
    paths: &Paths,
    snapshot: &Snapshot,
    ordered: &[(String, u64)],
) -> Vec<Candidate> {
    let current_saved = current_saved_id(paths, &snapshot.usage_map, &snapshot.tokens);
    build_candidates(ordered, snapshot, current_saved.as_deref())
}

fn pick_one(
    action: &str,
    label: Option<&str>,
    snapshot: &Snapshot,
    candidates: &[Candidate],
) -> Result<Candidate, String> {
    if let Some(label) = label {
        select_by_label(label, &snapshot.labels, candidates)
    } else {
        require_tty(action)?;
        select_single_profile("", candidates)
    }
}

fn pick_many(
    action: &str,
    label: Option<&str>,
    snapshot: &Snapshot,
    candidates: &[Candidate],
) -> Result<Vec<Candidate>, String> {
    if let Some(label) = label {
        Ok(vec![select_by_label(label, &snapshot.labels, candidates)?])
    } else {
        require_tty(action)?;
        select_multiple_profiles("", candidates)
    }
}

pub(crate) struct ProfileInfo {
    pub(crate) display: String,
    pub(crate) email: Option<String>,
    pub(crate) plan: Option<String>,
    pub(crate) is_free: bool,
}

pub(crate) fn profile_info(
    tokens: Option<&Tokens>,
    label: Option<String>,
    is_current: bool,
    use_color: bool,
) -> ProfileInfo {
    profile_info_with_fallback(tokens, None, label, is_current, use_color)
}

fn profile_info_with_fallback(
    tokens: Option<&Tokens>,
    fallback: Option<&ProfileIndexEntry>,
    label: Option<String>,
    is_current: bool,
    use_color: bool,
) -> ProfileInfo {
    let (email, plan) = if let Some(tokens) = tokens {
        extract_email_and_plan(tokens)
    } else if let Some(entry) = fallback {
        (entry.email.clone(), entry.plan.clone())
    } else {
        (None, None)
    };
    let is_free = is_free_plan(plan.as_deref());
    let display =
        crate::format_profile_display(email.clone(), plan.clone(), label, is_current, use_color);
    ProfileInfo {
        display,
        email,
        plan,
        is_free,
    }
}

#[derive(Debug)]
pub(crate) enum LoadChoice {
    SaveAndContinue,
    ContinueWithoutSaving,
    Cancel,
}

pub(crate) fn prompt_unsaved_load(paths: &Paths, reason: &str) -> Result<LoadChoice, String> {
    let is_tty = io::stdin().is_terminal();
    if !is_tty {
        let hint = format_save_before_load(paths, use_color_stderr());
        return Err(format!("Error: current profile is not saved. {hint}"));
    }
    let selection = Select::new(
        "",
        vec![
            "Save current profile and continue",
            "Continue without saving",
            "Cancel",
        ],
    )
    .with_render_config(inquire_select_render_config())
    .prompt();
    prompt_unsaved_load_with(paths, reason, is_tty, selection)
}

fn prompt_unsaved_load_with(
    paths: &Paths,
    reason: &str,
    is_tty: bool,
    selection: Result<&str, inquire::error::InquireError>,
) -> Result<LoadChoice, String> {
    if !is_tty {
        let hint = format_save_before_load(paths, use_color_stderr());
        return Err(format!("Error: current profile is not saved. {hint}"));
    }
    let warning = format_warning(
        &format!("Current profile is not saved ({reason})."),
        use_color_stderr(),
    );
    eprintln!("{warning}");
    match selection {
        Ok("Save current profile and continue") => Ok(LoadChoice::SaveAndContinue),
        Ok("Continue without saving") => Ok(LoadChoice::ContinueWithoutSaving),
        Ok(_) => Ok(LoadChoice::Cancel),
        Err(err) if is_inquire_cancel(&err) => Ok(LoadChoice::Cancel),
        Err(err) => Err(format!("Error: failed to prompt for load: {err}")),
    }
}

pub(crate) fn build_candidates(
    ordered: &[(String, u64)],
    snapshot: &Snapshot,
    current_saved_id: Option<&str>,
) -> Vec<Candidate> {
    let mut candidates = Vec::with_capacity(ordered.len());
    let use_color = use_color_stderr();
    for (id, ts) in ordered {
        let label = label_for_id(&snapshot.labels, id);
        let tokens = snapshot
            .tokens
            .get(id)
            .and_then(|result| result.as_ref().ok());
        let index_entry = snapshot.index.profiles.get(id);
        let is_current = current_saved_id == Some(id.as_str());
        let info = profile_info_with_fallback(tokens, index_entry, label, is_current, use_color);
        let last_used = if is_current {
            String::new()
        } else {
            format_last_used(*ts)
        };
        candidates.push(Candidate {
            id: id.clone(),
            display: info.display,
            last_used,
            is_current,
        });
    }
    candidates
}

pub(crate) fn require_tty(action: &str) -> Result<(), String> {
    require_tty_with(io::stdin().is_terminal(), action)
}

fn require_tty_with(is_tty: bool, action: &str) -> Result<(), String> {
    if is_tty {
        Ok(())
    } else {
        Err(format!(
            "Error: {action} selection requires a TTY. Run `{} {action}` interactively.",
            command_name()
        ))
    }
}

pub(crate) fn select_single_profile(
    title: &str,
    candidates: &[Candidate],
) -> Result<Candidate, String> {
    let options = candidates.to_vec();
    let render_config = inquire_select_render_config();
    let prompt = Select::new(title, options)
        .with_help_message(LOAD_HELP)
        .with_render_config(render_config)
        .prompt();
    handle_inquire_result(prompt, "selection")
}

pub(crate) fn select_multiple_profiles(
    title: &str,
    candidates: &[Candidate],
) -> Result<Vec<Candidate>, String> {
    let options = candidates.to_vec();
    let render_config = inquire_select_render_config();
    let prompt = MultiSelect::new(title, options)
        .with_help_message(DELETE_HELP)
        .with_render_config(render_config)
        .prompt();
    let selections = handle_inquire_result(prompt, "selection")?;
    if selections.is_empty() {
        return Err(CANCELLED_MESSAGE.to_string());
    }
    Ok(selections)
}

pub(crate) fn select_by_label(
    label: &str,
    labels: &Labels,
    candidates: &[Candidate],
) -> Result<Candidate, String> {
    let id = resolve_label_id(labels, label)?;
    let Some(candidate) = candidates.iter().find(|candidate| candidate.id == id) else {
        return Err(format!(
            "Error: label '{label}' does not match a saved profile. {}",
            format_list_hint(use_color_stderr())
        ));
    };
    Ok(candidate.clone())
}

pub(crate) fn confirm_delete_profiles(displays: &[String]) -> Result<bool, String> {
    let is_tty = io::stdin().is_terminal();
    if !is_tty {
        return Err(
            "Error: deletion requires confirmation. Re-run with `--yes` to skip the prompt."
                .to_string(),
        );
    }
    let prompt = if displays.len() == 1 {
        format!("Delete profile {}? This cannot be undone.", displays[0])
    } else {
        let count = displays.len();
        eprintln!("Delete {count} profiles? This cannot be undone.");
        for display in displays {
            eprintln!(" - {display}");
        }
        "Delete selected profiles? This cannot be undone.".to_string()
    };
    let selection = Confirm::new(&prompt)
        .with_default(false)
        .with_render_config(inquire_select_render_config())
        .prompt();
    confirm_delete_profiles_with(is_tty, selection)
}

fn confirm_delete_profiles_with(
    is_tty: bool,
    selection: Result<bool, inquire::error::InquireError>,
) -> Result<bool, String> {
    if !is_tty {
        return Err(
            "Error: deletion requires confirmation. Re-run with `--yes` to skip the prompt."
                .to_string(),
        );
    }
    match selection {
        Ok(value) => Ok(value),
        Err(err) if is_inquire_cancel(&err) => Err(CANCELLED_MESSAGE.to_string()),
        Err(err) => Err(format!("Error: failed to prompt for delete: {err}")),
    }
}

#[derive(Clone)]
pub(crate) struct Candidate {
    pub(crate) id: String,
    pub(crate) display: String,
    pub(crate) last_used: String,
    pub(crate) is_current: bool,
}

impl fmt::Display for Candidate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let header = format_entry_header(
            &self.display,
            &self.last_used,
            self.is_current,
            use_color_stderr(),
        );
        write!(f, "{header}")
    }
}

fn render_entries(
    entries: &[Entry],
    show_last_used: bool,
    ctx: &ListCtx,
    separator: Option<&str>,
    allow_plain_spacing: bool,
) -> Vec<String> {
    let mut lines = Vec::with_capacity((entries.len().max(1)) * 4);
    for (idx, entry) in entries.iter().enumerate() {
        let header = format_entry_header(
            &entry.display,
            if show_last_used { &entry.last_used } else { "" },
            entry.is_current,
            ctx.use_color,
        );
        let show_detail_lines = ctx.show_usage || entry.always_show_details;
        if !show_detail_lines {
            if let Some(err) = entry.error_summary.as_deref() {
                let mut header = header;
                header.push_str(&format!("  {err}"));
                lines.push(header);
            } else {
                lines.push(header);
            }
        } else {
            lines.push(header);
            lines.extend(entry.details.iter().cloned());
        }
        if idx + 1 < entries.len() {
            push_separator(&mut lines, separator, allow_plain_spacing);
        }
    }
    lines
}

fn push_separator(lines: &mut Vec<String>, separator: Option<&str>, allow_plain_spacing: bool) {
    match separator {
        Some(value) => lines.push(value.to_string()),
        None => {
            if !is_plain() || allow_plain_spacing {
                lines.push(String::new());
            }
        }
    }
}

fn separator_line(trim: usize) -> Option<String> {
    if is_plain() {
        return None;
    }
    let width = terminal_width()?;
    let len = width.saturating_sub(trim);
    if len == 0 {
        return None;
    }
    let line = "-".repeat(len);
    Some(style_text(&line, use_color_stdout(), |text| text.dimmed()))
}

fn make_error(
    label: Option<String>,
    index_entry: Option<&ProfileIndexEntry>,
    use_color: bool,
    last_used: String,
    message: &str,
    summary_label: &str,
    is_current: bool,
) -> Entry {
    let display =
        profile_info_with_fallback(None, index_entry, label, is_current, use_color).display;
    Entry {
        display,
        last_used,
        details: vec![format_error(message)],
        error_summary: Some(error_summary(summary_label, message)),
        always_show_details: false,
        is_current,
    }
}

fn unavailable_lines(message: &str, use_color: bool) -> Vec<String> {
    vec![format_usage_unavailable(message, use_color)]
}

fn detail_lines(
    tokens: &mut Tokens,
    email: Option<&str>,
    plan: Option<&str>,
    show_spinner: bool,
    profile_path: &Path,
    ctx: &ListCtx,
    allow_401_refresh: bool,
) -> (Vec<String>, Option<String>) {
    let plan_is_free = is_free_plan(plan);
    let use_color = ctx.use_color;
    let account_id = token_account_id(tokens).map(str::to_string);
    let access_token = tokens.access_token.clone();
    if is_api_key_profile(tokens) {
        if ctx.show_usage {
            return (
                unavailable_lines("Usage unavailable for API key login", use_color),
                None,
            );
        }
        return (Vec::new(), None);
    }
    let unavailable_text = usage_unavailable(plan_is_free);
    if let Some(message) = profile_error(tokens, email, plan) {
        let missing_access = access_token.is_none() || account_id.is_none();
        if ctx.show_usage && missing_access && email.is_some() && plan.is_some() {
            return (unavailable_lines(unavailable_text, use_color), None);
        }
        let details = vec![format_error(message)];
        let summary = Some(error_summary("Error", message));
        return (details, summary);
    }
    if ctx.show_usage {
        let Some(base_url) = ctx.base_url.as_deref() else {
            return (Vec::new(), None);
        };
        let Some(access_token) = access_token.as_deref() else {
            return (Vec::new(), None);
        };
        let Some(account_id) = account_id.as_deref() else {
            return (Vec::new(), None);
        };
        match fetch_usage_details(
            base_url,
            access_token,
            account_id,
            unavailable_text,
            ctx.now,
            show_spinner,
        ) {
            Ok(details) => (details, None),
            Err(err) if allow_401_refresh && err.status_code() == Some(401) => {
                match refresh_profile_tokens(profile_path, tokens) {
                    Ok(()) => {
                        let Some(access_token) = tokens.access_token.as_deref() else {
                            let message = "Error: refreshed access_token is missing.";
                            return (
                                vec![format_error(message)],
                                Some(error_summary("Auth error", message)),
                            );
                        };
                        match fetch_usage_details(
                            base_url,
                            access_token,
                            account_id,
                            unavailable_text,
                            ctx.now,
                            show_spinner,
                        ) {
                            Ok(details) => (details, None),
                            Err(err) => (
                                vec![format_error(&err.message())],
                                Some(error_summary("Usage error", &err.message())),
                            ),
                        }
                    }
                    Err(err) => (
                        vec![format_error(&err)],
                        Some(error_summary("Auth error", &err)),
                    ),
                }
            }
            Err(err) => (
                vec![format_error(&err.message())],
                Some(error_summary("Usage error", &err.message())),
            ),
        }
    } else if plan_is_free {
        (unavailable_lines(unavailable_text, use_color), None)
    } else {
        (Vec::new(), None)
    }
}

enum RefreshAttempt {
    Skipped,
    Succeeded,
    Failed(String),
}

fn refresh_for_status(tokens: &mut Tokens, profile_path: &Path, ctx: &ListCtx) -> RefreshAttempt {
    if !ctx.show_usage {
        return RefreshAttempt::Skipped;
    }
    if is_api_key_profile(tokens) {
        return RefreshAttempt::Skipped;
    }
    let has_refresh = tokens
        .refresh_token
        .as_deref()
        .map(|value| !value.is_empty())
        .unwrap_or(false);
    if !has_refresh {
        return RefreshAttempt::Failed(
            "Error: profile is missing refresh_token; run `codex login` and save it again."
                .to_string(),
        );
    }
    match refresh_profile_tokens(profile_path, tokens) {
        Ok(()) => RefreshAttempt::Succeeded,
        Err(err) => RefreshAttempt::Failed(err),
    }
}

fn make_entry(
    last_used: String,
    label: Option<String>,
    tokens_result: Option<&Result<Tokens, String>>,
    index_entry: Option<&ProfileIndexEntry>,
    profile_path: &Path,
    ctx: &ListCtx,
    is_current: bool,
) -> Entry {
    let use_color = ctx.use_color;
    let label_for_error = label.clone().or_else(|| profile_id_from_path(profile_path));
    let mut tokens = match tokens_result {
        Some(Ok(tokens)) => tokens.clone(),
        Some(Err(err)) => {
            return make_error(
                label_for_error,
                index_entry,
                use_color,
                last_used,
                err,
                "Error",
                is_current,
            );
        }
        None => {
            return make_error(
                label_for_error,
                index_entry,
                use_color,
                last_used,
                "profile file missing",
                "Error",
                is_current,
            );
        }
    };
    let refresh_attempt = refresh_for_status(&mut tokens, profile_path, ctx);
    let info = profile_info(Some(&tokens), label, is_current, use_color);
    let allow_401_refresh = matches!(refresh_attempt, RefreshAttempt::Skipped);
    let (mut details, mut summary) = detail_lines(
        &mut tokens,
        info.email.as_deref(),
        info.plan.as_deref(),
        false,
        profile_path,
        ctx,
        allow_401_refresh,
    );
    if let RefreshAttempt::Failed(err) = refresh_attempt {
        let warning = format_warning(&normalize_error(&err), use_color);
        details.insert(0, warning);
        if summary.is_none() {
            summary = Some(error_summary("Auth refresh", &err));
        }
    }
    Entry {
        display: info.display,
        last_used,
        details,
        error_summary: summary,
        always_show_details: info.is_free,
        is_current,
    }
}

fn make_saved(
    id: &str,
    ts: u64,
    snapshot: &Snapshot,
    current_saved_id: Option<&str>,
    ctx: &ListCtx,
) -> Entry {
    let profile_path = ctx.profiles_dir.join(format!("{id}.json"));
    let label = label_for_id(&snapshot.labels, id);
    let is_current = current_saved_id == Some(id);
    let last_used = if is_current {
        String::new()
    } else {
        format_last_used(ts)
    };
    make_entry(
        last_used,
        label,
        snapshot.tokens.get(id),
        snapshot.index.profiles.get(id),
        &profile_path,
        ctx,
        is_current,
    )
}

fn make_entries(
    ordered: &[(String, u64)],
    snapshot: &Snapshot,
    current_saved_id: Option<&str>,
    ctx: &ListCtx,
) -> Vec<Entry> {
    let build = |(id, ts): &(String, u64)| make_saved(id, *ts, snapshot, current_saved_id, ctx);
    if ctx.show_usage && ordered.len() >= 3 {
        if ordered.len() > MAX_USAGE_CONCURRENCY {
            let mut entries = Vec::with_capacity(ordered.len());
            for chunk in ordered.chunks(MAX_USAGE_CONCURRENCY) {
                let mut chunk_entries: Vec<Entry> = chunk.par_iter().map(build).collect();
                entries.append(&mut chunk_entries);
            }
            return entries;
        }
        return ordered.par_iter().map(build).collect();
    }

    ordered.iter().map(build).collect()
}

fn make_current(
    paths: &Paths,
    current_saved_id: Option<&str>,
    labels: &Labels,
    tokens_map: &BTreeMap<String, Result<Tokens, String>>,
    usage_map: &BTreeMap<String, u64>,
    ctx: &ListCtx,
) -> Option<Entry> {
    if !paths.auth.is_file() {
        return None;
    }
    let mut tokens = match read_tokens(&paths.auth) {
        Ok(tokens) => tokens,
        Err(err) => {
            return Some(make_error(
                None,
                None,
                ctx.use_color,
                String::new(),
                &err,
                "Error",
                true,
            ));
        }
    };
    let refresh_attempt = refresh_for_status(&mut tokens, &ctx.auth_path, ctx);
    let (email, _) = extract_email_and_plan(&tokens);
    let refreshed_saved_id =
        if matches!(refresh_attempt, RefreshAttempt::Succeeded) || current_saved_id.is_none() {
            match (token_account_id(&tokens), email.as_deref()) {
                (Some(account_id), Some(email)) => {
                    let candidates = cached_profile_ids(tokens_map, account_id, Some(email));
                    pick_primary(&candidates, usage_map)
                }
                _ => None,
            }
        } else {
            None
        };
    let effective_saved_id = refreshed_saved_id.as_deref().or(current_saved_id);
    if matches!(refresh_attempt, RefreshAttempt::Succeeded)
        && let Some(id) = effective_saved_id
    {
        let profile_path = ctx.profiles_dir.join(format!("{id}.json"));
        if profile_path.is_file()
            && let Err(err) = copy_atomic(&ctx.auth_path, &profile_path)
        {
            let warning = format_warning(&normalize_error(&err), use_color_stderr());
            eprintln!("{warning}");
        }
    }
    let label = effective_saved_id.and_then(|id| label_for_id(labels, id));
    let use_color = ctx.use_color;
    let info = profile_info(Some(&tokens), label, true, use_color);
    let plan_is_free = info.is_free;
    let can_save = is_profile_ready(&tokens);
    let is_unsaved = effective_saved_id.is_none() && can_save;
    let allow_401_refresh = matches!(refresh_attempt, RefreshAttempt::Skipped);
    let (mut details, mut summary) = detail_lines(
        &mut tokens,
        info.email.as_deref(),
        info.plan.as_deref(),
        ctx.show_spinner,
        &ctx.auth_path,
        ctx,
        allow_401_refresh,
    );
    if let RefreshAttempt::Failed(err) = refresh_attempt {
        let warning = format_warning(&normalize_error(&err), use_color);
        details.insert(0, warning);
        if summary.is_none() {
            summary = Some(error_summary("Auth refresh", &err));
        }
    }

    if is_unsaved && !plan_is_free {
        details.extend(format_unsaved_warning(use_color));
    }

    Some(Entry {
        display: info.display,
        last_used: String::new(),
        details,
        error_summary: summary,
        always_show_details: is_unsaved || (plan_is_free && !ctx.show_usage),
        is_current: true,
    })
}

fn error_summary(label: &str, message: &str) -> String {
    format!("{label}: {}", normalize_error(message))
}

struct ListCtx {
    base_url: Option<String>,
    now: DateTime<Local>,
    show_usage: bool,
    show_spinner: bool,
    use_color: bool,
    profiles_dir: PathBuf,
    auth_path: PathBuf,
}

impl ListCtx {
    fn new(paths: &Paths, show_usage: bool) -> Self {
        Self {
            base_url: show_usage.then(|| read_base_url(paths)),
            now: Local::now(),
            show_usage,
            show_spinner: show_usage,
            use_color: use_color_stdout(),
            profiles_dir: paths.profiles.clone(),
            auth_path: paths.auth.clone(),
        }
    }
}

struct Entry {
    display: String,
    last_used: String,
    details: Vec<String>,
    error_summary: Option<String>,
    always_show_details: bool,
    is_current: bool,
}

const LOAD_HELP: &str = "Type to search • Use ↑/↓ to select • ENTER to load";
const DELETE_HELP: &str = "Type to search • Use ↑/↓ to select • SPACE to select • ENTER to delete";

fn handle_inquire_result<T>(
    result: Result<T, inquire::error::InquireError>,
    context: &str,
) -> Result<T, String> {
    match result {
        Ok(value) => Ok(value),
        Err(err) if is_inquire_cancel(&err) => Err(CANCELLED_MESSAGE.to_string()),
        Err(err) => Err(format!("Error: failed to prompt for {context}: {err}")),
    }
}

fn trim_label(label: &str) -> Result<&str, String> {
    let trimmed = label.trim();
    if trimmed.is_empty() {
        return Err("Error: label cannot be empty".to_string());
    }
    Ok(trimmed)
}

fn normalize_labels(labels: &Labels) -> Labels {
    let mut normalized = BTreeMap::new();
    for (label, id) in labels {
        let trimmed = label.trim();
        if trimmed.is_empty() {
            continue;
        }
        let id = id.trim();
        if id.is_empty() {
            continue;
        }
        normalized.insert(trimmed.to_string(), id.to_string());
    }
    normalized
}

fn is_profile_file(path: &Path) -> bool {
    let Some(ext) = path.extension().and_then(|ext| ext.to_str()) else {
        return false;
    };
    if ext != "json" {
        return false;
    }
    !matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some("profiles.json" | "update.json")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{build_id_token, make_paths};
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::{Path, PathBuf};

    fn write_auth(
        path: &Path,
        account_id: &str,
        email: &str,
        plan: &str,
        access: &str,
        refresh: &str,
    ) {
        let id_token = build_id_token(email, plan);
        let value = serde_json::json!({
            "tokens": {
                "account_id": account_id,
                "id_token": id_token,
                "access_token": access,
                "refresh_token": refresh
            }
        });
        fs::write(path, serde_json::to_string(&value).unwrap()).unwrap();
    }

    fn write_profile(paths: &Paths, id: &str, account_id: &str, email: &str, plan: &str) {
        let id_token = build_id_token(email, plan);
        let value = serde_json::json!({
            "tokens": {
                "account_id": account_id,
                "id_token": id_token,
                "access_token": "acc",
                "refresh_token": "ref"
            }
        });
        let path = profile_path_for_id(&paths.profiles, id);
        fs::write(&path, serde_json::to_string(&value).unwrap()).unwrap();
    }

    #[test]
    fn require_tty_with_variants() {
        assert!(require_tty_with(true, "load").is_ok());
        let err = require_tty_with(false, "load").unwrap_err();
        assert!(err.contains("requires a TTY"));
    }

    #[test]
    fn prompt_unsaved_load_with_variants() {
        let dir = tempfile::tempdir().expect("tempdir");
        let paths = make_paths(dir.path());
        let err = prompt_unsaved_load_with(&paths, "reason", false, Ok("Cancel")).unwrap_err();
        assert!(err.contains("not saved"));
        assert!(matches!(
            prompt_unsaved_load_with(
                &paths,
                "reason",
                true,
                Ok("Save current profile and continue")
            )
            .unwrap(),
            LoadChoice::SaveAndContinue
        ));
        assert!(matches!(
            prompt_unsaved_load_with(&paths, "reason", true, Ok("Continue without saving"))
                .unwrap(),
            LoadChoice::ContinueWithoutSaving
        ));
        assert!(matches!(
            prompt_unsaved_load_with(&paths, "reason", true, Ok("Cancel")).unwrap(),
            LoadChoice::Cancel
        ));
        let err = prompt_unsaved_load_with(
            &paths,
            "reason",
            true,
            Err(inquire::error::InquireError::OperationCanceled),
        )
        .unwrap();
        assert!(matches!(err, LoadChoice::Cancel));
    }

    #[test]
    fn confirm_delete_profiles_with_variants() {
        let err = confirm_delete_profiles_with(false, Ok(true)).unwrap_err();
        assert!(err.contains("requires confirmation"));
        assert!(confirm_delete_profiles_with(true, Ok(true)).unwrap());
        let err = confirm_delete_profiles_with(
            true,
            Err(inquire::error::InquireError::OperationCanceled),
        )
        .unwrap_err();
        assert_eq!(err, CANCELLED_MESSAGE);
    }

    #[test]
    fn label_helpers() {
        let mut labels = Labels::new();
        assign_label(&mut labels, "Team", "id").unwrap();
        assert_eq!(label_for_id(&labels, "id").unwrap(), "Team");
        assert_eq!(resolve_label_id(&labels, "Team").unwrap(), "id");
        remove_labels_for_id(&mut labels, "id");
        assert!(labels.is_empty());
        assert!(trim_label(" ").is_err());
    }

    #[test]
    fn profiles_index_roundtrip() {
        let dir = tempfile::tempdir().expect("tempdir");
        let paths = make_paths(dir.path());
        let mut index = ProfilesIndex {
            active_profile_id: Some("id".to_string()),
            ..ProfilesIndex::default()
        };
        index.profiles.insert(
            "id".to_string(),
            ProfileIndexEntry {
                account_id: Some("acct".to_string()),
                email: Some("me@example.com".to_string()),
                plan: Some("Team".to_string()),
                label: Some("work".to_string()),
                added_at: 1,
                last_used: Some(2),
                is_api_key: false,
            },
        );
        write_profiles_index(&paths, &index).unwrap();
        let read_back = read_profiles_index(&paths).unwrap();
        let entry = read_back.profiles.get("id").unwrap();
        assert_eq!(read_back.active_profile_id.as_deref(), Some("id"));
        assert_eq!(entry.account_id.as_deref(), Some("acct"));
        assert_eq!(entry.email.as_deref(), Some("me@example.com"));
        assert_eq!(entry.plan.as_deref(), Some("Team"));
        assert_eq!(entry.label.as_deref(), Some("work"));
        assert_eq!(entry.added_at, 1);
        assert_eq!(entry.last_used, Some(2));
        assert!(!entry.is_api_key);
    }

    #[test]
    fn profiles_index_prunes_missing_profiles() {
        let dir = tempfile::tempdir().expect("tempdir");
        let paths = make_paths(dir.path());
        fs::create_dir_all(&paths.profiles).unwrap();
        let mut index = ProfilesIndex {
            active_profile_id: Some("missing".to_string()),
            ..ProfilesIndex::default()
        };
        index
            .profiles
            .insert("missing".to_string(), ProfileIndexEntry::default());
        prune_profiles_index(&mut index, &paths.profiles).unwrap();
        assert!(index.profiles.is_empty());
        assert!(index.active_profile_id.is_none());
    }

    #[test]
    fn sanitize_helpers() {
        assert_eq!(sanitize_part("A B"), "a-b");
        assert_eq!(profile_base("", ""), "unknown-unknown");
        assert_eq!(short_account_suffix("abcdef123"), "abcdef");
    }

    #[test]
    fn normalize_source_codex_dir_prefers_nested_codex() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().join("home");
        let nested = root.join(".codex");
        fs::create_dir_all(nested.join("profiles")).unwrap();
        let normalized = normalize_source_codex_dir(root);
        assert_eq!(normalized, nested);
    }

    #[test]
    fn pick_migration_source_skips_destination_and_picks_existing_source() {
        let dir = tempfile::tempdir().expect("tempdir");
        let dest_codex = dir.path().join("dest").join(".codex");
        let source_codex = dir.path().join("source").join(".codex");
        let dest_profiles = dest_codex.join("profiles");
        fs::create_dir_all(&dest_profiles).unwrap();
        fs::create_dir_all(source_codex.join("profiles")).unwrap();

        let picked =
            pick_migration_source_codex(&[dest_codex, source_codex.clone()], &dest_profiles);
        assert_eq!(picked.as_deref(), Some(source_codex.as_path()));
    }

    #[test]
    fn priority_order_prefers_tier_score_and_label() {
        let mut rows = vec![
            PriorityRow {
                id: "b".to_string(),
                profile_name: "b@example.com [b]".to_string(),
                label: Some("b".to_string()),
                is_current: false,
                candidate: true,
                state: PriorityState::Ready(PriorityUsage {
                    seven_day_left: 40,
                    seven_day_reset: Some("in 3d".to_string()),
                    five_hour_left: 0,
                    five_hour_reset: Some("in 1h".to_string()),
                    tier: 1,
                    score: 2800,
                }),
            },
            PriorityRow {
                id: "a".to_string(),
                profile_name: "a@example.com [a]".to_string(),
                label: Some("a".to_string()),
                is_current: false,
                candidate: true,
                state: PriorityState::Ready(PriorityUsage {
                    seven_day_left: 55,
                    seven_day_reset: Some("in 4d".to_string()),
                    five_hour_left: 45,
                    five_hour_reset: Some("in 2h".to_string()),
                    tier: 0,
                    score: 5200,
                }),
            },
            PriorityRow {
                id: "c".to_string(),
                profile_name: "c@example.com [c]".to_string(),
                label: Some("c".to_string()),
                is_current: false,
                candidate: true,
                state: PriorityState::Unavailable("usage failed".to_string()),
            },
        ];
        rows.sort_by(priority_row_cmp);
        assert_eq!(rows[0].id, "a");
        assert_eq!(rows[1].id, "b");
        assert_eq!(rows[2].id, "c");
    }

    #[test]
    fn render_priority_table_shows_unavailable_summary() {
        let rows = vec![
            PriorityRow {
                id: "a".to_string(),
                profile_name: "a@example.com [a]".to_string(),
                label: Some("a".to_string()),
                is_current: true,
                candidate: true,
                state: PriorityState::Ready(PriorityUsage {
                    seven_day_left: 90,
                    seven_day_reset: Some("in 6d".to_string()),
                    five_hour_left: 50,
                    five_hour_reset: Some("in 5h".to_string()),
                    tier: 0,
                    score: 7800,
                }),
            },
            PriorityRow {
                id: "b".to_string(),
                profile_name: "b@example.com [b]".to_string(),
                label: Some("b".to_string()),
                is_current: false,
                candidate: true,
                state: PriorityState::Unavailable("missing 7d window".to_string()),
            },
        ];
        let output = render_priority_table(&rows, false);
        assert!(output.contains("Priority ranking"));
        assert!(output.contains("5H RESET"));
        assert!(output.contains("7D RESET"));
        assert!(output.contains("UNAVAILABLE"));
        assert!(output.contains("Unavailable profiles"));
    }

    #[test]
    fn unique_id_conflicts() {
        let dir = tempfile::tempdir().expect("tempdir");
        let paths = make_paths(dir.path());
        fs::create_dir_all(&paths.profiles).unwrap();
        write_profile(&paths, "base", "acct", "a@b.com", "pro");
        let id = unique_id("base", "acct", &paths.profiles);
        assert_eq!(id, "base");
        let id = unique_id("base", "other", &paths.profiles);
        assert!(id.starts_with("base-"));
    }

    #[test]
    fn load_profile_tokens_map_handles_invalid() {
        let dir = tempfile::tempdir().expect("tempdir");
        let paths = make_paths(dir.path());
        fs::create_dir_all(&paths.profiles).unwrap();
        write_profile(&paths, "valid", "acct", "a@b.com", "pro");
        fs::write(paths.profiles.join("bad.json"), "not-json").unwrap();
        let index = serde_json::json!({
            "version": 1,
            "active_profile_id": null,
            "profiles": {
                "bad": {
                    "label": "bad",
                    "last_used": 1,
                    "added_at": 1
                }
            }
        });
        fs::write(
            &paths.profiles_index,
            serde_json::to_string(&index).unwrap(),
        )
        .unwrap();
        let map = load_profile_tokens_map(&paths).unwrap();
        assert!(map.contains_key("valid"));
    }

    #[cfg(unix)]
    #[test]
    fn load_profile_tokens_map_remove_error() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().expect("tempdir");
        let paths = make_paths(dir.path());
        fs::create_dir_all(&paths.profiles).unwrap();
        let bad_path = paths.profiles.join("bad.json");
        fs::write(&bad_path, "not-json").unwrap();
        let perms = fs::Permissions::from_mode(0o400);
        fs::set_permissions(&paths.profiles, perms).unwrap();
        let map = load_profile_tokens_map(&paths).unwrap();
        assert!(map.contains_key("bad"));
    }

    #[test]
    fn resolve_save_and_sync_ids() {
        let dir = tempfile::tempdir().expect("tempdir");
        let paths = make_paths(dir.path());
        fs::create_dir_all(&paths.profiles).unwrap();
        write_profile(&paths, "one", "acct", "a@b.com", "pro");
        let tokens = read_tokens(&paths.profiles.join("one.json")).unwrap();
        let mut usage_map = BTreeMap::new();
        let mut labels = Labels::new();
        let mut index = ProfilesIndex::default();
        let id = resolve_save_id(&paths, &mut usage_map, &mut labels, &mut index, &tokens).unwrap();
        assert!(!id.is_empty());
        let id = resolve_sync_id(&paths, &mut usage_map, &mut labels, &mut index, &tokens).unwrap();
        assert!(id.is_some());
    }

    #[test]
    fn rename_profile_id_errors_when_missing() {
        let dir = tempfile::tempdir().expect("tempdir");
        let paths = make_paths(dir.path());
        fs::create_dir_all(&paths.profiles).unwrap();
        let mut usage_map = BTreeMap::new();
        let mut labels = Labels::new();
        let mut index = ProfilesIndex::default();
        let err = rename_profile_id(
            &paths,
            &mut usage_map,
            &mut labels,
            &mut index,
            "missing",
            "base",
            "acct",
        )
        .unwrap_err();
        assert!(err.contains("not found"));
    }

    #[test]
    fn render_helpers() {
        let entry = Entry {
            display: "Display".to_string(),
            last_used: "".to_string(),
            details: vec!["detail".to_string()],
            error_summary: None,
            always_show_details: true,
            is_current: false,
        };
        let ctx = ListCtx {
            base_url: None,
            now: chrono::Local::now(),
            show_usage: false,
            show_spinner: false,
            use_color: false,
            profiles_dir: PathBuf::new(),
            auth_path: PathBuf::new(),
        };
        let lines = render_entries(&[entry], true, &ctx, None, true);
        assert!(!lines.is_empty());
        push_separator(&mut vec!["a".to_string()], None, true);
    }

    #[test]
    fn handle_inquire_result_variants() {
        let ok: Result<i32, inquire::error::InquireError> = Ok(1);
        assert_eq!(handle_inquire_result(ok, "selection").unwrap(), 1);
        let err: Result<(), inquire::error::InquireError> =
            Err(inquire::error::InquireError::OperationCanceled);
        let err = handle_inquire_result(err, "selection").unwrap_err();
        assert_eq!(err, CANCELLED_MESSAGE);
    }

    #[test]
    fn sync_and_status_paths() {
        let dir = tempfile::tempdir().expect("tempdir");
        let paths = make_paths(dir.path());
        fs::create_dir_all(&paths.profiles).unwrap();
        write_auth(&paths.auth, "acct", "a@b.com", "pro", "acc", "ref");
        crate::ensure_paths(&paths).unwrap();
        save_profile(&paths, Some("team".to_string())).unwrap();
        list_profiles(&paths, false, false, false, false).unwrap();
        status_profiles(&paths, false).unwrap();
        let label = read_labels(&paths).unwrap().keys().next().cloned().unwrap();
        status_label(&paths, &label).unwrap();
        sync_current_readonly(&paths).unwrap();
    }

    #[test]
    fn delete_profile_by_label() {
        let dir = tempfile::tempdir().expect("tempdir");
        let paths = make_paths(dir.path());
        fs::create_dir_all(&paths.profiles).unwrap();
        write_auth(&paths.auth, "acct", "a@b.com", "pro", "acc", "ref");
        crate::ensure_paths(&paths).unwrap();
        save_profile(&paths, Some("team".to_string())).unwrap();
        delete_profile(&paths, true, Some("team".to_string())).unwrap();
    }
}
