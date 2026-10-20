use std::{fs, path::PathBuf, process::Command};
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;
use thai_font_kit_core::photocraft::Bridge;

fn data(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_local_data_dir()
        .map_err(|e| e.to_string())?
        .join("photocraft-bridge");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}
fn connect(app: &tauri::AppHandle) -> Result<Bridge, String> {
    let token = data(app)?.join("control.token");
    if fs::symlink_metadata(&token)
        .map(|m| !m.is_file() || m.file_type().is_symlink() || m.len() > 256)
        .unwrap_or(true)
    {
        return Err("กรุณาเปิด PhotoCraft ผ่านปุ่มในแอปก่อน".into());
    }
    Bridge::connect(
        fs::read_to_string(token)
            .map_err(|_| "อ่านข้อมูลเชื่อมต่อไม่ได้")?
            .trim(),
    )
}
#[tauri::command]
pub async fn photocraft_open(app: tauri::AppHandle, select: bool) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let dir = data(&app)?;
        let config = dir.join("executable.json");
        let saved = if select {
            None
        } else {
            fs::read(&config)
                .ok()
                .and_then(|b| serde_json::from_slice::<PathBuf>(&b).ok())
        };
        let mut path = match saved {
            Some(path) => path,
            None => app
                .dialog()
                .file()
                .set_title("เลือกโปรแกรม PhotoCraft ที่ติดตั้งไว้")
                .blocking_pick_file()
                .ok_or("ยกเลิกการเลือก PhotoCraft")?
                .into_path()
                .map_err(|_| "ตำแหน่งโปรแกรมไม่รองรับ")?,
        };
        #[cfg(target_os = "macos")]
        if path.extension().is_some_and(|e| e == "app") {
            path = ["photocraft", "PhotoCraft"]
                .iter()
                .map(|n| path.join("Contents/MacOS").join(n))
                .find(|p| p.is_file())
                .ok_or("ไม่พบโปรแกรมใน PhotoCraft.app")?;
        }
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase();
        #[cfg(target_os = "windows")]
        let valid = name == "photocraft.exe";
        #[cfg(not(target_os = "windows"))]
        let valid =
            name == "photocraft" || (name.contains("photocraft") && name.ends_with(".appimage"));
        if !path.is_absolute() || !path.is_file() || !valid {
            return Err(
                "กรุณาเลือกไฟล์โปรแกรม PhotoCraft (.exe / .app / AppImage หรือ photocraft)".into(),
            );
        }
        let token = dir.join("control.token");
        if fs::symlink_metadata(&token).is_ok_and(|m| m.file_type().is_symlink()) {
            return Err("ตำแหน่งข้อมูลเชื่อมต่อไม่ปลอดภัย".into());
        }
        if connect(&app).and_then(|mut b| b.check()).is_ok() {
            return Ok(());
        }
        let mut command = Command::new(&path);
        command
            .args(["--control", "17878", "--control-token-file"])
            .arg(token);
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000);
        }
        let mut child = command
            .spawn()
            .map_err(|_| "เปิด PhotoCraft ไม่สำเร็จ กรุณาตรวจตำแหน่งโปรแกรม")?;
        fs::write(
            config,
            serde_json::to_vec(&path).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        // Reap only the process we launched; no service and no force-close of the editor.
        std::thread::spawn(move || {
            let _ = child.wait();
        });
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
        while std::time::Instant::now() < deadline {
            if connect(&app).and_then(|mut b| b.check()).is_ok() {
                return Ok(());
            }
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
        Err(
            "PhotoCraft อาจยังเปิดไม่เสร็จ กดเชื่อมต่ออีกครั้ง หากเปิดรุ่นเดิมอยู่ ให้บันทึกงานและปิดก่อนเปิดผ่านปุ่มนี้"
                .into(),
        )
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn photocraft_check(app: tauri::AppHandle) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || connect(&app)?.check())
        .await
        .map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn photocraft_apply(app: tauri::AppHandle, id: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let family = thai_font_kit_core::catalog()?
            .families
            .into_iter()
            .find(|f| f.id == id)
            .ok_or("ไม่พบฟอนต์ในคลัง")?;
        connect(&app)?.apply(&family.name)
    })
    .await
    .map_err(|e| e.to_string())?
}
