use serde::Serialize;
use serde_json::Value;
use chrono::{Utc, Duration};
use crate::entsoe;
use crate::mcp::ToolCallResult;

#[derive(Debug, Serialize)]
struct CarbonIntensityResult {
    country: String,
    timestamp: String,
    carbon_intensity_gco2_kwh: f64,
    rating: String,
    renewable_share_percent: f64,
    breakdown: Vec<SourceContribution>,
    context: CarbonContext,
}

#[derive(Debug, Serialize)]
struct SourceContribution {
    source: String,
    generation_mw: f64,
    share_percent: f64,
    emission_factor_gco2_kwh: f64,
    contribution_gco2_kwh: f64,
}

#[derive(Debug, Serialize)]
struct CarbonContext {
    eu_average_gco2_kwh: f64,
    rating_scale: String,
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
                    "No generation data available for {} to calculate carbon intensity",
                    country
                ));
            }

            let total_mw: f64 = sources.iter().map(|s| s.generation_mw).sum();
            let renewable_mw: f64 = sources.iter()
                .filter(|s| s.is_renewable)
                .map(|s| s.generation_mw)
                .sum();

            // Calculate weighted carbon intensity
            let mut total_emissions: f64 = 0.0;
            let mut breakdown: Vec<SourceContribution> = Vec::new();

            for source in &sources {
                let ef = entsoe::emission_factor(&source.psr_type);
                let contribution = source.generation_mw / total_mw * ef;
                total_emissions += contribution;

                breakdown.push(SourceContribution {
                    source: source.source.clone(),
                    generation_mw: source.generation_mw,
                    share_percent: source.share_percent,
                    emission_factor_gco2_kwh: ef,
                    contribution_gco2_kwh: (contribution * 100.0).round() / 100.0,
                });
            }

            // Sort by contribution (highest emitters first)
            breakdown.sort_by(|a, b| b.contribution_gco2_kwh.partial_cmp(&a.contribution_gco2_kwh).unwrap());

            let carbon_intensity = (total_emissions * 10.0).round() / 10.0;
            let rating = match carbon_intensity as u32 {
                0..=50 => "Very Low",
                51..=150 => "Low",
                151..=300 => "Medium",
                301..=500 => "High",
                _ => "Very High",
            };

            let result = CarbonIntensityResult {
                country,
                timestamp: now.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                carbon_intensity_gco2_kwh: carbon_intensity,
                rating: rating.into(),
                renewable_share_percent: ((renewable_mw / total_mw * 100.0) * 10.0).round() / 10.0,
                breakdown,
                context: CarbonContext {
                    eu_average_gco2_kwh: 230.0,
                    rating_scale: "Very Low (<50) | Low (50-150) | Medium (150-300) | High (300-500) | Very High (>500)".into(),
                },
            };

            ToolCallResult::text(serde_json::to_string_pretty(&result).unwrap_or_default())
        }
        Err(e) => ToolCallResult::error(e),
    }
}
