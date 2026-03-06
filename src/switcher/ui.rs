use colored::Colorize;
use inquire::ui::{Color, RenderConfig, StyleSheet, Styled};
use std::sync::atomic::{AtomicBool, Ordering};
use supports_color::Stream;

use crate::switcher::has_auth;
use crate::switcher::{Paths, command_name};

static PLAIN: AtomicBool = AtomicBool::new(false);

pub const CANCELLED_MESSAGE: &str = "Cancelled.";

pub fn set_plain(value: bool) {
    PLAIN.store(value, Ordering::Relaxed);
}

pub fn is_plain() -> bool {
    PLAIN.load(Ordering::Relaxed)
}

pub fn use_color_stdout() -> bool {
    supports_color(Stream::Stdout)
}

pub fn use_color_stderr() -> bool {
    supports_color(Stream::Stderr)
}

pub fn use_tty_stderr() -> bool {
    use_color_stderr()
}

pub fn terminal_width() -> Option<usize> {
    if is_plain() {
        return None;
    }
    std::env::var("COLUMNS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
}

fn supports_color(stream: Stream) -> bool {
    if is_plain() {
        return false;
    }
    if std::env::var_os("NO_COLOR").is_some() {
        return false;
    }
    supports_color::on(stream).is_some()
}

pub fn style_text<F>(text: &str, use_color: bool, style: F) -> String
where
    F: FnOnce(colored::ColoredString) -> colored::ColoredString,
{
    if use_color && !is_plain() {
        style(text.normal()).to_string()
    } else {
        text.to_string()
    }
}

pub fn format_cmd(command: &str, use_color: bool) -> String {
    let text = format!("`{command}`");
    style_text(&text, use_color, |text| text.yellow().bold())
}

pub fn format_action(message: &str, use_color: bool) -> String {
    let text = format!("✅ {message}");
    style_text(&text, use_color, |text| text.green().bold())
}

pub fn format_warning(message: &str, use_color: bool) -> String {
    let text = if is_plain() {
        format!("WARNING: {message}")
    } else {
        format!("Warning: {message}")
    };
    style_text(&text, use_color, |text| text.yellow().dimmed().italic())
}

pub fn format_cancel(use_color: bool) -> String {
    style_text(CANCELLED_MESSAGE, use_color, |text| text.dimmed().italic())
}

pub fn format_hint(message: &str, use_color: bool) -> String {
    if is_plain() {
        format!("INFO: {message}")
    } else {
        let message = format!("\n\n{message}");
        style_text(&message, use_color, |text| text.italic())
    }
}

pub fn format_no_profiles(paths: &Paths, use_color: bool) -> String {
    let hint = format_save_hint(
        paths,
        use_color,
        "Run {save} to save this profile.",
        "Run {login} • then {save}.",
    );
    format!("No saved profiles. {hint}")
}

pub fn format_save_before_load(paths: &Paths, use_color: bool) -> String {
    format_save_hint(
        paths,
        use_color,
        "Run {save} before loading.",
        "Run {login}, then {save} before loading.",
    )
}

pub fn format_unsaved_warning(use_color: bool) -> Vec<String> {
    let warning = "WARNING: This profile is not saved yet.";
    let save_line = format!(
        "Run {} to save this profile.",
        format_command("save", false)
    );
    if !use_color {
        return vec![warning.to_string(), save_line];
    }
    vec![
        style_text(warning, use_color, |text| text.yellow().dimmed().italic()),
        style_text(&save_line, use_color, |text| text.dimmed().italic()),
    ]
}

pub fn format_list_hint(use_color: bool) -> String {
    let list = format_command("list", use_color);
    format_hint(&format!("Run {list} to see saved profiles."), use_color)
}

pub fn normalize_error(message: &str) -> String {
    let message = message.strip_prefix("Error: ").unwrap_or(message);
    if message.contains("codex login") {
        if message.contains("not found") {
            return "Not logged in. Run `codex login`.".to_string();
        }
        if message.contains("invalid JSON") {
            return "Auth file is invalid. Run `codex login`.".to_string();
        }
        return "Auth is incomplete. Run `codex login`.".to_string();
    }
    message.to_string()
}

pub fn format_error(message: &str) -> String {
    let normalized = normalize_error(message);
    let prefix = if use_color_stdout() {
        "Error:".red().bold().blink().to_string()
    } else {
        "Error:".to_string()
    };
    format!("{prefix} {normalized}")
}

pub fn format_profile_display(
    email: Option<String>,
    plan: Option<String>,
    label: Option<String>,
    is_current: bool,
    use_color: bool,
) -> String {
    let label = label.as_deref();
    if email
        .as_deref()
        .map(|value| value.eq_ignore_ascii_case("Key"))
        .unwrap_or(false)
        && plan
            .as_deref()
            .map(|value| value.eq_ignore_ascii_case("Key"))
            .unwrap_or(false)
    {
        let badge = format_plan_badge("Key", is_current, use_color);
        let label_suffix = format_label(label, use_color);
        return format!("{badge}{label_suffix}");
    }
    let label_suffix = format_label(label, use_color);
    match email {
        Some(email) => {
            let plan = plan.unwrap_or_else(|| "Unknown".to_string());
            let plan_is_free = crate::switcher::is_free_plan(Some(&plan));
            let badge = format_plan_badge(&plan, is_current, use_color);
            if use_color {
                let email_badge = format_email_badge(&email, plan_is_free, is_current);
                format!("{badge}{email_badge}{label_suffix}")
            } else {
                format!("{badge} {email}{label_suffix}")
            }
        }
        None => format!("Unknown profile{label_suffix}"),
    }
}

pub fn format_entry_header(
    display: &str,
    last_used: &str,
    is_current: bool,
    use_color: bool,
) -> String {
    let mut base = if use_color {
        display.bold().to_string()
    } else {
        display.to_string()
    };
    if !is_current && !last_used.is_empty() && !last_used.eq_ignore_ascii_case("unknown") {
        base.push_str(&format_last_used_badge(last_used, use_color));
    }
    base
}

fn format_plan_badge(plan: &str, is_current: bool, use_color: bool) -> String {
    let plan_upper = plan.to_uppercase();
    let text = format!(" {} ", plan_upper);
    let plan_is_free = crate::switcher::is_free_plan(Some(plan));
    if use_color {
        if plan_is_free {
            text.white().on_bright_red().bold().to_string()
        } else if is_current {
            text.white().on_bright_green().bold().to_string()
        } else {
            text.white().on_bright_magenta().bold().to_string()
        }
    } else {
        format!("[{plan_upper}]")
    }
}

fn format_last_used_badge(last_used: &str, use_color: bool) -> String {
    if use_color {
        let text = format!(" {last_used}");
        style_text(&text, use_color, |text| text.dimmed().italic())
    } else {
        format!(" ({last_used})")
    }
}

fn format_label(label: Option<&str>, use_color: bool) -> String {
    match label {
        Some(value) if use_color => format!(" {value} ").white().on_bright_black().to_string(),
        Some(value) => format!(" ({value})"),
        None => String::new(),
    }
}

fn format_email_badge(email: &str, plan_is_free: bool, is_current: bool) -> String {
    if plan_is_free {
        format!(" {email} ").white().on_red().to_string()
    } else if is_current {
        format!(" {email} ").white().on_green().to_string()
    } else {
        format!(" {email} ").white().on_magenta().to_string()
    }
}

pub fn inquire_select_render_config() -> RenderConfig<'static> {
    let mut config = if use_color_stderr() {
        let mut config = RenderConfig::default_colored();
        config.help_message = StyleSheet::new().with_fg(Color::DarkGrey);
        config
    } else {
        RenderConfig::empty()
    };
    config.prompt_prefix = Styled::new("");
    config.answered_prompt_prefix = Styled::new("");
    config
}

