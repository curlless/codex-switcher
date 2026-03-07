use crate::switcher::{
    Paths, ReloadAppTarget, codex_app_override, ensure_codex_app_override, format_action,
    format_hint, format_no_profiles, format_warning, inspect_ide_reload_target_with_codex_override,
    print_output_block, reload_ide_target_best_effort_with_codex_override, use_color_stdout,
};

use super::{
    best_ready_row, current_saved_id, load_snapshot, priority_rows, profile_load,
    render_priority_table,
};

const CURSOR_PROTOCOL_HELPER_HINT: &str = "Cursor automation: install the Commands Executor extension (ionutvmi.vscode-commands-executor) to enable protocol-based Reload Window.";

pub(super) fn switch_best_profile(
    paths: &Paths,
    dry_run: bool,
    reload_target: Option<ReloadAppTarget>,
) -> Result<(), String> {
    let use_color = use_color_stdout();
    let no_profiles = format_no_profiles(paths, use_color);
    let snapshot = load_snapshot(paths, false)?;
    if snapshot.usage_map.is_empty() {
        print_output_block(&no_profiles);
        return Ok(());
    }
    let current_saved = current_saved_id(paths, &snapshot.usage_map, &snapshot.tokens);
    let rows = priority_rows(paths, &snapshot, current_saved.as_deref(), false);
    if rows.is_empty() {
        print_output_block(&no_profiles);
        return Ok(());
    }
    let table = render_priority_table(&rows, use_color);
    print_output_block(&table);

    let Some(best) = best_ready_row(&rows) else {
        let hint = format_hint(
            "No switch performed because usage data is unavailable for all profiles.",
            use_color,
        );
        return Err(format!(
            "Error: no eligible profile found for auto-switch.{hint}"
        ));
    };

    if dry_run {
        let message = format_action(
            &format!("Dry run: best profile is {}", best.profile_name),
            use_color,
        );
        print_output_block(&message);
        return Ok(());
    }

    profile_load::load_profile_by_id(paths, &best.id, &best.profile_name)?;

    if let Some(reload_target) = reload_target {
        let codex_override = codex_override_for_reload_target(paths, reload_target)?;
        let outcome = reload_ide_target_best_effort_with_codex_override(
            reload_target,
            codex_override.as_ref(),
        );
        print_reload_outcome(
            outcome.message,
            outcome.restarted,
            outcome.manual_hints,
            reload_target,
        );
    }

    Ok(())
}

pub(super) fn reload_app(
    paths: &Paths,
    dry_run: bool,
    target: ReloadAppTarget,
) -> Result<(), String> {
    let codex_override = codex_override_for_reload_target(paths, target)?;
    let outcome = if dry_run {
        inspect_ide_reload_target_with_codex_override(target, codex_override.as_ref())
    } else {
        reload_ide_target_best_effort_with_codex_override(target, codex_override.as_ref())
    };
    print_reload_outcome(
        outcome.message,
        outcome.restarted,
        outcome.manual_hints,
        target,
    );
    Ok(())
}

pub(super) fn codex_override_for_reload_target(
    paths: &Paths,
    target: ReloadAppTarget,
) -> Result<Option<crate::switcher::CodexAppOverride>, String> {
    if matches!(target, ReloadAppTarget::All | ReloadAppTarget::Codex) {
        ensure_codex_app_override(paths)
    } else {
        codex_app_override(paths)
    }
}

fn print_reload_outcome(
    message: String,
    restarted: bool,
    mut manual_hints: Vec<String>,
    target: ReloadAppTarget,
) {
    let use_color = use_color_stdout();
    let mut lines = Vec::new();
    if matches!(target, ReloadAppTarget::All | ReloadAppTarget::Cursor)
        && !message.contains("protocol reload is available")
        && !manual_hints
            .iter()
            .any(|hint| hint.contains("ionutvmi.vscode-commands-executor"))
    {
        manual_hints.push(CURSOR_PROTOCOL_HELPER_HINT.to_string());
    }
    if restarted {
        lines.push(format_action(&message, use_color));
    } else {
        lines.push(format_warning(&message, use_color));
    }
    for hint in manual_hints {
        lines.push(format_hint(&hint, use_color));
    }
    print_output_block(&lines.join("\n"));
}
