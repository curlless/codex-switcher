use chrono::{DateTime, Local};
use colored::Colorize;
use directories::BaseDirs;
use inquire::{Confirm, MultiSelect, Select};
use rayon::prelude::*;
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt;
use std::fs;
use std::io::{self, IsTerminal as _};
use std::path::{Path, PathBuf};

use crate::switcher::ReloadAppTarget;
use crate::switcher::{
    CANCELLED_MESSAGE, format_action, format_entry_header, format_error, format_hint,
    format_list_hint, format_no_profiles, format_save_before_load, format_unsaved_warning,
    format_warning, inquire_select_render_config, is_inquire_cancel, is_plain, normalize_error,
    print_output_block, print_output_block_with_frame, style_text, terminal_width,
    use_color_stderr, use_color_stdout,
};
use crate::switcher::{
    Labels, ProfileIndexEntry, ProfilesIndex, assign_label, cached_profile_ids,
    collect_profile_ids, label_for_id, labels_from_index, load_profile_tokens_map_locked,
    pick_primary, profile_files, profile_id_from_path, profile_path_for_id, prune_labels,
    prune_profiles_index, read_profiles_index, read_profiles_index_relaxed, remove_labels_for_id,
    resolve_label_id, resolve_save_id, resolve_sync_id, sync_profiles_index,
    update_profiles_index_entry, usage_map_from_index, write_profiles_index,
};
use crate::switcher::{Paths, command_name, copy_atomic};
use crate::switcher::{
    Tokens, extract_email_and_plan, is_api_key_profile, is_free_plan, is_profile_ready,
    profile_error, read_tokens, read_tokens_opt, refresh_profile_tokens, token_account_id,
};
use crate::switcher::{
    UsageLock, UsageWindow, fetch_usage_details, fetch_usage_limits, format_last_used,
    format_usage_unavailable, lock_usage, now_seconds, ordered_profiles, read_base_url,
    start_usage_spinner, stop_usage_spinner, usage_unavailable,
};

const MAX_USAGE_CONCURRENCY: usize = 4;
const SCORE_7D_WEIGHT: i64 = 70;
const SCORE_5H_WEIGHT: i64 = 30;
const RESERVED_DISPLAY_MARKER: &str = " [reserved]";
#[path = "profiles_priority.rs"]
mod profile_priority;
#[cfg(all(test, feature = "switcher-unit-tests"))]
use profile_priority::{PriorityRow, PriorityState, PriorityUsage, priority_row_cmp};
pub use profile_priority::AvailabilityPayload;
use profile_priority::{
    best_ready_row, ordered_profiles_by_usage, priority_rows, render_priority_table,
};

pub fn save_profile(paths: &Paths, label: Option<String>) -> Result<(), String> {
    profile_load::save_profile(paths, label)
}

pub fn load_profile(paths: &Paths, label: Option<String>) -> Result<(), String> {
    profile_load::load_profile(paths, label)
}

pub fn switch_best_profile(
    paths: &Paths,
    dry_run: bool,
    reload_target: Option<ReloadAppTarget>,
) -> Result<(), String> {
    profile_switch::switch_best_profile(paths, dry_run, reload_target)
}

pub fn reload_app(paths: &Paths, dry_run: bool, target: ReloadAppTarget) -> Result<(), String> {
    profile_switch::reload_app(paths, dry_run, target)
}

pub fn reserve_profile(paths: &Paths, label: Option<String>) -> Result<(), String> {
    profile_reserve::reserve_profile(paths, label)
}

pub fn unreserve_profile(paths: &Paths, label: Option<String>) -> Result<(), String> {
    profile_reserve::unreserve_profile(paths, label)
}

pub fn migrate_profiles(
    paths: &Paths,
    from: Option<String>,
    overwrite: bool,
) -> Result<(), String> {
    profile_migrate::migrate_profiles(paths, from, overwrite)
}

pub fn delete_profile(paths: &Paths, yes: bool, label: Option<String>) -> Result<(), String> {
    profile_delete::delete_profile(paths, yes, label)
}

pub fn list_profiles(
    paths: &Paths,
    show_usage: bool,
    show_last_used: bool,
    allow_plain_spacing: bool,
    frame_with_separator: bool,
) -> Result<(), String> {
    profile_status::list_profiles(
        paths,
        show_usage,
        show_last_used,
        allow_plain_spacing,
        frame_with_separator,
    )
}

pub fn status_profiles(paths: &Paths, all: bool) -> Result<(), String> {
    profile_status::status_profiles(paths, all)
}

pub fn status_label(paths: &Paths, label: &str) -> Result<(), String> {
    profile_status::status_label(paths, label)
}

pub fn sync_current_readonly(paths: &Paths) -> Result<(), String> {
    profile_status::sync_current_readonly(paths)
}

pub fn profiles_overview(paths: &Paths) -> Result<ProfilesOverviewPayload, String> {
    profile_service::profiles_overview(paths)
}

pub fn active_profile_status(paths: &Paths) -> Result<ActiveProfileStatusPayload, String> {
    profile_service::active_profile_status(paths)
}

pub fn switch_preview(
    paths: &Paths,
    requested_profile: &str,
) -> Result<SwitchPreviewPayload, String> {
    profile_service::switch_preview(paths, requested_profile)
}

pub fn execute_best_switch(
    paths: &Paths,
    reload_target: Option<ReloadAppTarget>,
) -> Result<SwitchExecutionPayload, String> {
    profile_service::execute_best_switch(paths, reload_target)
}

pub fn execute_switch(
    paths: &Paths,
    requested_profile: &str,
    reload_target: Option<ReloadAppTarget>,
) -> Result<SwitchExecutionPayload, String> {
    profile_service::execute_switch(paths, requested_profile, reload_target)
}

pub fn inspect_reload_outcome(
    paths: &Paths,
    target: ReloadAppTarget,
) -> Result<ReloadOutcomePayload, String> {
    profile_service::inspect_reload_outcome(paths, target)
}

pub fn execute_reload_outcome(
    paths: &Paths,
    target: ReloadAppTarget,
) -> Result<ReloadOutcomePayload, String> {
    profile_service::execute_reload_outcome(paths, target)
}

#[path = "profiles_ui.rs"]
mod profile_ui;
use profile_ui::*;

#[path = "profiles_migrate.rs"]
mod profile_migrate;
#[cfg(all(test, feature = "switcher-unit-tests"))]
use profile_migrate::*;

#[path = "profiles_status.rs"]
mod profile_status;

#[path = "profiles_reserve.rs"]
mod profile_reserve;

#[path = "profiles_load.rs"]
mod profile_load;

#[path = "profiles_delete.rs"]
mod profile_delete;

#[path = "profiles_switch.rs"]
mod profile_switch;

#[path = "profiles_runtime.rs"]
mod profile_runtime;
use profile_runtime::*;

#[path = "profiles_service.rs"]
mod profile_service;
pub use profile_service::{
    ActiveProfileStatusPayload, ProfileCard, ProfilesOverviewPayload, ReloadOutcomePayload,
    SwitchExecutionPayload, SwitchPreviewPayload, SwitchProfilePayload,
};

#[cfg(all(test, feature = "switcher-unit-tests"))]
#[path = "profiles_tests.rs"]
mod tests;
