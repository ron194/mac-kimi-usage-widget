#![allow(dead_code)]

use serde::Deserialize;
use serde_json::Value;

const USAGES_URL: &str = "https://api.kimi.com/coding/v1/usages";

#[derive(Debug, Clone, Deserialize)]
pub struct UsageSummary {
    pub limit: String,
    pub used: String,
    pub remaining: String,
    #[serde(default)]
    pub reset_time: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WindowDetail {
    pub limit: String,
    pub used: String,
    pub remaining: String,
    #[serde(default)]
    pub reset_time: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RateLimit {
    pub window: Window,
    pub detail: WindowDetail,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Window {
    pub duration: u64,
    #[serde(default)]
    pub time_unit: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ParallelLimit {
    pub limit: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TotalQuota {
    pub limit: String,
    pub remaining: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UsagesResponse {
    pub usage: UsageSummary,
    #[serde(default)]
    pub limits: Vec<RateLimit>,
    #[serde(default)]
    pub parallel: Option<ParallelLimit>,
    #[serde(default)]
    pub total_quota: Option<TotalQuota>,
}

#[derive(Debug, Clone)]
pub struct QuotaStats {
    pub weekly_used: u64,
    pub weekly_limit: u64,
    pub weekly_remaining: u64,
    pub window_used: u64,
    pub window_limit: u64,
    pub window_remaining: u64,
    pub window_duration_minutes: u64,
    pub reset_time: Option<String>,
    pub parallel_limit: Option<u64>,
}

impl QuotaStats {
    pub fn weekly_percentage(&self) -> u8 {
        if self.weekly_limit == 0 {
            return 0;
        }
        let pct = (self.weekly_used as f64 / self.weekly_limit as f64) * 100.0;
        pct.min(100.0) as u8
    }
}

fn parse_u64_str(s: &str) -> u64 {
    s.parse().unwrap_or(0)
}

fn value_to_u64(value: &Value) -> Option<u64> {
    match value {
        Value::Number(n) => n.as_u64().or_else(|| n.as_f64().map(|f| f as u64)),
        Value::String(s) => s.parse().ok(),
        _ => None,
    }
}

fn value_to_string(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

/// Extract quota statistics from a loosely-typed API response.
///
/// The live API has returned fields as either strings or numbers at different
/// times, so this parser accepts both and falls back to zero for missing
/// fields rather than failing the whole request.
fn quota_from_value(data: &Value) -> QuotaStats {
    let usage = data.get("usage");

    let weekly_used = usage
        .and_then(|u| u.get("used"))
        .and_then(value_to_u64)
        .unwrap_or(0);
    let weekly_limit = usage
        .and_then(|u| u.get("limit"))
        .and_then(value_to_u64)
        .unwrap_or(0);
    let weekly_remaining = usage
        .and_then(|u| u.get("remaining"))
        .and_then(value_to_u64)
        .unwrap_or(0);
    let reset_time = usage
        .and_then(|u| u.get("reset_time"))
        .and_then(value_to_string);

    let mut window_used = 0;
    let mut window_limit = 0;
    let mut window_remaining = 0;
    let mut window_duration_minutes = 0;

    if let Some(Value::Array(limits)) = data.get("limits") {
        if let Some(first) = limits.first() {
            if let Some(detail) = first.get("detail") {
                window_used = detail.get("used").and_then(value_to_u64).unwrap_or(0);
                window_limit = detail.get("limit").and_then(value_to_u64).unwrap_or(0);
                window_remaining = detail.get("remaining").and_then(value_to_u64).unwrap_or(0);
            }
            if let Some(window) = first.get("window") {
                window_duration_minutes =
                    window.get("duration").and_then(value_to_u64).unwrap_or(0);
            }
        }
    }

    let parallel_limit = data
        .get("parallel")
        .and_then(|p| p.get("limit"))
        .and_then(value_to_u64);

    QuotaStats {
        weekly_used,
        weekly_limit,
        weekly_remaining,
        window_used,
        window_limit,
        window_remaining,
        window_duration_minutes,
        reset_time,
        parallel_limit,
    }
}

pub fn fetch_quota(api_key: &str) -> Result<QuotaStats, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    let response = client
        .get(USAGES_URL)
        .header("Authorization", format!("Bearer {api_key}"))
        .header("User-Agent", "kimi-usage-widget/0.1.0")
        .send()?;

    if !response.status().is_success() {
        return Err(format!(
            "API returned {}: {}",
            response.status(),
            response.text().unwrap_or_default()
        )
        .into());
    }

    let body = response.text()?;
    crate::log::write_payload("API response", &body);

    let data: Value = serde_json::from_str(&body).map_err(|e| {
        crate::log::write(&format!(
            "Failed to parse API response as JSON: {e}. Body length: {}",
            body.len()
        ));
        format!("error decoding response body for url ({}): {e}", USAGES_URL)
    })?;

    let quota = quota_from_value(&data);
    crate::log::write(&format!(
        "Parsed quota: used={} limit={} remaining={} window_duration={}m",
        quota.weekly_used,
        quota.weekly_limit,
        quota.weekly_remaining,
        quota.window_duration_minutes
    ));

    Ok(quota)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_string_quota_values() {
        let json = serde_json::json!({
            "usage": {
                "limit": "1000",
                "used": "480",
                "remaining": "520",
                "reset_time": "2026-07-14T00:00:00Z"
            },
            "limits": [
                {
                    "window": { "duration": 60 },
                    "detail": {
                        "limit": "100",
                        "used": "30",
                        "remaining": "70"
                    }
                }
            ],
            "parallel": { "limit": "10" }
        });

        let quota = quota_from_value(&json);
        assert_eq!(quota.weekly_used, 480);
        assert_eq!(quota.weekly_limit, 1000);
        assert_eq!(quota.weekly_remaining, 520);
        assert_eq!(quota.reset_time, Some("2026-07-14T00:00:00Z".to_string()));
        assert_eq!(quota.window_used, 30);
        assert_eq!(quota.window_limit, 100);
        assert_eq!(quota.window_remaining, 70);
        assert_eq!(quota.window_duration_minutes, 60);
        assert_eq!(quota.parallel_limit, Some(10));
        assert_eq!(quota.weekly_percentage(), 48);
    }

    #[test]
    fn parses_numeric_quota_values() {
        let json = serde_json::json!({
            "usage": {
                "limit": 1000,
                "used": 480,
                "remaining": 520
            },
            "limits": [
                {
                    "window": { "duration": 60 },
                    "detail": {
                        "limit": 100,
                        "used": 30,
                        "remaining": 70
                    }
                }
            ]
        });

        let quota = quota_from_value(&json);
        assert_eq!(quota.weekly_used, 480);
        assert_eq!(quota.weekly_limit, 1000);
        assert_eq!(quota.weekly_remaining, 520);
        assert_eq!(quota.window_used, 30);
        assert_eq!(quota.window_limit, 100);
        assert_eq!(quota.window_remaining, 70);
        assert_eq!(quota.window_duration_minutes, 60);
        assert_eq!(quota.parallel_limit, None);
    }

    #[test]
    fn handles_partial_response() {
        let json = serde_json::json!({
            "usage": {
                "limit": "1000",
                "used": "480"
            }
        });

        let quota = quota_from_value(&json);
        assert_eq!(quota.weekly_used, 480);
        assert_eq!(quota.weekly_limit, 1000);
        assert_eq!(quota.weekly_remaining, 0);
        assert_eq!(quota.window_duration_minutes, 0);
        assert_eq!(quota.weekly_percentage(), 48);
    }
}
