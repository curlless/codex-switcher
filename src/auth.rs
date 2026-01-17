use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde::{Deserialize, Serialize};
use serde_with::{NoneAsEmptyString, serde_as};
use std::path::Path;
use std::time::Duration;

use crate::write_atomic;

const API_KEY_PREFIX: &str = "api-key-";
const API_KEY_LABEL: &str = "Key";
const API_KEY_SEPARATOR: &str = "~";
const API_KEY_PREFIX_LEN: usize = 12;
const API_KEY_SUFFIX_LEN: usize = 16;
const REFRESH_TOKEN_URL: &str = "https://auth.openai.com/oauth/token";
const REFRESH_TOKEN_URL_OVERRIDE_ENV_VAR: &str = "CODEX_REFRESH_TOKEN_URL_OVERRIDE";
const CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";

#[derive(Deserialize)]
pub struct AuthFile {
    #[serde(rename = "OPENAI_API_KEY")]
    pub openai_api_key: Option<String>,
    pub tokens: Option<Tokens>,
    #[serde(default)]
    pub last_refresh: Option<String>,
}

#[serde_as]
#[derive(Clone, Deserialize)]
pub struct Tokens {
    #[serde(default)]
    #[serde_as(as = "NoneAsEmptyString")]
    pub account_id: Option<String>,
    #[serde(default)]
    #[serde_as(as = "NoneAsEmptyString")]
    pub id_token: Option<String>,
    #[serde(default)]
    #[serde_as(as = "NoneAsEmptyString")]
    pub access_token: Option<String>,
    #[serde(default)]
    #[serde_as(as = "NoneAsEmptyString")]
    pub refresh_token: Option<String>,
}

#[serde_as]
#[derive(Deserialize)]
struct IdTokenClaims {
    #[serde(default)]
    #[serde_as(as = "NoneAsEmptyString")]
    email: Option<String>,
    #[serde(rename = "https://api.openai.com/auth")]
    auth: Option<AuthClaims>,
}

#[serde_as]
#[derive(Deserialize)]
struct AuthClaims {
    #[serde(default)]
    #[serde_as(as = "NoneAsEmptyString")]
    chatgpt_plan_type: Option<String>,
}

pub fn read_tokens(path: &Path) -> Result<Tokens, String> {
    let auth = read_auth_file(path)?;
    if let Some(tokens) = auth.tokens {
        return Ok(tokens);
    }
    if let Some(api_key) = auth.openai_api_key.as_deref() {
        return Ok(tokens_from_api_key(api_key));
    }
    Err(format!(
        "Error: missing tokens in {}. Run `codex login` to authenticate.",
        path.display()
    ))
}

pub fn read_auth_file(path: &Path) -> Result<AuthFile, String> {
    let data = std::fs::read_to_string(path).map_err(|err| {
        if err.kind() == std::io::ErrorKind::NotFound {
            "Error: Codex auth file not found. Run `codex login` first.".to_string()
        } else {
            format!("Error: failed to read {}: {err}", path.display())
        }
    })?;
    let auth: AuthFile = serde_json::from_str(&data).map_err(|err| {
        format!(
            "Error: invalid JSON in {}: {err}. Run `codex login` to regenerate it.",
            path.display()
        )
    })?;
    Ok(auth)
}

pub fn read_tokens_opt(path: &Path) -> Option<Tokens> {
    if !path.is_file() {
        return None;
    }
    read_tokens(path).ok()
}

pub fn tokens_from_api_key(api_key: &str) -> Tokens {
    Tokens {
        account_id: Some(api_key_profile_id(api_key)),
        id_token: None,
        access_token: None,
        refresh_token: None,
    }
}

pub fn has_auth(path: &Path) -> bool {
    read_tokens_opt(path).is_some_and(|tokens| is_profile_ready(&tokens))
}

pub fn is_profile_ready(tokens: &Tokens) -> bool {
    if is_api_key_profile(tokens) {
        return true;
    }
    if token_account_id(tokens).is_none() {
        return false;
    }
    if !tokens
        .access_token
        .as_deref()
        .map(|value| !value.is_empty())
        .unwrap_or(false)
    {
        return false;
    }
    let (email, plan) = extract_email_and_plan(tokens);
    email.is_some() && plan.is_some()
}

