# Release acceptance checklist

Alpha source is not a production compatibility guarantee. Record OS versions, app versions, installer hashes, screenshots and results before release.

1. CI builds succeed on Windows x64, macOS Apple Silicon, macOS Intel, Linux x64. Download each installer and compare its SHA256SUMS.
2. Clean user account: install app, launch, search and change font selection, resize window, use keyboard focus and both confirmation dialogs.
3. Preview Anuphan and Sarabun. Text changes only after verified font bytes load. Test `น้ำ ผู้ใหญ่ เก้าอี้ ปี่ กุ้ง กิ๊ฟต์ ๑๒๓ ฿` and mixed Thai/English.
4. Install one variable-font family and one static-font family. Confirm all expected files / styles in system font tools, restart PhotoCraft / VectorCraft / DesignCraft, type Thai, save / reopen and export PNG / PDF as applicable. Compare marks, line breaks, caret and selection. Record compatibility separately for each app.
5. Reboot / sign out and verify fonts persist. Open another program that uses system fonts and check the family is available.
6. Disconnect internet: cached previews and fully cached installations still work. A never-loaded family shows an error and existing files remain intact.
7. Reinstall a managed family, uninstall it, and verify system registration / font listing disappears after refreshing or restarting the design app.
8. Create an unrelated font and verify uninstall preserves it. Edit a managed font or replace it with a symlink; uninstall must refuse to remove it. Close apps using fonts before testing removal on Windows.
9. Terminate during install, reopen, inspect managed status and retry / uninstall. Test full disk, unwritable user font directory, damaged receipt JSON, unavailable network and corrupted cache. Record partial-install behavior.
10. Verify two running windows cannot mutate receipts at once; operation errors restore UI controls.
11. On Linux, test with and without `fc-cache` and confirm the app reports cache-refresh guidance.
12. Complete publisher signing / macOS notarization, or clearly distribute only as an unsigned alpha to testers. Confirm installer uninstall leaves fonts by design and document how to remove them first.

## Verification completed in the authoring environment

See `VERIFICATION.md` at repository root for actual executed checks and remaining platform checks. Native installers are not available until a build runner completes them.
