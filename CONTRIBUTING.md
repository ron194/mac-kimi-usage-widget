# Contributing to Kimi Usage Widget

Thank you for considering a contribution! This project is a small, native macOS menu-bar app, and we aim to keep it lightweight and focused.

## How to contribute

1. **Open an issue first** for significant changes or new features so we can discuss direction before you invest time.
2. **Fork the repository** and create a feature branch.
3. **Make your changes** with clear, focused commits.
4. **Run the checks locally** (see below).
5. **Open a pull request** with a concise description of what changed and why.

## Development setup

```bash
# Clone your fork
git clone https://github.com/<your-username>/kimi-usage-widget.git
cd kimi-usage-widget

# Build
cargo build --release

# Run tests
cargo test
```

## Code style

- Format your code with `cargo fmt`.
- Fix any Clippy warnings: `cargo clippy --all-targets --all-features -- -D warnings`.
- Keep dependencies minimal. If you add a new crate, explain why it is needed.
- Gate macOS-specific code with `#[cfg(target_os = "macos")]` and provide sensible fallbacks.

## Testing

- Add unit tests that do not depend on your local `~/.kimi-code` logs.
- The integration test `parses_actual_kimi_usage` is ignored by default because it requires real wire logs. Run it explicitly with:

  ```bash
  cargo test -- --ignored
  ```

## Commit messages

Write clear commit messages in the imperative mood, for example:

> Add cache-read token display to dropdown

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
