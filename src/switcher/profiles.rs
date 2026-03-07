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
) -> Result<Option<crate::switcher::CodexAppOverride>, String> {
    if matches!(target, ReloadAppTarget::All | ReloadAppTarget::Codex) {
        ensure_codex_app_override(paths)
    } else {
        codex_app_override(paths)
    }
}

pub fn reserve_profile(paths: &Paths, label: Option<String>) -> Result<(), String> {
    set_profile_reserved(paths, label, true)
}

pub fn unreserve_profile(paths: &Paths, label: Option<String>) -> Result<(), String> {
    set_profile_reserved(paths, label, false)
}

fn set_profile_reserved(
    paths: &Paths,
    label: Option<String>,
    reserved: bool,
) -> Result<(), String> {
    let use_color_err = use_color_stderr();
    let use_color_out = use_color_stdout();
    let no_profiles = format_no_profiles(paths, use_color_err);
    let (snapshot, ordered) = load_snapshot_ordered(paths, true, &no_profiles)?;
    let candidates = make_candidates(paths, &snapshot, &ordered);
    let selected = pick_one(
        if reserved { "reserve" } else { "unreserve" },
        label.as_deref(),
        &snapshot,
        &candidates,
    )?;
    let mut store = ProfileStore::load(paths)?;
    let entry = store
        .profiles_index
        .profiles
        .entry(selected.id.clone())
        .or_default();
    if entry.reserved == reserved {
        let state = if reserved {
            "already reserved"
        } else {
            "already unreserved"
        };
        let message = format_action(
            &format!(
                "Profile {} is {state}.",
                display_without_reserved_marker(&selected.display)
            ),
            use_color_out,
        );
        print_output_block(&message);
        return Ok(());
    }
    entry.reserved = reserved;
    store.save(paths)?;
    let verb = if reserved { "Reserved" } else { "Unreserved" };
    let message = format_action(
        &format!(
            "{verb} profile {}",
            display_without_reserved_marker(&selected.display)
        ),
        use_color_out,
    );
    print_output_block(&message);
    Ok(())
}

pub fn migrate_profiles(
    paths: &Paths,
    from: Option<String>,
    overwrite: bool,
) -> Result<(), String> {
    profile_migrate::migrate_profiles(paths, from, overwrite)
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

#[cfg(all(test, feature = "switcher-unit-tests"))]
#[path = "profiles_tests.rs"]
mod tests;
