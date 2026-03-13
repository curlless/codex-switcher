mod common;

use common::build_id_token;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

fn resolve_switcher_bin_path() -> PathBuf {
    if let Ok(path) = env::var("CARGO_BIN_EXE_codex-switcher") {
        return PathBuf::from(path);
    }
    let exe = env::current_exe().expect("current exe");
    let target_dir = exe
        .parent()
        .and_then(|path| path.parent())
        .expect("target dir");
    let bin_name = if cfg!(windows) {
        "codex-switcher.exe"
    } else {
        "codex-switcher"
    };
    target_dir.join(bin_name)
}

fn make_home() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    let home = env::temp_dir().join(format!(
        "codex-switcher-test-{}-{nanos}",
        std::process::id()
    ));
    fs::create_dir_all(home.join(".codex")).expect("create home");
    home
}

fn run_switcher(args: &[&str], home: &Path) -> std::process::Output {
    let bin = resolve_switcher_bin_path();
    let mut cmd = Command::new(bin);
    cmd.args(args)
        .env("HOME", home)
        .env("CODEX_SWITCHER_HOME", home)
        .env("CODEX_SWITCHER_SKIP_UPDATE", "1")
        .env("CODEX_SWITCHER_COMMAND", "codex-switcher")
        .env("NO_COLOR", "1")
        .stdin(Stdio::null());
    if cfg!(windows) {
        cmd.env("USERPROFILE", home);
    }
    cmd.output().expect("run switcher")
}

fn write_auth(home: &Path, account_id: &str, email: &str, plan: &str, access_token: &str) {
    let auth_path = home.join(".codex").join("auth.json");
    let id_token = build_id_token(email, plan);
    let payload = serde_json::json!({
        "tokens": {
            "account_id": account_id,
            "id_token": id_token,
            "access_token": access_token
        }
    });
    fs::write(
        auth_path,
        serde_json::to_string(&payload).expect("serialize auth"),
    )
    .expect("write auth");
}

fn write_saved_profile(
    home: &Path,
    id: &str,
    account_id: &str,
    email: &str,
    plan: &str,
    access_token: &str,
) {
    let profiles_dir = home.join(".codex").join("profiles");
    fs::create_dir_all(&profiles_dir).expect("create profiles dir");
    let id_token = build_id_token(email, plan);
    let payload = serde_json::json!({
        "tokens": {
            "account_id": account_id,
            "id_token": id_token,
            "access_token": access_token
        }
    });
    fs::write(
        profiles_dir.join(format!("{id}.json")),
        serde_json::to_string(&payload).expect("serialize profile"),
    )
    .expect("write profile");
}

fn write_profiles_index(
    home: &Path,
    entries: &[(&str, u64)],
    labels: &[(&str, &str)],
    active_id: Option<&str>,
) {
    let profiles_dir = home.join(".codex").join("profiles");
    fs::create_dir_all(&profiles_dir).expect("create profiles dir");
    let mut profiles = serde_json::Map::new();
    let label_map: std::collections::HashMap<_, _> = labels.iter().copied().collect();
    for (id, last_used) in entries {
        let mut entry = serde_json::Map::new();
        entry.insert("last_used".to_string(), serde_json::json!(last_used));
        entry.insert("added_at".to_string(), serde_json::json!(1));
        if let Some(label) = label_map.get(id) {
            entry.insert("label".to_string(), serde_json::json!(label));
        }
        profiles.insert(id.to_string(), serde_json::Value::Object(entry));
    }
    let index = serde_json::json!({
        "version": 1,
        "active_profile_id": active_id,
        "profiles": serde_json::Value::Object(profiles)
    });
    fs::write(
        profiles_dir.join("profiles.json"),
        serde_json::to_string(&index).expect("serialize profiles index"),
    )
    .expect("write profiles index");
}

fn write_config(home: &Path, base_url: &str) {
    fs::write(
        home.join(".codex").join("config.toml"),
        format!("chatgpt_base_url = \"{base_url}\"\n"),
    )
    .expect("write config");
}

fn write_switcher_reload_config(home: &Path, target: &str, reload_after_switch: bool) {
    let profiles_dir = home.join(".codex").join("profiles");
    fs::create_dir_all(&profiles_dir).expect("create profiles dir");
    fs::write(
        profiles_dir.join("config.toml"),
        format!(
            "[reload]\nprimary_target = \"{target}\"\n\n[switch]\nreload_after_switch = {reload_after_switch}\n"
        ),
    )
    .expect("write switcher config");
}

fn read_profiles_index(home: &Path) -> serde_json::Value {
    let path = home.join(".codex").join("profiles").join("profiles.json");
    serde_json::from_str(&fs::read_to_string(path).expect("read profiles index"))
        .expect("parse profiles index")
}