pub fn extract_email_and_plan(tokens: &Tokens) -> (Option<String>, Option<String>) {
    if is_api_key_profile(tokens) {
        let display = api_key_display_label(tokens).unwrap_or_else(|| API_KEY_LABEL.to_string());
        return (Some(display), Some(API_KEY_LABEL.to_string()));
    }
    let claims = tokens.id_token.as_deref().and_then(decode_id_token_claims);
    let email = claims.as_ref().and_then(|c| c.email.clone());
    let plan = claims
        .and_then(|c| c.auth)
        .and_then(|auth| auth.chatgpt_plan_type)
        .map(|plan| format_plan(&plan));
    (email, plan)
}

pub fn require_identity(tokens: &Tokens) -> Result<(String, String, String), String> {
    let Some(account_id) = token_account_id(tokens) else {
        return Err(
            "Error: auth.json is missing tokens.account_id. Run `codex login` to reauthenticate."
                .to_string(),
        );
    };
    let (email, plan) = extract_email_and_plan(tokens);
    let email = email.ok_or_else(|| {
        "Error: auth.json is missing id_token email. Run `codex login` to reauthenticate."
            .to_string()
    })?;
    let plan = plan.ok_or_else(|| {
        "Error: auth.json is missing id_token plan. Run `codex login` to reauthenticate."
            .to_string()
    })?;
    Ok((account_id.to_string(), email, plan))
}

pub fn profile_error(
    tokens: &Tokens,
    email: Option<&str>,
    plan: Option<&str>,
) -> Option<&'static str> {
    if is_api_key_profile(tokens) {
        return None;
    }
    if email.is_none() || plan.is_none() {
        return Some("profile missing id_token email/plan");
    }
    if token_account_id(tokens).is_none() {
        return Some("profile missing tokens.account_id");
    }
    if tokens.access_token.is_none() {
        return Some("profile missing tokens.access_token");
    }
    None
}

pub fn token_account_id(tokens: &Tokens) -> Option<&str> {
    tokens
        .account_id
        .as_deref()
        .filter(|value| !value.is_empty())
}

pub fn is_api_key_profile(tokens: &Tokens) -> bool {
    tokens
        .account_id
        .as_deref()
        .map(|value| value.starts_with(API_KEY_PREFIX))
        .unwrap_or(false)
        && tokens.id_token.is_none()
        && tokens.access_token.is_none()
        && tokens.refresh_token.is_none()
}

pub fn format_plan(plan: &str) -> String {
    let mut out = String::new();
    for word in plan.split(['_', '-']) {
        if word.is_empty() {
            continue;
        }
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(&title_case(word));
    }
    if out.is_empty() {
        "Unknown".to_string()
    } else {
        out
    }
}

pub fn is_free_plan(plan: Option<&str>) -> bool {
    plan.map(|value| value.eq_ignore_ascii_case("free"))
        .unwrap_or(false)
}

fn title_case(word: &str) -> String {
    let mut chars = word.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    let mut out = String::new();
    out.push(first.to_ascii_uppercase());
    out.extend(chars.flat_map(|ch| ch.to_lowercase()));
    out
}

fn decode_id_token_claims(token: &str) -> Option<IdTokenClaims> {
    let mut parts = token.split('.');
    let _header = parts.next()?;
    let payload = parts.next()?;
    let _sig = parts.next()?;
    let decoded = URL_SAFE_NO_PAD.decode(payload).ok()?;
    serde_json::from_slice(&decoded).ok()
}

fn api_key_profile_id(api_key: &str) -> String {
    let prefix = api_key_prefix(api_key);
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in api_key.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{API_KEY_PREFIX}{prefix}{API_KEY_SEPARATOR}{hash:016x}")
}

fn api_key_display_label(tokens: &Tokens) -> Option<String> {
    let account_id = tokens.account_id.as_deref()?;
    let rest = account_id.strip_prefix(API_KEY_PREFIX)?;
    let (prefix, hash) = rest.split_once(API_KEY_SEPARATOR)?;
    if prefix.is_empty() {
        return None;
    }
    let suffix: String = hash.chars().rev().take(API_KEY_SUFFIX_LEN).collect();
    let suffix: String = suffix.chars().rev().collect();
    if suffix.is_empty() {
        return None;
    }
    Some(format!("{API_KEY_SEPARATOR}{suffix}"))
}

