mod commands;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            commands::desktop_profiles_overview,
            commands::desktop_active_profile_status,
            commands::desktop_switch_preview,
            commands::desktop_switch_execute,
            commands::desktop_switch_best_execute,
            commands::desktop_smoke_mode,
            commands::desktop_record_smoke_trace,
            commands::desktop_reload_targets,
            commands::desktop_reload_target
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Codex Switcher Desktop");
}
