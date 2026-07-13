use serde::Serialize;
use serde_json::Value;
use chrono::{Utc, Duration};
use crate::entsoe;
use crate::mcp::ToolCallResult;

#[derive(Debug, Serialize)]
struct GenerationMixResult {
    country: String,
    timestamp: String,
    total_generation_mw: f64,
    renewable_share_percent: f64,
    fossil_share_percent: f64,
    sources: Vec<entsoe::GenerationSource>,
}

pub async fn execute(params: &Value) -> ToolCallResult {
    let country = params.get("country")
        .and_then(|v| v.as_str())
        .unwrap_or("DE")
        .to_uppercase();

    let now = Utc::now();
    let start = now.format("%Y%m%d0000").to_string();
    let end = (now + Duration::days(1)).format("%Y%m%d0000").to_string();

    match entsoe::fetch_generation(&country, &start, &end).await {
        Ok(sources) => {
            if sources.is_empty() {
                return ToolCallResult::error(format!(
                    "No generation data available for {}. Try: DE, FR, NL, BE, AT, ES, PL, CZ, FI, SE3",
                    country
                ));
            }

            let total: f64 = sources.iter().map(|s| s.generation_mw).sum();
            let renewable: f64 = sources.iter()
                .filter(|s| s.is_renewable)
                .map(|s| s.generation_mw)
                .sum();
            let fossil: f64 = sources.iter()
                .filter(|s| !s.is_renewable && !matches!(s.psr_type.as_str(), "B14"))
                .map(|s| s.generation_mw)
                .sum();

            let result = GenerationMixResult {
                country,
                timestamp: now.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                total_generation_mw: (total * 10.0).round() / 10.0,
                renewable_share_percent: ((renewable / total * 100.0) * 10.0).round() / 10.0,
                fossil_share_percent: ((fossil / total * 100.0) * 10.0).round() / 10.0,
                sources,
            };

            ToolCallResult::text(serde_json::to_string_pretty(&result).unwrap_or_default())
        }
        Err(e) => ToolCallResult::error(e),
    }
}
