use chrono::{DateTime, Local};
use colored::Colorize;
use directories::BaseDirs;
use inquire::{Confirm, MultiSelect, Select};
use rayon::prelude::*;
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt;
use std::fs;
use std::io::{self, IsTerminal as _};
use std::path::{Path, PathBuf};

use crate::switcher::{
    CANCELLED_MESSAGE, format_action, format_entry_header, format_error, format_hint,
    format_list_hint, format_no_profiles, format_save_before_load, format_unsaved_warning,
    format_warning, inquire_select_render_config, is_inquire_cancel, is_plain, normalize_error,
    print_output_block, print_output_block_with_frame, style_text, terminal_width,
    use_color_stderr, use_color_stdout,
};
use crate::switcher::{
    Labels, ProfileIndexEntry, ProfilesIndex, assign_label, cached_profile_ids,
    collect_profile_ids, label_for_id, labels_from_index, load_profile_tokens_map_locked,
    pick_primary, profile_files, profile_id_from_path, profile_path_for_id, prune_labels,
    prune_profiles_index, read_profiles_index, read_profiles_index_relaxed, remove_labels_for_id,
    resolve_label_id, resolve_save_id, resolve_sync_id, sync_profiles_index,
    update_profiles_index_entry, usage_map_from_index, write_profiles_index,
};
use crate::switcher::{
    Paths, codex_app_override, command_name, copy_atomic, ensure_codex_app_override,
};
use crate::switcher::{
    ReloadAppTarget, inspect_ide_reload_target_with_codex_override,
    reload_ide_target_best_effort_with_codex_override,
};
use crate::switcher::{
    Tokens, extract_email_and_plan, is_api_key_profile, is_free_plan, is_profile_ready,
    profile_error, read_tokens, read_tokens_opt, refresh_profile_tokens, token_account_id,
};
use crate::switcher::{
    UsageLock, UsageWindow, fetch_usage_details, fetch_usage_limits, format_last_used,
    format_usage_unavailable, lock_usage, now_seconds, ordered_profiles, read_base_url,
    start_usage_spinner, stop_usage_spinner, usage_unavailable,
};

const MAX_USAGE_CONCURRENCY: usize = 4;
const SCORE_7D_WEIGHT: i64 = 70;
const SCORE_5H_WEIGHT: i64 = 30;
const RESERVED_DISPLAY_MARKER: &str = " [reserved]";
const CURSOR_PROTOCOL_HELPER_HINT: &str = "Cursor automation: install the Commands Executor extension (ionutvmi.vscode-commands-executor) to enable protocol-based Reload Window.";

#[path = "profiles_priority.rs"]
mod profile_priority;
#[cfg(all(test, feature = "switcher-unit-tests"))]
use profile_priority::{PriorityRow, PriorityState, PriorityUsage, priority_row_cmp};
use profile_priority::{
    best_ready_row, ordered_profiles_by_usage, priority_rows, render_priority_table,
};

pub fn save_profile(paths: &Paths, label: Option<String>) -> Result<(), String> {
    profile_load::save_profile(paths, label)
}

pub fn load_profile(paths: &Paths, label: Option<String>) -> Result<(), String> {
    profile_load::load_profile(paths, label)
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

    profile_load::load_profile_by_id(paths, &best.id, &best.profile_name)?;

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
) -> Result<Option<crate::switcher::CodexAppOverride>, String> {
    if matches!(target, ReloadAppTarget::All | ReloadAppTarget::Codex) {
        ensure_codex_app_override(paths)
    } else {
        codex_app_override(paths)
    }
}

pub fn reserve_profile(paths: &Paths, label: Option<String>) -> Result<(), String> {
    profile_reserve::reserve_profile(paths, label)
}

pub fn unreserve_profile(paths: &Paths, label: Option<String>) -> Result<(), String> {
    profile_reserve::unreserve_profile(paths, label)
}

pub fn migrate_profiles(
    paths: &Paths,
    from: Option<String>,
    overwrite: bool,
) -> Result<(), String> {
    profile_migrate::migrate_profiles(paths, from, overwrite)
}

pub fn delete_profile(paths: &Paths, yes: bool, label: Option<String>) -> Result<(), String> {
    profile_delete::delete_profile(paths, yes, label)
}

pub fn list_profiles(
    paths: &Paths,
    show_usage: bool,
    show_last_used: bool,
    allow_plain_spacing: bool,
    frame_with_separator: bool,
) -> Result<(), String> {
    profile_status::list_profiles(
        paths,
        show_usage,
        show_last_used,
        allow_plain_spacing,
        frame_with_separator,
    )
}

pub fn status_profiles(paths: &Paths, all: bool) -> Result<(), String> {
    profile_status::status_profiles(paths, all)
}

pub fn status_label(paths: &Paths, label: &str) -> Result<(), String> {
    profile_status::status_label(paths, label)
}

pub fn sync_current_readonly(paths: &Paths) -> Result<(), String> {
    profile_status::sync_current_readonly(paths)
}

fn profile_is_reserved(id: &str, snapshot: &Snapshot) -> bool {
    snapshot
        .index
        .profiles
        .get(id)
        .is_some_and(|entry| entry.reserved)
}

fn with_reserved_marker(display: String, reserved: bool) -> String {
    if reserved && !display.ends_with(RESERVED_DISPLAY_MARKER) {
        format!("{display}{RESERVED_DISPLAY_MARKER}")
    } else {
        display
    }
}

fn display_without_reserved_marker(display: &str) -> String {
    display
        .strip_suffix(RESERVED_DISPLAY_MARKER)
        .unwrap_or(display)
        .to_string()
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
    let tokens = load_profile_tokens_map_locked(paths)?;
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

#[path = "profiles_ui.rs"]
mod profile_ui;
use profile_ui::*;

#[path = "profiles_migrate.rs"]
mod profile_migrate;
#[cfg(all(test, feature = "switcher-unit-tests"))]
use profile_migrate::*;

#[path = "profiles_status.rs"]
mod profile_status;

#[path = "profiles_reserve.rs"]
mod profile_reserve;

#[path = "profiles_load.rs"]
mod profile_load;

#[path = "profiles_delete.rs"]
mod profile_delete;

#[cfg(all(test, feature = "switcher-unit-tests"))]
#[path = "profiles_tests.rs"]
mod tests;
