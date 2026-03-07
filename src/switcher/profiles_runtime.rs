use super::*;

pub(super) fn profile_is_reserved(id: &str, snapshot: &Snapshot) -> bool {
    snapshot
        .index
        .profiles
        .get(id)
        .is_some_and(|entry| entry.reserved)
}

pub(super) fn with_reserved_marker(display: String, reserved: bool) -> String {
    if reserved && !display.ends_with(RESERVED_DISPLAY_MARKER) {
        format!("{display}{RESERVED_DISPLAY_MARKER}")
    } else {
        display
    }
}

pub(super) fn display_without_reserved_marker(display: &str) -> String {
    display
        .strip_suffix(RESERVED_DISPLAY_MARKER)
        .unwrap_or(display)
        .to_string()
}

pub(super) struct Snapshot {
    pub(super) usage_map: BTreeMap<String, u64>,
    pub(super) labels: Labels,
    pub(super) tokens: BTreeMap<String, Result<Tokens, String>>,
    pub(super) index: ProfilesIndex,
}

pub(super) fn sync_current(
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

pub(super) fn sync_profile(paths: &Paths, target: &Path) -> Result<(), String> {
    copy_atomic(&paths.auth, target)
        .map_err(|err| format!("Error: failed to sync current profile: {err}"))?;
    Ok(())
}

pub(super) fn load_snapshot(paths: &Paths, strict_labels: bool) -> Result<Snapshot, String> {
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

pub(super) fn unsaved_reason(
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

pub(super) fn current_saved_id(
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

pub(super) struct ProfileStore {
    _lock: UsageLock,
    pub(super) usage_map: BTreeMap<String, u64>,
    pub(super) labels: Labels,
    pub(super) profiles_index: ProfilesIndex,
}

impl ProfileStore {
    pub(super) fn load(paths: &Paths) -> Result<Self, String> {
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

    pub(super) fn save(&mut self, paths: &Paths) -> Result<(), String> {
        prune_labels(&mut self.labels, &paths.profiles);
        prune_profiles_index(&mut self.profiles_index, &paths.profiles)?;
        sync_profiles_index(&mut self.profiles_index, &self.usage_map, &self.labels);
        write_profiles_index(paths, &self.profiles_index)?;
        Ok(())
    }
}

pub(super) fn profile_not_found(use_color: bool) -> String {
    format!(
        "Selected profile not found. {}",
        format_list_hint(use_color)
    )
}

pub(super) fn load_snapshot_ordered(
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

pub(super) fn copy_profile(source: &Path, dest: &Path, context: &str) -> Result<(), String> {
    copy_atomic(source, dest)
        .map_err(|err| format!("Error: failed to {context} {}: {err}", dest.display()))?;
    Ok(())
}