pub fn is_inquire_cancel(err: &inquire::error::InquireError) -> bool {
    matches!(
        err,
        inquire::error::InquireError::OperationCanceled
            | inquire::error::InquireError::OperationInterrupted
    )
}

const OUTPUT_INDENT: &str = " ";

pub fn print_output_block(message: &str) {
    let message = if is_plain() {
        message.to_string()
    } else {
        indent_output(message)
    };
    println!("\n{message}\n");
}

pub fn print_output_block_with_frame(message: &str, separator: &str) {
    if is_plain() {
        print_output_block(message);
        return;
    }
    let message = indent_output(message);
    let separator = indent_output(separator);
    println!("\n{separator}\n{message}\n{separator}\n");
}

fn indent_output(message: &str) -> String {
    message
        .lines()
        .map(|line| {
            if line.is_empty() {
                String::new()
            } else {
                format!("{OUTPUT_INDENT}{line}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_command(cmd: &str, use_color: bool) -> String {
    let name = command_name();
    let full = if cmd.is_empty() {
        name.to_string()
    } else {
        format!("{name} {cmd}")
    };
    format_cmd(&full, use_color)
}

fn format_save_hint(paths: &Paths, use_color: bool, save_only: &str, with_login: &str) -> String {
    let save = format_command("save", use_color);
    let message = if has_auth(&paths.auth) {
        save_only.replace("{save}", &save)
    } else {
        let login = format_cmd("codex login", use_color);
        with_login
            .replace("{login}", &login)
            .replace("{save}", &save)
    };
    format_hint(&message, use_color)
}

#[cfg(all(test, feature = "switcher-unit-tests"))]
mod tests {
    use super::*;
    use crate::switcher::test_utils::{make_paths, set_env_guard, set_plain_guard};
    use std::fs;

    #[test]
    fn plain_toggle_affects_output() {
        {
            let _plain = set_plain_guard(true);
            assert!(is_plain());
            let warning = format_warning("oops", false);
            assert!(warning.contains("WARNING"));
        }
        assert!(!is_plain());
    }

    #[test]
    fn terminal_width_parses_columns() {
        let _plain = set_plain_guard(false);
        let _env = set_env_guard("COLUMNS", Some("80"));
        assert_eq!(terminal_width(), Some(80));
    }

    #[test]
    fn terminal_width_none_when_plain() {
        let _plain = set_plain_guard(true);
        assert_eq!(terminal_width(), None);
    }

    #[test]
    fn supports_color_respects_no_color() {
        let _env = set_env_guard("NO_COLOR", Some("1"));
        assert!(!use_color_stdout());
        assert!(!use_color_stderr());
    }

    #[test]
    fn format_helpers_basic() {
        let _plain = set_plain_guard(false);
        let cmd = format_cmd("codex login", false);
        assert!(cmd.contains("codex login"));
        let action = format_action("done", false);
        assert!(action.contains("done"));
        let hint = format_hint("hint", false);
        assert!(hint.contains("hint"));
        let cancel = format_cancel(false);
        assert_eq!(cancel, CANCELLED_MESSAGE);
    }

    #[test]
    fn format_no_profiles_and_save_before_load() {
        let dir = tempfile::tempdir().expect("tempdir");
        let paths = make_paths(dir.path());
        let msg = format_no_profiles(&paths, false);
        assert!(msg.contains("No saved profiles"));
        let msg = format_save_before_load(&paths, false);
        assert!(msg.contains("save"));
    }

    #[test]
    fn format_unsaved_warning_plain() {
        let lines = format_unsaved_warning(false);
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("WARNING"));
    }

    #[test]
    fn normalize_error_variants() {
        assert_eq!(
            normalize_error("Error: Codex auth file not found. Run `codex login` first."),
            "Not logged in. Run `codex login`."
        );
        assert_eq!(
            normalize_error(
                "Error: invalid JSON in auth.json: oops. Run `codex login` to regenerate it."
            ),
            "Auth file is invalid. Run `codex login`."
        );
        assert_eq!(
            normalize_error(
                "Error: auth.json is missing tokens.account_id. Run `codex login` to reauthenticate."
            ),
            "Auth is incomplete. Run `codex login`."
        );
        assert_eq!(normalize_error("other"), "other");
    }

    #[test]
    fn format_error_plain() {
        let _env = set_env_guard("NO_COLOR", Some("1"));
        let err = format_error("oops");
        assert!(err.contains("Error:"));
    }

    #[test]
    fn format_profile_display_variants() {
        let key = format_profile_display(
            Some("Key".to_string()),
            Some("Key".to_string()),
            Some("label".to_string()),
            false,
            false,
        );
        assert!(key.to_lowercase().contains("key"));
        let display = format_profile_display(
            Some("me@example.com".to_string()),
            Some("Free".to_string()),
            None,
            true,
            false,
        );
        assert!(display.contains("me@example.com"));
        let unknown = format_profile_display(None, None, None, false, false);
        assert!(unknown.contains("Unknown"));
    }

    #[test]
    fn format_entry_header_and_separator() {
        let header = format_entry_header("Display", "1d", false, false);
        assert!(header.contains("Display"));
        let indented = super::indent_output("line\n\nline2");
        assert!(indented.contains("line2"));
    }

    #[test]
    fn render_config_and_cancel() {
        let _env = set_env_guard("NO_COLOR", Some("1"));
        let config = inquire_select_render_config();
        assert_eq!(config.prompt_prefix.content, "");
        let err = inquire::error::InquireError::OperationCanceled;
        assert!(is_inquire_cancel(&err));
    }

    #[test]
    fn print_output_blocks() {
        let _plain = set_plain_guard(true);
        print_output_block("hi");
        print_output_block_with_frame("hi", "-");
    }

    #[test]
    fn format_command_uses_name() {
        let cmd = super::format_command("list", false);
        assert!(cmd.contains("list"));
    }

    #[test]
    fn format_save_hint_with_auth() {
        let dir = tempfile::tempdir().expect("tempdir");
        let paths = make_paths(dir.path());
        fs::write(&paths.auth, "{}").expect("write auth");
        let hint = super::format_save_hint(&paths, false, "Run {save}", "Run {login} {save}");
        assert!(hint.contains("save"));
    }
}
