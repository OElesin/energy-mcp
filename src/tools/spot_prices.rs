use serde_json::Value;
use crate::awattar;
use crate::mcp::ToolCallResult;
use chrono::Utc;

pub async fn execute(params: &Value) -> ToolCallResult {
    let hours_ahead = params.get("hours_ahead")
        .and_then(|v| v.as_i64())
        .unwrap_or(48)
        .min(72) as i64;

    let now = Utc::now();
    let start_ms = now.timestamp_millis();
    let end_ms = (now + chrono::Duration::hours(hours_ahead)).timestamp_millis();

    match awattar::fetch_spot_prices(Some(start_ms), Some(end_ms)).await {
        Ok(data) => {
            let json = serde_json::to_string_pretty(&data).unwrap_or_default();
            ToolCallResult::text(json)
        }
        Err(e) => ToolCallResult::error(format!("Failed to fetch spot prices: {}", e)),
    }
}
