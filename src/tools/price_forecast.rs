use serde_json::Value;
use crate::awattar;
use crate::mcp::ToolCallResult;
use chrono::{Utc, Duration, Datelike, NaiveDate, NaiveTime, NaiveDateTime};

pub async fn execute(_params: &Value) -> ToolCallResult {
    // Calculate tomorrow's start and end timestamps
    let now = Utc::now();
    let tomorrow = now.date_naive() + Duration::days(1);
    let tomorrow_start = NaiveDateTime::new(tomorrow, NaiveTime::from_hms_opt(0, 0, 0).unwrap());
    let tomorrow_end = NaiveDateTime::new(tomorrow, NaiveTime::from_hms_opt(23, 59, 59).unwrap());

    let start_ms = tomorrow_start.and_utc().timestamp_millis();
    let end_ms = tomorrow_end.and_utc().timestamp_millis();

    match awattar::fetch_spot_prices(Some(start_ms), Some(end_ms)).await {
        Ok(data) => {
            let result = serde_json::json!({
                "date": tomorrow.to_string(),
                "country": data.country,
                "currency": data.currency,
                "prices_available": !data.prices.is_empty(),
                "note": if data.prices.is_empty() {
                    "Day-ahead prices are published daily at 14:00 CET. If empty, prices for tomorrow are not yet available."
                } else {
                    "Day-ahead prices for tomorrow."
                },
                "prices": data.prices,
                "stats": data.stats
            });
            ToolCallResult::text(serde_json::to_string_pretty(&result).unwrap_or_default())
        }
        Err(e) => ToolCallResult::error(format!("Failed to fetch forecast: {}", e)),
    }
}
