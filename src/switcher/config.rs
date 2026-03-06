use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::process::Command as ProcessCommand;

use crate::switcher::{
    Paths, ReloadAppTarget, format_action, print_output_block, use_color_stdout, write_atomic,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SwitcherConfig {
    pub reload: ReloadConfig,
    pub switch: SwitchConfig,
    pub editor: EditorConfig,
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

impl Default for SwitcherConfig {
    fn default() -> Self {
        Self {
            reload: ReloadConfig::default(),
            switch: SwitchConfig::default(),
            editor: EditorConfig::default(),
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
        ""
    )
    .to_string()
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
