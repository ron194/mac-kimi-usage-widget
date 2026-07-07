use crate::usage::{self, DetailedUsage, SessionUsage};
use chrono::NaiveDate;
use std::collections::HashMap;
use std::path::Path;

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

fn format_number(n: u64) -> String {
    usage::format_number(n)
}

fn format_number_full(n: u64) -> String {
    n.to_string()
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(std::str::from_utf8)
        .collect::<Result<Vec<_>, _>>()
        .unwrap_or_default()
        .join(",")
}

fn percent(part: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        (part as f64 / total as f64) * 100.0
    }
}

fn build_daily_rows(daily_totals: &HashMap<NaiveDate, u64>) -> String {
    let mut dates: Vec<&NaiveDate> = daily_totals.keys().collect();
    dates.sort();

    let mut rows = String::new();
    for date in dates {
        let total = daily_totals[date];
        rows.push_str(&format!(
            "<tr><td>{}</td><td class=\"num\">{}</td><td class=\"num\">{}</td></tr>\n",
            date,
            format_number_full(total),
            format_number(total)
        ));
    }
    rows
}

fn build_session_rows(sessions: &[SessionUsage]) -> String {
    let mut rows = String::new();
    for session in sessions {
        for agent in &session.agents {
            rows.push_str(&format!(
                "<tr>
                <td>{}</td>
                <td>{}</td>
                <td class=\"num\">{}</td>
                <td class=\"num\">{}</td>
                <td class=\"num\">{}</td>
                <td class=\"num\">{}</td>
                <td class=\"num\">{}</td>
                </tr>\n",
                escape_html(&session.session_id),
                escape_html(&agent.agent_id),
                format_number_full(agent.categories.total()),
                format_number(agent.categories.total()),
                format_number_full(agent.categories.output),
                format_number_full(agent.categories.input_other),
                format_number_full(
                    agent.categories.input_cache_read + agent.categories.input_cache_creation
                )
            ));
        }
    }
    rows
}

fn build_bar_chart(daily_totals: &HashMap<NaiveDate, u64>) -> String {
    if daily_totals.is_empty() {
        return "<p class=\"muted\">No daily data available.</p>".to_string();
    }

    let mut dates: Vec<&NaiveDate> = daily_totals.keys().collect();
    dates.sort();
    let max = dates
        .iter()
        .map(|d| daily_totals[d])
        .max()
        .unwrap_or(1)
        .max(1);

    let mut bars = String::new();
    for date in dates {
        let value = daily_totals[date];
        let pct = (value as f64 / max as f64) * 100.0;
        let label = date.format("%m-%d").to_string();
        bars.push_str(&format!(
            "<div class=\"bar\" style=\"height:{pct:.1}%\" title=\"{date}: {full} ({short})\">
                <span class=\"bar-label\">{label}</span>
                <span class=\"bar-value\">{short}</span>
            </div>\n",
            pct = pct,
            date = date,
            full = format_number_full(value),
            short = format_number(value),
            label = label
        ));
    }

    format!("<div class=\"bar-chart\">{}</div>\n", bars)
}

