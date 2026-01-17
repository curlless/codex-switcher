use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener};
use std::path::PathBuf;
use std::process::{Command, Output};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

struct TestEnv {
    home: PathBuf,
}

impl TestEnv {
    fn new() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let home = env::temp_dir().join(format!(
            "codex-profiles-test-{}-{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(home.join(".codex")).expect("create codex dir");
        Self { home }
    }

    fn codex_dir(&self) -> PathBuf {
        self.home.join(".codex")
    }

    fn profiles_dir(&self) -> PathBuf {
        self.codex_dir().join("profiles")
    }

    fn write_config(&self, base_url: &str) {
        let path = self.codex_dir().join("config.toml");
        let contents = format!("chatgpt_base_url = \"{}\"\n", base_url);
        fs::write(path, contents).expect("write config");
    }

    fn write_auth(&self, account_id: &str, email: &str, plan: &str, access_token: &str) {
        let id_token = build_id_token(email, plan);
        let path = self.codex_dir().join("auth.json");
        let value = serde_json::json!({
            "tokens": {
                "account_id": account_id,
                "id_token": id_token,
                "access_token": access_token
            }
        });
        fs::write(path, serde_json::to_string(&value).expect("serialize auth"))
            .expect("write auth.json");
    }

    fn write_auth_with_refresh(
        &self,
        account_id: &str,
        email: &str,
        plan: &str,
        access_token: &str,
        refresh_token: &str,
    ) {
        let id_token = build_id_token(email, plan);
        let path = self.codex_dir().join("auth.json");
        let value = serde_json::json!({
            "tokens": {
                "account_id": account_id,
                "id_token": id_token,
                "access_token": access_token,
                "refresh_token": refresh_token
            }
        });
        fs::write(path, serde_json::to_string(&value).expect("serialize auth"))
            .expect("write auth.json");
    }

    fn write_usage(&self, entries: &[(&str, u64)]) {
        fs::create_dir_all(self.profiles_dir()).expect("create profiles dir");
        let out = format_usage_contents(entries);
        let path = self.profiles_dir().join("usage.tsv");
        fs::write(path, out).expect("write usage.tsv");
    }

    fn write_usage_raw(&self, contents: &str) {
        fs::create_dir_all(self.profiles_dir()).expect("create profiles dir");
        let path = self.profiles_dir().join("usage.tsv");
        fs::write(path, contents).expect("write usage.tsv");
    }

    fn read_auth(&self) -> String {
        let path = self.codex_dir().join("auth.json");
        fs::read_to_string(path).expect("read auth.json")
    }

    fn run(&self, args: &[&str]) -> String {
        let output = self.run_output(args);
        if !output.status.success() {
            panic!(
                "command failed: {:?}\nstdout:\n{}\nstderr:\n{}",
                args,
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        ascii_only(String::from_utf8_lossy(&output.stdout).as_ref())
    }

    fn run_expect_error(&self, args: &[&str]) -> String {
        let output = self.run_output(args);
        if output.status.success() {
            panic!(
                "command unexpectedly succeeded: {:?}\nstdout:\n{}",
                args,
                String::from_utf8_lossy(&output.stdout)
            );
        }
        ascii_only(String::from_utf8_lossy(&output.stderr).as_ref())
    }

    fn run_output(&self, args: &[&str]) -> Output {
        self.run_output_with_env(args, &[])
    }

    fn run_output_with_env(&self, args: &[&str], extra_env: &[(&str, &str)]) -> Output {
        let bin = resolve_bin_path();
        let mut cmd = Command::new(bin);
        cmd.args(args)
            .env("HOME", &self.home)
            .env("CODEX_PROFILES_HOME", &self.home)
            .env("CODEX_PROFILES_COMMAND", "codex-profiles")
            .env("CODEX_PROFILES_SKIP_UPDATE", "1")
            .env("NO_COLOR", "1")
            .env("LANG", "C")
            .env("LC_ALL", "C");
        for (key, value) in extra_env {
            cmd.env(key, value);
        }
        if cfg!(windows) {
            cmd.env("USERPROFILE", &self.home);
            if let Some(home_str) = self.home.to_str()
                && let Some(idx) = home_str.find(':')
            {
                let (drive, rest) = home_str.split_at(idx + 1);
                cmd.env("HOMEDRIVE", drive);
                cmd.env("HOMEPATH", rest);
            }
        }
        cmd.output().expect("run command")
    }

    fn run_with_env(&self, args: &[&str], extra_env: &[(&str, &str)]) -> String {
        let output = self.run_output_with_env(args, extra_env);
        if !output.status.success() {
            panic!(
                "command failed: {:?}\nstdout:\n{}\nstderr:\n{}",
                args,
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        ascii_only(String::from_utf8_lossy(&output.stdout).as_ref())
    }
}

impl Drop for TestEnv {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.home);
    }
}

fn format_usage_contents(entries: &[(&str, u64)]) -> String {
    let mut out = String::new();
    for (id, ts) in entries {
        out.push_str(id);
        out.push('\t');
        out.push_str(&ts.to_string());
        out.push('\n');
    }
    out
}

fn build_id_token(email: &str, plan: &str) -> String {
    let header = r#"{"alg":"none","typ":"JWT"}"#;
    let payload = format!(
        "{{\"email\":\"{email}\",\"https://api.openai.com/auth\":{{\"chatgpt_plan_type\":\"{plan}\"}}}}"
    );
    let header = URL_SAFE_NO_PAD.encode(header);
    let payload = URL_SAFE_NO_PAD.encode(payload);
    format!("{header}.{payload}.")
}

fn ascii_only(raw: &str) -> String {
    let output = raw.replace('\r', "");
    let filtered: String = output.chars().filter(|ch| ch.is_ascii()).collect();
    filtered
        .lines()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n")
}

fn resolve_bin_path() -> PathBuf {
    if let Ok(path) = env::var("CARGO_BIN_EXE_codex-profiles") {
        return PathBuf::from(path);
    }
    let exe = env::current_exe().expect("current exe");
    let target_dir = exe
        .parent()
        .and_then(|path| path.parent())
        .expect("target dir");
    let bin_name = if cfg!(windows) {
        "codex-profiles.exe"
    } else {
        "codex-profiles"
    };
    target_dir.join(bin_name)
}

fn seed_profiles(env: &TestEnv) {
    env.write_auth("acct-alpha", "alpha@example.com", "team", "token-alpha");
    env.run(&["save", "--label", "alpha"]);
    env.write_auth("acct-beta", "beta@example.com", "team", "token-beta");
    env.run(&["save", "--label", "beta"]);
}

fn start_usage_server(
    body: &'static str,
    max_requests: usize,
) -> std::io::Result<(SocketAddr, thread::JoinHandle<()>)> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    listener.set_nonblocking(true)?;
    let addr = listener.local_addr()?;
    let handle = thread::spawn(move || {
        let mut handled = 0usize;
        let mut last_activity = Instant::now();
        loop {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let mut buf = [0u8; 1024];
                    let _ = stream.read(&mut buf);
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    let _ = stream.write_all(response.as_bytes());
                    handled += 1;
                    last_activity = Instant::now();
                    if handled >= max_requests {
                        break;
                    }
                }
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    if last_activity.elapsed() > Duration::from_secs(2) {
                        break;
                    }
                    thread::sleep(Duration::from_millis(25));
                }
                Err(_) => break,
            }
        }
    });
    Ok((addr, handle))
}

