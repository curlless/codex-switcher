use super::*;

pub(super) fn migrate_profiles(
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
            if source_last_used > dest_last_used || !store.usage_map.contains_key(id) {
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

pub(super) fn normalize_source_codex_dir(path: PathBuf) -> PathBuf {
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

pub(super) fn pick_migration_source_codex(
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
