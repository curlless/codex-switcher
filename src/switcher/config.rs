use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::process::Command as ProcessCommand;

use crate::switcher::{
    CodexAppOverride, Paths, ReloadAppTarget, detect_codex_app_discovery, format_action,
    format_hint, print_output_block, use_color_stdout, write_atomic,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SwitcherConfig {
    pub reload: ReloadConfig,
    pub switch: SwitchConfig,
    pub editor: EditorConfig,
    pub codex_app: CodexAppConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ReloadConfig {
    pub primary_target: ReloadAppTarget,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SwitchConfig {
    pub reload_after_switch: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct EditorConfig {
    pub command: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CodexAppConfig {
    pub path: Option<String>,
    pub app_user_model_id: Option<String>,
}

impl Default for ReloadConfig {
    fn default() -> Self {
        Self {
            primary_target: ReloadAppTarget::Codex,
        }
    }
}

pub fn load_switcher_config(paths: &Paths) -> Result<SwitcherConfig, String> {
    if !paths.switcher_config.exists() {
        return Ok(SwitcherConfig::default());
    }
    let raw = fs::read_to_string(&paths.switcher_config).map_err(|err| {
        format!(
            "Error: cannot read config file {}: {err}",
            paths.switcher_config.display()
        )
    })?;
    toml::from_str(&raw).map_err(|err| {
        format!(
            "Error: config file {} is invalid TOML: {err}",
            paths.switcher_config.display()
        )
    })
}

pub fn show_config(paths: &Paths) -> Result<(), String> {
    ensure_switcher_config_exists(paths)?;
    let raw = fs::read_to_string(&paths.switcher_config).map_err(|err| {
        format!(
            "Error: cannot read config file {}: {err}",
            paths.switcher_config.display()
        )
    })?;
    let use_color = use_color_stdout();
    let header = format_action(
        &format!("Config file: {}", paths.switcher_config.display()),
        use_color,
    );
    print_output_block(&format!("{header}\n\n{}", raw.trim_end()));
    Ok(())
}

pub fn edit_config(paths: &Paths) -> Result<(), String> {
    ensure_switcher_config_exists(paths)?;
    let config = load_switcher_config(paths)?;
    let mut editor_parts = resolve_editor_command(&config)?;
    let config_path = paths.switcher_config.to_string_lossy().into_owned();
    let placeholder_used = apply_path_placeholder(&mut editor_parts, &config_path);
    let program = editor_parts
        .first()
        .cloned()
        .ok_or_else(|| "Error: editor command is empty.".to_string())?;
    let args = if placeholder_used {
        editor_parts.into_iter().skip(1).collect::<Vec<_>>()
    } else {
        let mut args = editor_parts.into_iter().skip(1).collect::<Vec<_>>();
        args.push(config_path);
        args
    };
    let status = ProcessCommand::new(&program)
        .args(&args)
        .status()
        .map_err(|err| format!("Error: failed to launch editor '{program}': {err}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("Error: editor exited with status {status}"))
    }
}

pub fn detect_codex_app(paths: &Paths, write_env: bool) -> Result<(), String> {
    ensure_switcher_config_exists(paths)?;
    let mut config = load_switcher_config(paths)?;
    let discovery = detect_codex_app_discovery(codex_app_override_from_config(&config).as_ref())?;
    let saved_to_config = persist_detected_codex_app(paths, &mut config, &discovery)?;
    if write_env {
        persist_user_env_var("CODEX_SWITCHER_CODEX_APP_PATH", &discovery.executable_path)?;
        persist_user_env_var("CODEX_PROFILES_CODEX_APP_PATH", &discovery.executable_path)?;
        if let Some(aumid) = discovery.app_user_model_id.as_deref() {
            persist_user_env_text("CODEX_SWITCHER_CODEX_APP_AUMID", aumid)?;
            persist_user_env_text("CODEX_PROFILES_CODEX_APP_AUMID", aumid)?;
        }
    }
    let use_color = use_color_stdout();
    let mut lines = vec![format_action(
        &format!(
            "Detected Codex app via {}: {}",
            discovery.source,
            discovery.executable_path.display()
        ),
        use_color,
    )];
    if let Some(aumid) = discovery.app_user_model_id.as_deref() {
        lines.push(format_hint(&format!("AppUserModelID: {aumid}"), use_color));
    }
    lines.push(format_hint(
        &format!(
            "Config override fields: [codex_app].path = \"{}\"",
            discovery.executable_path.display()
        ),
        use_color,
    ));
    lines.push(format_hint(
        if saved_to_config {
            "Saved detected Codex app path into [codex_app] in config.toml."
        } else {
            "Config already contains the detected Codex app path."
        },
        use_color,
    ));
    if write_env {
        lines.push(format_hint(
            "Wrote user env vars CODEX_SWITCHER_CODEX_APP_PATH / CODEX_PROFILES_CODEX_APP_PATH and, when available, matching *_AUMID vars. Open a new terminal to see them in your shell.",
            use_color,
        ));
    }
    print_output_block(&lines.join("\n"));
    Ok(())
}

pub fn effective_reload_target(
    paths: &Paths,
    explicit_target: Option<ReloadAppTarget>,
) -> Result<ReloadAppTarget, String> {
    if let Some(target) = explicit_target {
        return Ok(target);
    }
    Ok(load_switcher_config(paths)?.reload.primary_target)
}

pub fn switch_reload_target(
    paths: &Paths,
    explicit_target: Option<ReloadAppTarget>,
) -> Result<Option<ReloadAppTarget>, String> {
    if explicit_target.is_some() {
        return Ok(explicit_target);
    }
    let config = load_switcher_config(paths)?;
    Ok(config
        .switch
        .reload_after_switch
        .then_some(config.reload.primary_target))
}

pub fn codex_app_override(paths: &Paths) -> Result<Option<CodexAppOverride>, String> {
    let config = load_switcher_config(paths)?;
    Ok(codex_app_override_from_config(&config))
}

pub fn ensure_codex_app_override(paths: &Paths) -> Result<Option<CodexAppOverride>, String> {
    ensure_switcher_config_exists(paths)?;
    let mut config = load_switcher_config(paths)?;
    if has_valid_codex_app_path(&config) {
        return Ok(codex_app_override_from_config(&config));
    }
    let discovery = detect_codex_app_discovery(codex_app_override_from_config(&config).as_ref())?;
    persist_detected_codex_app(paths, &mut config, &discovery)?;
    Ok(codex_app_override_from_config(&config))
}

fn ensure_switcher_config_exists(paths: &Paths) -> Result<(), String> {
    if paths.switcher_config.exists() {
        return Ok(());
    }
    let template = default_switcher_config_template();
    write_atomic(&paths.switcher_config, template.as_bytes())
        .map_err(|err| format!("Error: failed to write config file: {err}"))
}

fn default_switcher_config_template() -> String {
    render_switcher_config(&SwitcherConfig::default())
}

fn render_switcher_config(config: &SwitcherConfig) -> String {
    let mut rendered = String::from("# codex-switcher configuration\n");
    rendered.push_str("# This file is stored next to your saved profiles index.\n");
    rendered.push_str("#\n");
    rendered.push_str("# reload.primary_target controls the default target for `reload-app`\n");
    rendered.push_str("# and for auto-reload after `switch` when enabled below.\n");
    rendered.push_str("\n[reload]\n");
    rendered.push_str(&format!(
        "primary_target = {}\n",
        toml_basic_string(reload_target_name(config.reload.primary_target))
    ));
    rendered.push_str("\n[switch]\n");
    rendered.push_str(&format!(
        "reload_after_switch = {}\n",
        config.switch.reload_after_switch
    ));
    rendered.push_str("\n[editor]\n");
    rendered.push_str("# Optional editor command for `codex-switcher config edit`.\n");
    rendered.push_str("# Examples: \"code --wait\", \"cursor --wait\", \"notepad\"\n");
    if let Some(command) = config
        .editor
        .command
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        rendered.push_str(&format!("command = {}\n", toml_basic_string(command)));
    } else {
        rendered.push_str("# command = \"code --wait\"\n");
    }
    rendered.push_str("\n[codex_app]\n");
    rendered.push_str("# Optional explicit path to standalone Codex.exe\n");
    if let Some(path) = config
        .codex_app
        .path
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        rendered.push_str(&format!("path = {}\n", toml_basic_string(path)));
    } else {
        rendered.push_str(
            "# path = \"C:\\\\Program Files\\\\WindowsApps\\\\OpenAI.Codex_...\\\\app\\\\Codex.exe\"\n",
        );
    }
    rendered.push_str("# Optional explicit AppUserModelID for AppsFolder relaunch\n");
    if let Some(app_user_model_id) = config
        .codex_app
        .app_user_model_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        rendered.push_str(&format!(
            "app_user_model_id = {}\n",
            toml_basic_string(app_user_model_id)
        ));
    } else {
        rendered.push_str("# app_user_model_id = \"OpenAI.Codex_xxxxx!App\"\n");
    }
    rendered
}

fn codex_app_override_from_config(config: &SwitcherConfig) -> Option<CodexAppOverride> {
    let executable_path = config
        .codex_app
        .path
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(std::path::PathBuf::from);
    let app_user_model_id = config
        .codex_app
        .app_user_model_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    (executable_path.is_some() || app_user_model_id.is_some()).then_some(CodexAppOverride {
        executable_path,
        app_user_model_id,
    })
}

fn has_valid_codex_app_path(config: &SwitcherConfig) -> bool {
    config
        .codex_app
        .path
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(std::path::Path::new)
        .is_some_and(|path| path.is_file())
}

fn persist_detected_codex_app(
    paths: &Paths,
    config: &mut SwitcherConfig,
    discovery: &crate::switcher::CodexAppDiscovery,
) -> Result<bool, String> {
    let detected_path = discovery.executable_path.to_string_lossy().into_owned();
    let current_path = config
        .codex_app
        .path
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let current_aumid = config
        .codex_app
        .app_user_model_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let detected_aumid = discovery
        .app_user_model_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let changed = current_path != Some(detected_path.as_str()) || current_aumid != detected_aumid;
    if !changed {
        return Ok(false);
    }
    config.codex_app.path = Some(detected_path);
    config.codex_app.app_user_model_id = detected_aumid.map(str::to_string);
    save_switcher_config(paths, config)?;
    Ok(true)
}

fn save_switcher_config(paths: &Paths, config: &SwitcherConfig) -> Result<(), String> {
    let rendered = render_switcher_config(config);
    write_atomic(&paths.switcher_config, rendered.as_bytes())
        .map_err(|err| format!("Error: failed to write config file: {err}"))
}

fn reload_target_name(target: ReloadAppTarget) -> &'static str {
    match target {
        ReloadAppTarget::All => "all",
        ReloadAppTarget::Codex => "codex",
        ReloadAppTarget::Cursor => "cursor",
    }
}

fn toml_basic_string(value: &str) -> String {
    format!("{value:?}")
}

fn resolve_editor_command(config: &SwitcherConfig) -> Result<Vec<String>, String> {
    let configured = config
        .editor
        .command
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let env_editor = env::var("VISUAL")
        .ok()
        .or_else(|| env::var("EDITOR").ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let fallback = if cfg!(windows) { "notepad" } else { "vi" }.to_string();
    let raw = configured.or(env_editor).unwrap_or(fallback);
    shlex::split(&raw)
        .filter(|parts| !parts.is_empty())
        .ok_or_else(|| format!("Error: could not parse editor command '{raw}'."))
}

fn apply_path_placeholder(parts: &mut [String], path: &str) -> bool {
    let mut replaced = false;
    for part in parts.iter_mut() {
        if part.contains("{path}") {
            *part = part.replace("{path}", path);
            replaced = true;
        }
    }
    replaced
}

#[cfg(windows)]
fn persist_user_env_var(name: &str, value: &std::path::Path) -> Result<(), String> {
    persist_user_env_text(name, &value.to_string_lossy())
}

#[cfg(windows)]
fn persist_user_env_text(name: &str, value: &str) -> Result<(), String> {
    let escaped_name = powershell_single_quote(name);
    let escaped_value = powershell_single_quote(value);
    let command =
        format!("[Environment]::SetEnvironmentVariable('{escaped_name}','{escaped_value}','User')");
    let status = ProcessCommand::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &command])
        .status()
        .map_err(|err| format!("Error: failed to write user env var {name}: {err}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "Error: failed to write user env var {name}: PowerShell exited with {status}"
        ))
    }
}

