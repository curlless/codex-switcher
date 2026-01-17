use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::{
    AuthFile, Labels, Paths, ProfileStore, Tokens, assign_label, extract_email_and_plan,
    format_warning, is_api_key_profile, label_for_id, normalize_error, now_seconds, profile_files,
    profile_id_from_path, profile_path_for_id, read_auth_file, read_labels, resolve_save_id,
    tokens_from_api_key, use_color_stderr, write_atomic, write_atomic_with_mode,
};

const EVERY_CODE_ACCOUNTS_FILE: &str = "auth_accounts.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum AuthMode {
    ApiKey,
    Chatgpt,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct AccountsFile {
    #[serde(default = "default_version")]
    version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    active_account_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    accounts: Vec<StoredAccount>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
struct AccountsFileRaw {
    #[serde(default)]
    accounts: Vec<StoredAccountRaw>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
struct StoredAccountRaw {
    #[serde(default)]
    mode: Option<String>,
    #[serde(default)]
    label: Option<String>,
    #[serde(default, rename = "openai_api_key")]
    openai_api_key: Option<String>,
    #[serde(default)]
    tokens: Option<EveryCodeTokens>,
    #[serde(default)]
    last_refresh: Option<String>,
    #[serde(default)]
    last_used_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct StoredAccount {
    id: String,
    mode: AuthMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    openai_api_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    tokens: Option<EveryCodeTokens>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    last_refresh: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    created_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    last_used_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct EveryCodeTokens {
    id_token: String,
    access_token: String,
    refresh_token: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    account_id: Option<String>,
}

fn default_version() -> u32 {
    1
}

pub fn export_every_code(
    paths: &Paths,
    code_home: Option<String>,
    overwrite: bool,
) -> Result<(), String> {
    let code_home = resolve_code_home(code_home)?;
    let accounts_path = code_home.join(EVERY_CODE_ACCOUNTS_FILE);
    let mut data = if overwrite {
        AccountsFile {
            version: default_version(),
            active_account_id: None,
            accounts: Vec::new(),
        }
    } else {
        read_accounts_file(&accounts_path)?
    };

    let labels = read_labels(paths)?;
    let mut id_map = std::collections::HashMap::new();
    let mut exported = 0usize;
    let mut skipped = 0usize;

    for path in profile_files(&paths.profiles)? {
        let Some(profile_id) = profile_id_from_path(&path) else {
            continue;
        };
        let label = label_for_id(&labels, &profile_id);
        let auth = match read_auth_file(&path) {
            Ok(auth) => auth,
            Err(err) => {
                let warning = format_warning(&normalize_error(&err), use_color_stderr());
                eprintln!("{warning}");
                skipped += 1;
                continue;
            }
        };
        let account = match build_account(&profile_id, label, &auth) {
            Ok(Some(account)) => account,
            Ok(None) => {
                skipped += 1;
                continue;
            }
            Err(err) => {
                let warning = format_warning(&normalize_error(&err), use_color_stderr());
                eprintln!("{warning}");
                skipped += 1;
                continue;
            }
        };
        let stored_id = upsert_account(&mut data, account);
        id_map.insert(profile_id, stored_id);
        exported += 1;
    }

    if exported == 0 {
        return Err("Error: no valid profiles to export.".to_string());
    }

    if overwrite || data.active_account_id.is_none() {
        let active = resolve_active_account_id(paths, &data, &id_map);
        if active.is_some() {
            data.active_account_id = active;
        }
    }

    write_every_code_accounts(&accounts_path, &data)?;
    let message = if skipped > 0 {
        format!(
            "Exported {exported} profile{} (skipped {skipped}).",
            if exported == 1 { "" } else { "s" }
        )
    } else {
        format!(
            "Exported {exported} profile{}.",
            if exported == 1 { "" } else { "s" }
        )
    };
    println!("{message}");
    Ok(())
}

pub fn import_every_code(paths: &Paths, code_home: Option<String>) -> Result<(), String> {
    let code_home = resolve_code_home(code_home)?;
    let accounts_path = code_home.join(EVERY_CODE_ACCOUNTS_FILE);
    let data = read_accounts_file_raw(&accounts_path)?;
    if data.accounts.is_empty() {
        return Err(format!(
            "Error: no accounts found in {}",
            accounts_path.display()
        ));
    }

    let mut store = ProfileStore::load(paths)?;
    let mut imported = 0usize;
    let mut skipped = 0usize;

    for account in data.accounts {
        let mode = account
            .mode
            .as_deref()
            .unwrap_or("")
            .trim()
            .to_ascii_lowercase();
        match mode.as_str() {
            "apikey" => {
                let Some(api_key) = account
                    .openai_api_key
                    .as_ref()
                    .filter(|value| !value.is_empty())
                else {
                    let warning = format_warning(
                        "Skipping API key account missing key value",
                        use_color_stderr(),
                    );
                    eprintln!("{warning}");
                    skipped += 1;
                    continue;
                };
                let tokens = tokens_from_api_key(api_key);
                let id = resolve_save_id(paths, &mut store.usage_map, &mut store.labels, &tokens)?;
                let label = account.label.as_deref();
                assign_label_unique(&mut store.labels, label, &id)?;
                let auth = auth_value_for_api_key(api_key, account.last_refresh.as_deref());
                write_profile_value(paths, &id, &auth)?;
                let last_used = parse_last_used(account.last_used_at.as_deref());
                store.usage_map.insert(id, last_used);
                imported += 1;
            }
            "chatgpt" => {
                let Some(tokens) = account.tokens.as_ref() else {
                    let warning = format_warning(
                        "Skipping ChatGPT account missing tokens",
                        use_color_stderr(),
                    );
                    eprintln!("{warning}");
                    skipped += 1;
                    continue;
                };
                let tokens = match build_tokens_from_everycode(tokens) {
                    Ok(tokens) => tokens,
                    Err(err) => {
                        let warning = format_warning(&normalize_error(&err), use_color_stderr());
                        eprintln!("{warning}");
                        skipped += 1;
                        continue;
                    }
                };
                let id = resolve_save_id(paths, &mut store.usage_map, &mut store.labels, &tokens)?;
                let label = account.label.as_deref();
                assign_label_unique(&mut store.labels, label, &id)?;
                let auth = auth_value_for_tokens(&tokens, account.last_refresh.as_deref());
                write_profile_value(paths, &id, &auth)?;
                let last_used = parse_last_used(account.last_used_at.as_deref());
                store.usage_map.insert(id, last_used);
                imported += 1;
            }
            _ => {
                let warning = format_warning(
                    "Skipping unsupported account mode in Every Code accounts",
                    use_color_stderr(),
                );
                eprintln!("{warning}");
                skipped += 1;
            }
        }
    }

    if imported == 0 {
        return Err("Error: no valid accounts to import.".to_string());
    }

    store.save(paths)?;
    let message = if skipped > 0 {
        format!(
            "Imported {imported} profile{} (skipped {skipped}).",
            if imported == 1 { "" } else { "s" }
        )
    } else {
        format!(
            "Imported {imported} profile{}.",
            if imported == 1 { "" } else { "s" }
        )
    };
    println!("{message}");
    Ok(())
}

fn resolve_code_home(code_home: Option<String>) -> Result<PathBuf, String> {
    if let Some(path) = code_home {
        let path = PathBuf::from(path);
        if path.as_os_str().is_empty() {
            return Err("Error: code home cannot be empty".to_string());
        }
        return Ok(path);
    }
    if let Some(path) = env::var_os("CODE_HOME") {
        let path = PathBuf::from(path);
        if !path.as_os_str().is_empty() {
            return Ok(path);
        }
    }
    let base_dirs = BaseDirs::new()
        .ok_or_else(|| "Error: could not resolve home directory for CODE_HOME".to_string())?;
    Ok(base_dirs.home_dir().join(".code"))
}

fn read_accounts_file(path: &Path) -> Result<AccountsFile, String> {
    match fs::read_to_string(path) {
        Ok(contents) => serde_json::from_str(&contents)
            .map_err(|err| format!("Error: invalid {}: {err}", path.display())),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(AccountsFile {
            version: default_version(),
            active_account_id: None,
            accounts: Vec::new(),
        }),
        Err(err) => Err(format!("Error: failed to read {}: {err}", path.display())),
    }
}

fn read_accounts_file_raw(path: &Path) -> Result<AccountsFileRaw, String> {
    match fs::read_to_string(path) {
        Ok(contents) => serde_json::from_str(&contents)
            .map_err(|err| format!("Error: invalid {}: {err}", path.display())),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(AccountsFileRaw {
            accounts: Vec::new(),
        }),
        Err(err) => Err(format!("Error: failed to read {}: {err}", path.display())),
    }
}

fn write_every_code_accounts(path: &Path, data: &AccountsFile) -> Result<(), String> {
    let json = serde_json::to_string_pretty(data)
        .map_err(|err| format!("Error: failed to serialize Every Code accounts: {err}"))?;
    write_atomic_with_mode(path, format!("{json}\n").as_bytes(), 0o600)
        .map_err(|err| format!("Error: failed to write {}: {err}", path.display()))?;
    Ok(())
}

fn auth_value_for_api_key(api_key: &str, last_refresh: Option<&str>) -> Value {
    let mut map = serde_json::Map::new();
    map.insert(
        "OPENAI_API_KEY".to_string(),
        Value::String(api_key.to_string()),
    );
    if let Some(last_refresh) = last_refresh.filter(|value| !value.trim().is_empty()) {
        map.insert(
            "last_refresh".to_string(),
            Value::String(last_refresh.to_string()),
        );
    }
    Value::Object(map)
}

fn auth_value_for_tokens(tokens: &Tokens, last_refresh: Option<&str>) -> Value {
    let mut map = serde_json::Map::new();
    let mut token_map = serde_json::Map::new();
    if let Some(account_id) = tokens.account_id.as_ref().filter(|v| !v.is_empty()) {
        token_map.insert("account_id".to_string(), Value::String(account_id.clone()));
    }
    if let Some(id_token) = tokens.id_token.as_ref().filter(|v| !v.is_empty()) {
        token_map.insert("id_token".to_string(), Value::String(id_token.clone()));
    }
    if let Some(access_token) = tokens.access_token.as_ref().filter(|v| !v.is_empty()) {
        token_map.insert(
            "access_token".to_string(),
            Value::String(access_token.clone()),
        );
    }
    if let Some(refresh_token) = tokens.refresh_token.as_ref().filter(|v| !v.is_empty()) {
        token_map.insert(
            "refresh_token".to_string(),
            Value::String(refresh_token.clone()),
        );
    }
    map.insert("tokens".to_string(), Value::Object(token_map));
    if let Some(last_refresh) = last_refresh.filter(|value| !value.trim().is_empty()) {
        map.insert(
            "last_refresh".to_string(),
            Value::String(last_refresh.to_string()),
        );
    }
    Value::Object(map)
}

fn write_profile_value(paths: &Paths, id: &str, value: &Value) -> Result<(), String> {
    let path = profile_path_for_id(&paths.profiles, id);
    let json = serde_json::to_string_pretty(value)
        .map_err(|err| format!("Error: failed to serialize profile {id}: {err}"))?;
    write_atomic(&path, format!("{json}\n").as_bytes())
        .map_err(|err| format!("Error: failed to write {}: {err}", path.display()))
}

fn build_tokens_from_everycode(tokens: &EveryCodeTokens) -> Result<Tokens, String> {
    if tokens.id_token.trim().is_empty() {
        return Err("Error: account tokens missing id_token".to_string());
    }
    if !id_token_valid(&tokens.id_token) {
        return Err("Error: account id_token is invalid".to_string());
    }
    if tokens.access_token.trim().is_empty() {
        return Err("Error: account tokens missing access_token".to_string());
    }
    if tokens.refresh_token.trim().is_empty() {
        return Err("Error: account tokens missing refresh_token".to_string());
    }
    let Some(account_id) = tokens.account_id.clone() else {
        return Err("Error: account tokens missing account_id".to_string());
    };
    Ok(Tokens {
        account_id: Some(account_id),
        id_token: Some(tokens.id_token.clone()),
        access_token: Some(tokens.access_token.clone()),
        refresh_token: Some(tokens.refresh_token.clone()),
    })
}

fn assign_label_unique(labels: &mut Labels, label: Option<&str>, id: &str) -> Result<(), String> {
    let Some(label) = label.map(str::trim) else {
        return Ok(());
    };
    if label.is_empty() {
        return Ok(());
    }
    let id_value = id.to_string();
    if labels.get(label).is_none() {
        return assign_label(labels, label, id);
    }
    if labels.get(label) == Some(&id_value) {
        return Ok(());
    }
    let mut suffix = 2;
    loop {
        let candidate = format!("{label}-{suffix}");
        if labels.get(&candidate).is_none() {
            return assign_label(labels, &candidate, id);
        }
        suffix += 1;
    }
}

fn parse_last_used(value: Option<&str>) -> u64 {
    let Some(value) = value else {
        return 0;
    };
    let value = value.trim();
    if value.is_empty() {
        return 0;
    }
    match chrono::DateTime::parse_from_rfc3339(value) {
        Ok(dt) => dt.timestamp().max(0) as u64,
        Err(_) => now_seconds(),
    }
}

fn build_account(
    profile_id: &str,
    label: Option<String>,
    auth: &AuthFile,
) -> Result<Option<StoredAccount>, String> {
    if let Some(api_key) = auth
        .openai_api_key
        .as_ref()
        .filter(|value| !value.is_empty())
    {
        return Ok(Some(StoredAccount {
            id: profile_id.to_string(),
            mode: AuthMode::ApiKey,
            label,
            openai_api_key: Some(api_key.to_string()),
            tokens: None,
            last_refresh: auth.last_refresh.clone(),
            created_at: None,
            last_used_at: None,
        }));
    }

    let tokens = auth.tokens.as_ref().ok_or_else(|| {
        "Error: profile missing tokens and API key; re-save the profile".to_string()
    })?;

    if is_api_key_profile(tokens) {
        return Err("Error: profile uses API key but key is missing".to_string());
    }

    let id_token = tokens
        .id_token
        .as_deref()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "Error: profile is missing id_token".to_string())?;
    if !id_token_valid(id_token) {
        return Err("Error: profile has invalid id_token".to_string());
    }
    let access_token = tokens
        .access_token
        .as_deref()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "Error: profile is missing access_token".to_string())?;
    let refresh_token = tokens
        .refresh_token
        .as_deref()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "Error: profile is missing refresh_token".to_string())?;

    Ok(Some(StoredAccount {
        id: profile_id.to_string(),
        mode: AuthMode::Chatgpt,
        label,
        openai_api_key: None,
        tokens: Some(EveryCodeTokens {
            id_token: id_token.to_string(),
            access_token: access_token.to_string(),
            refresh_token: refresh_token.to_string(),
            account_id: tokens.account_id.clone(),
        }),
        last_refresh: auth.last_refresh.clone(),
        created_at: None,
        last_used_at: None,
    }))
}

fn upsert_account(data: &mut AccountsFile, mut account: StoredAccount) -> String {
    if let Some(idx) = data
        .accounts
        .iter()
        .position(|existing| match_account(existing, &account))
    {
        let stored = &mut data.accounts[idx];
        if account.label.is_some() {
            stored.label = account.label.take();
        }
        if account.last_refresh.is_some() {
            stored.last_refresh = account.last_refresh.take();
        }
        if account.tokens.is_some() {
            stored.tokens = account.tokens.take();
        }
        if account.openai_api_key.is_some() {
            stored.openai_api_key = account.openai_api_key.take();
        }
        return stored.id.clone();
    }
    let id = unique_account_id(&account.id, &data.accounts);
    account.id = id.clone();
    data.accounts.push(account);
    id
}

fn unique_account_id(base: &str, accounts: &[StoredAccount]) -> String {
    if accounts.iter().all(|acc| acc.id != base) {
        return base.to_string();
    }
    let mut idx = 2;
    loop {
        let candidate = format!("{base}-{idx}");
        if accounts.iter().all(|acc| acc.id != candidate) {
            return candidate;
        }
        idx += 1;
    }
}

fn match_account(existing: &StoredAccount, candidate: &StoredAccount) -> bool {
    match (&existing.mode, &candidate.mode) {
        (AuthMode::ApiKey, AuthMode::ApiKey) => existing.openai_api_key == candidate.openai_api_key,
        (AuthMode::Chatgpt, AuthMode::Chatgpt) => {
            let Some(existing_tokens) = existing.tokens.as_ref() else {
                return false;
            };
            let Some(candidate_tokens) = candidate.tokens.as_ref() else {
                return false;
            };
            let account_id_matches = match (
                existing_tokens.account_id.as_deref(),
                candidate_tokens.account_id.as_deref(),
            ) {
                (Some(a), Some(b)) => a == b,
                _ => false,
            };
            let email_matches = match (
                email_from_id_token(&existing_tokens.id_token),
                email_from_id_token(&candidate_tokens.id_token),
            ) {
                (Some(a), Some(b)) => a.eq_ignore_ascii_case(&b),
                _ => false,
            };
            account_id_matches && email_matches
        }
        _ => false,
    }
}

fn resolve_active_account_id(
    paths: &Paths,
    data: &AccountsFile,
    id_map: &std::collections::HashMap<String, String>,
) -> Option<String> {
    let auth = read_auth_file(&paths.auth).ok()?;
    if let Some(account) = auth
        .openai_api_key
        .as_ref()
        .filter(|value| !value.is_empty())
        .and_then(|api_key| {
            data.accounts.iter().find(|acc| {
                acc.mode == AuthMode::ApiKey && acc.openai_api_key.as_deref() == Some(api_key)
            })
        })
    {
        return Some(account.id.clone());
    }
    if let Some(tokens) = auth.tokens.as_ref() {
        let (email, _) = extract_email_and_plan(tokens);
        if let Some(account_id) = tokens.account_id.as_deref() {
            let target_email = email.as_deref();
            let matches = data.accounts.iter().find(|acc| {
                if acc.mode != AuthMode::Chatgpt {
                    return false;
                }
                let Some(acc_tokens) = acc.tokens.as_ref() else {
                    return false;
                };
                if acc_tokens.account_id.as_deref() != Some(account_id) {
                    return false;
                }
                let stored_email = email_from_id_token(&acc_tokens.id_token);
                match (target_email, stored_email.as_deref()) {
                    (Some(expected), Some(actual)) => actual.eq_ignore_ascii_case(expected),
                    _ => false,
                }
            });
            if let Some(account) = matches {
                return Some(account.id.clone());
            }
        }
    }
    id_map
        .values()
        .next()
        .cloned()
        .or_else(|| data.accounts.first().map(|acc| acc.id.clone()))
}

fn email_from_id_token(id_token: &str) -> Option<String> {
    let mut parts = id_token.split('.');
    let _header = parts.next()?;
    let payload = parts.next()?;
    let _sig = parts.next()?;
    let decoded = URL_SAFE_NO_PAD.decode(payload).ok()?;
    let value: Value = serde_json::from_slice(&decoded).ok()?;
    value
        .get("email")
        .and_then(Value::as_str)
        .map(|s| s.to_string())
}

fn id_token_valid(id_token: &str) -> bool {
    let mut parts = id_token.split('.');
    let _header = parts.next();
    let payload = parts.next();
    let _sig = parts.next();
    let Some(payload) = payload else {
        return false;
    };
    let Ok(decoded) = URL_SAFE_NO_PAD.decode(payload) else {
        return false;
    };
    serde_json::from_slice::<Value>(&decoded).is_ok()
}