pub fn generate_html_report(usage: &DetailedUsage) -> String {
    let total = usage.total_tokens();
    let cat = &usage.categories;

    let daily_rows = build_daily_rows(&usage.daily_totals);
    let session_rows = build_session_rows(&usage.sessions);
    let chart = build_bar_chart(&usage.daily_totals);

    let generated = chrono::Local::now()
        .format("%Y-%m-%d %H:%M:%S %z")
        .to_string();

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Kimi Code Token Usage Report</title>
<style>
:root {{ color-scheme: light dark; }}
body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif; margin: 0; padding: 2rem; background: #f5f5f7; color: #1d1d1f; }}
.container {{ max-width: 960px; margin: 0 auto; }}
h1 {{ font-size: 1.75rem; margin-bottom: 0.25rem; }}
.muted {{ color: #6e6e73; font-size: 0.9rem; }}
.cards {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(160px, 1fr)); gap: 1rem; margin: 1.5rem 0; }}
.card {{ background: white; border-radius: 12px; padding: 1rem; box-shadow: 0 1px 3px rgba(0,0,0,0.08); }}
.card h3 {{ margin: 0 0 0.5rem; font-size: 0.85rem; color: #6e6e73; text-transform: uppercase; letter-spacing: 0.03em; }}
.card .value {{ font-size: 1.5rem; font-weight: 600; }}
.card .sub {{ font-size: 0.85rem; color: #6e6e73; margin-top: 0.25rem; }}
section {{ background: white; border-radius: 12px; padding: 1.25rem; margin: 1.25rem 0; box-shadow: 0 1px 3px rgba(0,0,0,0.08); }}
h2 {{ font-size: 1.2rem; margin-top: 0; }}
table {{ width: 100%; border-collapse: collapse; margin-top: 0.75rem; }}
th, td {{ text-align: left; padding: 0.55rem 0.5rem; border-bottom: 1px solid #e5e5e5; }}
th {{ font-weight: 600; font-size: 0.85rem; color: #6e6e73; }}
td.num, th.num {{ text-align: right; }}
tr:hover {{ background: #f9f9fb; }}
.bar-chart {{ display: flex; align-items: flex-end; gap: 0.35rem; height: 220px; padding: 1rem 0 2.2rem; border-bottom: 1px solid #d1d1d6; overflow-x: auto; }}
.bar {{ flex: 1 0 24px; min-width: 24px; background: #007aff; border-radius: 4px 4px 0 0; position: relative; display: flex; flex-direction: column; justify-content: flex-end; align-items: center; }}
.bar-label {{ position: absolute; bottom: -1.4rem; font-size: 0.7rem; color: #6e6e73; transform: rotate(-35deg); transform-origin: left top; white-space: nowrap; }}
.bar-value {{ position: absolute; top: -1.3rem; font-size: 0.7rem; color: #1d1d1f; }}
.progress {{ background: #e5e5e5; border-radius: 6px; height: 8px; margin-top: 0.5rem; overflow: hidden; }}
.progress-fill {{ background: #34c759; height: 100%; border-radius: 6px; }}
@media (prefers-color-scheme: dark) {{
  body {{ background: #1c1c1e; color: #f2f2f7; }}
  .card, section {{ background: #2c2c2e; box-shadow: none; }}
  tr:hover {{ background: #3a3a3c; }}
  th, td {{ border-bottom-color: #38383a; }}
  .bar-value {{ color: #f2f2f7; }}
  .progress {{ background: #3a3a3c; }}
}}
</style>
</head>
<body>
<div class="container">
<h1>Kimi Code Token Usage Report</h1>
<p class="muted">Generated on {generated}</p>

<div class="cards">
  <div class="card"><h3>Total Tokens</h3><div class="value">{total_short}</div><div class="sub">{total_full}</div></div>
  <div class="card"><h3>Output</h3><div class="value">{output_short}</div><div class="sub">{output_pct:.1}%</div></div>
  <div class="card"><h3>Input (other)</h3><div class="value">{input_short}</div><div class="sub">{input_pct:.1}%</div></div>
  <div class="card"><h3>Cache Read</h3><div class="value">{cache_read_short}</div><div class="sub">{cache_read_pct:.1}%</div></div>
  <div class="card"><h3>Cache Creation</h3><div class="value">{cache_creation_short}</div><div class="sub">{cache_creation_pct:.1}%</div></div>
</div>

<section>
  <h2>Recent Activity</h2>
  <div class="cards">
    <div class="card"><h3>Today</h3><div class="value">{today_short}</div><div class="sub">{today_full} tokens</div></div>
    <div class="card"><h3>Last 7 Days</h3><div class="value">{week_short}</div><div class="sub">{week_full} tokens</div></div>
  </div>
</section>

<section>
  <h2>Daily Totals</h2>
  {chart}
  <table>
    <thead><tr><th>Date</th><th class="num">Tokens</th><th class="num">Short</th></tr></thead>
    <tbody>{daily_rows}</tbody>
  </table>
</section>

<section>
  <h2>Usage by Session &amp; Agent</h2>
  <table>
    <thead>
      <tr>
        <th>Session</th><th>Agent</th><th class="num">Total</th>
        <th class="num">Total (short)</th><th class="num">Output</th>
        <th class="num">Input</th><th class="num">Cache</th>
      </tr>
    </thead>
    <tbody>{session_rows}</tbody>
  </table>
</section>

</div>
</body>
</html>"#,
        generated = generated,
        total_short = format_number(total),
        total_full = format_number_full(total),
        output_short = format_number(cat.output),
        output_pct = percent(cat.output, total),
        input_short = format_number(cat.input_other),
        input_pct = percent(cat.input_other, total),
        cache_read_short = format_number(cat.input_cache_read),
        cache_read_pct = percent(cat.input_cache_read, total),
        cache_creation_short = format_number(cat.input_cache_creation),
        cache_creation_pct = percent(cat.input_cache_creation, total),
        today_short = format_number(usage.today_total),
        today_full = format_number_full(usage.today_total),
        week_short = format_number(usage.last_7_days_total),
        week_full = format_number_full(usage.last_7_days_total),
        chart = chart,
        daily_rows = daily_rows,
        session_rows = session_rows,
    )
}

fn report_path() -> Option<std::path::PathBuf> {
    dirs::cache_dir().map(|dir| dir.join("kimi-usage-widget").join("usage-report.html"))
}

fn write_report_to_disk(html: &str) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let path = report_path().ok_or("could not determine cache directory")?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, html)?;
    Ok(path)
}

fn open_file(path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(path).spawn()?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(path).spawn()?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args([
                "/c",
                "start",
                "",
                path.as_os_str().to_string_lossy().as_ref(),
            ])
            .spawn()?;
    }
    Ok(())
}

pub fn open_usage_report(base_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let usage = usage::load_detailed_usage(base_dir);
    let html = generate_html_report(&usage);
    let path = write_report_to_disk(&html)?;
    open_file(&path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn write_wire_log(dir: &Path, lines: &[String]) {
        std::fs::create_dir_all(dir).unwrap();
        let mut file = std::fs::File::create(dir.join("wire.jsonl")).unwrap();
        for line in lines {
            writeln!(file, "{}", line).unwrap();
        }
    }

    #[test]
    fn generates_html_report() {
        let base = TempDir::new().unwrap();
        let session = base
            .path()
            .join("sessions/session-1/agent-1/agents/agent-1");
        let ts = chrono::Local::now().timestamp_millis();
        write_wire_log(
            &session,
            &[
                r#"{"type":"usage.record","time":TS,"usage":{"input_other":100,"output":50,"inputCacheRead":25,"inputCacheCreation":5}}"#
                    .replace("TS", &ts.to_string()),
                r#"{"type":"usage.record","time":TS,"usage":{"input_other":10,"output":5,"inputCacheRead":0,"inputCacheCreation":0}}"#
                    .replace("TS", &ts.to_string()),
            ],
        );

        let usage = usage::load_detailed_usage(base.path());
        let html = generate_html_report(&usage);

        assert!(html.contains("Kimi Code Token Usage Report"));
        assert!(html.contains("Total Tokens"));
        assert!(html.contains(&format_number(usage.total_tokens())));
        assert!(html.contains("Daily Totals"));
        assert!(html.contains("Usage by Session &amp; Agent"));
        assert!(html.contains("session-1"));
        assert!(html.contains("agent-1"));
    }

    #[test]
    fn generates_empty_report() {
        let base = TempDir::new().unwrap();
        let usage = usage::load_detailed_usage(base.path());
        let html = generate_html_report(&usage);
        assert!(html.contains("No daily data available"));
        assert!(html.contains("Total Tokens"));
    }

    #[test]
    fn writes_report_to_disk() {
        let html = "<html><body>test</body></html>";
        let path = write_report_to_disk(html).unwrap();
        assert!(path.exists());
        let contents = std::fs::read_to_string(&path).unwrap();
        assert_eq!(contents, html);
    }
}
