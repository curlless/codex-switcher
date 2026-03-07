use super::*;
use crate::switcher::test_utils::{build_id_token, make_paths};
use crate::switcher::{
    Labels, ProfileIndexEntry, ProfilesIndex, assign_label, label_for_id, load_profile_tokens_map,
    profile_base, profile_path_for_id, read_labels, read_profiles_index, remove_labels_for_id,
    rename_profile_id, resolve_label_id, resolve_save_id, resolve_sync_id, sanitize_part,
    short_account_suffix, trim_label, unique_id, write_profiles_index,
};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

fn write_auth(path: &Path, account_id: &str, email: &str, plan: &str, access: &str, refresh: &str) {
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
        prompt_unsaved_load_with(&paths, "reason", true, Ok("Continue without saving")).unwrap(),
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
    let err =
        confirm_delete_profiles_with(true, Err(inquire::error::InquireError::OperationCanceled))
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
fn profiles_index_roundtrip() {
    let dir = tempfile::tempdir().expect("tempdir");
    let paths = make_paths(dir.path());
    let mut index = ProfilesIndex {
        active_profile_id: Some("id".to_string()),
        ..ProfilesIndex::default()
    };
    index.profiles.insert(
        "id".to_string(),
        ProfileIndexEntry {
            account_id: Some("acct".to_string()),
            email: Some("me@example.com".to_string()),
            plan: Some("Team".to_string()),
            label: Some("work".to_string()),
            added_at: 1,
            last_used: Some(2),
            is_api_key: false,
            reserved: true,
        },
    );
    write_profiles_index(&paths, &index).unwrap();
    let read_back = read_profiles_index(&paths).unwrap();
    let entry = read_back.profiles.get("id").unwrap();
    assert_eq!(read_back.active_profile_id.as_deref(), Some("id"));
    assert_eq!(entry.account_id.as_deref(), Some("acct"));
    assert_eq!(entry.email.as_deref(), Some("me@example.com"));
    assert_eq!(entry.plan.as_deref(), Some("Team"));
    assert_eq!(entry.label.as_deref(), Some("work"));
    assert_eq!(entry.added_at, 1);
    assert_eq!(entry.last_used, Some(2));
    assert!(!entry.is_api_key);
    assert!(entry.reserved);
}

#[test]
fn profiles_index_prunes_missing_profiles() {
    let dir = tempfile::tempdir().expect("tempdir");
    let paths = make_paths(dir.path());
    fs::create_dir_all(&paths.profiles).unwrap();
    let mut index = ProfilesIndex {
        active_profile_id: Some("missing".to_string()),
        ..ProfilesIndex::default()
    };
    index
        .profiles
        .insert("missing".to_string(), ProfileIndexEntry::default());
    prune_profiles_index(&mut index, &paths.profiles).unwrap();
    assert!(index.profiles.is_empty());
    assert!(index.active_profile_id.is_none());
}

#[test]
fn sanitize_helpers() {
    assert_eq!(sanitize_part("A B"), "a-b");
    assert_eq!(profile_base("", ""), "unknown-unknown");
    assert_eq!(short_account_suffix("abcdef123"), "abcdef");
}

#[test]
fn normalize_source_codex_dir_prefers_nested_codex() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().join("home");
    let nested = root.join(".codex");
    fs::create_dir_all(nested.join("profiles")).unwrap();
    let normalized = normalize_source_codex_dir(root);
    assert_eq!(normalized, nested);
}

#[test]
fn pick_migration_source_skips_destination_and_picks_existing_source() {
    let dir = tempfile::tempdir().expect("tempdir");
    let dest_codex = dir.path().join("dest").join(".codex");
    let source_codex = dir.path().join("source").join(".codex");
    let dest_profiles = dest_codex.join("profiles");
    fs::create_dir_all(&dest_profiles).unwrap();
    fs::create_dir_all(source_codex.join("profiles")).unwrap();

    let picked = pick_migration_source_codex(&[dest_codex, source_codex.clone()], &dest_profiles);
    assert_eq!(picked.as_deref(), Some(source_codex.as_path()));
}

