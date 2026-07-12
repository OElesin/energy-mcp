use serde_json::Value;
use crate::awattar;
use crate::mcp::ToolCallResult;

pub async fn execute(_params: &Value) -> ToolCallResult {
    match awattar::fetch_spot_prices(None, None).await {
        Ok(data) => {
            let json = serde_json::to_string_pretty(&data.stats).unwrap_or_default();
            ToolCallResult::text(json)
        }
        Err(e) => ToolCallResult::error(format!("Failed to fetch price stats: {}", e)),
    }
}
