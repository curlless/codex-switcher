use super::*;

pub(super) fn list_profiles(
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

pub(super) fn status_profiles(paths: &Paths, all: bool) -> Result<(), String> {
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

pub(super) fn status_label(paths: &Paths, label: &str) -> Result<(), String> {
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

pub(super) fn sync_current_readonly(paths: &Paths) -> Result<(), String> {
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