#[test]
fn switcher_help_shows_relay_login() {
    let home = make_home();
    let output = run_switcher(&["--help"], &home);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("relay-login"));
    assert!(stdout.contains("reload-app"));
    assert!(stdout.contains("reserve"));
    assert!(stdout.contains("unreserve"));
    let _ = fs::remove_dir_all(home);
}

#[test]
fn switcher_version_uses_binary_name() {
    let home = make_home();
    let output = run_switcher(&["--version"], &home);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(&format!("codex-switcher {}", env!("CARGO_PKG_VERSION"))));
    assert!(!stdout.contains("codex-switcher.exe"));
    let _ = fs::remove_dir_all(home);
}

#[test]
fn switcher_relay_login_requires_url_without_tty() {
    let home = make_home();
    let output = run_switcher(&["relay-login"], &home);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--url is required in non-interactive mode"));
    let _ = fs::remove_dir_all(home);
}

#[test]
fn switcher_list_works_with_empty_profiles() {
    let home = make_home();
    let output = run_switcher(&["list"], &home);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.to_lowercase().contains("no saved profiles"));
    let _ = fs::remove_dir_all(home);
}

#[test]
fn switcher_save_ignores_update_json_file() {
    let home = make_home();
    write_auth(
        &home,
        "acct-test",
        "tester@example.com",
        "plus",
        "access-token",
    );

    let profiles_dir = home.join(".codex").join("profiles");
    fs::create_dir_all(&profiles_dir).expect("create profiles dir");
    let update_json = profiles_dir.join("update.json");
    fs::write(&update_json, "{\"noop\":true}").expect("write update.json");

    let output = run_switcher(&["save"], &home);
    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("Removed invalid profile"));
    assert!(!stderr.contains("update.json"));
    assert!(update_json.is_file());
    let _ = fs::remove_dir_all(home);
}

#[test]
fn switcher_status_dedupes_same_account_rows() {
    let home = make_home();
    write_saved_profile(
        &home,
        "dup@example.com-team",
        "acct-dup",
        "dup@example.com",
        "team",
        "access-1",
    );
    write_saved_profile(
        &home,
        "dup@example.com-team-copy",
        "acct-dup",
        "dup@example.com",
        "team",
        "access-2",
    );
    write_profiles_index(
        &home,
        &[
            ("dup@example.com-team", 100),
            ("dup@example.com-team-copy", 200),
        ],
        &[("dup@example.com-team-copy", "work")],
        None,
    );
    write_config(&home, "http://127.0.0.1:1/backend-api");

    let output = run_switcher(&["status"], &home);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let ranking = stdout
        .split("Unavailable profiles")
        .next()
        .unwrap_or(stdout.as_ref());
    let profile_lines = ranking
        .lines()
        .filter(|line| line.contains("dup@example.com"))
        .collect::<Vec<_>>();
    assert_eq!(profile_lines.len(), 1, "status output:\n{stdout}");
    assert!(profile_lines[0].contains("dup@example.com [work]"));

    let _ = fs::remove_dir_all(home);
}

#[test]
fn switcher_reserve_and_unreserve_update_profile_metadata() {
    let home = make_home();
    write_auth(
        &home,
        "acct-test",
        "tester@example.com",
        "plus",
        "access-token",
    );

    let output = run_switcher(&["save", "--label", "vps"], &home);
    assert!(output.status.success());

    let output = run_switcher(&["reserve", "--label", "vps"], &home);
    assert!(output.status.success());
    let index = read_profiles_index(&home);
    assert_eq!(
        index["profiles"]["tester@example.com-plus"]["reserved"],
        serde_json::json!(true)
    );

    let output = run_switcher(&["unreserve", "--label", "vps"], &home);
    assert!(output.status.success());
    let index = read_profiles_index(&home);
    assert_eq!(
        index["profiles"]["tester@example.com-plus"]["reserved"],
        serde_json::json!(false)
    );

    let _ = fs::remove_dir_all(home);
}

