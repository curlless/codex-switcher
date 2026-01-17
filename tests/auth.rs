use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use codex_profiles::{AuthFile, Tokens, extract_email_and_plan};

fn build_id_token(email: &str, plan: &str) -> String {
    let header = serde_json::json!({
        "alg": "none",
        "typ": "JWT",
    });
    let auth = serde_json::json!({
        "chatgpt_plan_type": plan,
    });
    let payload = serde_json::json!({
        "email": email,
        "https://api.openai.com/auth": auth,
    });
    let header = URL_SAFE_NO_PAD.encode(serde_json::to_string(&header).unwrap());
    let payload = URL_SAFE_NO_PAD.encode(serde_json::to_string(&payload).unwrap());
    format!("{header}.{payload}.")
}

#[test]
fn extracts_email_and_plan_from_id_token() {
    let id_token = build_id_token("alpha@example.com", "team");
    let tokens = Tokens {
        account_id: None,
        id_token: Some(id_token),
        access_token: None,
        refresh_token: None,
    };
    let (email, plan) = extract_email_and_plan(&tokens);
    assert_eq!(email.as_deref(), Some("alpha@example.com"));
    assert_eq!(plan.as_deref(), Some("Team"));
}

#[test]
fn extracts_email_and_plan_for_api_key_profile() {
    let tokens = Tokens {
        account_id: Some("api-key-sk-proj-a3x~1234".to_string()),
        id_token: None,
        access_token: None,
        refresh_token: None,
    };
    let (email, plan) = extract_email_and_plan(&tokens);
    assert_eq!(email.as_deref(), Some("~1234"));
    assert_eq!(plan.as_deref(), Some("Key"));
}

#[test]
fn extracts_email_and_plan_for_legacy_api_key_profile() {
    let tokens = Tokens {
        account_id: Some("api-key-1234".to_string()),
        id_token: None,
        access_token: None,
        refresh_token: None,
    };
    let (email, plan) = extract_email_and_plan(&tokens);
    assert_eq!(email.as_deref(), Some("Key"));
    assert_eq!(plan.as_deref(), Some("Key"));
}

#[test]
fn auth_file_parses_without_refresh_token() {
    let id_token = build_id_token("alpha@example.com", "team");
    let json = format!(
        "{{\"tokens\":{{\"account_id\":\"acct-alpha\",\"id_token\":\"{id_token}\",\"access_token\":\"token\"}}}}"
    );
    let auth: AuthFile = serde_json::from_str(&json).expect("parse auth");
    let tokens = auth.tokens.expect("tokens");
    assert_eq!(tokens.account_id.as_deref(), Some("acct-alpha"));
}
