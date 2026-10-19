#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

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
async fn preview_font(id: String) -> Result<Vec<u8>, String> {
    tauri::async_runtime::spawn_blocking(move || Manager::open()?.font_bytes(&id))
        .await
        .map_err(|e| e.to_string())?
}
#[tauri::command]
async fn install_font(id: String) -> Result<Outcome, String> {
    tauri::async_runtime::spawn_blocking(move || Manager::open()?.install(&id))
        .await
        .map_err(|e| e.to_string())?
}
#[tauri::command]
async fn uninstall_font(id: String) -> Result<Outcome, String> {
    tauri::async_runtime::spawn_blocking(move || Manager::open()?.uninstall(&id))
        .await
        .map_err(|e| e.to_string())?
}
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            catalog,
            font_status,
            preview_font,
            install_font,
            uninstall_font
        ])
        .run(tauri::generate_context!())
        .expect("Unable to start Thai Font Kit");
}
