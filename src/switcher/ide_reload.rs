#[cfg(windows)]
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdeReloadOutcome {
    pub attempted: bool,
    pub restarted: bool,
    pub message: String,
}

pub fn reload_ide_best_effort() -> IdeReloadOutcome {
    #[cfg(windows)]
    {
        reload_windows()
    }
    #[cfg(not(windows))]
    {
        IdeReloadOutcome {
            attempted: false,
            restarted: false,
            message: "IDE auto-reload is only implemented for Windows in this build.".to_string(),
        }
    }
}

pub fn reload_ide_manual_hint() -> &'static str {
    "If account state does not update in Cursor, run Command Palette: Developer: Reload Window."
}

#[cfg(windows)]
fn reload_windows() -> IdeReloadOutcome {
    let process_names = ["codex-app-server.exe", "codex.exe"];
    let mut killed = Vec::new();
    let mut issues = Vec::new();
    let mut attempted = false;

    for process in process_names {
        attempted = true;
        let output = Command::new("taskkill")
            .args(["/IM", process, "/F"])
            .output();
        match output {
            Ok(output) => {
                if output.status.success() {
                    killed.push(process.to_string());
                    continue;
                }
                let stderr = String::from_utf8_lossy(&output.stderr).to_lowercase();
                if stderr.contains("not found") || stderr.contains("no running instance") {
                    continue;
                }
                issues.push(format!("{process}: taskkill exited with {}", output.status));
            }
            Err(err) => {
                issues.push(format!("{process}: failed to run taskkill ({err})"));
            }
        }
    }

    if !killed.is_empty() {
        return IdeReloadOutcome {
            attempted,
            restarted: true,
            message: format!("Reload hint: terminated {}.", killed.join(", ")),
        };
    }

    if !issues.is_empty() {
        return IdeReloadOutcome {
            attempted,
            restarted: false,
            message: format!("Reload hint: {}", issues.join("; ")),
        };
    }

    IdeReloadOutcome {
        attempted,
        restarted: false,
        message: "Reload hint: no matching IDE/Codex processes were running.".to_string(),
    }
}
