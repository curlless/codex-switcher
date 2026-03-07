use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::switcher::{
    Labels, Paths, ProfilesIndex, Tokens, extract_email_and_plan, profile_files,
    profile_id_from_path, profile_path_for_id, read_tokens, require_identity, token_account_id,
};

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

pub(crate) fn profile_base(email: &str, plan_label: &str) -> String {
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

pub(crate) fn sanitize_part(value: &str) -> String {
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

pub(crate) fn unique_id(base: &str, account_id: &str, profiles_dir: &Path) -> String {
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

pub(crate) fn short_account_suffix(account_id: &str) -> String {
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

pub(crate) fn rename_profile_id(
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
