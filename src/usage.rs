use chrono::{DateTime, Local};
use colored::Colorize;
use fslock::LockFile;
use serde::Deserialize;
use std::collections::{BTreeMap, HashSet};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::thread::sleep;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::collect_profile_ids;
use crate::{Paths, command_name};
use crate::{is_plain, style_text, use_color_stdout};

const DEFAULT_BASE_URL: &str = "https://chatgpt.com/backend-api";
const USER_AGENT: &str = "codex-profiles";
const LOCK_TIMEOUT: Duration = Duration::from_secs(10);
const LOCK_RETRY_DELAY: Duration = Duration::from_secs(1);

#[derive(Clone, Default)]
pub(crate) struct UsageLimits {
    pub(crate) five_hour: Option<UsageWindow>,
    pub(crate) weekly: Option<UsageWindow>,
}

#[derive(Clone)]
pub(crate) struct UsageWindow {
    pub(crate) left_percent: f64,
    pub(crate) reset_at: i64,
    pub(crate) reset_at_relative: Option<String>,
}

#[derive(Debug)]
pub enum UsageFetchError {
    Status(u16),
    Transport(String),
    Parse(String),
}

impl UsageFetchError {
    pub fn status_code(&self) -> Option<u16> {
        match self {
            UsageFetchError::Status(code) => Some(*code),
            _ => None,
        }
    }

    pub fn message(&self) -> String {
        match self {
            UsageFetchError::Status(code) => {
                format!("Error: failed to fetch usage: http status: {code}")
            }
            UsageFetchError::Transport(err) => {
                format!("Error: failed to fetch usage: {err}")
            }
            UsageFetchError::Parse(err) => format!("Error: failed to parse usage: {err}"),
        }
    }
}

impl std::fmt::Display for UsageFetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message())
    }
}

#[derive(Deserialize)]
struct UsagePayload {
    #[serde(default)]
    rate_limit: Option<RateLimitDetails>,
}

#[derive(Clone, Deserialize)]
struct RateLimitDetails {
    #[serde(default)]
    primary_window: Option<RateLimitWindowSnapshot>,
    #[serde(default)]
    secondary_window: Option<RateLimitWindowSnapshot>,
}

#[derive(Clone, Deserialize)]
struct RateLimitWindowSnapshot {
    used_percent: f64,
    limit_window_seconds: i64,
    reset_at: i64,
}

pub fn read_base_url(paths: &Paths) -> String {
    let config_path = paths.codex.join("config.toml");
    if let Ok(contents) = fs::read_to_string(config_path) {
        for line in contents.lines() {
            if let Some(value) = parse_config_value(line, "chatgpt_base_url") {
                return normalize_base_url(&value);
            }
        }
    }
    DEFAULT_BASE_URL.to_string()
}

#[doc(hidden)]
pub fn parse_config_value(line: &str, key: &str) -> Option<String> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }
    let (config_key, raw_value) = line.split_once('=')?;
    if config_key.trim() != key {
        return None;
    }
    let value = strip_inline_comment(raw_value).trim();
    if value.is_empty() {
        return None;
    }
    let value = value.trim_matches('"').trim_matches('\'').trim();
    if value.is_empty() {
        return None;
    }
    Some(value.to_string())
}

fn strip_inline_comment(value: &str) -> &str {
    let mut in_single = false;
    let mut in_double = false;
    let mut escape = false;
    for (idx, ch) in value.char_indices() {
        match ch {
            '"' if !in_single && !escape => in_double = !in_double,
            '\'' if !in_double => in_single = !in_single,
            '#' if !in_single && !in_double => return value[..idx].trim_end(),
            _ => {}
        }
        escape = in_double && ch == '\\' && !escape;
        if ch != '\\' {
            escape = false;
        }
    }
    value.trim_end()
}

fn normalize_base_url(value: &str) -> String {
    let mut base = value.trim_end_matches('/').to_string();
    if (base.starts_with("https://chatgpt.com") || base.starts_with("https://chat.openai.com"))
        && !base.contains("/backend-api")
    {
        base = format!("{base}/backend-api");
    }
    base
}

fn usage_endpoint(base_url: &str) -> String {
    if base_url.contains("/backend-api") {
        format!("{base_url}/wham/usage")
    } else {
        format!("{base_url}/api/codex/usage")
    }
}