fn assert_status_output(env: &TestEnv, args: &[&str], expected_profiles: &[&str]) {
    let body = r#"{"rate_limit":{"primary_window":{"used_percent":20,"limit_window_seconds":18000,"reset_at":2000000000},"secondary_window":{"used_percent":50,"limit_window_seconds":604800,"reset_at":2000600000}}}"#;
    let server = start_usage_server(body, 6);
    if let Ok((addr, handle)) = server {
        env.write_config(&format!("http://{addr}/backend-api"));
        let output = env.run(args);
        for name in expected_profiles {
            assert!(output.contains(name));
        }
        if !output.contains("resets ") {
            assert!(output.contains("Error: failed to "));
        }
        let _ = handle.join();
    } else {
        env.write_config("http://127.0.0.1:1/backend-api");
        let output = env.run(args);
        for name in expected_profiles {
            assert!(output.contains(name));
        }
        assert!(output.contains("Error: failed to fetch usage"));
    }
}

fn assert_order(output: &str, first: &str, second: &str) {
    let first_idx = output
        .find(first)
        .unwrap_or_else(|| panic!("missing expected text: {first}"));
    let second_idx = output
        .find(second)
        .unwrap_or_else(|| panic!("missing expected text: {second}"));
    assert!(
        first_idx < second_idx,
        "expected '{first}' before '{second}' in output"
    );
}

