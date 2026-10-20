#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod photocraft;

use thai_font_kit_core::{Catalog, Manager, Outcome, Status};

#[tauri::command]
fn catalog() -> Result<Catalog, String> {
    thai_font_kit_core::catalog()
}

#[tauri::command]
async fn font_status() -> Result<Vec<Status>, String> {
    tauri::async_runtime::spawn_blocking(|| Manager::open()?.status())
        .await
        .map_err(|e| e.to_string())?
}
#[tauri::command]
async fn preview_font(id: String) -> Result<tauri::ipc::Response, String> {
    tauri::async_runtime::spawn_blocking(move || {
        Manager::open()?
            .font_bytes(&id)
            .map(tauri::ipc::Response::new)
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
async fn install_font(
    id: String,
    on_progress: tauri::ipc::Channel<serde_json::Value>,
) -> Result<Outcome, String> {
    tauri::async_runtime::spawn_blocking(move || {
        Manager::open()?.install_with_progress(&id, |phase, done, total| {
            let _ =
                on_progress.send(serde_json::json!({"phase": phase, "done": done, "total": total}));
        })
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
async fn uninstall_font(id: String) -> Result<Outcome, String> {
    tauri::async_runtime::spawn_blocking(move || Manager::open()?.uninstall(&id))
        .await
        .map_err(|e| e.to_string())?
}
#[tauri::command]
fn companion_mode(window: tauri::WebviewWindow, enabled: bool) -> Result<(), String> {
    let (width, height) = if enabled {
        (460.0, 820.0)
    } else {
        (1140.0, 800.0)
    };
    window
        .set_min_size(Some(tauri::LogicalSize::new(
            if enabled { 420.0 } else { 780.0 },
            600.0,
        )))
        .map_err(|e| e.to_string())?;
    window
        .set_size(tauri::LogicalSize::new(width, height))
        .map_err(|e| e.to_string())?;
    window.set_always_on_top(enabled).map_err(|e| e.to_string())
}
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            photocraft::photocraft_open,
            photocraft::photocraft_check,
            photocraft::photocraft_apply,
            companion_mode,
            catalog,
            font_status,
            preview_font,
            install_font,
            uninstall_font
        ])
        .run(tauri::generate_context!())
        .expect("Unable to start Thai Font Kit");
}