fn fetch_usage_payload(
    base_url: &str,
    access_token: &str,
    account_id: &str,
) -> Result<UsagePayload, UsageFetchError> {
    let endpoint = usage_endpoint(base_url);
    let config = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(5)))
        .build();
    let agent: ureq::Agent = config.into();
    let response = match agent
        .get(&endpoint)
        .header("Authorization", &format!("Bearer {access_token}"))
        .header("ChatGPT-Account-Id", account_id)
        .header("User-Agent", USER_AGENT)
        .call()
    {
        Ok(response) => response,
        Err(ureq::Error::StatusCode(code)) => return Err(UsageFetchError::Status(code)),
        Err(err) => return Err(UsageFetchError::Transport(err.to_string())),
    };
    response
        .into_body()
        .read_json::<UsagePayload>()
        .map_err(|err| UsageFetchError::Parse(err.to_string()))
}

pub fn fetch_usage_details(
    base_url: &str,
    access_token: &str,
    account_id: &str,
    unavailable_text: &str,
    now: DateTime<Local>,
    show_spinner: bool,
) -> Result<Vec<String>, UsageFetchError> {
    let spinner = show_spinner.then(|| start_spinner("Fetching profile..."));
    let payload = fetch_usage_payload(base_url, access_token, account_id);
    if let Some(spinner) = spinner {
        stop_spinner(spinner);
    }
    let payload = payload?;
    let limits = build_usage_limits(&payload, now);
    Ok(format_usage(
        format_limit(limits.five_hour.as_ref(), now, unavailable_text),
        format_limit(limits.weekly.as_ref(), now, unavailable_text),
        unavailable_text,
    ))
}

fn build_usage_limits(payload: &UsagePayload, now: DateTime<Local>) -> UsageLimits {
    let mut limits = UsageLimits::default();
    let Some(rate_limit) = payload.rate_limit.as_ref() else {
        return limits;
    };
    let mut windows: Vec<(i64, UsageWindow)> = [
        rate_limit.primary_window.as_ref(),
        rate_limit.secondary_window.as_ref(),
    ]
    .into_iter()
    .flatten()
    .map(|window| {
        (
            window.limit_window_seconds,
            usage_window_output(window, now),
        )
    })
    .collect();
    if windows.is_empty() {
        return limits;
    }
    windows.sort_by_key(|(secs, _)| *secs);
    if let Some((_, first)) = windows.first() {
        limits.five_hour = Some(first.clone());
    }
    if let Some((_, second)) = windows.get(1) {
        limits.weekly = Some(second.clone());
    }
    limits
}

fn usage_window_output(window: &RateLimitWindowSnapshot, now: DateTime<Local>) -> UsageWindow {
    let left_percent = (100.0 - window.used_percent).clamp(0.0, 100.0);
    let reset_at = window.reset_at;
    let reset_at_relative = format_reset_relative(reset_at, now);
    UsageWindow {
        left_percent,
        reset_at,
        reset_at_relative,
    }
}

fn start_spinner(message: &str) -> spinner::SpinnerHandle {
    spinner::SpinnerBuilder::new(message.to_string())
        .spinner(vec![".  ", ".. ", "...", " ..", "  .", "   "])
        .step(Duration::from_millis(80))
        .start()
}

fn stop_spinner(spinner: spinner::SpinnerHandle) {
    spinner.close();
    eprint!("\r\x1b[2K");
    let _ = std::io::stderr().flush();
}

pub(crate) struct UsageLine {
    pub(crate) bar: String,
    pub(crate) percent: String,
    pub(crate) reset: String,
    pub(crate) left_percent: Option<i64>,
}

impl UsageLine {
    fn unavailable(text: &str) -> Self {
        UsageLine {
            bar: text.to_string(),
            percent: String::new(),
            reset: String::new(),
            left_percent: None,
        }
    }
}

pub(crate) fn format_limit(
    window: Option<&UsageWindow>,
    now: DateTime<Local>,
    unavailable_text: &str,
) -> UsageLine {
    let Some(window) = window else {
        return UsageLine::unavailable(unavailable_text);
    };
    let left_percent = window.left_percent;
    let left_percent_rounded = left_percent.round() as i64;
    let bar = render_bar(left_percent);
    let bar = style_usage_bar(&bar, left_percent);
    let percent = format!("{left_percent_rounded}%");
    let reset = window.reset_at_relative.clone().unwrap_or_else(|| {
        let local = local_from_timestamp(window.reset_at).unwrap_or(now);
        local.format("%H:%M on %d %b").to_string()
    });
    UsageLine {
        bar,
        percent,
        reset,
        left_percent: Some(left_percent_rounded),
    }
}

pub fn usage_unavailable(plan_is_free: bool) -> &'static str {
    if plan_is_free {
        "You need a ChatGPT subscription to use Codex CLI"
    } else {
        "Data not available"
    }
}

pub fn format_usage_unavailable(text: &str, use_color: bool) -> String {
    if is_plain() {
        format!("INFO: {text}")
    } else if use_color {
        text.red().bold().to_string()
    } else {
        text.to_string()
    }
}