#[cfg(not(windows))]
fn persist_user_env_var(name: &str, _value: &std::path::Path) -> Result<(), String> {
    Err(format!(
        "Error: writing persistent user env vars is only implemented for Windows ({name})."
    ))
}

#[cfg(not(windows))]
fn persist_user_env_text(name: &str, _value: &str) -> Result<(), String> {
    Err(format!(
        "Error: writing persistent user env vars is only implemented for Windows ({name})."
    ))
}

fn powershell_single_quote(value: &str) -> String {
    value.replace('\'', "''")
}

#[cfg(all(test, feature = "switcher-unit-tests"))]
mod tests {
    use super::*;
    use crate::switcher::test_utils::{make_paths, set_env_guard};

    #[test]
    fn load_switcher_config_defaults_when_missing() {
        let dir = tempfile::tempdir().expect("tempdir");
        let paths = make_paths(dir.path());
        let config = load_switcher_config(&paths).expect("defaults");
        assert_eq!(config.reload.primary_target, ReloadAppTarget::Codex);
        assert!(!config.switch.reload_after_switch);
    }

    #[test]
    fn load_switcher_config_reads_toml_values() {
        let dir = tempfile::tempdir().expect("tempdir");
        let paths = make_paths(dir.path());
        fs::create_dir_all(&paths.profiles).expect("profiles dir");
        fs::write(
            &paths.switcher_config,
            "[reload]\nprimary_target = \"cursor\"\n\n[switch]\nreload_after_switch = true\n",
        )
        .expect("config write");
        let config = load_switcher_config(&paths).expect("config");
        assert_eq!(config.reload.primary_target, ReloadAppTarget::Cursor);
        assert!(config.switch.reload_after_switch);
    }

