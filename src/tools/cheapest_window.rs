use serde_json::Value;
use crate::awattar;
use crate::mcp::ToolCallResult;

pub async fn execute(params: &Value) -> ToolCallResult {
    let duration_hours = match params.get("duration_hours").and_then(|v| v.as_u64()) {
        Some(h) if h > 0 && h <= 24 => h as usize,
        _ => return ToolCallResult::error("duration_hours is required (1-24)".into()),
    };

    let consumption_kwh = params.get("consumption_kwh")
        .and_then(|v| v.as_f64())
        .unwrap_or(1.0);

    // Fetch next 48 hours of prices
    match awattar::fetch_spot_prices(None, None).await {
        Ok(data) => {
            match awattar::find_cheapest_window(&data.prices, duration_hours, consumption_kwh) {
                Some(window) => {
                    let json = serde_json::to_string_pretty(&window).unwrap_or_default();
                    ToolCallResult::text(json)
                }
                None => ToolCallResult::error("Not enough price data to find a window of that duration".into()),
            }
        }
        Err(e) => ToolCallResult::error(format!("Failed to fetch prices: {}", e)),
    }
}
