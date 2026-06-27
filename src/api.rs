#![allow(dead_code)]

use serde::Deserialize;

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

    let data: UsagesResponse = response.json()?;

    let weekly_used = parse_u64_str(&data.usage.used);
    let weekly_limit = parse_u64_str(&data.usage.limit);
    let weekly_remaining = parse_u64_str(&data.usage.remaining);

    let mut window_used = 0;
    let mut window_limit = 0;
    let mut window_remaining = 0;
    let mut window_duration_minutes = 0;

    if let Some(first) = data.limits.first() {
        window_used = parse_u64_str(&first.detail.used);
        window_limit = parse_u64_str(&first.detail.limit);
        window_remaining = parse_u64_str(&first.detail.remaining);
        window_duration_minutes = first.window.duration;
    }

    let parallel_limit = data.parallel.as_ref().map(|p| parse_u64_str(&p.limit));

    Ok(QuotaStats {
        weekly_used,
        weekly_limit,
        weekly_remaining,
        window_used,
        window_limit,
        window_remaining,
        window_duration_minutes,
        reset_time: data.usage.reset_time.clone(),
        parallel_limit,
    })
}
