mod commands;

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::desktop_profiles_overview,
            commands::desktop_active_profile_status,
            commands::desktop_switch_preview,
            commands::desktop_reload_targets,
            commands::desktop_reload_target
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Codex Switcher Desktop");
}
