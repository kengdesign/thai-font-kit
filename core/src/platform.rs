use std::path::{Path, PathBuf};

pub fn font_dir() -> Result<PathBuf, String> {
    let home = directories::BaseDirs::new().ok_or("Cannot resolve user home")?;
    #[cfg(target_os = "windows")]
    let path = home.data_local_dir().join("Microsoft/Windows/Fonts");
    #[cfg(target_os = "macos")]
    let path = home.home_dir().join("Library/Fonts");
    #[cfg(target_os = "linux")]
    let path = home.data_dir().join("fonts");
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    return Err("Unsupported operating system".into());
    Ok(path)
}

#[cfg(target_os = "windows")]
pub fn register(path: &Path, add: bool) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    use winreg::{enums::HKEY_CURRENT_USER, RegKey};
    #[link(name = "gdi32")]
    extern "system" {
        fn AddFontResourceExW(path: *const u16, flags: u32, reserved: *mut std::ffi::c_void)
            -> i32;
        fn RemoveFontResourceExW(
            path: *const u16,
            flags: u32,
            reserved: *mut std::ffi::c_void,
        ) -> i32;
    }
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let (key, _) = RegKey::predef(HKEY_CURRENT_USER)
        .create_subkey("Software\\Microsoft\\Windows NT\\CurrentVersion\\Fonts")
        .map_err(|e| e.to_string())?;
    let name = format!(
        "{} (TrueType)",
        path.file_stem()
            .ok_or("Invalid font path")?
            .to_string_lossy()
    );
    if add {
        if key.get_value::<String, _>(&name).ok().as_deref()
            == Some(path.to_string_lossy().as_ref())
        {
            return Ok(());
        }
        unsafe {
            if AddFontResourceExW(wide.as_ptr(), 0, std::ptr::null_mut()) == 0 {
                return Err("Windows could not activate this font".into());
            }
        }
        if let Err(e) = key.set_value(&name, &path.to_string_lossy().as_ref()) {
            unsafe {
                RemoveFontResourceExW(wide.as_ptr(), 0, std::ptr::null_mut());
            }
            return Err(e.to_string());
        }
    } else {
        match key.delete_value(&name) {
            Ok(()) => (),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(e) => return Err(e.to_string()),
        }
        unsafe {
            RemoveFontResourceExW(wide.as_ptr(), 0, std::ptr::null_mut());
        }
    }
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn register(path: &Path, add: bool) -> Result<(), String> {
    use std::ffi::c_void;
    use std::os::unix::ffi::OsStrExt;
    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFURLCreateFromFileSystemRepresentation(
            allocator: *const c_void,
            bytes: *const u8,
            len: isize,
            directory: u8,
        ) -> *const c_void;
        fn CFRelease(object: *const c_void);
        fn CFErrorGetCode(error: *const c_void) -> isize;
    }
    #[link(name = "CoreText", kind = "framework")]
    extern "C" {
        fn CTFontManagerRegisterFontsForURL(
            url: *const c_void,
            scope: u32,
            error: *mut *const c_void,
        ) -> u8;
        fn CTFontManagerUnregisterFontsForURL(
            url: *const c_void,
            scope: u32,
            error: *mut *const c_void,
        ) -> u8;
    }
    let bytes = path.as_os_str().as_bytes();
    unsafe {
        let url = CFURLCreateFromFileSystemRepresentation(
            std::ptr::null(),
            bytes.as_ptr(),
            bytes.len() as isize,
            0,
        );
        if url.is_null() {
            return Err("Cannot create font URL".into());
        }
        let mut error = std::ptr::null();
        let ok = if add {
            CTFontManagerRegisterFontsForURL(url, 2, &mut error)
        } else {
            CTFontManagerUnregisterFontsForURL(url, 2, &mut error)
        };
        let code = if error.is_null() {
            0
        } else {
            CFErrorGetCode(error)
        };
        if !error.is_null() {
            CFRelease(error);
        }
        CFRelease(url);
        if ok == 0 && add && code != 105 {
            return Err(
                "macOS could not activate this font. Check Font Book for duplicates.".into(),
            );
        }
    }
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn register(_path: &Path, _add: bool) -> Result<(), String> {
    Ok(())
}

pub fn refresh(dir: &Path) -> String {
    #[cfg(target_os = "windows")]
    {
        #[link(name = "user32")]
        extern "system" {
            fn SendNotifyMessageW(hwnd: isize, msg: u32, wparam: usize, lparam: isize) -> i32;
        }
        unsafe {
            SendNotifyMessageW(0xffff, 0x001d, 0, 0);
        }
    }
    #[cfg(target_os = "linux")]
    {
        use std::{
            process::{Command, Stdio},
            thread,
            time::{Duration, Instant},
        };
        match Command::new("fc-cache")
            .arg("-f")
            .arg(dir)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(mut child) => {
                let start = Instant::now();
                loop {
                    match child.try_wait() {
                        Ok(Some(status)) if status.success() => return String::new(),
                        Ok(Some(_)) | Err(_) => return "Font cache refresh failed. Restart the design app or sign in again.".into(),
                        _ if start.elapsed() > Duration::from_secs(15) => {
                            let _ = child.kill(); let _ = child.wait();
                            return "Font cache refresh timed out. Restart the design app or sign in again.".into();
                        }
                        _ => thread::sleep(Duration::from_millis(100)),
                    }
                }
            }
            Err(_) => return "Install fontconfig to refresh fonts, or sign in again.".into(),
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = dir;
        String::new()
    }
}
