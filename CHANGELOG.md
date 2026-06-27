# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- macOS `.app` bundle packaging so the app runs as a menu-bar-only app without a keep-alive terminal.
  - `packaging/macos/Info.plist` with `LSUIElement` enabled.
  - `scripts/build-macos-app.sh` to build, sign, and package the release.
- Generated `AppIcon.icns` from `assets/icon.png` as part of the macOS bundle build.
- Ad-hoc code signing of the `.app` bundle so it launches from the DMG.
- GitHub Actions release workflow (`.github/workflows/release.yml`) that builds and uploads a DMG to GitHub Releases on `v*.*.*` tags.
- Install and distribution instructions in `README.md` and `AGENTS.md`.

## [0.1.0] - 2026-06-27

### Added

- Initial release of Kimi Usage Widget.
- Live console quota fetching from the Kimi Code API.
- Local token usage aggregation from `~/.kimi-code` wire logs.
- macOS menu-bar UI with refresh, API-key prompt, open-console, and quit actions.
- Config file and environment-variable support for API key, base URL, and daily budget.

[Unreleased]: https://github.com/rontam/kimi-usage-widget/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/rontam/kimi-usage-widget/releases/tag/v0.1.0