pub(crate) fn format_usage(
    five: UsageLine,
    weekly: UsageLine,
    unavailable_text: &str,
) -> Vec<String> {
    let use_color = use_color_stdout();
    let available: Vec<UsageLine> = [five, weekly]
        .into_iter()
        .filter(|line| line.left_percent.is_some())
        .collect();
    if available.is_empty() {
        return vec![format_usage_unavailable(unavailable_text, use_color)];
    }
    let has_zero = available.iter().any(|line| line.left_percent == Some(0));
    let multiple = available.len() > 1;
    available
        .into_iter()
        .map(|line| {
            let dim = use_color && multiple && has_zero && line.left_percent != Some(0);
            format_usage_line(&line, dim, use_color)
        })
        .collect()
}

pub fn format_last_used(ts: u64) -> String {
    if ts == 0 {
        return "unknown".to_string();
    }
    let timestamp = UNIX_EPOCH + Duration::from_secs(ts);
    match SystemTime::now().duration_since(timestamp) {
        Ok(duration) => format_relative_duration(duration, true),
        Err(err) => format_relative_duration(err.duration(), false),
    }
}

pub(crate) fn format_reset_relative(reset_at: i64, now: DateTime<Local>) -> Option<String> {
    let reset_at = local_from_timestamp(reset_at)?;
    let duration = reset_at.signed_duration_since(now);
    if duration.num_seconds() <= 0 {
        return Some("now".to_string());
    }
    let duration = duration.to_std().ok()?;
    Some(format_duration(duration, DurationStyle::ResetTimer))
}

fn format_usage_line(line: &UsageLine, dim: bool, use_color: bool) -> String {
    let reset = reset_label(&line.reset);
    let reset = reset.to_string();
    let percent = if line.percent.is_empty() {
        String::new()
    } else {
        format!("{} left", line.percent)
    };
    let resets = format_resets_suffix(&reset, use_color);
    if is_plain() {
        let mut out = String::new();
        if !percent.is_empty() {
            out.push_str(&percent);
        }
        if !resets.is_empty() {
            if !out.is_empty() {
                out.push(' ');
            }
            out.push_str(&resets);
        }
        return out;
    }
    let resets = if resets.is_empty() {
        resets
    } else {
        format!(" {resets}")
    };
    let bar = if dim {
        strip_ansi(&line.bar)
    } else {
        line.bar.clone()
    };
    let formatted = if percent.is_empty() {
        format!("{bar}{resets}")
    } else {
        format!("{bar} {percent}{resets}")
    };
    if dim && use_color {
        formatted.dimmed().to_string()
    } else {
        formatted
    }
}

fn reset_label(reset: &str) -> &str {
    if reset.is_empty() { "unknown" } else { reset }
}

fn format_resets_suffix(reset: &str, use_color: bool) -> String {
    let text = format!("(resets {reset})");
    style_text(&text, use_color, |text| text.dimmed().italic())
}

fn render_bar(left_percent: f64) -> String {
    let total = 20;
    let filled = ((left_percent / 100.0) * total as f64).round() as usize;
    let filled = filled.min(total);
    let empty = total.saturating_sub(filled);
    format!(
        "{}{}",
        "▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮"
            .chars()
            .take(filled)
            .collect::<String>(),
        "▯▯▯▯▯▯▯▯▯▯▯▯▯▯▯▯▯▯▯▯"
            .chars()
            .take(empty)
            .collect::<String>()
    )
}

fn style_usage_bar(bar: &str, left_percent: f64) -> String {
    if !use_color_stdout() {
        return bar.to_string();
    }
    if left_percent >= 66.0 {
        bar.green().to_string()
    } else if left_percent >= 33.0 {
        bar.yellow().to_string()
    } else {
        bar.red().to_string()
    }
}

fn strip_ansi(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    loop {
        let Some(ch) = chars.next() else {
            break;
        };
        if ch == '\x1b' && consume_ansi_escape(&mut chars) {
            continue;
        }
        out.push(ch);
    }
    out
}

fn consume_ansi_escape<I>(chars: &mut std::iter::Peekable<I>) -> bool
where
    I: Iterator<Item = char>,
{
    if chars.peek() != Some(&'[') {
        return false;
    }
    chars.next();
    for c in chars.by_ref() {
        if c == 'm' {
            break;
        }
    }
    true
}

fn format_relative_duration(duration: Duration, past: bool) -> String {
    let text = format_duration(duration, DurationStyle::LastUsed);
    if past {
        format!("{text} ago")
    } else {
        format!("in {text}")
    }
}

enum DurationStyle {
    LastUsed,
    ResetTimer,
}