#[test]
fn priority_order_prefers_tier_score_and_label() {
    let mut rows = [
        PriorityRow {
            id: "b".to_string(),
            profile_name: "b@example.com [b]".to_string(),
            label: Some("b".to_string()),
            is_current: false,
            candidate: true,
            state: PriorityState::Ready(PriorityUsage {
                seven_day_left: 40,
                seven_day_reset: Some("in 3d".to_string()),
                five_hour_left: 0,
                five_hour_reset: Some("in 1h".to_string()),
                tier: 1,
                score: 2800,
            }),
        },
        PriorityRow {
            id: "a".to_string(),
            profile_name: "a@example.com [a]".to_string(),
            label: Some("a".to_string()),
            is_current: false,
            candidate: true,
            state: PriorityState::Ready(PriorityUsage {
                seven_day_left: 55,
                seven_day_reset: Some("in 4d".to_string()),
                five_hour_left: 45,
                five_hour_reset: Some("in 2h".to_string()),
                tier: 0,
                score: 5200,
            }),
        },
        PriorityRow {
            id: "c".to_string(),
            profile_name: "c@example.com [c]".to_string(),
            label: Some("c".to_string()),
            is_current: false,
            candidate: true,
            state: PriorityState::Unavailable("usage failed".to_string()),
        },
    ];
    rows.sort_by(priority_row_cmp);
    assert_eq!(rows[0].id, "a");
    assert_eq!(rows[1].id, "b");
    assert_eq!(rows[2].id, "c");
}

#[test]
fn render_priority_table_shows_unavailable_summary() {
    let rows = vec![
        PriorityRow {
            id: "a".to_string(),
            profile_name: "a@example.com [a]".to_string(),
            label: Some("a".to_string()),
            is_current: true,
            candidate: true,
            state: PriorityState::Ready(PriorityUsage {
                seven_day_left: 90,
                seven_day_reset: Some("in 6d".to_string()),
                five_hour_left: 50,
                five_hour_reset: Some("in 5h".to_string()),
                tier: 0,
                score: 7800,
            }),
        },
        PriorityRow {
            id: "b".to_string(),
            profile_name: "b@example.com [b]".to_string(),
            label: Some("b".to_string()),
            is_current: false,
            candidate: true,
            state: PriorityState::Unavailable("missing 7d window".to_string()),
        },
    ];
    let output = render_priority_table(&rows, false);
    assert!(output.contains("Priority ranking"));
    assert!(output.contains("5H RESET"));
    assert!(output.contains("7D RESET"));
    assert!(output.contains("UNAVAILABLE"));
    assert!(output.contains("Unavailable profiles"));
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
    let index = serde_json::json!({
        "version": 1,
        "active_profile_id": null,
        "profiles": {
            "bad": {
                "label": "bad",
                "last_used": 1,
                "added_at": 1
            }
        }
    });
    fs::write(
        &paths.profiles_index,
        serde_json::to_string(&index).unwrap(),
    )
    .unwrap();
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
    let mut index = ProfilesIndex::default();
    let id = resolve_save_id(&paths, &mut usage_map, &mut labels, &mut index, &tokens).unwrap();
    assert!(!id.is_empty());
    let id = resolve_sync_id(&paths, &mut usage_map, &mut labels, &mut index, &tokens).unwrap();
    assert!(id.is_some());
}

