use chrono::{DateTime, Local};
use colored::Colorize;
use inquire::{Confirm, MultiSelect, Select};
use rayon::prelude::*;
use std::collections::{BTreeMap, HashSet};
use std::fmt;
use std::fs;
use std::io::{self, IsTerminal as _};
use std::path::{Path, PathBuf};

use crate::{
    CANCELLED_MESSAGE, format_action, format_entry_header, format_error, format_list_hint,
    format_no_profiles, format_save_before_load, format_unsaved_warning, format_warning,
    inquire_select_render_config, is_inquire_cancel, is_plain, normalize_error, print_output_block,
    print_output_block_with_frame, style_text, terminal_width, use_color_stderr, use_color_stdout,
};
use crate::{Paths, command_name, copy_atomic, write_atomic};
use crate::{
    Tokens, extract_email_and_plan, is_api_key_profile, is_free_plan, is_profile_ready,
    profile_error, read_tokens, read_tokens_opt, refresh_profile_tokens, require_identity,
    token_account_id,
};
use crate::{
    UsageLock, ensure_usage, fetch_usage_details, format_last_used, format_usage_unavailable,
    lock_usage, normalize_usage, now_seconds, ordered_profiles, read_base_url, read_usage,
    usage_unavailable, write_usage,
};

const MAX_USAGE_CONCURRENCY: usize = 4;

