use crate::{InstallSource, format_cmd};

pub fn ensure_codex_cli(source: InstallSource) -> Result<(), String> {
    if cfg!(debug_assertions) {
        return Ok(());
    }
    if command_exists("codex") {
        return Ok(());
    }
    let install_cmd = format_cmd(&install_command(source), false);
    Err(format!(
        "Error: Codex CLI not found. Install it with {install_cmd}."
    ))
}

fn install_command(source: InstallSource) -> String {
    match source {
        InstallSource::Npm => "npm install -g @openai/codex".to_string(),
        InstallSource::Bun => "bun install -g @openai/codex".to_string(),
        InstallSource::Brew => "brew install --cask codex".to_string(),
        InstallSource::Unknown => platform_default_install_command(),
    }
}

fn command_exists(command: &str) -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    let candidates = command_candidates(command);
    for dir in std::env::split_paths(&path) {
        for candidate in &candidates {
            if dir.join(candidate).is_file() {
                return true;
            }
        }
    }
    false
}

fn command_candidates(command: &str) -> Vec<String> {
    if cfg!(windows) {
        let mut candidates = Vec::new();
        let path = std::path::Path::new(command);
        if path.extension().is_some() {
            candidates.push(command.to_string());
            return candidates;
        }
        let pathext = std::env::var_os("PATHEXT")
            .and_then(|value| value.into_string().ok())
            .unwrap_or_else(|| ".EXE;.CMD;.BAT;.COM".to_string());
        for ext in pathext.split(';').filter(|ext| !ext.is_empty()) {
            candidates.push(format!("{command}{ext}"));
        }
        candidates
    } else {
        vec![command.to_string()]
    }
}

fn platform_default_install_command() -> String {
    if cfg!(windows) {
        "winget install OpenAI.Codex".to_string()
    } else if cfg!(target_os = "macos") {
        "brew install --cask codex or npm install -g @openai/codex".to_string()
    } else {
        "npm install -g @openai/codex".to_string()
    }
}