#[test]
fn rename_profile_id_errors_when_missing() {
    let dir = tempfile::tempdir().expect("tempdir");
    let paths = make_paths(dir.path());
    fs::create_dir_all(&paths.profiles).unwrap();
    let mut usage_map = BTreeMap::new();
    let mut labels = Labels::new();
    let mut index = ProfilesIndex::default();
    let err = rename_profile_id(
        &paths,
        &mut usage_map,
        &mut labels,
        &mut index,
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
        show_spinner: false,
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
    crate::switcher::ensure_paths(&paths).unwrap();
    save_profile(&paths, Some("team".to_string())).unwrap();
    list_profiles(&paths, false, false, false, false).unwrap();
    status_profiles(&paths, false).unwrap();
    let label = read_labels(&paths).unwrap().keys().next().cloned().unwrap();
    status_label(&paths, &label).unwrap();
    sync_current_readonly(&paths).unwrap();
}

#[test]
fn reserve_and_unreserve_profile_by_label() {
    let dir = tempfile::tempdir().expect("tempdir");
    let paths = make_paths(dir.path());
    fs::create_dir_all(&paths.profiles).unwrap();
    write_auth(&paths.auth, "acct", "a@b.com", "pro", "acc", "ref");
    crate::switcher::ensure_paths(&paths).unwrap();
    save_profile(&paths, Some("vps".to_string())).unwrap();

    reserve_profile(&paths, Some("vps".to_string())).unwrap();
    let labels = read_labels(&paths).unwrap();
    let id = resolve_label_id(&labels, "vps").unwrap();
    let index = read_profiles_index(&paths).unwrap();
    assert!(index.profiles.get(&id).is_some_and(|entry| entry.reserved));

    unreserve_profile(&paths, Some("vps".to_string())).unwrap();
    let index = read_profiles_index(&paths).unwrap();
    assert!(index.profiles.get(&id).is_some_and(|entry| !entry.reserved));
}

#[test]
fn priority_rows_mark_reserved_profiles_as_non_candidates() {
    let dir = tempfile::tempdir().expect("tempdir");
    let paths = make_paths(dir.path());
    fs::create_dir_all(&paths.profiles).unwrap();
    write_profile(&paths, "reserved", "acct-r", "reserved@example.com", "pro");
    write_profile(&paths, "normal", "acct-n", "normal@example.com", "pro");
    let index = serde_json::json!({
        "version": 1,
        "active_profile_id": null,
        "profiles": {
            "reserved": {
                "label": "vps",
                "last_used": 20,
                "added_at": 1,
                "reserved": true
            },
            "normal": {
                "label": "local",
                "last_used": 10,
                "added_at": 1
            }
        }
    });
    fs::write(
        &paths.profiles_index,
        serde_json::to_string(&index).unwrap(),
    )
    .unwrap();

    let snapshot = load_snapshot(&paths, false).unwrap();
    let rows = priority_rows(&paths, &snapshot, None, false);
    let reserved = rows.iter().find(|row| row.id == "reserved").unwrap();
    let normal = rows.iter().find(|row| row.id == "normal").unwrap();
    assert!(!reserved.candidate);
    assert!(reserved.profile_name.ends_with(RESERVED_DISPLAY_MARKER));
    assert!(normal.candidate);
}

#[test]
fn reserved_profiles_remain_selectable_by_label() {
    let dir = tempfile::tempdir().expect("tempdir");
    let paths = make_paths(dir.path());
    fs::create_dir_all(&paths.profiles).unwrap();
    write_profile(&paths, "reserved", "acct-r", "reserved@example.com", "pro");
    let index = serde_json::json!({
        "version": 1,
        "active_profile_id": null,
        "profiles": {
            "reserved": {
                "label": "vps",
                "last_used": 20,
                "added_at": 1,
                "reserved": true
            }
        }
    });
    fs::write(
        &paths.profiles_index,
        serde_json::to_string(&index).unwrap(),
    )
    .unwrap();

    let no_profiles = format_no_profiles(&paths, false);
    let (snapshot, ordered) = load_snapshot_ordered(&paths, true, &no_profiles).unwrap();
    let candidates = make_candidates(&paths, &snapshot, &ordered);
    let selected = select_by_label("vps", &snapshot.labels, &candidates).unwrap();
    assert_eq!(selected.id, "reserved");
    assert!(selected.display.ends_with(RESERVED_DISPLAY_MARKER));
}

#[test]
fn delete_profile_by_label() {
    let dir = tempfile::tempdir().expect("tempdir");
    let paths = make_paths(dir.path());
    fs::create_dir_all(&paths.profiles).unwrap();
    write_auth(&paths.auth, "acct", "a@b.com", "pro", "acc", "ref");
    crate::switcher::ensure_paths(&paths).unwrap();
    save_profile(&paths, Some("team".to_string())).unwrap();
    delete_profile(&paths, true, Some("team".to_string())).unwrap();
}
