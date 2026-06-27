# Agent Guide for `kimi-usage-widget`

## Project Overview

`kimi-usage-widget` is a lightweight, native macOS menu-bar application written in Rust. It displays Kimi Code usage information in the macOS menu bar and a dropdown menu.

- **Language**: Rust (edition 2024, requires Rust 1.85+)
- **Platform**: macOS (the interactive API-key prompt is macOS-only)
- **License**: MIT
- **Repository layout**: Standard Cargo project with source under `src/` and a single binary crate.

The app performs two main jobs:

1. **Live console quota**: Calls the Kimi Code API (`GET https://api.kimi.com/coding/v1/usages`) with a user API key and shows the weekly usage percentage in the menu bar (e.g. `Kimi 48%`).
2. **Local token usage**: Reads `~/.kimi-code/sessions/*/agents/*/wire.jsonl`, filters `usage.record` events, and aggregates total / output / input / cache-read tokens, plus today and last-7-days totals.

If no API key is configured or the API call fails, the menu bar falls back to showing the local daily-budget percentage.

## Build and Run Commands

```bash
# Development build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run the raw binary (useful during development)
cargo run --release
# or after building:
./target/release/kimi-usage-widget

# Build a distributable macOS .app bundle and DMG
./scripts/build-macos-app.sh
```

The release artifact is a signed `.app` bundle packaged as a DMG; the raw binary is still available under `target/release/kimi-usage-widget` for development.

## Technology Stack and Architecture

### Key dependencies (`Cargo.toml`)

| Crate | Purpose |
|-------|---------|
| `tao` | Cross-platform windowing/event-loop primitives; used here for the macOS event loop. |
| `tray-icon` | System tray / menu-bar icon and dropdown menu. |
| `reqwest` (blocking, json) | HTTP client for the Kimi Code API. |
| `serde` / `serde_json` | JSON/TOML serialization. |
| `chrono` (with `serde`) | Timestamp parsing and date arithmetic for local usage aggregation. |
| `toml` | Config file serialization. |
| `dirs` | Standard platform directories for config and home paths. |
| `image` (png only) | Loading the embedded menu-bar icon. |

### Module layout (`src/`)

- `src/main.rs` — Entry point. Builds the tray icon, starts the `tao` event loop, handles menu clicks (Refresh / Set API Key / Quit), and refreshes the UI every 60 seconds.
- `src/api.rs` — Kimi Code HTTP API client. Defines request/response types and `fetch_quota`, which returns a `QuotaStats` struct.
- `src/config.rs` — Config file loading/saving and environment-variable handling.
- `src/prompt.rs` — Native macOS dialog for entering the API key via `osascript`. Non-macOS builds return an error.
- `src/usage.rs` — Discovers local `wire.jsonl` logs under `~/.kimi-code`, parses `usage.record` events, and aggregates token counts by day.
- `assets/icon.png` — Embedded menu-bar icon, loaded at compile time with `include_bytes!`.
- `packaging/macos/Info.plist` — macOS app-bundle metadata (`LSUIElement`, bundle identifier, version, etc.).
- `scripts/build-macos-app.sh` — Builds the release binary, produces `Kimi Usage Widget.app`, and packages it into a DMG.

### Runtime behavior

- On launch, `config::ensure_default_config()` creates a default config file if one does not exist.
- The event loop uses `ControlFlow::WaitUntil` to refresh the UI every 60 seconds (`REFRESH_INTERVAL`).
- `Rc<RefCell<...>>` is used for shared mutable state (`config`, `tray_icon`, `base_dir`, UI state) inside the single-threaded event loop.
- Menu items are recreated on every UI refresh because `tray-icon` does not support updating existing item labels in place.

## Configuration

The app looks for config at the standard OS config directory:

- macOS: `~/Library/Application Support/kimi-usage-widget/config.toml`
- Linux: `~/.config/kimi-usage-widget/config.toml`

Example config:

```toml
daily_budget = 1000000
api_key = "your-kimi-code-api-key"
base_url = "https://api.kimi.com/coding/v1"
```

- `daily_budget` — Used to compute the local fallback percentage shown in the menu bar. Default: `1_000_000`.
- `api_key` — Kimi Code API key. Optional; if absent, the app falls back to local usage.
- `base_url` — API base URL. Default: `https://api.kimi.com/coding/v1`.

The API key can also be supplied via the `KIMI_CODE_API_KEY` environment variable, which overrides the config file value. The in-app **Set API Key...** menu writes the key to the config file.

## Code Style Guidelines

- Follow standard Rust formatting with `cargo fmt`.
- Keep error handling simple: functions return `Result<..., Box<dyn std::error::Error>>` where practical.
- Use `serde` derives for JSON/TOML data types.
- Avoid adding heavy dependencies; the project intentionally avoids a webview.
- macOS-specific code is gated with `#[cfg(target_os = "macos")]`; provide a sensible fallback for other platforms.
- Percentage calculations clamp results (`min(100.0)` for quota, `min(255.0)` for the daily budget) before casting to `u8`.

## Testing Instructions

```bash
cargo test
```

Tests live in `src/usage.rs` under `#[cfg(test)]`.

- `aggregates_usage_from_wire_jsonl` uses a temporary directory with sample `wire.jsonl` data and runs in CI.
- `parses_actual_kimi_usage` is marked `#[ignore]` because it requires real `~/.kimi-code` wire logs on the local machine. Run it explicitly with `cargo test -- --ignored`.

When adding new functionality, prefer adding unit tests that do not depend on the developer's local Kimi Code logs. Run `cargo test` before committing.

## Security Considerations

- The API key is stored in **plaintext** in the config file. This is consistent with the current design but should be noted if extending the app.
- The API key can be passed via the `KIMI_CODE_API_KEY` environment variable; this avoids writing it to disk but may still be visible in process listings.
- All API calls use HTTPS and send the key in an `Authorization: Bearer <key>` header with a short timeout (15 seconds).
- The app reads local wire logs from `~/.kimi-code`. It does not write to that directory.
- No secrets or credentials are logged or shown in the UI.

## Deployment Notes

- The release workflow (`.github/workflows/release.yml`) builds the app on `macos-latest`, runs `./scripts/build-macos-app.sh`, and uploads the resulting DMG to a GitHub Release when a `v*.*.*` tag is pushed.
- The script produces:
  - `dist/Kimi Usage Widget.app` — ad-hoc signed macOS app bundle with `LSUIElement` set so it runs as a menu-bar-only app.
  - `dist/Kimi-Usage-Widget-X.Y.Z.dmg` — compressed disk image for distribution.
- The app is signed ad-hoc (`codesign --sign -`) so it can be launched from the DMG, but it is **not** notarized. Users on macOS 10.15+ may need to right-click the app and choose **Open** the first time they run it.
- To ship a fully Gatekeeper-compliant build, configure a paid Apple Developer account and update the workflow to import a Developer ID certificate and submit the app to Apple Notary Service.
