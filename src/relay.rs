use std::io::{self, IsTerminal};
use std::time::Duration;

use ureq::http::Uri;

use crate::{format_action, print_output_block, use_color_stdout};

const RELAY_TIMEOUT_SECS: u64 = 5;

pub fn relay_login(url_arg: Option<String>) -> Result<(), String> {
    let callback_url = resolve_callback_url(url_arg)?;
    validate_callback_url(&callback_url)?;
    relay_callback(&callback_url)?;
    let message = format_action(
        "Relayed callback to local login listener. Complete login in Roo/Codex terminal.",
        use_color_stdout(),
    );
    print_output_block(&message);
    Ok(())
}

fn resolve_callback_url(url_arg: Option<String>) -> Result<String, String> {
    resolve_callback_url_with(url_arg, io::stdin().is_terminal(), || {
        println!("Paste Roo/Codex callback URL:");
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|err| format!("Error: failed to read callback URL: {err}"))?;
        Ok(input)
    })
}

fn resolve_callback_url_with<F>(
    url_arg: Option<String>,
    stdin_is_tty: bool,
    read_input: F,
) -> Result<String, String>
where
    F: FnOnce() -> Result<String, String>,
{
    if let Some(url) = url_arg {
        return normalize_callback_url(url);
    }
    if !stdin_is_tty {
        return Err("Error: --url is required in non-interactive mode.".to_string());
    }
    let input = read_input()?;
    normalize_callback_url(input)
}

fn normalize_callback_url(value: String) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err("Error: callback URL is empty.".to_string());
    }
    Ok(trimmed.to_string())
}

fn validate_callback_url(callback_url: &str) -> Result<(), String> {
    if callback_url.contains('#') {
        return Err("Error: callback URL must not include a fragment.".to_string());
    }

    let uri: Uri = callback_url
        .parse()
        .map_err(|err| format!("Error: callback URL is invalid: {err}"))?;

    if !matches!(uri.scheme_str(), Some(scheme) if scheme.eq_ignore_ascii_case("http")) {
        return Err("Error: callback URL must use http.".to_string());
    }

    let host = uri.host().unwrap_or_default();
    if !host.eq_ignore_ascii_case("localhost") && host != "127.0.0.1" {
        return Err("Error: callback URL host must be localhost or 127.0.0.1.".to_string());
    }

    if uri.port_u16().is_none() {
        return Err("Error: callback URL must include an explicit port.".to_string());
    }

    if uri.path() != "/auth/callback" {
        return Err("Error: callback URL path must be /auth/callback.".to_string());
    }

    let query = uri.query().unwrap_or_default();
    if !has_non_empty_query_param(query, "code") {
        return Err("Error: callback URL must include a non-empty code query param.".to_string());
    }
    if !has_non_empty_query_param(query, "state") {
        return Err("Error: callback URL must include a non-empty state query param.".to_string());
    }

    Ok(())
}

fn has_non_empty_query_param(query: &str, key: &str) -> bool {
    query.split('&').any(|part| {
        let (name, value) = part.split_once('=').unwrap_or((part, ""));
        name == key && !value.is_empty()
    })
}

fn relay_callback(callback_url: &str) -> Result<(), String> {
    let config = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(RELAY_TIMEOUT_SECS)))
        .max_redirects(0)
        .build();
    let agent: ureq::Agent = config.into();
    match agent.get(callback_url).call() {
        Ok(response) => map_listener_status(response.status().as_u16()),
        Err(ureq::Error::StatusCode(status)) => map_listener_status(status),
        Err(err) => Err(map_transport_error(err)),
    }
}

fn map_listener_status(status: u16) -> Result<(), String> {
    if (200..=399).contains(&status) {
        return Ok(());
    }
    if (400..=499).contains(&status) {
        return Err(format!(
            "Error: local login listener returned http status: {status}. Verify the listener is running and retry with a fresh callback URL."
        ));
    }
    if (500..=599).contains(&status) {
        return Err(format!(
            "Error: local login listener returned http status: {status}. The listener failed to process the callback; restart it and retry."
        ));
    }
    Err(format!(
        "Error: local login listener returned unexpected http status: {status}."
    ))
}