#[test]
fn ui_save_command() {
    let env = TestEnv::new();
    env.write_auth("acct-alpha", "alpha@example.com", "team", "token-alpha");
    let output = env.run(&["save", "--label", "alpha"]);
    assert!(output.contains("Saved profile"));
    assert!(output.contains("alpha@example.com"));
    let profile_path = env.profiles_dir().join("alpha@example.com-team.json");
    assert!(profile_path.is_file());
}

#[test]
fn ui_save_missing_auth() {
    let env = TestEnv::new();
    let err = env.run_expect_error(&["save"]);
    assert!(err.contains("Codex auth file not found"));
}

#[test]
fn ui_save_empty_label() {
    let env = TestEnv::new();
    env.write_auth("acct-alpha", "alpha@example.com", "team", "token-alpha");
    let err = env.run_expect_error(&["save", "--label", "   "]);
    assert!(err.contains("label cannot be empty"));
}

#[test]
fn ui_save_trims_label() {
    let env = TestEnv::new();
    env.write_auth("acct-alpha", "alpha@example.com", "team", "token-alpha");
    env.run(&["save", "--label", "  work  "]);
    let labels_path = env.profiles_dir().join("labels.json");
    let labels = fs::read_to_string(labels_path).expect("read labels.json");
    let json: serde_json::Value = serde_json::from_str(&labels).expect("parse labels");
    assert!(json.get("work").is_some());
}

#[test]
fn ui_save_skips_rename_without_usage_signal() {
    let env = TestEnv::new();
    env.write_auth("acct-alpha", "alpha@example.com", "team", "token-alpha");
    fs::create_dir_all(env.profiles_dir()).expect("create profiles dir");
    let profile_one = env.profiles_dir().join("alpha@example.com-team-old.json");
    fs::copy(env.codex_dir().join("auth.json"), &profile_one).expect("seed profile one");
    env.write_auth(
        "acct-alpha",
        "alpha@example.com",
        "team",
        "token-alpha-rotated",
    );
    let profile_two = env.profiles_dir().join("alpha@example.com-team-alt.json");
    fs::copy(env.codex_dir().join("auth.json"), &profile_two).expect("seed profile two");
    env.write_usage_raw("");
    env.write_auth("acct-alpha", "alpha@example.com", "team", "token-alpha-new");
    env.run(&["save"]);
    assert!(profile_one.is_file());
    assert!(profile_two.is_file());
}

#[test]
fn ui_save_duplicate_label() {
    let env = TestEnv::new();
    env.write_auth("acct-alpha", "alpha@example.com", "team", "token-alpha");
    env.run(&["save", "--label", "alpha"]);
    env.write_auth("acct-beta", "beta@example.com", "team", "token-beta");
    let err = env.run_expect_error(&["save", "--label", "alpha"]);
    assert!(err.contains("label 'alpha' already exists"));
}

#[test]
fn ui_load_command() {
    let env = TestEnv::new();
    seed_profiles(&env);
    env.write_auth("acct-alpha", "alpha@example.com", "team", "token-alpha");
    let output = env.run(&["load", "--label", "beta"]);
    assert!(output.contains("Loaded profile"));
    assert!(output.contains("beta@example.com"));
    assert!(env.read_auth().contains("acct-beta"));
}

#[test]
fn ui_load_label_not_found() {
    let env = TestEnv::new();
    seed_profiles(&env);
    let err = env.run_expect_error(&["load", "--label", "missing"]);
    assert!(err.contains("label 'missing' was not found"));
}

