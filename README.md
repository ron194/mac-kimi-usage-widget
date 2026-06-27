# Kimi Code Usage Widget

A lightweight macOS menu-bar app written in Rust that displays Kimi Code usage.

![CI](https://github.com/rontam/kimi-usage-widget/actions/workflows/ci.yml/badge.svg)
![Rust](https://img.shields.io/badge/rust-1.85%2B-orange)
![License](https://img.shields.io/badge/license-MIT-blue)

## Features

- Sits in the macOS menu bar
- Fetches **real console quota** from the Kimi Code API and shows the weekly usage percentage directly in the menu bar (e.g. `Kimi 48%`)
- Shows both console quota and local token usage in the dropdown:
  - Weekly quota used / limit / remaining
  - Rolling rate-limit window usage
  - Quota reset time
  - Total / output / input / cache-read tokens
  - Today / last 7 days token usage
- Refresh on demand
- Auto-refreshes every 60 seconds
- Lightweight native implementation (no webview)

## Setup

Create an API key in the [Kimi Code Console](https://www.kimi.com/code/console), then configure the app.

### Option 1: In-app menu (easiest)

1. Click the menu-bar icon.
2. Select **Set API Key...**.
3. Paste your API key in the native dialog and click **Save**.

### Option 2: Config file

The app creates a config file on first run:

- macOS: `~/Library/Application Support/kimi-usage-widget/config.toml`
- Linux: `~/.config/kimi-usage-widget/config.toml`

Add your API key:

```toml
daily_budget = 1000000
api_key = "your-kimi-code-api-key"
base_url = "https://api.kimi.com/coding/v1"
```

### Option 3: Environment variable

```bash
export KIMI_CODE_API_KEY="your-kimi-code-api-key"
./target/release/kimi-usage-widget
```

The environment variable overrides the config file.

## How it works

- Reads `~/.kimi-code/sessions/*/agents/*/wire.jsonl`, filters `usage.record` events, and aggregates token counts locally.
- Calls `GET https://api.kimi.com/coding/v1/usages` with your API key to fetch the live console quota.
- If the API key is missing or the call fails, the app falls back to showing the local token-usage percentage.

## Requirements

- macOS
- Rust 1.85 or later
- Kimi Code API key (for live quota)

## Build

```bash
cargo build --release
```

## Run

```bash
./target/release/kimi-usage-widget
```

To keep it running in the background:

```bash
./target/release/kimi-usage-widget &
```

## Test

```bash
cargo test
```

## Project structure

- `src/main.rs` — menu-bar setup and event loop
- `src/api.rs` — Kimi Code usage API client
- `src/config.rs` — config file and environment handling
- `src/usage.rs` — usage log discovery and aggregation
- `assets/icon.png` — menu-bar icon

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, code style, and pull-request guidelines.

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.
