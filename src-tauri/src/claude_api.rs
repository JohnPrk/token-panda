use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiUsage {
    /// 0.0 ~ 100.0 (utilization %, NOT remaining)
    pub five_hour_pct: f64,
    pub weekly_pct: f64,
    pub five_hour_resets_at: Option<DateTime<Utc>>,
    pub weekly_resets_at: Option<DateTime<Utc>>,
    pub fetched_at: DateTime<Utc>,
}

#[derive(Deserialize)]
struct RawResponse {
    five_hour: Option<RawWindow>,
    seven_day: Option<RawWindow>,
}

#[derive(Deserialize)]
struct RawWindow {
    utilization: Option<f64>,
    resets_at: Option<String>,
}

/// Hit claude.ai's internal usage endpoint with the user's session cookie.
/// Returns the parsed % + reset timestamps, or an error string suitable for
/// displaying in the UI.
pub fn fetch_usage(org_id: &str, cookie: &str) -> Result<ApiUsage, String> {
    let url = format!("https://claude.ai/api/organizations/{}/usage", org_id);
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("client build: {}", e))?;

    let resp = client
        .get(&url)
        .header("Cookie", cookie)
        .header("Accept", "application/json")
        .header(
            "User-Agent",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 11_0) AppleWebKit/605.1.15",
        )
        .send()
        .map_err(|e| format!("request: {}", e))?;

    let status = resp.status();
    if !status.is_success() {
        return Err(format!(
            "HTTP {}: 세션 쿠키가 만료됐거나 잘못됐을 가능성이 높습니다",
            status.as_u16()
        ));
    }

    let raw: RawResponse = resp
        .json()
        .map_err(|e| format!("응답 파싱 실패: {}", e))?;

    let parse_dt = |s: Option<String>| -> Option<DateTime<Utc>> {
        let s = s?;
        DateTime::parse_from_rfc3339(&s)
            .ok()
            .map(|d| d.with_timezone(&Utc))
    };

    let five_pct = raw
        .five_hour
        .as_ref()
        .and_then(|w| w.utilization)
        .unwrap_or(0.0);
    let weekly_pct = raw
        .seven_day
        .as_ref()
        .and_then(|w| w.utilization)
        .unwrap_or(0.0);
    let five_reset = parse_dt(raw.five_hour.and_then(|w| w.resets_at));
    let weekly_reset = parse_dt(raw.seven_day.and_then(|w| w.resets_at));

    Ok(ApiUsage {
        five_hour_pct: five_pct,
        weekly_pct,
        five_hour_resets_at: five_reset,
        weekly_resets_at: weekly_reset,
        fetched_at: Utc::now(),
    })
}