pub fn save_profile(paths: &Paths, label: Option<String>) -> Result<(), String> {
    let use_color = use_color_stdout();
    let mut store = ProfileStore::load(paths)?;
    let tokens = read_tokens(&paths.auth)?;
    let id = resolve_save_id(paths, &mut store.usage_map, &mut store.labels, &tokens)?;

    if let Some(label) = label.as_deref() {
        assign_label(&mut store.labels, label, &id)?;
    }

    let target = profile_path_for_id(&paths.profiles, &id);
    copy_profile(&paths.auth, &target, "save profile to")?;

    store.usage_map.insert(id.clone(), now_seconds());
    store.save(paths)?;

    let label_display = label_for_id(&store.labels, &id);
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
    let use_color_out = use_color_stdout();
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

    let mut store = ProfileStore::load(paths)?;

    if let Err(err) = sync_current(paths, &mut store.usage_map, &mut store.labels) {
        let warning = format_warning(&err, use_color_err);
        eprintln!("{warning}");
    }

    let source = profile_path_for_id(&paths.profiles, &selected_id);
    if !source.is_file() {
        return Err(profile_not_found(use_color_err));
    }

    copy_profile(&source, &paths.auth, "load selected profile to")?;

    store.usage_map.insert(selected_id.clone(), now_seconds());
    store.save(paths)?;

    let message = format_action(&format!("Loaded profile {selected_display}"), use_color_out);
    print_output_block(&message);
    Ok(())
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
    let ctx = ListCtx::new(paths, show_usage);

    let ordered = ordered_profiles(usage_map);
    let separator = separator_line(2);
    let frame_separator = if frame_with_separator {
        separator_line(0)
    } else {
        None
    };
    let has_saved = !ordered.is_empty();
    if !has_saved {
        if !render_current(
            paths,
            current_saved_id.as_deref(),
            &snapshot.labels,
            &snapshot.tokens,
            &snapshot.usage_map,
            false,
            &ctx,
        )? {
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

    let mut lines = Vec::new();
    if let Some(entry) = make_current(
        paths,
        current_saved_id.as_deref(),
        &snapshot.labels,
        &snapshot.tokens,
        &snapshot.usage_map,
        &ctx,
    ) {
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
        return list_profiles(paths, true, true, true, true);
    }
    let snapshot = load_snapshot(paths, false).ok();
    let current_saved_id = snapshot
        .as_ref()
        .and_then(|snap| current_saved_id(paths, &snap.usage_map, &snap.tokens));
    let ctx = ListCtx::new(paths, true);
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
    if !render_current(
        paths,
        current_saved_id.as_deref(),
        labels,
        tokens_map,
        usage_map,
        false,
        &ctx,
    )? {
        let message = format_no_profiles(paths, ctx.use_color);
        print_output_block(&message);
    }
    Ok(())
}

pub fn status_label(paths: &Paths, label: &str) -> Result<(), String> {
    let snapshot = load_snapshot(paths, false)?;
    let id = resolve_label_id(&snapshot.labels, label)?;
    let current_saved_id = current_saved_id(paths, &snapshot.usage_map, &snapshot.tokens);
    let ctx = ListCtx::new(paths, true);
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
        &profile_path,
        &ctx,
        is_current,
    );
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

pub fn read_labels(paths: &Paths) -> Result<Labels, String> {
    if !paths.labels.exists() {
        return Ok(BTreeMap::new());
    }
    let contents = fs::read_to_string(&paths.labels).map_err(|err| {
        format!(
            "Error: cannot read labels file {}: {err}",
            paths.labels.display()
        )
    })?;
    let labels: Labels = serde_json::from_str(&contents).map_err(|_| {
        format!(
            "Error: labels file {} is invalid JSON",
            paths.labels.display()
        )
    })?;
    Ok(normalize_labels(&labels))
}

pub fn write_labels(paths: &Paths, labels: &Labels) -> Result<(), String> {
    let normalized = normalize_labels(labels);
    let json = serde_json::to_string_pretty(&normalized)
        .map_err(|err| format!("Error: failed to serialize labels: {err}"))?;
    write_atomic(&paths.labels, format!("{json}\n").as_bytes())
        .map_err(|err| format!("Error: failed to write labels file: {err}"))
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
    if !removed_ids.is_empty()
        && let Ok(mut labels) = read_labels(paths)
    {
        for id in &removed_ids {
            remove_labels_for_id(&mut labels, id);
        }
        let _ = write_labels(paths, &labels);
    }
    Ok(map)
}

pub(crate) fn resolve_save_id(
    paths: &Paths,
    map: &mut BTreeMap<String, u64>,
    labels: &mut Labels,
    tokens: &Tokens,
) -> Result<String, String> {
    let (account_id, email, plan) = require_identity(tokens)?;
    let (desired_base, desired, candidates) =
        desired_candidates(paths, &account_id, &email, &plan)?;
    if has_usage_signal(&candidates, map)
        && let Some(primary) = pick_primary(&candidates, map).filter(|primary| primary != &desired)
    {
        return rename_profile_id(paths, map, labels, &primary, &desired_base, &account_id);
    }
    Ok(desired)
}

pub(crate) fn resolve_sync_id(
    paths: &Paths,
    map: &mut BTreeMap<String, u64>,
    labels: &mut Labels,
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
        let renamed = rename_profile_id(paths, map, labels, &primary, &desired_base, &account_id)?;
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
    Ok(desired)
}

pub(crate) struct Snapshot {
    pub(crate) usage_map: BTreeMap<String, u64>,
    pub(crate) labels: Labels,
    pub(crate) tokens: BTreeMap<String, Result<Tokens, String>>,
}

pub(crate) fn sync_current(
    paths: &Paths,
    map: &mut BTreeMap<String, u64>,
    labels: &mut Labels,
) -> Result<(), String> {
    let Some(tokens) = read_tokens_opt(&paths.auth) else {
        return Ok(());
    };
    let id = match resolve_sync_id(paths, map, labels, &tokens)? {
        Some(id) => id,
        None => return Ok(()),
    };
    let target = profile_path_for_id(&paths.profiles, &id);
    sync_profile(paths, &target)?;
    map.insert(id, now_seconds());
    Ok(())
}

fn sync_profile(paths: &Paths, target: &Path) -> Result<(), String> {
    copy_atomic(&paths.auth, target)
        .map_err(|err| format!("Error: failed to sync current profile: {err}"))?;
    Ok(())
}

pub(crate) fn load_snapshot(paths: &Paths, strict_labels: bool) -> Result<Snapshot, String> {
    let mut lock = lock_usage(paths)?;
    let usage_entries = read_usage(&mut lock.file)?;
    drop(lock);

    let tokens = load_profile_tokens_map(paths)?;
    let ids: HashSet<String> = tokens.keys().cloned().collect();
    let usage_map = normalize_usage(&usage_entries, &ids);
    let labels = if strict_labels {
        read_labels(paths)?
    } else {
        read_labels_relaxed(paths)
    };

    Ok(Snapshot {
        usage_map,
        labels,
        tokens,
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

fn read_labels_relaxed(paths: &Paths) -> Labels {
    match read_labels(paths) {
        Ok(labels) => labels,
        Err(err) => {
            let normalized = normalize_error(&err);
            let warning = format_warning(&normalized, use_color_stderr());
            eprintln!("{warning}");
            Labels::new()
        }
    }
}

pub(crate) struct ProfileStore {
    lock: UsageLock,
    pub(crate) usage_map: BTreeMap<String, u64>,
    pub(crate) labels: Labels,
}

impl ProfileStore {
    pub(crate) fn load(paths: &Paths) -> Result<Self, String> {
        let mut lock = lock_usage(paths)?;
        let usage_map = ensure_usage(&mut lock.file, &paths.profiles)?;
        let labels = read_labels(paths)?;
        Ok(Self {
            lock,
            usage_map,
            labels,
        })
    }

    pub(crate) fn save(&mut self, paths: &Paths) -> Result<(), String> {
        prune_labels(&mut self.labels, &paths.profiles);
        write_labels(paths, &self.labels)?;
        write_usage(&mut self.lock.file, &self.usage_map)
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
    let (email, plan) = tokens.map(extract_email_and_plan).unwrap_or((None, None));
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
        let is_current = current_saved_id == Some(id.as_str());
        let info = profile_info(tokens, label, is_current, use_color);
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

fn render_current(
    paths: &Paths,
    current_saved_id: Option<&str>,
    labels: &Labels,
    tokens_map: &BTreeMap<String, Result<Tokens, String>>,
    usage_map: &BTreeMap<String, u64>,
    hide_last_used: bool,
    ctx: &ListCtx,
) -> Result<bool, String> {
    if let Some(entry) = make_current(paths, current_saved_id, labels, tokens_map, usage_map, ctx) {
        let lines = render_entries(&[entry], !hide_last_used, ctx, None, false);
        print_output_block(&lines.join("\n"));
        Ok(true)
    } else {
        Ok(false)
    }
}

fn make_error(
    label: Option<String>,
    use_color: bool,
    last_used: String,
    message: &str,
    summary_label: &str,
    is_current: bool,
) -> Entry {
    let display = profile_info(None, label, is_current, use_color).display;
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
    is_current: bool,
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
            is_current,
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
                            is_current,
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
    if ctx.base_url.is_some() && ordered.len() >= 3 {
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
        true,
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
        Some("labels.json") | Some("version.json")
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
    fn sanitize_helpers() {
        assert_eq!(sanitize_part("A B"), "a-b");
        assert_eq!(profile_base("", ""), "unknown-unknown");
        assert_eq!(short_account_suffix("abcdef123"), "abcdef");
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
        let labels = serde_json::json!({"bad": "bad"});
        fs::write(&paths.labels, serde_json::to_string(&labels).unwrap()).unwrap();
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
        let id = resolve_save_id(&paths, &mut usage_map, &mut labels, &tokens).unwrap();
        assert!(!id.is_empty());
        let id = resolve_sync_id(&paths, &mut usage_map, &mut labels, &tokens).unwrap();
        assert!(id.is_some());
    }

    #[test]
    fn rename_profile_id_errors_when_missing() {
        let dir = tempfile::tempdir().expect("tempdir");
        let paths = make_paths(dir.path());
        fs::create_dir_all(&paths.profiles).unwrap();
        let mut usage_map = BTreeMap::new();
        let mut labels = Labels::new();
        let err = rename_profile_id(
            &paths,
            &mut usage_map,
            &mut labels,
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
