use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::process::Command as ProcessCommand;

use crate::switcher::{
    CodexAppOverride, Paths, ReloadAppTarget, detect_codex_app_discovery, format_action,
    format_hint, print_output_block, use_color_stdout, write_atomic,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SwitchConfig {
    pub reload_after_switch: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct EditorConfig {
    pub command: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct CodexAppConfig {
    pub path: Option<String>,
    pub app_user_model_id: Option<String>,
}

impl Default for SwitcherConfig {
    fn default() -> Self {
        Self {
            reload: ReloadConfig::default(),
            switch: SwitchConfig::default(),
            editor: EditorConfig::default(),
            codex_app: CodexAppConfig::default(),
        }
    }
}

impl Default for ReloadConfig {
    fn default() -> Self {
        Self {
            primary_target: ReloadAppTarget::Codex,
        }
    }
}

impl Default for SwitchConfig {
    fn default() -> Self {
        Self {
            reload_after_switch: false,
        }
    }
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self { command: None }
    }
}

impl Default for CodexAppConfig {
    fn default() -> Self {
        Self {
            path: None,
            app_user_model_id: None,
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
    let config = load_switcher_config(paths)?;
    let discovery = detect_codex_app_discovery(codex_app_override_from_config(&config).as_ref())?;
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

fn ensure_switcher_config_exists(paths: &Paths) -> Result<(), String> {
    if paths.switcher_config.exists() {
        return Ok(());
    }
    let template = default_switcher_config_template();
    write_atomic(&paths.switcher_config, template.as_bytes())
        .map_err(|err| format!("Error: failed to write config file: {err}"))
}

fn default_switcher_config_template() -> String {
    concat!(
        "# codex-switcher configuration\n",
        "# This file is stored next to your saved profiles index.\n",
        "#\n",
        "# reload.primary_target controls the default target for `reload-app`\n",
        "# and for auto-reload after `switch` when enabled below.\n",
        "\n",
        "[reload]\n",
        "primary_target = \"codex\"\n",
        "\n",
        "[switch]\n",
        "reload_after_switch = false\n",
        "\n",
        "[editor]\n",
        "# Optional editor command for `codex-switcher config edit`.\n",
        "# Examples: \"code --wait\", \"cursor --wait\", \"notepad\"\n",
        "# command = \"code --wait\"\n",
        "\n",
        "[codex_app]\n",
        "# Optional explicit path to standalone Codex.exe\n",
        "# path = \"C:\\\\Program Files\\\\WindowsApps\\\\OpenAI.Codex_...\\\\app\\\\Codex.exe\"\n",
        "# Optional explicit AppUserModelID for AppsFolder relaunch\n",
        "# app_user_model_id = \"OpenAI.Codex_xxxxx!App\"\n",
        ""
    )
    .to_string()
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
