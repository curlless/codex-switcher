use colored::Colorize;
use inquire::ui::{Color, RenderConfig, StyleSheet, Styled};
use std::sync::atomic::{AtomicBool, Ordering};
use supports_color::Stream;

use crate::has_auth;
use crate::{Paths, command_name};

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
            let plan_is_free = crate::is_free_plan(Some(&plan));
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
    if !is_current && !last_used.is_empty() {
        base.push_str(&format_last_used_badge(last_used, use_color));
    }
    base
}

fn format_plan_badge(plan: &str, is_current: bool, use_color: bool) -> String {
    let plan_upper = plan.to_uppercase();
    let text = format!(" {} ", plan_upper);
    let plan_is_free = crate::is_free_plan(Some(plan));
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
