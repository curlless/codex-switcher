mod auth;
mod cli;
mod cli_runtime;
mod common;
mod config;
mod ide_reload;
mod profile_identity;
mod profile_store;
mod profiles;
mod relay;
mod requirements;
#[cfg(all(test, feature = "switcher-unit-tests"))]
mod test_utils;
mod ui;
mod updates;
mod usage;

pub use auth::{
    AuthFile, Tokens, extract_email_and_plan, format_plan, has_auth, is_api_key_profile,
    is_free_plan, is_profile_ready, profile_error, read_auth_file, read_tokens, read_tokens_opt,
    refresh_profile_tokens, require_identity, token_account_id, tokens_from_api_key,
};
pub use cli_runtime::run_cli;
pub use common::{
    Paths, command_name, copy_atomic, ensure_paths, package_command_name, resolve_paths,
    write_atomic, write_atomic_with_mode,
};
pub use config::{
    CodexAppConfig, EditorConfig, ReloadConfig, SwitchConfig, SwitcherConfig, codex_app_override,
    detect_codex_app, edit_config, effective_reload_target, ensure_codex_app_override,
    load_switcher_config, show_config, switch_reload_target,
};
pub use ide_reload::{
    CodexAppDiscovery, CodexAppOverride, IdeReloadOutcome, ReloadAppTarget,
    detect_codex_app_discovery, inspect_ide_reload, inspect_ide_reload_target,
    inspect_ide_reload_target_with_codex_override, reload_ide_best_effort,
    reload_ide_target_best_effort, reload_ide_target_best_effort_with_codex_override,
};
pub(crate) use profile_identity::{
    cached_profile_ids, pick_primary, resolve_save_id, resolve_sync_id,
};
pub use profile_store::{
    Labels, assign_label, collect_profile_ids, label_for_id, load_profile_tokens_map,
    profile_files, profile_id_from_path, profile_path_for_id, prune_labels, read_labels,
    remove_labels_for_id, resolve_label_id, write_labels,
};
pub(crate) use profile_store::{
    ProfileIndexEntry, ProfilesIndex, UpdateCache, labels_from_index,
    load_profile_tokens_map_locked, prune_profiles_index, read_profiles_index,
    read_profiles_index_relaxed, sync_profiles_index, update_profiles_index_entry,
    usage_map_from_index, write_profiles_index,
};
pub use profiles::{
    ActiveProfileStatusPayload, ProfileCard, ProfilesOverviewPayload, ReloadOutcomePayload,
    SwitchExecutionPayload, SwitchPreviewPayload, SwitchProfilePayload, active_profile_status,
    delete_profile, execute_best_switch, execute_reload_outcome, execute_switch,
    inspect_reload_outcome, list_profiles, load_profile, migrate_profiles, profiles_overview,
    reload_app, reserve_profile, save_profile, status_label, status_profiles, switch_best_profile,
    switch_preview, sync_current_readonly, unreserve_profile,
};
pub use relay::relay_login;
pub use requirements::ensure_codex_cli;
pub use ui::{
    CANCELLED_MESSAGE, format_action, format_cancel, format_cmd, format_entry_header, format_error,
    format_hint, format_list_hint, format_no_profiles, format_profile_display,
    format_save_before_load, format_unsaved_warning, format_warning, inquire_select_render_config,
    is_inquire_cancel, is_plain, normalize_error, print_output_block,
    print_output_block_with_frame, set_plain, style_text, terminal_width, use_color_stderr,
    use_color_stdout, use_tty_stderr,
};
pub use updates::{
    InstallSource, UpdateAction, UpdateConfig, UpdatePromptOutcome, detect_install_source,
    detect_install_source_inner, dismiss_version, extract_version_from_cask,
    extract_version_from_latest_tag, get_upgrade_version, get_upgrade_version_for_popup, is_newer,
    run_update_prompt_if_needed,
};
pub use usage::{
    UsageFetchError, UsageLock, fetch_usage_details, format_last_used, format_usage_unavailable,
    lock_usage, normalize_usage, now_seconds, ordered_profiles, parse_config_value, read_base_url,
    usage_unavailable,
};
pub(crate) use usage::{UsageWindow, fetch_usage_limits, start_usage_spinner, stop_usage_spinner};
