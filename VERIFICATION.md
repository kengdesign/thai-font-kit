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

## Not yet completed

- Full native Tauri compilation / installer creation: this authoring host lacks WebKitGTK / GTK development libraries; package installation is unavailable in the host. The GitHub workflow installs dependencies on its build runners.
- GitHub workflow execution: repository `kengdesign/thai-font-kit` has been created by its owner. The initial source upload triggers the configured native build matrix; results will be recorded after completion.
- Installer execution, OS font registration persistence, reboot behavior, uninstall while a font is in use, real PhotoCraft / VectorCraft / DesignCraft editing and export, code signing and notarization.
- Windows MSVC and macOS Intel full build verification will run in the configured GitHub matrix.

All public release claims must remain alpha until native builds and manual QA are completed. The source archive is not a desktop installer.