#[test]
fn ui_load_rejects_invalid_profile_json() {
    let env = TestEnv::new();
    fs::create_dir_all(env.profiles_dir()).expect("create profiles dir");
    let profile_path = env.profiles_dir().join("broken.json");
    fs::write(&profile_path, "{").expect("write profile");
    env.write_usage(&[("broken", 123)]);
    let labels_path = env.profiles_dir().join("labels.json");
    fs::write(labels_path, r#"{"broken":"broken"}"#).expect("write labels");
    let err = env.run_expect_error(&["load", "--label", "broken"]);
    assert!(err.contains("No saved profiles.") || err.contains("label 'broken' was not found"));
    assert!(!profile_path.is_file());
}

#[test]
fn ui_load_requires_tty() {
    let env = TestEnv::new();
    seed_profiles(&env);
    env.write_auth("acct-alpha", "alpha@example.com", "team", "token-alpha");
    let err = env.run_expect_error(&["load"]);
    assert!(err.contains("load selection requires a TTY"));
}

#[test]
fn ui_load_unsaved_profile_requires_prompt() {
    let env = TestEnv::new();
    seed_profiles(&env);
    env.write_auth(
        "acct-current",
        "current@example.com",
        "team",
        "token-current",
    );
    let err = env.run_expect_error(&["load", "--label", "alpha"]);
    assert!(err.contains("current profile is not saved"));
}

#[test]
fn ui_delete_command() {
    let env = TestEnv::new();
    seed_profiles(&env);
    let output = env.run(&["delete", "--label", "beta", "--yes"]);
    assert!(output.contains("Deleted profile"));
    assert!(output.contains("beta@example.com"));
    let profile_path = env.profiles_dir().join("beta@example.com-team.json");
    assert!(!profile_path.is_file());
}

#[test]
fn ui_delete_requires_tty() {
    let env = TestEnv::new();
    seed_profiles(&env);
    let err = env.run_expect_error(&["delete"]);
    assert!(err.contains("delete selection requires a TTY"));
}

#[test]
fn ui_delete_requires_confirmation() {
    let env = TestEnv::new();
    seed_profiles(&env);
    let err = env.run_expect_error(&["delete", "--label", "beta"]);
    assert!(err.contains("deletion requires confirmation"));
}

#[test]
fn ui_delete_no_profiles() {
    let env = TestEnv::new();
    env.write_auth("acct-alpha", "alpha@example.com", "team", "token-alpha");
    let output = env.run(&["delete", "--yes"]);
    assert!(output.contains("No saved profiles."));
}

#[test]
fn ui_list_command() {
    let env = TestEnv::new();
    seed_profiles(&env);
    env.write_usage(&[
        ("alpha@example.com-team", 200),
        ("beta@example.com-team", 100),
    ]);
    env.write_auth(
        "acct-current",
        "current@example.com",
        "team",
        "token-current",
    );
    let output = env.run(&["list"]);
    assert!(output.contains("current@example.com"));
    assert!(output.contains("WARNING: This profile is not saved yet."));
    assert!(output.contains("Run `codex-profiles save` to save this profile."));
    assert!(output.contains("alpha@example.com"));
    assert!(output.contains("beta@example.com"));
    assert_order(&output, "current@example.com", "alpha@example.com");
    assert_order(&output, "alpha@example.com", "beta@example.com");
}

#[test]
fn ui_list_free_plan() {
    let env = TestEnv::new();
    env.write_auth("acct-free", "free@example.com", "free", "token-free");
    env.run(&["save", "--label", "free"]);
    let output = env.run(&["list"]);
    assert!(output.contains("You need a ChatGPT subscription to use Codex CLI"));
}

#[test]
fn ui_sync_current_updates_profile() {
    let env = TestEnv::new();
    env.write_auth("acct-alpha", "alpha@example.com", "team", "token-alpha");
    env.run(&["save", "--label", "alpha"]);
    env.write_auth(
        "acct-alpha",
        "alpha@example.com",
        "team",
        "token-alpha-rotated",
    );
    env.run(&["list"]);
    let profile_path = env.profiles_dir().join("alpha@example.com-team.json");
    let contents = fs::read_to_string(profile_path).expect("read profile");
    assert!(contents.contains("token-alpha-rotated"));
}

#[test]
fn ui_usage_normalizes_entries() {
    let env = TestEnv::new();
    seed_profiles(&env);
    env.write_usage_raw(
        "alpha@example.com-team\t100\nalpha@example.com-team\t200\n\nbeta@example.com-team\t150\ninvalid\nbeta@example.com-team\t140\n",
    );
    env.write_auth("acct-alpha", "alpha@example.com", "team", "token-alpha");
    env.run(&["save", "--label", "alpha"]);
    let contents =
        fs::read_to_string(env.profiles_dir().join("usage.tsv")).expect("read usage.tsv");
    let mut ids = std::collections::HashMap::new();
    for line in contents.lines() {
        assert!(!line.trim().is_empty());
        let (id, _ts) = line
            .split_once('\t')
            .unwrap_or_else(|| panic!("missing tab in usage line: {line}"));
        *ids.entry(id.to_string()).or_insert(0usize) += 1;
    }
    assert_eq!(ids.get("alpha@example.com-team"), Some(&1));
    assert_eq!(ids.get("beta@example.com-team"), Some(&1));
}

#[test]
fn ui_save_missing_usage_entries() {
    let env = TestEnv::new();
    env.write_auth("acct-alpha", "alpha@example.com", "team", "token-alpha");
    let profile_path = env.profiles_dir().join("alpha@example.com-team.json");
    fs::create_dir_all(env.profiles_dir()).expect("create profiles dir");
    fs::copy(env.codex_dir().join("auth.json"), &profile_path).expect("seed profile file");
    env.write_usage_raw("");
    env.write_auth("acct-beta", "beta@example.com", "team", "token-beta");
    let output = env.run(&["save", "--label", "beta"]);
    assert!(output.contains("Saved profile"));
    let contents =
        fs::read_to_string(env.profiles_dir().join("usage.tsv")).expect("read usage.tsv");
    assert!(contents.contains("alpha@example.com-team\t"));
    assert!(contents.contains("beta@example.com-team\t"));
}

#[test]
fn ui_status_command() {
    let env = TestEnv::new();
    seed_profiles(&env);
    env.write_auth("acct-alpha", "alpha@example.com", "team", "token-alpha");
    assert_status_output(&env, &["status"], &["alpha@example.com"]);
    let output = env.run(&["status"]);
    assert!(output.contains("alpha@example.com"));
    assert!(!output.contains("beta@example.com"));
}

#[test]
fn ui_status_label_command() {
    let env = TestEnv::new();
    seed_profiles(&env);
    env.write_auth("acct-alpha", "alpha@example.com", "team", "token-alpha");
    assert_status_output(&env, &["status", "--label", "beta"], &["beta@example.com"]);
    let output = env.run(&["status", "--label", "beta"]);
    assert!(output.contains("beta@example.com"));
    assert!(!output.contains("alpha@example.com"));
}

#[test]
fn ui_status_all_command() {
    let env = TestEnv::new();
    seed_profiles(&env);
    env.write_usage(&[
        ("alpha@example.com-team", 200),
        ("beta@example.com-team", 100),
    ]);
    env.write_auth("acct-alpha", "alpha@example.com", "team", "token-alpha");
    assert_status_output(
        &env,
        &["status", "--all"],
        &["alpha@example.com", "beta@example.com"],
    );
    let output = env.run(&["status", "--all"]);
    assert_order(&output, "alpha@example.com", "beta@example.com");
}

#[test]
fn ui_status_all_no_usage() {
    let env = TestEnv::new();
    seed_profiles(&env);
    env.write_usage(&[
        ("alpha@example.com-team", 200),
        ("beta@example.com-team", 100),
    ]);
    env.write_auth("acct-alpha", "alpha@example.com", "team", "token-alpha");
    env.write_config("http://127.0.0.1:1/backend-api");
    let output = env.run(&["status", "--all"]);
    assert!(output.contains("alpha@example.com"));
    assert!(output.contains("beta@example.com"));
    assert!(output.contains("Error: failed to fetch usage"));
}

#[test]
fn ui_import_every_code_accounts() {
    let env = TestEnv::new();
    let code_home = env.home.join(".code");
    fs::create_dir_all(&code_home).expect("create code home");
    let id_token = build_id_token("alpha@example.com", "team");
    let accounts = serde_json::json!({
        "version": 1,
        "active_account_id": "acc-1",
        "accounts": [
            {
                "id": "acc-1",
                "mode": "chatgpt",
                "label": "work",
                "tokens": {
                    "id_token": id_token,
                    "access_token": "access-1",
                    "refresh_token": "refresh-1",
                    "account_id": "acct-alpha"
                },
                "last_used_at": "2026-01-01T00:00:00Z"
            }
        ]
    });
    let accounts_path = code_home.join("auth_accounts.json");
    fs::write(
        &accounts_path,
        serde_json::to_string_pretty(&accounts).expect("serialize accounts"),
    )
    .expect("write accounts");

    env.run(&[
        "import",
        "--every-code",
        "--code-home",
        code_home.to_str().expect("code home str"),
    ]);

    let mut found = false;
    for entry in fs::read_dir(env.profiles_dir()).expect("read profiles") {
        let entry = entry.expect("entry");
        let path = entry.path();
        if path.file_name().and_then(|name| name.to_str()) == Some("labels.json") {
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let contents = fs::read_to_string(&path).expect("read profile");
        if contents.contains("access-1") {
            found = true;
            break;
        }
    }
    assert!(found, "imported profile should exist");
}

#[test]
fn ui_export_every_code_accounts() {
    let env = TestEnv::new();
    env.write_auth_with_refresh(
        "acct-alpha",
        "alpha@example.com",
        "team",
        "token-alpha",
        "refresh-alpha",
    );
    env.run(&["save", "--label", "alpha"]);

    let api_key_path = env.codex_dir().join("auth.json");
    fs::write(&api_key_path, "{\"OPENAI_API_KEY\":\"sk-test\"}\n").expect("write auth");
    env.run(&["save", "--label", "api"]);

    let code_home = env.home.join(".code");
    env.run(&[
        "export",
        "--every-code",
        "--code-home",
        code_home.to_str().expect("code home str"),
        "--overwrite",
    ]);

    let accounts_path = code_home.join("auth_accounts.json");
    let contents = fs::read_to_string(&accounts_path).expect("read accounts");
    let value: serde_json::Value = serde_json::from_str(&contents).expect("parse accounts");
    let accounts = value
        .get("accounts")
        .and_then(|v| v.as_array())
        .expect("accounts array");
    assert_eq!(accounts.len(), 2);
    let modes: Vec<&str> = accounts
        .iter()
        .filter_map(|acc| acc.get("mode").and_then(|v| v.as_str()))
        .collect();
    assert!(modes.contains(&"chatgpt"));
    assert!(modes.contains(&"apikey"));
}

#[cfg(unix)]
#[test]
fn ui_export_every_code_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let env = TestEnv::new();
    env.write_auth_with_refresh(
        "acct-alpha",
        "alpha@example.com",
        "team",
        "token-alpha",
        "refresh-alpha",
    );
    env.run(&["save", "--label", "alpha"]);
    let code_home = env.home.join(".code");
    env.run(&[
        "export",
        "--every-code",
        "--code-home",
        code_home.to_str().expect("code home str"),
        "--overwrite",
    ]);
    let accounts_path = code_home.join("auth_accounts.json");
    let mode = fs::metadata(&accounts_path)
        .expect("metadata")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o600);
}

