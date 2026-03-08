use crate::switcher::{
    Paths, ReloadAppTarget, codex_app_override, ensure_codex_app_override, format_action,
    format_hint, format_no_profiles, format_warning, print_output_block, use_color_stdout,
};

use super::{ReloadOutcomePayload, profile_service, render_priority_table};

pub(super) fn switch_best_profile(
    paths: &Paths,
    dry_run: bool,
    reload_target: Option<ReloadAppTarget>,
) -> Result<(), String> {
    let use_color = use_color_stdout();
    let no_profiles = format_no_profiles(paths, use_color);
    let plan = profile_service::best_switch_plan(paths)?;
    if plan.rows.is_empty() {
        print_output_block(&no_profiles);
        return Ok(());
    }
    let table = render_priority_table(&plan.rows, use_color);
    print_output_block(&table);

    let Some(best_display) = plan.selected_display.as_deref() else {
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
            &format!("Dry run: best profile is {best_display}"),
            use_color,
        );
        print_output_block(&message);
        return Ok(());
    }

    let outcome = profile_service::execute_best_switch(paths, reload_target)?;
    if let Some(reload) = outcome.reload {
        print_reload_outcome(&reload);
    }

    Ok(())
}

pub(super) fn reload_app(
    paths: &Paths,
    dry_run: bool,
    target: ReloadAppTarget,
) -> Result<(), String> {
    let outcome = if dry_run {
        profile_service::inspect_reload_outcome(paths, target)?
    } else {
        profile_service::execute_reload_outcome(paths, target)?
    };
    print_reload_outcome(&outcome);
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

fn print_reload_outcome(outcome: &ReloadOutcomePayload) {
    let use_color = use_color_stdout();
    let mut lines = Vec::new();
    if outcome.restarted {
        lines.push(format_action(&outcome.message, use_color));
    } else {
        lines.push(format_warning(&outcome.message, use_color));
    }
    for hint in &outcome.manual_hints {
        lines.push(format_hint(hint, use_color));
    }
    print_output_block(&lines.join("\n"));
}
