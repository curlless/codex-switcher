use super::*;

pub(super) fn delete_profile(
    paths: &Paths,
    yes: bool,
    label: Option<String>,
) -> Result<(), String> {
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
