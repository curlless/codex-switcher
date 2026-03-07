use super::*;

pub(super) fn reserve_profile(paths: &Paths, label: Option<String>) -> Result<(), String> {
    set_profile_reserved(paths, label, true)
}

pub(super) fn unreserve_profile(paths: &Paths, label: Option<String>) -> Result<(), String> {
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
