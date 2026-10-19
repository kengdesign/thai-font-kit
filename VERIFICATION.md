# Executed verification — 2026-10-20

## Version 0.2.0 — 34-family expansion

- Matched the complete Thai subset from the official Google Fonts metadata: exactly 34 families and 257 font files.
- Downloaded and validated SHA-256, font parsing and representative Thai glyph coverage for all 257 files using `verify_all_catalog_fonts`.
- Compared all 8 legacy family entries to version 0.1: unchanged IDs, filenames, checksums, sizes and licences. Added a regression test preserving existing installation receipts.
- Default core tests: 9 passed on Linux; the complete-catalog test was then run explicitly and passed.
- Playwright: 3 passed, including search and selection of the newly added Google Sans family.
- TypeScript/Vite build, Rust formatting and Python syntax checks passed.
- Native installers are built by the same GitHub Actions matrix before the 0.2.0 alpha is published. Results appear in the repository Actions page and release assets.

## Earlier version 0.1 build evidence

## Passed

- `npm run build`: TypeScript type checking and Vite production bundle.
- `cargo test --locked -p thai-font-kit-core`: 8 passing tests, including real verified Anuphan bytes → offline install → idempotent reinstall → uninstall; preserving unmanaged and modified fonts; refusing symlinks and forged paths; missing-file receipt cleanup; two-instance lock.
- `npm test`: 3 passing Playwright Chromium UI tests. Covers search / desktop-only controls, real FontFace preview bytes through a mocked native bridge, install confirmation / cancel, recovery after a simulated download error, and cleanup access for missing managed fonts.
- `cargo fmt --all --check`.
- `python3 -m py_compile scripts/catalog.py scripts/collect_release.py`.
- Release collector smoke check: copied a fixture installer and independently verified its SHA-256 file.
- Platform adapter type checking for Windows GNU x64 and macOS Apple Silicon in a small temporary crate containing the same `platform.rs`. These are compile checks only, not native linking, registry / Core Text runtime tests, or full Tauri builds.
- UI rendered in Chromium and visually inspected; fixed the preview fallback to the bundled Thai UI font so first-run Linux does not show missing Thai glyphs.

## Native build results on GitHub

All four native build targets and the interface tests passed in [run 39294623034](https://github.com/kengdesign/thai-font-kit/actions/runs/39294623034):

- Windows x64 MSVC — NSIS .exe
- macOS Apple Silicon — .dmg
- macOS Intel — .dmg
- Linux x64 — .deb and .AppImage
- Playwright interface tests — 3 passed

Native runner core tests passed on Windows, macOS and Linux. Installers were collected with SHA-256 checksum files. This verifies compilation, packaging and automated checks; it does not verify OS font activation or ArtCraft compatibility on real user machines.

The alpha release pipeline is [run 39295035898](https://github.com/kengdesign/thai-font-kit/actions/runs/39295035898). It publishes only after the same native build matrix and interface tests succeed.

## Not yet completed

- Installer execution, OS font registration persistence, reboot behavior, uninstall while a font is in use, and real PhotoCraft / VectorCraft / DesignCraft editing and export.
- Publisher code signing and macOS notarization.

Public installers remain explicitly labelled experimental Alpha. No stable release or ArtCraft compatibility guarantee is claimed.