fn format_duration(duration: Duration, style: DurationStyle) -> String {
    let secs = duration.as_secs();
    let (value, unit) = if secs < 60 {
        (secs, "s")
    } else if secs < 60 * 60 {
        (secs / 60, "m")
    } else if secs < 60 * 60 * 24 {
        (secs / (60 * 60), "h")
    } else {
        (secs / (60 * 60 * 24), "d")
    };
    match style {
        DurationStyle::LastUsed => format!("{value}{unit}"),
        DurationStyle::ResetTimer => format!("in {value}{unit}"),
    }
}

fn local_from_timestamp(ts: i64) -> Option<DateTime<Local>> {
    let dt = chrono::DateTime::<chrono::Utc>::from_timestamp(ts, 0)?;
    Some(dt.with_timezone(&Local))
}

pub struct UsageLock {
    pub file: File,
    _lock: LockFile,
}

pub fn lock_usage(paths: &Paths) -> Result<UsageLock, String> {
    let start = Instant::now();
    let mut lock = LockFile::open(&paths.usage_lock)
        .map_err(|err| format!("Error: failed to open usage lock: {err}"))?;
    loop {
        match lock.try_lock() {
            Ok(true) => break,
            Ok(false) => {
                if start.elapsed() > LOCK_TIMEOUT {
                    return Err(format!(
                        "Error: could not acquire usage lock. Ensure no other {} is running and retry.",
                        command_name()
                    ));
                }
                sleep(LOCK_RETRY_DELAY);
            }
            Err(err) => {
                return Err(format!("Error: failed to lock usage file: {err}"));
            }
        }
    }
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&paths.usage)
        .map_err(|err| usage_io_error("open", err))?;
    Ok(UsageLock { file, _lock: lock })
}

pub fn read_usage(file: &mut File) -> Result<Vec<(String, u64)>, String> {
    let contents = read_usage_contents(file)?;
    Ok(parse_usage_entries(&contents))
}

fn read_usage_contents(file: &mut File) -> Result<String, String> {
    file.seek(SeekFrom::Start(0))
        .map_err(|err| usage_io_error("read", err))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|err| usage_io_error("read", err))?;
    Ok(contents)
}

fn parse_usage_entries(contents: &str) -> Vec<(String, u64)> {
    contents
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }
            let (id, ts) = line.split_once('\t')?;
            let ts = ts.parse::<u64>().ok()?;
            if id.is_empty() {
                None
            } else {
                Some((id.to_string(), ts))
            }
        })
        .collect()
}

pub fn write_usage(file: &mut File, entries: &BTreeMap<String, u64>) -> Result<(), String> {
    let out = format_usage_contents(entries);
    file.set_len(0)
        .map_err(|err| usage_io_error("write", err))?;
    file.seek(SeekFrom::Start(0))
        .map_err(|err| usage_io_error("write", err))?;
    file.write_all(out.as_bytes())
        .map_err(|err| usage_io_error("write", err))?;
    file.flush().map_err(|err| usage_io_error("write", err))?;
    Ok(())
}

pub fn normalize_usage(entries: &[(String, u64)], ids: &HashSet<String>) -> BTreeMap<String, u64> {
    let mut map = BTreeMap::new();
    for id in ids {
        map.insert(id.clone(), 0);
    }
    for (id, ts) in entries {
        if !ids.contains(id) {
            continue;
        }
        let entry = map.entry(id.clone()).or_insert(0);
        if *ts > *entry {
            *entry = *ts;
        }
    }
    map
}

pub fn ensure_usage(
    file: &mut File,
    profiles_dir: &std::path::Path,
) -> Result<BTreeMap<String, u64>, String> {
    let contents = read_usage_contents(file)?;
    let entries = parse_usage_entries(&contents);
    let ids = collect_profile_ids(profiles_dir)?;

    let map = normalize_usage(&entries, &ids);

    let expected = format_usage_contents(&map);
    if contents != expected {
        write_usage(file, &map)?;
    }
    Ok(map)
}

pub fn ordered_profiles(map: &BTreeMap<String, u64>) -> Vec<(String, u64)> {
    let mut ordered = map
        .iter()
        .map(|(id, ts)| (id.clone(), *ts))
        .collect::<Vec<_>>();
    ordered.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    ordered
}

pub fn now_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn format_usage_contents(entries: &BTreeMap<String, u64>) -> String {
    let mut out = String::new();
    for (id, ts) in entries {
        out.push_str(id);
        out.push('\t');
        out.push_str(&ts.to_string());
        out.push('\n');
    }
    out
}

fn usage_io_error(action: &str, err: impl std::fmt::Display) -> String {
    format!("Error: failed to {action} usage file: {err}")
}