#[cfg(windows)]
#[test]
fn ui_export_every_code_overwrite_windows() {
    let env = TestEnv::new();
    env.write_auth("acct-alpha", "alpha@example.com", "team", "token-alpha");
    env.run(&["save", "--label", "alpha"]);
    let code_home = env.home.join(".code");
    env.run(&[
        "export",
        "--every-code",
        "--code-home",
        code_home.to_str().expect("code home str"),
        "--overwrite",
    ]);
    env.run(&[
        "export",
        "--every-code",
        "--code-home",
        code_home.to_str().expect("code home str"),
        "--overwrite",
    ]);
    assert!(code_home.join("auth_accounts.json").is_file());
}

#[test]
fn ui_import_every_code_filters_modes_and_labels() {
    let env = TestEnv::new();
    let code_home = env.home.join(".code");
    fs::create_dir_all(&code_home).expect("create code home");

    let valid_id_token = build_id_token("alpha@example.com", "team");
    let missing_id_token = build_id_token("missing@example.com", "team");
    let accounts = serde_json::json!({
        "version": 1,
        "accounts": [
            {
                "id": "acc-missing",
                "mode": "chatgpt",
                "label": "work",
                "tokens": {
                    "id_token": missing_id_token,
                    "access_token": "access-missing",
                    "refresh_token": "refresh-missing"
                }
            },
            {
                "id": "acc-anthropic",
                "mode": "anthropic",
                "label": "other",
                "tokens": {
                    "id_token": "bad",
                    "access_token": "anthropic-access",
                    "refresh_token": "anthropic-refresh",
                    "account_id": "acct-other"
                }
            },
            {
                "id": "acc-api",
                "mode": "apikey",
                "label": "work",
                "openai_api_key": "sk-test"
            },
            {
                "id": "acc-ok",
                "mode": "chatgpt",
                "label": "work",
                "tokens": {
                    "id_token": valid_id_token,
                    "access_token": "access-ok",
                    "refresh_token": "refresh-ok",
                    "account_id": "acct-ok"
                },
                "last_used_at": "2026-01-02T00:00:00Z"
            }
        ]
    });
    fs::write(
        code_home.join("auth_accounts.json"),
        serde_json::to_string_pretty(&accounts).expect("serialize accounts"),
    )
    .expect("write accounts");

    env.run(&[
        "import",
        "--every-code",
        "--code-home",
        code_home.to_str().expect("code home str"),
    ]);

    let mut profile_count = 0usize;
    let mut contains_missing = false;
    let mut contains_ok = false;
    for entry in fs::read_dir(env.profiles_dir()).expect("read profiles") {
        let entry = entry.expect("entry");
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        if path.file_name().and_then(|name| name.to_str()) == Some("labels.json") {
            continue;
        }
        profile_count += 1;
        let contents = fs::read_to_string(&path).expect("read profile");
        if contents.contains("access-missing") {
            contains_missing = true;
        }
        if contents.contains("access-ok") {
            contains_ok = true;
        }
        if contents.contains("anthropic-access") {
            panic!("unsupported mode imported");
        }
    }
    assert_eq!(profile_count, 2);
    assert!(!contains_missing, "missing account_id should be skipped");
    assert!(contains_ok, "valid account imported");

    let labels_path = env.profiles_dir().join("labels.json");
    let labels = fs::read_to_string(labels_path).expect("read labels.json");
    let json: serde_json::Value = serde_json::from_str(&labels).expect("parse labels");
    assert!(json.get("work").is_some());
    assert!(json.get("work-2").is_some());
}