fn api_key_prefix(api_key: &str) -> String {
    let mut out = String::new();
    for ch in api_key.chars().take(API_KEY_PREFIX_LEN) {
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
            out.push(ch);
        } else {
            out.push('-');
        }
    }
    out
}

#[derive(Serialize)]
struct RefreshRequest {
    client_id: &'static str,
    grant_type: &'static str,
    refresh_token: String,
    scope: &'static str,
}

#[derive(Clone, Deserialize)]
struct RefreshResponse {
    id_token: Option<String>,
    access_token: Option<String>,
    refresh_token: Option<String>,
}

pub fn refresh_profile_tokens(path: &Path, tokens: &mut Tokens) -> Result<(), String> {
    let refresh_token = tokens
        .refresh_token
        .as_deref()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            "Error: profile is missing refresh_token; run `codex login` and save it again."
                .to_string()
        })?;
    let refreshed = refresh_access_token(refresh_token)?;
    apply_refresh(tokens, &refreshed)?;
    update_auth_tokens(path, &refreshed)?;
    Ok(())
}

fn refresh_access_token(refresh_token: &str) -> Result<RefreshResponse, String> {
    let request = RefreshRequest {
        client_id: CLIENT_ID,
        grant_type: "refresh_token",
        refresh_token: refresh_token.to_string(),
        scope: "openid profile email",
    };
    let endpoint = std::env::var(REFRESH_TOKEN_URL_OVERRIDE_ENV_VAR)
        .unwrap_or_else(|_| REFRESH_TOKEN_URL.to_string());
    let config = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(5)))
        .build();
    let agent: ureq::Agent = config.into();
    let response = agent
        .post(&endpoint)
        .header("Content-Type", "application/json")
        .send_json(&request)
        .map_err(|err| match err {
            ureq::Error::StatusCode(code) => {
                format!("Error: failed to refresh access token: http status: {code}")
            }
            other => format!("Error: failed to refresh access token: {other}"),
        })?;
    response
        .into_body()
        .read_json::<RefreshResponse>()
        .map_err(|err| format!("Error: failed to parse refresh response: {err}"))
}

fn apply_refresh(tokens: &mut Tokens, refreshed: &RefreshResponse) -> Result<(), String> {
    let Some(access_token) = refreshed.access_token.as_ref() else {
        return Err("Error: refresh response missing access_token.".to_string());
    };
    tokens.access_token = Some(access_token.clone());
    if let Some(id_token) = refreshed.id_token.as_ref() {
        tokens.id_token = Some(id_token.clone());
    }
    if let Some(refresh_token) = refreshed.refresh_token.as_ref() {
        tokens.refresh_token = Some(refresh_token.clone());
    }
    Ok(())
}

fn update_auth_tokens(path: &Path, refreshed: &RefreshResponse) -> Result<(), String> {
    let contents = std::fs::read_to_string(path)
        .map_err(|err| format!("Error: failed to read {}: {err}", path.display()))?;
    let mut value: serde_json::Value = serde_json::from_str(&contents)
        .map_err(|err| format!("Error: invalid JSON in {}: {err}", path.display()))?;
    let Some(root) = value.as_object_mut() else {
        return Err(format!(
            "Error: invalid JSON in {} (expected object)",
            path.display()
        ));
    };
    let tokens = root
        .entry("tokens")
        .or_insert_with(|| serde_json::json!({}));
    let Some(tokens_map) = tokens.as_object_mut() else {
        return Err(format!(
            "Error: invalid tokens in {} (expected object)",
            path.display()
        ));
    };
    if let Some(id_token) = refreshed.id_token.as_ref() {
        tokens_map.insert(
            "id_token".to_string(),
            serde_json::Value::String(id_token.clone()),
        );
    }
    if let Some(access_token) = refreshed.access_token.as_ref() {
        tokens_map.insert(
            "access_token".to_string(),
            serde_json::Value::String(access_token.clone()),
        );
    }
    if let Some(refresh_token) = refreshed.refresh_token.as_ref() {
        tokens_map.insert(
            "refresh_token".to_string(),
            serde_json::Value::String(refresh_token.clone()),
        );
    }
    let json = serde_json::to_string_pretty(&value)
        .map_err(|err| format!("Error: failed to serialize auth file: {err}"))?;
    write_atomic(path, format!("{json}\n").as_bytes())
        .map_err(|err| format!("Error: failed to write {}: {err}", path.display()))
}
