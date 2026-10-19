# Executed verification — 2026-10-20

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
