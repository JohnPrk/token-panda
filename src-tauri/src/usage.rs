use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const FIVE_HOUR_GAP_THRESHOLD: i64 = 5 * 3600;
const WEEKLY_LOOKBACK_DAYS: i64 = 7;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct UsageEntry {
    pub timestamp: DateTime<Utc>,
    pub tokens: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct UsageSnapshot {
    pub five_hour_tokens: u64,
    pub weekly_tokens: u64,
    pub last_request_at: Option<DateTime<Utc>>,
    /// First message timestamp of the current 5h window (or None if no recent activity).
    pub five_hour_window_start: Option<DateTime<Utc>>,
    /// When the current 5h window expires (window_start + 5h).
    pub five_hour_resets_at: Option<DateTime<Utc>>,
    /// First message timestamp of the current 7d weekly window.
    pub weekly_window_start: Option<DateTime<Utc>>,
    pub weekly_resets_at: Option<DateTime<Utc>>,
    pub now: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct RawLine {
    timestamp: Option<String>,
    message: Option<RawMessage>,
}

#[derive(Debug, Deserialize)]
struct RawMessage {
    role: Option<String>,
    usage: Option<RawUsage>,
}

#[derive(Debug, Deserialize)]
struct RawUsage {
    input_tokens: Option<u64>,
    output_tokens: Option<u64>,
    cache_creation_input_tokens: Option<u64>,
    // cache_read_input_tokens intentionally ignored — billed at 0.1× and
    // including it inflated counts ~10× in real usage tests.
}

pub fn claude_projects_dir() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let p = home.join(".claude").join("projects");
    if p.exists() { Some(p) } else { None }
}

pub fn collect_entries_since(since: DateTime<Utc>) -> Vec<UsageEntry> {
    let Some(root) = claude_projects_dir() else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for entry in WalkDir::new(&root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("jsonl"))
    {
        if let Some(modified) = entry
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .map(DateTime::<Utc>::from)
        {
            if modified < since - Duration::hours(1) {
                continue;
            }
        }
        scan_file(entry.path(), since, &mut out);
    }
    out.sort_by_key(|e| e.timestamp);
    out
}

fn scan_file(path: &Path, since: DateTime<Utc>, out: &mut Vec<UsageEntry>) {
    let Ok(file) = File::open(path) else { return };
    let reader = BufReader::new(file);
    for line in reader.lines().map_while(Result::ok) {
        if !line.contains("\"usage\"") {
            continue;
        }
        let Ok(raw) = serde_json::from_str::<RawLine>(&line) else {
            continue;
        };
        let Some(msg) = raw.message else { continue };
        if msg.role.as_deref() != Some("assistant") {
            continue;
        }
        let Some(u) = msg.usage else { continue };
        let Some(ts_str) = raw.timestamp else { continue };
        let Ok(ts) = DateTime::parse_from_rfc3339(&ts_str) else {
            continue;
        };
        let ts = ts.with_timezone(&Utc);
        if ts < since {
            continue;
        }
        let tokens = u.input_tokens.unwrap_or(0)
            + u.output_tokens.unwrap_or(0)
            + u.cache_creation_input_tokens.unwrap_or(0);
        if tokens == 0 {
            continue;
        }
        out.push(UsageEntry { timestamp: ts, tokens });
    }
}

/// Find the start of the active 5-hour window: the first entry whose preceding
/// gap is ≥ 5h (or the earliest entry overall if no such gap exists in the
/// loaded range). This mirrors how Anthropic's 5h quota window actually rolls.
fn five_hour_window_start(entries: &[UsageEntry], now: DateTime<Utc>) -> Option<DateTime<Utc>> {
    if entries.is_empty() {
        return None;
    }
    let last = entries.last().unwrap();
    // If the most recent entry is older than 5h, no active window.
    if (now - last.timestamp).num_seconds() >= FIVE_HOUR_GAP_THRESHOLD {
        return None;
    }
    // Walk backwards. Window start = first entry where the previous entry was
    // ≥ 5h earlier (or the earliest entry).
    for i in (1..entries.len()).rev() {
        let gap = (entries[i].timestamp - entries[i - 1].timestamp).num_seconds();
        if gap >= FIVE_HOUR_GAP_THRESHOLD {
            return Some(entries[i].timestamp);
        }
    }
    Some(entries[0].timestamp)
}

pub fn snapshot() -> UsageSnapshot {
    let now = Utc::now();
    let lookback = now - Duration::days(WEEKLY_LOOKBACK_DAYS);
    let entries = collect_entries_since(lookback);

    let five_start = five_hour_window_start(&entries, now);
    let five_reset = five_start.map(|s| s + Duration::hours(5));

    let mut five_hour: u64 = 0;
    let mut weekly: u64 = 0;
    let mut last: Option<DateTime<Utc>> = None;
    let mut weekly_first: Option<DateTime<Utc>> = None;
    for e in &entries {
        weekly = weekly.saturating_add(e.tokens);
        if weekly_first.is_none() {
            weekly_first = Some(e.timestamp);
        }
        if let Some(start) = five_start {
            if e.timestamp >= start && e.timestamp <= now {
                five_hour = five_hour.saturating_add(e.tokens);
            }
        }
        last = Some(match last {
            Some(prev) if prev > e.timestamp => prev,
            _ => e.timestamp,
        });
    }

    let weekly_reset = weekly_first.map(|s| s + Duration::days(WEEKLY_LOOKBACK_DAYS));

    UsageSnapshot {
        five_hour_tokens: five_hour,
        weekly_tokens: weekly,
        last_request_at: last,
        five_hour_window_start: five_start,
        five_hour_resets_at: five_reset,
        weekly_window_start: weekly_first,
        weekly_resets_at: weekly_reset,
        now,
    }
}
