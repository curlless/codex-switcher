use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::switcher::{
    Paths, Tokens, extract_email_and_plan, format_list_hint, format_warning, is_api_key_profile,
    lock_usage, normalize_error, read_tokens, token_account_id, use_color_stderr, write_atomic,
};

pub type Labels = BTreeMap<String, String>;

const PROFILES_INDEX_VERSION: u8 = 1;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ProfilesIndex {
    #[serde(default = "profiles_index_version")]
    pub(crate) version: u8,
    #[serde(default)]
    pub(crate) active_profile_id: Option<String>,
    #[serde(default)]
    pub(crate) profiles: BTreeMap<String, ProfileIndexEntry>,
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
pub(crate) struct ProfileIndexEntry {
    #[serde(default)]
    pub(crate) account_id: Option<String>,
    #[serde(default)]
    pub(crate) email: Option<String>,
    #[serde(default)]
    pub(crate) plan: Option<String>,
    #[serde(default)]
    pub(crate) label: Option<String>,
    #[serde(default)]
    pub(crate) added_at: u64,
    #[serde(default)]
    pub(crate) last_used: Option<u64>,
    #[serde(default)]
    pub(crate) is_api_key: bool,
    #[serde(default)]
    pub(crate) reserved: bool,
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

pub(crate) fn prune_profiles_index(
    index: &mut ProfilesIndex,
    profiles_dir: &Path,
) -> Result<(), String> {
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

pub(crate) fn sync_profiles_index(
    index: &mut ProfilesIndex,
    usage_map: &BTreeMap<String, u64>,
    labels: &Labels,
) {
    for (id, entry) in index.profiles.iter_mut() {
        entry.last_used = usage_map.get(id).copied();
        entry.label = label_for_id(labels, id);
    }
}

pub(crate) fn labels_from_index(index: &ProfilesIndex) -> Labels {
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

pub(crate) fn usage_map_from_index(
    index: &ProfilesIndex,
    ids: &HashSet<String>,
) -> BTreeMap<String, u64> {
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

pub(crate) fn update_profiles_index_entry(
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

fn write_labels_locked(paths: &Paths, labels: &Labels) -> Result<(), String> {
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

pub fn write_labels(paths: &Paths, labels: &Labels) -> Result<(), String> {
    let _lock = lock_usage(paths)?;
    write_labels_locked(paths, labels)
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

pub(crate) fn load_profile_tokens_map_locked(
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

pub fn load_profile_tokens_map(
    paths: &Paths,
) -> Result<BTreeMap<String, Result<Tokens, String>>, String> {
    let _lock = lock_usage(paths)?;
    load_profile_tokens_map_locked(paths)
}

pub(crate) fn trim_label(label: &str) -> Result<&str, String> {
    let trimmed = label.trim();
    if trimmed.is_empty() {
        return Err("Error: label cannot be empty".to_string());
    }
    Ok(trimmed)
}

pub(crate) fn normalize_labels(labels: &Labels) -> Labels {
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

pub(crate) fn is_profile_file(path: &Path) -> bool {
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
