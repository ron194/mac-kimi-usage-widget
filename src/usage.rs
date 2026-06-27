use chrono::{DateTime, Local, NaiveDate, Utc};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
struct WireEvent {
    #[serde(rename = "type")]
    event_type: String,
    usage: Option<UsageRecord>,
    #[serde(default)]
    time: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct UsageRecord {
    #[serde(default)]
    input_other: u64,
    #[serde(default)]
    output: u64,
    #[serde(default, rename = "inputCacheRead")]
    input_cache_read: u64,
    #[serde(default, rename = "inputCacheCreation")]
    input_cache_creation: u64,
}

impl UsageRecord {
    fn total(&self) -> u64 {
        self.input_other + self.output + self.input_cache_read + self.input_cache_creation
    }
}

#[derive(Debug, Clone, Default)]
pub struct AggregatedUsage {
    pub total_input_other: u64,
    pub total_output: u64,
    pub total_input_cache_read: u64,
    pub total_input_cache_creation: u64,
    pub today_total: u64,
    pub last_7_days_total: u64,
    pub daily_totals: HashMap<NaiveDate, u64>,
}

impl AggregatedUsage {
    pub fn total_tokens(&self) -> u64 {
        self.total_input_other
            + self.total_output
            + self.total_input_cache_read
            + self.total_input_cache_creation
    }
}

fn discover_wire_logs(base_dir: &Path) -> Vec<PathBuf> {
    let mut logs = Vec::new();
    let sessions_dir = base_dir.join("sessions");
    let Ok(entries) = std::fs::read_dir(&sessions_dir) else {
        return logs;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Ok(sub_entries) = std::fs::read_dir(&path) else {
            continue;
        };
        for sub in sub_entries.flatten() {
            let sub_path = sub.path();
            if !sub_path.is_dir() {
                continue;
            }
            let agents_dir = sub_path.join("agents");
            let Ok(agent_entries) = std::fs::read_dir(&agents_dir) else {
                continue;
            };
            for agent in agent_entries.flatten() {
                let agent_path = agent.path();
                if !agent_path.is_dir() {
                    continue;
                }
                let wire = agent_path.join("wire.jsonl");
                if wire.is_file() {
                    logs.push(wire);
                }
            }
        }
    }

    logs
}

pub fn load_usage(base_dir: &Path) -> AggregatedUsage {
    let logs = discover_wire_logs(base_dir);
    let mut aggregated = AggregatedUsage::default();
    let today = Local::now().date_naive();
    let week_ago = today - chrono::Duration::days(6);

    for log in logs {
        let Ok(file) = File::open(&log) else {
            continue;
        };
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let Ok(line) = line else { continue };
            let Ok(event) = serde_json::from_str::<WireEvent>(&line) else {
                continue;
            };
            if event.event_type != "usage.record" {
                continue;
            }
            let Some(record) = event.usage else { continue };

            aggregated.total_input_other += record.input_other;
            aggregated.total_output += record.output;
            aggregated.total_input_cache_read += record.input_cache_read;
            aggregated.total_input_cache_creation += record.input_cache_creation;

            let record_total = record.total();
            if let Some(ts_ms) = event.time {
                let dt: DateTime<Utc> =
                    DateTime::from_timestamp_millis(ts_ms).unwrap_or_else(Utc::now);
                let date = dt.with_timezone(&Local).date_naive();
                *aggregated.daily_totals.entry(date).or_insert(0) += record_total;
                if date == today {
                    aggregated.today_total += record_total;
                }
                if date >= week_ago && date <= today {
                    aggregated.last_7_days_total += record_total;
                }
            }
        }
    }

    aggregated
}

pub fn format_number(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.2}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

pub fn kimi_code_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".kimi-code"))
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Write;
    use tempfile::TempDir;

    fn write_wire_log(dir: &Path, lines: &[String]) {
        std::fs::create_dir_all(dir).unwrap();
        let mut file = File::create(dir.join("wire.jsonl")).unwrap();
        for line in lines {
            writeln!(file, "{}", line).unwrap();
        }
    }

    #[test]
    fn aggregates_usage_from_wire_jsonl() {
        let base = TempDir::new().unwrap();
        let session = base
            .path()
            .join("sessions/session-1/agent-1/agents/agent-1");
        let ts = Local::now().timestamp_millis();
        write_wire_log(
            &session,
            &[
                r#"{"type":"usage.record","time":TS,"usage":{"input_other":100,"output":50,"inputCacheRead":25,"inputCacheCreation":5}}"#.replace("TS", &ts.to_string()),
                r#"{"type":"other.event","time":TS}"#.replace("TS", &ts.to_string()),
                r#"{"type":"usage.record","time":TS,"usage":{"input_other":10,"output":5,"inputCacheRead":0,"inputCacheCreation":0}}"#.replace("TS", &ts.to_string()),
            ],
        );

        let usage = load_usage(base.path());
        assert_eq!(usage.total_input_other, 110);
        assert_eq!(usage.total_output, 55);
        assert_eq!(usage.total_input_cache_read, 25);
        assert_eq!(usage.total_input_cache_creation, 5);
        assert_eq!(usage.total_tokens(), 195);
        assert!(usage.today_total > 0);
        assert!(usage.last_7_days_total > 0);
    }

    #[test]
    #[ignore = "requires local ~/.kimi-code wire logs"]
    fn parses_actual_kimi_usage() {
        let dir = kimi_code_dir().expect("home directory available");
        let usage = load_usage(&dir);
        println!("total tokens: {}", usage.total_tokens());
        println!("today: {}", usage.today_total);
        assert!(
            usage.total_tokens() > 0,
            "expected some usage to be recorded"
        );
    }
}