    #[test]
    fn codex_app_override_reads_config_values() {
        let config = SwitcherConfig {
            codex_app: CodexAppConfig {
                path: Some(r"C:\Apps\Codex.exe".to_string()),
                app_user_model_id: Some("OpenAI.Codex_test!App".to_string()),
            },
            ..SwitcherConfig::default()
        };
        let override_ = codex_app_override_from_config(&config).expect("override");
        assert_eq!(
            override_.executable_path.as_deref(),
            Some(std::path::Path::new(r"C:\Apps\Codex.exe"))
        );
        assert_eq!(
            override_.app_user_model_id.as_deref(),
            Some("OpenAI.Codex_test!App")
        );
    }

    #[test]
    fn switch_reload_target_uses_config_when_enabled() {
        let dir = tempfile::tempdir().expect("tempdir");
        let paths = make_paths(dir.path());
        fs::create_dir_all(&paths.profiles).expect("profiles dir");
        fs::write(
            &paths.switcher_config,
            "[reload]\nprimary_target = \"cursor\"\n\n[switch]\nreload_after_switch = true\n",
        )
        .expect("config write");
        let target = switch_reload_target(&paths, None).expect("target");
        assert_eq!(target, Some(ReloadAppTarget::Cursor));
    }

    #[test]
    fn effective_reload_target_prefers_explicit_value() {
        let dir = tempfile::tempdir().expect("tempdir");
        let paths = make_paths(dir.path());
        let target =
            effective_reload_target(&paths, Some(ReloadAppTarget::Cursor)).expect("target");
        assert_eq!(target, ReloadAppTarget::Cursor);
    }

