use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

const LOG_FILE_NAME: &str = "debug.log";

pub fn log_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("kimi-usage-widget"))
}

pub fn log_path() -> Option<PathBuf> {
    log_dir().map(|dir| dir.join(LOG_FILE_NAME))
}

/// Append a timestamped message to the debug log.
///
/// Failures are silently ignored so logging never breaks the app.
pub fn write(message: &str) {
    let Some(path) = log_path() else { return };

    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    let line = format!("[{}] {}\n", now, message);

    let _ = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .and_then(|mut file| file.write_all(line.as_bytes()));
}

/// Write a multi-line payload (e.g. a raw HTTP response) to the debug log.
pub fn write_payload(label: &str, payload: &str) {
    let Some(path) = log_path() else { return };

    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    let header = format!("[{}] --- {} ---\n", now, label);
    let footer = format!("[{}] --- end {} ---\n", now, label);

    let _ = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .and_then(|mut file| {
            file.write_all(header.as_bytes())?;
            file.write_all(payload.as_bytes())?;
            if !payload.ends_with('\n') {
                file.write_all(b"\n")?;
            }
            file.write_all(footer.as_bytes())
        });
}
