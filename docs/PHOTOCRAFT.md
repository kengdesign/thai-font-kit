# PhotoCraft companion (0.3.0 alpha)

This is an external companion, not an embedded PhotoCraft panel. It uses the documented authenticated loopback JSON-line control protocol, `engine.execute`, `type.fonts`, `type.info`, and `type.setStyle`.

## User setup

Save work and close any existing PhotoCraft window. Click **เปิด PhotoCraft และเชื่อมต่อ** and select the installed PhotoCraft executable once. The path is remembered. On Windows choose PhotoCraft.exe; on Linux choose the executable or AppImage; on macOS choose PhotoCraft.app (or its photocraft executable inside Contents/MacOS). If the OS file picker does not allow selecting an app bundle, use the executable inside it.

The launcher supplies `--control 17878 --control-token-file <app-local-data>/photocraft-bridge/control.token`. PhotoCraft creates the token. No automation read/write roots are granted. On Windows keep the token folder in your normal private user profile. Do not share the token file.

Click **แผงเล็ก · อยู่ด้านหน้า** for a compact 460px-wide always-on-top companion, and click **กลับหน้าจอเต็ม** to restore the full view and stop keeping it on top. This does not embed into the editor.

Open a document in that PhotoCraft window and select an editable text layer. Choose an installed font in Thai Font Kit and click **ใช้ใน PhotoCraft**. It applies the family to the whole active text layer, retaining text and other style attributes. Undo is in PhotoCraft. The companion does not save the document or replace its text.

Install missing fonts before opening PhotoCraft. When PhotoCraft does not yet see a new font, save/close it and launch again through Thai Font Kit. The bridge refuses application when the requested family has no faces in PhotoCraft; no silent fallback.

## Limits and testing

Experimental: protocol compatibility is checked at connection time, but installed PhotoCraft versions differ. Both apps must run under the same user. The bridge only connects to 127.0.0.1:17878, authenticates every connection, bounds replies to 1 MB, and never retries a document edit. A timeout may mean the edit already happened: check the document before retrying. No background polling or service. Closing Thai Font Kit does not force-close PhotoCraft.

The selected layer at the time you press Apply is the target, and the whole layer is styled. Existing canvas text selection is not used as a range. This version does not implement live hover previews, automatic font activation inside PhotoCraft, project collections or multi-font comparison.

Tests cover authenticated TCP protocol exchanges against a local fixture server, correct command/target, invalid connection data, auth error redaction, favorites, filtering and UI command calls. Native installer builds do not establish that every installed PhotoCraft release works; real-machine PhotoCraft document checks are still needed.

## Embedded Thai tab proposal

PhotoCraft's documented WASM plug-in ABI v1 is filter-only and cannot inject a font picker tab. A proper embedded filter needs an upstream PhotoCraft UI change. Suggested scope: Thai-script filter, favorites and recent fonts in its shared `family_picker_in` so Type options, Character, styles and Glyphs panels stay consistent. Detect Thai glyph coverage, cache it once per font database refresh, use virtual scrolling, and preserve keyboard search/selection. Do not rename system font families or patch installed executable binaries. No upstream PR has been sent as part of this companion release.

Sources checked: https://github.com/storytold/photocraft/blob/main/docs/control-protocol.md and https://github.com/storytold/photocraft/blob/main/docs/plugins.md.
