#[derive(Debug, Clone)]
pub enum PromptResult {
    Value(String),
    Cancelled,
    Error(String),
}

/// Prompt the user for the Kimi Code API key using a native macOS dialog.
#[cfg(target_os = "macos")]
pub fn prompt_api_key() -> PromptResult {
    let script = r#"
        tell application "System Events"
            activate
            set dialogResult to display dialog "Enter your Kimi Code API Key:" default answer "" with title "Kimi Usage Widget" buttons {"Cancel", "Save"} default button "Save" with hidden answer
            if button returned of dialogResult is "Save" then
                return text returned of dialogResult
            else
                return "__CANCELLED__"
            end if
        end tell
    "#;

    match std::process::Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
    {
        Ok(output) => {
            let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if text == "__CANCELLED__" {
                PromptResult::Cancelled
            } else if text.is_empty() {
                PromptResult::Error("API key cannot be empty".to_string())
            } else {
                PromptResult::Value(text)
            }
        }
        Err(e) => PromptResult::Error(format!("failed to run dialog: {e}")),
    }
}

#[cfg(not(target_os = "macos"))]
pub fn prompt_api_key() -> PromptResult {
    PromptResult::Error("interactive API key prompt is only supported on macOS".to_string())
}
