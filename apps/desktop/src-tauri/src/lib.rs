mod commands;

use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use tauri::{AppHandle, LogicalSize, Manager, Runtime, WebviewWindow, Window, WindowEvent};

const WINDOW_STATE_FILENAME: &str = "window-state.json";
const MIN_WINDOW_WIDTH: f64 = 800.0;
const MIN_WINDOW_HEIGHT: f64 = 600.0;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
struct PersistedWindowState {
    width: f64,
    height: f64,
}

fn clamp_window_state(state: PersistedWindowState) -> PersistedWindowState {
    PersistedWindowState {
        width: state.width.max(MIN_WINDOW_WIDTH),
        height: state.height.max(MIN_WINDOW_HEIGHT),
    }
}

fn window_state_path<R: Runtime>(app_handle: &AppHandle<R>) -> Option<PathBuf> {
    app_handle
        .path()
        .app_config_dir()
        .ok()
        .map(|dir| dir.join(WINDOW_STATE_FILENAME))
}

fn load_window_state<R: Runtime>(app_handle: &AppHandle<R>) -> Option<PersistedWindowState> {
    let path = window_state_path(app_handle)?;
    let payload = fs::read_to_string(path).ok()?;
    let state = serde_json::from_str::<PersistedWindowState>(&payload).ok()?;
    Some(clamp_window_state(state))
}

fn save_window_state<R: Runtime>(window: &Window<R>) {
    let Ok(is_maximized) = window.is_maximized() else {
        return;
    };
    if is_maximized {
        return;
    }

    let Ok(scale_factor) = window.scale_factor() else {
        return;
    };
    let Ok(size) = window.outer_size() else {
        return;
    };
    let logical = size.to_logical::<f64>(scale_factor);
    let state = clamp_window_state(PersistedWindowState {
        width: logical.width,
        height: logical.height,
    });
    let Some(path) = window_state_path(&window.app_handle()) else {
        return;
    };

    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    if let Ok(payload) = serde_json::to_vec_pretty(&state) {
        let _ = fs::write(path, payload);
    }
}

fn restore_main_window_state<R: Runtime>(window: &WebviewWindow<R>) {
    let Some(state) = load_window_state(&window.app_handle()) else {
        return;
    };

    let _ = window.unmaximize();
    let _ = window.set_size(LogicalSize::new(state.width, state.height));
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if let Some(window) = app.handle().get_webview_window("main") {
                restore_main_window_state(&window);
            }
            Ok(())
        })
        .on_window_event(|window, event| match event {
            WindowEvent::Resized(_) | WindowEvent::Destroyed | WindowEvent::CloseRequested { .. } => {
                save_window_state(window);
            }
            _ => {}
        })
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