fn map_transport_error(err: ureq::Error) -> String {
    format!(
        "Error: failed to reach local login listener: {err}. Ensure the listener is running, then retry `relay-login --url <callback_url>`."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_LOCALHOST: &str =
        "http://localhost:1455/auth/callback?code=test-code&state=test-state";
    const VALID_LOOPBACK: &str =
        "http://127.0.0.1:1455/auth/callback?state=test-state&code=test-code";

    #[test]
    fn resolve_callback_url_uses_url_arg() {
        let value =
            resolve_callback_url_with(Some(format!("  {VALID_LOCALHOST}  ")), false, || {
                Ok(String::new())
            })
            .unwrap();
        assert_eq!(value, VALID_LOCALHOST);
    }

    #[test]
    fn resolve_callback_url_requires_url_when_non_interactive() {
        let err = resolve_callback_url_with(None, false, || Ok(String::new())).unwrap_err();
        assert_eq!(err, "Error: --url is required in non-interactive mode.");
    }

    #[test]
    fn resolve_callback_url_reports_read_error() {
        let err = resolve_callback_url_with(None, true, || {
            Err("Error: failed to read callback URL: broken pipe".to_string())
        })
        .unwrap_err();
        assert_eq!(err, "Error: failed to read callback URL: broken pipe");
    }

    #[test]
    fn resolve_callback_url_rejects_empty_input() {
        let err = resolve_callback_url_with(None, true, || Ok("   \n".to_string())).unwrap_err();
        assert_eq!(err, "Error: callback URL is empty.");
    }

    #[test]
    fn validate_callback_url_accepts_localhost_url() {
        validate_callback_url(VALID_LOCALHOST).unwrap();
    }

    #[test]
    fn validate_callback_url_accepts_loopback_url() {
        validate_callback_url(VALID_LOOPBACK).unwrap();
    }

    #[test]
    fn validate_callback_url_rejects_fragment() {
        let err = validate_callback_url(&format!("{VALID_LOCALHOST}#frag")).unwrap_err();
        assert_eq!(err, "Error: callback URL must not include a fragment.");
    }

    #[test]
    fn validate_callback_url_rejects_non_http_scheme() {
        let err = validate_callback_url("https://localhost:1455/auth/callback?code=a&state=b")
            .unwrap_err();
        assert_eq!(err, "Error: callback URL must use http.");
    }

    #[test]
    fn validate_callback_url_rejects_non_loopback_host() {
        let err = validate_callback_url("http://example.com:1455/auth/callback?code=a&state=b")
            .unwrap_err();
        assert_eq!(
            err,
            "Error: callback URL host must be localhost or 127.0.0.1."
        );
    }

    #[test]
    fn validate_callback_url_rejects_missing_port() {
        let err =
            validate_callback_url("http://localhost/auth/callback?code=a&state=b").unwrap_err();
        assert_eq!(err, "Error: callback URL must include an explicit port.");
    }

    #[test]
    fn validate_callback_url_rejects_wrong_path() {
        let err =
            validate_callback_url("http://localhost:1455/callback?code=a&state=b").unwrap_err();
        assert_eq!(err, "Error: callback URL path must be /auth/callback.");
    }

    #[test]
    fn validate_callback_url_rejects_missing_code() {
        let err = validate_callback_url("http://localhost:1455/auth/callback?state=b").unwrap_err();
        assert_eq!(
            err,
            "Error: callback URL must include a non-empty code query param."
        );
    }

    #[test]
    fn validate_callback_url_rejects_empty_state() {
        let err =
            validate_callback_url("http://localhost:1455/auth/callback?code=a&state=").unwrap_err();
        assert_eq!(
            err,
            "Error: callback URL must include a non-empty state query param."
        );
    }

    #[test]
    fn map_listener_status_accepts_2xx_and_3xx() {
        assert!(map_listener_status(204).is_ok());
        assert!(map_listener_status(302).is_ok());
    }

    #[test]
    fn map_listener_status_preserves_4xx_and_5xx_status() {
        let client_err = map_listener_status(401).unwrap_err();
        assert!(client_err.contains("401"));
        assert!(client_err.contains("retry"));

        let server_err = map_listener_status(503).unwrap_err();
        assert!(server_err.contains("503"));
        assert!(server_err.contains("restart"));
    }

    #[test]
    fn map_transport_error_includes_listener_hint() {
        let err = map_transport_error(ureq::Error::Timeout(ureq::Timeout::Global));
        assert!(err.starts_with("Error: failed to reach local login listener:"));
        assert!(err.contains("listener is running"));
        assert!(err.contains("relay-login --url <callback_url>"));
    }
}
