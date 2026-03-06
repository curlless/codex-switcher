mod common;

use common::build_id_token;
use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::sync::Mutex;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const ALPHA_ACCOUNT: &str = "acct-alpha";
const ALPHA_EMAIL: &str = "alpha@example.com";
const ALPHA_PLAN: &str = "team";
const ALPHA_TOKEN: &str = "token-alpha";
const ALPHA_ID: &str = "alpha@example.com-team";
const BETA_ACCOUNT: &str = "acct-beta";
const BETA_EMAIL: &str = "beta@example.com";
const BETA_PLAN: &str = "team";
const BETA_TOKEN: &str = "token-beta";
const BETA_ID: &str = "beta@example.com-team";
const FREE_ACCOUNT: &str = "acct-free";
const FREE_EMAIL: &str = "free@example.com";
const FREE_PLAN: &str = "free";
const FREE_TOKEN: &str = "token-free";

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
            "codex-switcher-test-{}-{nanos}",
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

    fn write_auth_base(
        &self,
        account_id: &str,
        email: &str,
        plan: &str,
        access_token: &str,
        refresh_token: Option<&str>,
    ) {
        let id_token = build_id_token(email, plan);
        let mut tokens = serde_json::Map::new();
        tokens.insert(
            "account_id".to_string(),
            serde_json::Value::String(account_id.to_string()),
        );
        tokens.insert("id_token".to_string(), serde_json::Value::String(id_token));
        tokens.insert(
            "access_token".to_string(),
            serde_json::Value::String(access_token.to_string()),
        );
        if let Some(refresh_token) = refresh_token {
            tokens.insert(
                "refresh_token".to_string(),
                serde_json::Value::String(refresh_token.to_string()),
            );
        }
        let value = serde_json::Value::Object({
            let mut root = serde_json::Map::new();
            root.insert("tokens".to_string(), serde_json::Value::Object(tokens));
            root
        });
        let path = self.codex_dir().join("auth.json");
        fs::write(path, serde_json::to_string(&value).expect("serialize auth"))
            .expect("write auth.json");
    }

    fn write_auth(&self, account_id: &str, email: &str, plan: &str, access_token: &str) {
        self.write_auth_base(account_id, email, plan, access_token, None);
    }

    fn write_auth_with_refresh(
        &self,
        account_id: &str,
        email: &str,
        plan: &str,
        access_token: &str,
        refresh_token: &str,
    ) {
        self.write_auth_base(account_id, email, plan, access_token, Some(refresh_token));
    }

    fn write_profiles_index(
        &self,
        entries: &[(&str, u64)],
        labels: &[(&str, &str)],
        active_id: Option<&str>,
    ) {
        fs::create_dir_all(self.profiles_dir()).expect("create profiles dir");
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
        let path = self.profiles_dir().join("profiles.json");
        fs::write(
            path,
            serde_json::to_string(&index).expect("serialize profiles.json"),
        )
        .expect("write profiles.json");
    }

    fn read_auth(&self) -> String {
        let path = self.codex_dir().join("auth.json");
        fs::read_to_string(path).expect("read auth.json")
    }

    fn run(&self, args: &[&str]) -> String {
        let output = self.run_output(args);
        self.assert_success(args, output)
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
            .env("CODEX_PROFILES_COMMAND", "codex-switcher")
            .env("CODEX_PROFILES_SKIP_UPDATE", "1")
            .env("NO_COLOR", "1")
            .env("LANG", "C")
            .env("LC_ALL", "C")
            .stdin(Stdio::null());
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
        self.assert_success(args, output)
    }

    fn assert_success(&self, args: &[&str], output: Output) -> String {
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

fn seed_alpha(env: &TestEnv) {
    env.write_auth(ALPHA_ACCOUNT, ALPHA_EMAIL, ALPHA_PLAN, ALPHA_TOKEN);
}

fn seed_alpha_with_token(env: &TestEnv, token: &str) {
    env.write_auth(ALPHA_ACCOUNT, ALPHA_EMAIL, ALPHA_PLAN, token);
}

fn seed_beta(env: &TestEnv) {
    env.write_auth(BETA_ACCOUNT, BETA_EMAIL, BETA_PLAN, BETA_TOKEN);
}

fn seed_free(env: &TestEnv) {
    env.write_auth(FREE_ACCOUNT, FREE_EMAIL, FREE_PLAN, FREE_TOKEN);
}

fn seed_current(env: &TestEnv) {
    env.write_auth(
        "acct-current",
        "current@example.com",
        "team",
        "token-current",
    );
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

fn seed_profiles(env: &TestEnv) {
    seed_alpha(env);
    env.run(&["save", "--label", "alpha"]);
    seed_beta(env);
    env.run(&["save", "--label", "beta"]);
}

#[allow(dead_code)]
struct RelayCallbackListener {
    addr: SocketAddr,
    request_target_rx: Mutex<Receiver<String>>,
    shutdown_tx: Option<Sender<()>>,
    handle: Option<thread::JoinHandle<()>>,
}

#[allow(dead_code)]
impl RelayCallbackListener {
    fn start(status_code: u16, body: &str) -> std::io::Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        listener.set_nonblocking(true)?;
        let addr = listener.local_addr()?;
        let response = build_http_response(status_code, body);
        let (request_target_tx, request_target_rx) = mpsc::channel();
        let (shutdown_tx, shutdown_rx) = mpsc::channel();

        let handle = thread::spawn(move || {
            loop {
                if shutdown_rx.try_recv().is_ok() {
                    break;
                }
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        let _ = stream.set_nonblocking(false);
                        let _ = stream.set_read_timeout(Some(Duration::from_millis(500)));
                        let _ = stream.set_write_timeout(Some(Duration::from_millis(500)));
                        if let Some(target) = read_request_target(&mut stream) {
                            let _ = request_target_tx.send(target);
                        }
                        let _ = stream.write_all(&response);
                        let _ = stream.flush();
                        break;
                    }
                    Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(Self {
            addr,
            request_target_rx: Mutex::new(request_target_rx),
            shutdown_tx: Some(shutdown_tx),
            handle: Some(handle),
        })
    }

    fn addr(&self) -> SocketAddr {
        self.addr
    }

    fn recv_request_target(&self, timeout: Duration) -> Option<String> {
        let receiver = self
            .request_target_rx
            .lock()
            .expect("lock relay listener request receiver");
        receiver.recv_timeout(timeout).ok()
    }

    fn shutdown_and_join(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for RelayCallbackListener {
    fn drop(&mut self) {
        self.shutdown_and_join();
    }
}

fn build_http_response(status_code: u16, body: &str) -> Vec<u8> {
    format!(
        "HTTP/1.1 {status_code} {}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        status_reason_phrase(status_code),
        body.len()
    )
    .into_bytes()
}

fn status_reason_phrase(status_code: u16) -> &'static str {
    match status_code {
        200 => "OK",
        201 => "Created",
        202 => "Accepted",
        204 => "No Content",
        301 => "Moved Permanently",
        302 => "Found",
        303 => "See Other",
        307 => "Temporary Redirect",
        308 => "Permanent Redirect",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        409 => "Conflict",
        422 => "Unprocessable Content",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        _ => "Test Response",
    }
}

fn read_request_target(stream: &mut TcpStream) -> Option<String> {
    let raw = read_http_request(stream);
    if raw.is_empty() {
        return None;
    }

    let request_line_end = raw
        .windows(2)
        .position(|pair| pair == b"\r\n")
        .unwrap_or(raw.len());
    let request_line = String::from_utf8_lossy(&raw[..request_line_end]);
    let mut parts = request_line.split_whitespace();
    let _ = parts.next()?;
    let request_target = parts.next()?;
    Some(request_target.to_string())
}

fn read_http_request(stream: &mut TcpStream) -> Vec<u8> {
    let _ = stream.set_read_timeout(Some(Duration::from_millis(500)));
    let mut raw = Vec::with_capacity(1024);
    let mut chunk = [0u8; 512];
    let mut idle_reads = 0u8;
    loop {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(read) => {
                raw.extend_from_slice(&chunk[..read]);
                idle_reads = 0;
                if http_request_complete(&raw) || raw.len() >= 64 * 1024 {
                    break;
                }
            }
            Err(err)
                if err.kind() == std::io::ErrorKind::WouldBlock
                    || err.kind() == std::io::ErrorKind::TimedOut =>
            {
                idle_reads = idle_reads.saturating_add(1);
                if idle_reads >= 2 {
                    break;
                }
            }
            Err(_) => break,
        }
    }
    raw
}

fn http_request_complete(raw: &[u8]) -> bool {
    let header_end = match raw.windows(4).position(|chunk| chunk == b"\r\n\r\n") {
        Some(pos) => pos + 4,
        None => return false,
    };
    let headers = String::from_utf8_lossy(&raw[..header_end]);
    let content_length = headers
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            if name.eq_ignore_ascii_case("content-length") {
                value.trim().parse::<usize>().ok()
            } else {
                None
            }
        })
        .unwrap_or(0);
    let chunked = headers.lines().any(|line| {
        let Some((name, value)) = line.split_once(':') else {
            return false;
        };
        name.eq_ignore_ascii_case("transfer-encoding")
            && value.to_ascii_lowercase().contains("chunked")
    });
    if chunked {
        return raw[header_end..]
            .windows(5)
            .any(|window| window == b"0\r\n\r\n");
    }
    raw.len() >= header_end + content_length
}

fn start_usage_server(
    body: &'static str,
    max_requests: usize,
) -> std::io::Result<(SocketAddr, thread::JoinHandle<()>)> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    listener.set_nonblocking(true)?;
    let addr = listener.local_addr()?;
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
        body.len(),
        body
    )
    .into_bytes();
    let handle = thread::spawn(move || {
        let mut handled = 0usize;
        let mut last_activity = Instant::now();
        loop {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let _ = stream.set_nonblocking(false);
                    let _ = stream.set_read_timeout(Some(Duration::from_millis(500)));
                    let _ = stream.set_write_timeout(Some(Duration::from_millis(500)));
                    let _ = read_http_request(&mut stream);
                    let _ = stream.write_all(&response);
                    let _ = stream.flush();
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
        assert_contains_all(&output, expected_profiles);
        if !output.contains("resets ") && !output.contains("Priority ranking") {
            assert!(output.contains("Error: failed to "));
        }
        let _ = handle.join();
    } else {
        env.write_config("http://127.0.0.1:1/backend-api");
        let output = env.run(args);
        assert_contains_all(&output, expected_profiles);
        assert!(output.contains("Error: failed to fetch usage"));
    }
}

fn assert_contains_all(output: &str, expected: &[&str]) {
    for name in expected {
        assert!(output.contains(name));
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
    seed_alpha(&env);
    let output = env.run(&["save", "--label", "alpha"]);
    assert!(output.contains("Saved profile"));
    assert!(output.contains("alpha@example.com"));
    let profile_path = env.profiles_dir().join(format!("{ALPHA_ID}.json"));
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
    seed_alpha(&env);
    let err = env.run_expect_error(&["save", "--label", "   "]);
    assert!(err.contains("label cannot be empty"));
}

#[test]
fn ui_save_trims_label() {
    let env = TestEnv::new();
    seed_alpha(&env);
    env.run(&["save", "--label", "  work  "]);
    let index_path = env.profiles_dir().join("profiles.json");
    let index = fs::read_to_string(index_path).expect("read profiles.json");
    let json: serde_json::Value = serde_json::from_str(&index).expect("parse profiles.json");
    let label = json
        .get("profiles")
        .and_then(|profiles| profiles.get(ALPHA_ID))
        .and_then(|entry| entry.get("label"))
        .and_then(|value| value.as_str());
    assert_eq!(label, Some("work"));
}

#[test]
fn ui_save_skips_rename_without_usage_signal() {
    let env = TestEnv::new();
    seed_alpha(&env);
    fs::create_dir_all(env.profiles_dir()).expect("create profiles dir");
    let profile_one = env.profiles_dir().join(format!("{ALPHA_ID}-old.json"));
    fs::copy(env.codex_dir().join("auth.json"), &profile_one).expect("seed profile one");
    seed_alpha_with_token(&env, "token-alpha-rotated");
    let profile_two = env.profiles_dir().join(format!("{ALPHA_ID}-alt.json"));
    fs::copy(env.codex_dir().join("auth.json"), &profile_two).expect("seed profile two");
    env.write_profiles_index(&[], &[], None);
    seed_alpha_with_token(&env, "token-alpha-new");
    env.run(&["save"]);
    assert!(profile_one.is_file());
    assert!(profile_two.is_file());
}

#[test]
fn ui_save_duplicate_label() {
    let env = TestEnv::new();
    seed_alpha(&env);
    env.run(&["save", "--label", "alpha"]);
    seed_beta(&env);
    let err = env.run_expect_error(&["save", "--label", "alpha"]);
    assert!(err.contains("label 'alpha' already exists"));
}

#[test]
fn ui_load_command() {
    let env = TestEnv::new();
    seed_profiles(&env);
    seed_alpha(&env);
    let output = env.run(&["load", "--label", "beta"]);
    assert!(output.contains("Loaded profile"));
    assert!(output.contains("beta@example.com"));
    assert!(env.read_auth().contains(BETA_ACCOUNT));
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
    env.write_profiles_index(&[("broken", 123)], &[("broken", "broken")], None);
    let err = env.run_expect_error(&["load", "--label", "broken"]);
    assert!(err.contains("No saved profiles.") || err.contains("label 'broken' was not found"));
    assert!(!profile_path.is_file());
}

#[test]
fn ui_load_requires_tty() {
    let env = TestEnv::new();
    seed_profiles(&env);
    seed_alpha(&env);
    let err = env.run_expect_error(&["load"]);
    assert!(err.contains("load selection requires a TTY"));
}

#[test]
fn ui_relay_login_success() {
    let env = TestEnv::new();
    let mut listener = RelayCallbackListener::start(302, "ok").expect("start relay listener");
    let addr = listener.addr();
    let url = format!("http://{addr}/auth/callback?code=test-code&state=test-state",);

    let output = env.run(&["relay-login", "--url", &url]);
    assert!(output.contains("Relayed callback to local login listener."));
    let target = listener
        .recv_request_target(Duration::from_secs(1))
        .expect("receive callback target");
    assert_eq!(target, "/auth/callback?code=test-code&state=test-state");
    listener.shutdown_and_join();
}

#[test]
fn ui_relay_login_requires_url_non_interactive() {
    let env = TestEnv::new();
    let err = env.run_expect_error(&["relay-login"]);
    assert!(err.contains("--url is required in non-interactive mode"));
}

#[test]
fn ui_relay_login_rejects_invalid_url() {
    let env = TestEnv::new();
    let err = env.run_expect_error(&[
        "relay-login",
        "--url",
        "https://localhost:1455/auth/callback?code=test&state=test",
    ]);
    assert!(err.contains("callback URL must use http"));
}

#[test]
fn ui_relay_login_reports_listener_status() {
    let env = TestEnv::new();
    let mut listener =
        RelayCallbackListener::start(401, "unauthorized").expect("start relay listener");
    let addr = listener.addr();
    let url = format!("http://{addr}/auth/callback?code=test-code&state=test-state");
    let err = env.run_expect_error(&["relay-login", "--url", &url]);
    assert!(err.contains("returned http status: 401"));
    listener.shutdown_and_join();
}

#[test]
fn ui_relay_login_reports_unreachable_listener() {
    let env = TestEnv::new();
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
    let addr = listener.local_addr().expect("local addr");
    drop(listener);
    let url = format!("http://{addr}/auth/callback?code=test-code&state=test-state");
    let err = env.run_expect_error(&["relay-login", "--url", &url]);
    assert!(err.contains("failed to reach local login listener"));
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
    let profile_path = env.profiles_dir().join(format!("{BETA_ID}.json"));
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
    seed_alpha(&env);
    let output = env.run(&["delete", "--yes"]);
    assert!(output.contains("No saved profiles."));
}

#[test]
fn ui_list_command() {
    let env = TestEnv::new();
    seed_profiles(&env);
    env.write_profiles_index(
        &[(ALPHA_ID, 200), (BETA_ID, 100)],
        &[(ALPHA_ID, "alpha"), (BETA_ID, "beta")],
        None,
    );
    seed_current(&env);
    let output = env.run(&["list"]);
    assert!(output.contains("current@example.com"));
    assert!(output.contains("WARNING: This profile is not saved yet."));
    assert!(output.contains("Run `codex-switcher save` to save this profile."));
    assert!(output.contains("alpha@example.com"));
    assert!(output.contains("beta@example.com"));
    assert_order(&output, "current@example.com", "alpha@example.com");
    assert_order(&output, "alpha@example.com", "beta@example.com");
}

#[test]
fn ui_list_free_plan() {
    let env = TestEnv::new();
    seed_free(&env);
    env.run(&["save", "--label", "free"]);
    let output = env.run(&["list"]);
    assert!(output.contains("You need a ChatGPT subscription to use Codex CLI"));
}

#[test]
fn ui_sync_current_updates_profile() {
    let env = TestEnv::new();
    seed_alpha(&env);
    env.run(&["save", "--label", "alpha"]);
    seed_alpha_with_token(&env, "token-alpha-rotated");
    env.run(&["list"]);
    let profile_path = env.profiles_dir().join(format!("{ALPHA_ID}.json"));
    let contents = fs::read_to_string(profile_path).expect("read profile");
    assert!(contents.contains("token-alpha-rotated"));
}

#[test]
fn ui_profiles_index_tracks_last_used() {
    let env = TestEnv::new();
    seed_profiles(&env);
    let index_path = env.profiles_dir().join("profiles.json");
    let index = fs::read_to_string(index_path).expect("read profiles.json");
    let json: serde_json::Value = serde_json::from_str(&index).expect("parse profiles.json");
    let profiles = json.get("profiles").expect("profiles map");
    let alpha_last_used = profiles
        .get(ALPHA_ID)
        .and_then(|entry| entry.get("last_used"))
        .and_then(|value| value.as_u64());
    let beta_last_used = profiles
        .get(BETA_ID)
        .and_then(|entry| entry.get("last_used"))
        .and_then(|value| value.as_u64());
    assert!(alpha_last_used.unwrap_or_default() > 0);
    assert!(beta_last_used.unwrap_or_default() > 0);
}

#[test]
fn ui_save_adds_missing_profiles_to_index() {
    let env = TestEnv::new();
    seed_alpha(&env);
    let profile_path = env.profiles_dir().join(format!("{ALPHA_ID}.json"));
    fs::create_dir_all(env.profiles_dir()).expect("create profiles dir");
    fs::copy(env.codex_dir().join("auth.json"), &profile_path).expect("seed profile file");
    env.write_profiles_index(&[], &[], None);
    seed_beta(&env);
    let output = env.run(&["save", "--label", "beta"]);
    assert!(output.contains("Saved profile"));
    let contents =
        fs::read_to_string(env.profiles_dir().join("profiles.json")).expect("read profiles.json");
    let json: serde_json::Value = serde_json::from_str(&contents).expect("parse profiles.json");
    let profiles = json.get("profiles").expect("profiles map");
    assert!(profiles.get(ALPHA_ID).is_some());
    assert!(profiles.get(BETA_ID).is_some());
}

#[test]
fn ui_status_command() {
    let env = TestEnv::new();
    seed_profiles(&env);
    seed_alpha(&env);
    assert_status_output(
        &env,
        &["status"],
        &["alpha@example.com", "beta@example.com", "Priority ranking"],
    );
    let output = env.run(&["status"]);
    assert!(output.contains("alpha@example.com"));
    assert!(output.contains("beta@example.com"));
}

#[test]
fn ui_status_current_command() {
    let env = TestEnv::new();
    seed_profiles(&env);
    seed_alpha(&env);
    assert_status_output(&env, &["status", "--current"], &["alpha@example.com"]);
    let output = env.run(&["status", "--current"]);
    assert!(output.contains("alpha@example.com"));
    assert!(!output.contains("beta@example.com"));
}

#[test]
fn ui_status_all_includes_unsaved_current_profile() {
    let env = TestEnv::new();
    seed_profiles(&env);
    seed_current(&env);
    let output = env.run(&["status"]);
    assert!(output.contains("current@example.com"));
}

#[test]
fn ui_status_label_command() {
    let env = TestEnv::new();
    seed_profiles(&env);
    seed_alpha(&env);
    assert_status_output(&env, &["status", "--label", "beta"], &["beta@example.com"]);
    let output = env.run(&["status", "--label", "beta"]);
    assert!(output.contains("beta@example.com"));
    assert!(!output.contains("alpha@example.com"));
}

#[test]
fn ui_status_all_command() {
    let env = TestEnv::new();
    seed_profiles(&env);
    env.write_profiles_index(
        &[(ALPHA_ID, 200), (BETA_ID, 100)],
        &[(ALPHA_ID, "alpha"), (BETA_ID, "beta")],
        None,
    );
    seed_alpha(&env);
    assert_status_output(
        &env,
        &["status", "--all"],
        &["alpha@example.com", "beta@example.com", "Priority ranking"],
    );
    let output = env.run(&["status", "--all"]);
    assert_order(&output, "alpha@example.com", "beta@example.com");
}

#[test]
fn ui_status_all_no_usage() {
    let env = TestEnv::new();
    seed_profiles(&env);
    env.write_profiles_index(
        &[(ALPHA_ID, 200), (BETA_ID, 100)],
        &[(ALPHA_ID, "alpha"), (BETA_ID, "beta")],
        None,
    );
    seed_alpha(&env);
    env.write_config("http://127.0.0.1:1/backend-api");
    let output = env.run(&["status", "--all"]);
    assert!(output.contains("alpha@example.com"));
    assert!(output.contains("beta@example.com"));
    assert!(output.contains("UNAVAILABLE"));
}

#[test]
fn ui_switch_command() {
    let env = TestEnv::new();
    seed_profiles(&env);
    seed_beta(&env);
    let body = r#"{"rate_limit":{"primary_window":{"used_percent":20,"limit_window_seconds":18000,"reset_at":2000000000},"secondary_window":{"used_percent":50,"limit_window_seconds":604800,"reset_at":2000600000}}}"#;
    let (addr, handle) = start_usage_server(body, 8).expect("usage server");
    env.write_config(&format!("http://{addr}/backend-api"));

    let output = env.run(&["switch"]);
    assert!(output.contains("Priority ranking"));
    assert!(output.contains("Loaded profile"));
    assert!(env.read_auth().contains(ALPHA_ACCOUNT));

    let _ = handle.join();
}

#[test]
fn ui_switch_dry_run_does_not_modify_auth() {
    let env = TestEnv::new();
    seed_profiles(&env);
    seed_beta(&env);
    let before = env.read_auth();
    let body = r#"{"rate_limit":{"primary_window":{"used_percent":20,"limit_window_seconds":18000,"reset_at":2000000000},"secondary_window":{"used_percent":50,"limit_window_seconds":604800,"reset_at":2000600000}}}"#;
    let (addr, handle) = start_usage_server(body, 8).expect("usage server");
    env.write_config(&format!("http://{addr}/backend-api"));

    let output = env.run(&["switch", "--dry-run"]);
    assert!(output.contains("Dry run: best profile is"));
    let after = env.read_auth();
    assert_eq!(before, after);

    let _ = handle.join();
}

#[test]
fn ui_switch_fails_when_all_usage_unavailable() {
    let env = TestEnv::new();
    seed_profiles(&env);
    seed_beta(&env);
    env.write_config("http://127.0.0.1:1/backend-api");
    let err = env.run_expect_error(&["switch"]);
    assert!(err.contains("no eligible profile"));
}

#[test]
fn ui_migrate_profiles_keeps_source() {
    let source = TestEnv::new();
    seed_profiles(&source);

    let target = TestEnv::new();
    let from = source.codex_dir().to_string_lossy().into_owned();
    let output = target.run(&["migrate", "--from", &from]);
    assert!(output.contains("Migration complete"));
    assert!(output.contains("Source preserved"));

    let list = target.run(&["list"]);
    assert!(list.contains("alpha@example.com"));
    assert!(list.contains("beta@example.com"));

    let alpha_path = source.profiles_dir().join(format!("{ALPHA_ID}.json"));
    let beta_path = source.profiles_dir().join(format!("{BETA_ID}.json"));
    assert!(alpha_path.is_file());
    assert!(beta_path.is_file());
}

#[test]
fn ui_list_removes_invalid_profiles() {
    let env = TestEnv::new();
    fs::create_dir_all(env.profiles_dir()).expect("create profiles dir");
    let bad_profile = env.profiles_dir().join("bad.json");
    fs::write(&bad_profile, "{").expect("write bad profile");
    env.write_profiles_index(&[("bad", 123)], &[("bad", "bad")], None);

    env.run(&["list"]);

    assert!(!bad_profile.is_file());
    let index =
        fs::read_to_string(env.profiles_dir().join("profiles.json")).expect("read profiles.json");
    let json: serde_json::Value = serde_json::from_str(&index).expect("parse profiles.json");
    let profiles = json.get("profiles").expect("profiles map");
    assert!(profiles.get("bad").is_none());
}

#[test]
fn ui_status_refresh_updates_profile() {
    let env = TestEnv::new();
    let usage_body = r#"{"rate_limit":{"primary_window":{"used_percent":20,"limit_window_seconds":18000,"reset_at":2000000000}}}"#;
    let refresh_id_token = build_id_token(ALPHA_EMAIL, ALPHA_PLAN);
    let refresh_body = format!(
        "{{\"id_token\":\"{refresh_id_token}\",\"access_token\":\"new-access\",\"refresh_token\":\"new-refresh\"}}"
    );
    let (usage_addr, usage_handle) = start_usage_server(usage_body, 4).expect("usage server");
    let (refresh_addr, refresh_handle) =
        start_usage_server(Box::leak(refresh_body.into_boxed_str()), 2).expect("refresh server");

    env.write_config(&format!("http://{usage_addr}/backend-api"));
    env.write_auth_with_refresh(
        ALPHA_ACCOUNT,
        ALPHA_EMAIL,
        ALPHA_PLAN,
        "old-access",
        "refresh-old",
    );
    env.run(&["save", "--label", "alpha"]);

    let refresh_url = format!("http://{refresh_addr}/token");
    let output = env.run_with_env(
        &["status", "--current"],
        &[("CODEX_REFRESH_TOKEN_URL_OVERRIDE", refresh_url.as_str())],
    );

    let auth_contents = env.read_auth();
    assert!(
        auth_contents.contains("new-access"),
        "status output:\n{output}\nauth.json:\n{auth_contents}"
    );
    let profile_path = env.profiles_dir().join(format!("{ALPHA_ID}.json"));
    let profile_contents = fs::read_to_string(profile_path).expect("read profile");
    assert!(
        profile_contents.contains("new-access"),
        "status output:\n{output}\nprofile:\n{profile_contents}"
    );

    let _ = usage_handle.join();
    let _ = refresh_handle.join();
}
