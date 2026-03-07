use super::*;

pub(super) fn save_profile(paths: &Paths, label: Option<String>) -> Result<(), String> {
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

pub(super) fn load_profile(paths: &Paths, label: Option<String>) -> Result<(), String> {
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

pub(super) fn load_profile_by_id(
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