    #[test]
    fn ensure_codex_app_override_uses_existing_valid_path() {
        let dir = tempfile::tempdir().expect("tempdir");
        let paths = make_paths(dir.path());
        fs::create_dir_all(&paths.profiles).expect("profiles dir");
        let exe = dir.path().join("Codex.exe");
        fs::write(&exe, "").expect("exe");
        let config = SwitcherConfig {
            codex_app: CodexAppConfig {
                path: Some(exe.to_string_lossy().into_owned()),
                app_user_model_id: Some("OpenAI.Codex_test!App".to_string()),
            },
            ..SwitcherConfig::default()
        };
        save_switcher_config(&paths, &config).expect("config write");
        let override_ = ensure_codex_app_override(&paths)
            .expect("override result")
            .expect("override");
        assert_eq!(override_.executable_path.as_deref(), Some(exe.as_path()));
        assert_eq!(
            override_.app_user_model_id.as_deref(),
            Some("OpenAI.Codex_test!App")
        );
    }

    #[test]
    fn resolve_editor_command_uses_env_when_config_missing() {
        let _visual = set_env_guard("VISUAL", Some("code --wait"));
        let _editor = set_env_guard("EDITOR", None);
        let command = resolve_editor_command(&SwitcherConfig::default()).expect("editor");
        assert_eq!(command, vec!["code".to_string(), "--wait".to_string()]);
    }

    #[test]
    fn apply_path_placeholder_rewrites_inline_path() {
        let mut parts = vec![
            "code".to_string(),
            "--goto".to_string(),
            "{path}:1".to_string(),
        ];
        assert!(apply_path_placeholder(&mut parts, "C:\\temp\\config.toml"));
        assert_eq!(parts[2], "C:\\temp\\config.toml:1");
    }
}