#[test]
fn switcher_load_by_label_allows_reserved_profile() {
    let home = make_home();
    write_auth(
        &home,
        "acct-current",
        "current@example.com",
        "plus",
        "access-current",
    );
    write_saved_profile(
        &home,
        "current@example.com-plus",
        "acct-current",
        "current@example.com",
        "plus",
        "access-current",
    );
    write_saved_profile(
        &home,
        "reserved@example.com-plus",
        "acct-reserved",
        "reserved@example.com",
        "plus",
        "access-reserved",
    );
    let profiles_dir = home.join(".codex").join("profiles");
    let index = serde_json::json!({
        "version": 1,
        "active_profile_id": "current@example.com-plus",
        "profiles": {
            "current@example.com-plus": {
                "label": "current",
                "added_at": 1,
                "last_used": 3
            },
            "reserved@example.com-plus": {
                "label": "vps",
                "added_at": 1,
                "last_used": 2,
                "reserved": true
            }
        }
    });
    fs::write(
        profiles_dir.join("profiles.json"),
        serde_json::to_string(&index).expect("serialize profiles index"),
    )
    .expect("write profiles index");

    let output = run_switcher(&["load", "--label", "vps"], &home);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let auth = fs::read_to_string(home.join(".codex").join("auth.json")).expect("read auth");
    assert!(auth.contains("access-reserved"), "auth: {auth}");

    let _ = fs::remove_dir_all(home);
}

#[test]
fn switcher_load_uses_configured_reload_path() {
    let home = make_home();
    write_auth(
        &home,
        "acct-current",
        "current@example.com",
        "plus",
        "access-current",
    );
    write_saved_profile(
        &home,
        "current@example.com-plus",
        "acct-current",
        "current@example.com",
        "plus",
        "access-current",
    );
    write_saved_profile(
        &home,
        "reload@example.com-plus",
        "acct-reload",
        "reload@example.com",
        "plus",
        "access-reload",
    );
    write_profiles_index(
        &home,
        &[
            ("current@example.com-plus", 3),
            ("reload@example.com-plus", 2),
        ],
        &[("reload@example.com-plus", "reloadable")],
        Some("current@example.com-plus"),
    );
    write_switcher_reload_config(&home, "cursor", true);

    let output = run_switcher(&["load", "--label", "reloadable"], &home);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Loaded profile"), "stdout: {stdout}");
    assert!(stdout.contains("Reload hint:"), "stdout: {stdout}");

    let auth = fs::read_to_string(home.join(".codex").join("auth.json")).expect("read auth");
    assert!(auth.contains("access-reload"), "auth: {auth}");

    let _ = fs::remove_dir_all(home);
}

#[test]
fn switcher_load_can_skip_configured_reload_path() {
    let home = make_home();
    write_auth(
        &home,
        "acct-current",
        "current@example.com",
        "plus",
        "access-current",
    );
    write_saved_profile(
        &home,
        "current@example.com-plus",
        "acct-current",
        "current@example.com",
        "plus",
        "access-current",
    );
    write_saved_profile(
        &home,
        "reload@example.com-plus",
        "acct-reload",
        "reload@example.com",
        "plus",
        "access-reload",
    );
    write_profiles_index(
        &home,
        &[
            ("current@example.com-plus", 3),
            ("reload@example.com-plus", 2),
        ],
        &[("reload@example.com-plus", "reloadable")],
        Some("current@example.com-plus"),
    );
    write_switcher_reload_config(&home, "cursor", true);

    let output = run_switcher(&["load", "--label", "reloadable", "--no-reload-app"], &home);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Loaded profile"), "stdout: {stdout}");
    assert!(!stdout.contains("Reload hint:"), "stdout: {stdout}");

    let auth = fs::read_to_string(home.join(".codex").join("auth.json")).expect("read auth");
    assert!(auth.contains("access-reload"), "auth: {auth}");

    let _ = fs::remove_dir_all(home);
}

#[test]
fn switcher_load_can_force_reload_when_config_disabled() {
    let home = make_home();
    write_auth(
        &home,
        "acct-current",
        "current@example.com",
        "plus",
        "access-current",
    );
    write_saved_profile(
        &home,
        "current@example.com-plus",
        "acct-current",
        "current@example.com",
        "plus",
        "access-current",
    );
    write_saved_profile(
        &home,
        "reload@example.com-plus",
        "acct-reload",
        "reload@example.com",
        "plus",
        "access-reload",
    );
    write_profiles_index(
        &home,
        &[
            ("current@example.com-plus", 3),
            ("reload@example.com-plus", 2),
        ],
        &[("reload@example.com-plus", "reloadable")],
        Some("current@example.com-plus"),
    );
    write_switcher_reload_config(&home, "cursor", false);

    let output = run_switcher(
        &["load", "--label", "reloadable", "--reload-app", "codex"],
        &home,
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Loaded profile"), "stdout: {stdout}");
    assert!(stdout.contains("Reload hint:"), "stdout: {stdout}");

    let auth = fs::read_to_string(home.join(".codex").join("auth.json")).expect("read auth");
    assert!(auth.contains("access-reload"), "auth: {auth}");

    let _ = fs::remove_dir_all(home);
}