#[test]
fn ui_list_removes_invalid_profiles() {
    let env = TestEnv::new();
    fs::create_dir_all(env.profiles_dir()).expect("create profiles dir");
    let bad_profile = env.profiles_dir().join("bad.json");
    fs::write(&bad_profile, "{").expect("write bad profile");
    let labels_path = env.profiles_dir().join("labels.json");
    fs::write(&labels_path, r#"{"bad":"bad"}"#).expect("write labels");

    env.run(&["list"]);

    assert!(!bad_profile.is_file());
    let labels = fs::read_to_string(labels_path).expect("read labels.json");
    let json: serde_json::Value = serde_json::from_str(&labels).expect("parse labels");
    assert!(json.get("bad").is_none());
}

#[test]
fn ui_status_refresh_updates_profile() {
    let env = TestEnv::new();
    let usage_body = r#"{"rate_limit":{"primary_window":{"used_percent":20,"limit_window_seconds":18000,"reset_at":2000000000}}}"#;
    let refresh_id_token = build_id_token("alpha@example.com", "team");
    let refresh_body = format!(
        "{{\"id_token\":\"{refresh_id_token}\",\"access_token\":\"new-access\",\"refresh_token\":\"new-refresh\"}}"
    );
    let (usage_addr, usage_handle) = start_usage_server(usage_body, 4).expect("usage server");
    let (refresh_addr, refresh_handle) =
        start_usage_server(Box::leak(refresh_body.into_boxed_str()), 2).expect("refresh server");

    env.write_config(&format!("http://{usage_addr}/backend-api"));
    env.write_auth_with_refresh(
        "acct-alpha",
        "alpha@example.com",
        "team",
        "old-access",
        "refresh-old",
    );
    env.run(&["save", "--label", "alpha"]);

    let refresh_url = format!("http://{refresh_addr}/token");
    env.run_with_env(
        &["status"],
        &[("CODEX_REFRESH_TOKEN_URL_OVERRIDE", refresh_url.as_str())],
    );

    let auth_contents = env.read_auth();
    assert!(auth_contents.contains("new-access"));
    let profile_path = env.profiles_dir().join("alpha@example.com-team.json");
    let profile_contents = fs::read_to_string(profile_path).expect("read profile");
    assert!(profile_contents.contains("new-access"));

    let _ = usage_handle.join();
    let _ = refresh_handle.join();
}
