# Architecture and current limits

- `core`: OS independent font catalog, validation, verified cache, exclusive file lock, atomic JSON receipts, install / uninstall policy.
- `core/src/platform.rs`: Windows per-user Fonts directory + HKCU Fonts + GDI activation / WM_FONTCHANGE; macOS user Fonts + Core Text user scope; Linux XDG font directory + bounded fontconfig refresh.
- `src-tauri`: five narrow commands; the frontend cannot supply filesystem paths or download URLs.
- `src`: Thai UI with explicit font preview loading, confirmation dialogs and operation locking.
- `.github/workflows/build.yml`: four native build targets, artifacts with checksums, draft prerelease on a version tag.

Font URLs are built from an embedded commit + family ID + filename, never user input. HTTP redirects are rejected and downloads are bounded to 16 MB / 60 seconds. Font bytes must match embedded SHA-256 and parse as a font containing representative Thai consonants and combining marks. This checks coverage, not shaping correctness.

The installation uses an app-prefixed filename in the user's system font directory. It never adopts an existing unmanaged file, even when bytes match. Existing fonts elsewhere are not scanned for duplicates; installing an already available family under another filename may duplicate a family in some applications. The Installed view tracks only this app's receipts, not every system font.

There is no font-update button in alpha. A future catalog update needs versioned receipts and explicit migration so older files remain safely removable. An interrupted partial file write is normally cleaned up; a process or power failure at the exact point of a partial write can leave a modified file. The app refuses to delete such a file automatically. Show its path to the user for manual review.

Family installation is recoverable but not globally atomic: all downloads are validated first, then individual font styles are journaled and installed. If OS registration or disk I/O fails mid-family, already installed styles remain managed. Retry completes missing styles or uninstall removes those already written. Existing matching owned files are registered again to recover a crash between writing bytes and OS activation.

Uninstall refuses altered files or symlinks, validates receipt entries against the bundled catalog, removes OS registrations, removes bytes, then persists the updated receipt. If a font is locked by another app, closing that app and retrying is required.

Scope not implemented: internal ArtCraft plugin API, application-specific font injection, Thai shaping fixes, auto updates, system-wide installs, Windows ARM64 / Linux ARM64 builds, code signing and notarization. No installer size or ArtCraft compatibility has been measured yet.
