# PhotoCraft integration status — 0.4.0

The experimental external PhotoCraft launcher, token/port connector, apply button and always-on-top companion mode have been removed. Users no longer need to locate an executable or configure a connection. No PhotoCraft processes are launched by this app.

Thai Font Kit remains a system font installer and preview browser with favorites and categories. These categories belong to Thai Font Kit; they do not change PhotoCraft's font menu.

The requested Photoshop-like list inside PhotoCraft requires a change to PhotoCraft itself: a Thai script filter, classification, favorites and Thai sample previews in its native font picker. Its documented WASM plugin ABI v1 only hosts image filters and does not provide font-picker/panel injection. No embedded integration is included or claimed in this release.

Reference: https://github.com/storytold/photocraft/blob/main/docs/plugins.md
