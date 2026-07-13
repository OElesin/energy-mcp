use serde::Serialize;
use serde_json::Value;
use chrono::{Utc, Duration};
use crate::entsoe;
use crate::mcp::ToolCallResult;

#[derive(Debug, Serialize)]
struct EnergyPriceResult {
    country: String,
    period_start: String,
    period_end: String,
    currency: String,
    prices: Vec<entsoe::PricePoint>,
    stats: PriceStats,
}

#[derive(Debug, Serialize)]
struct PriceStats {
    count: usize,
    avg_eur_mwh: f64,
    avg_ct_kwh: f64,
    min_eur_mwh: f64,
    max_eur_mwh: f64,
    min_hour: String,
    max_hour: String,
}

pub async fn execute(params: &Value) -> ToolCallResult {
    let country = params.get("country")
        .and_then(|v| v.as_str())
        .unwrap_or("DE")
        .to_uppercase();

    // Date range: default to today
    let now = Utc::now();
    let start = params.get("date")
        .and_then(|v| v.as_str())
        .map(|d| format!("{}0000", d.replace("-", "")))
        .unwrap_or_else(|| now.format("%Y%m%d0000").to_string());

    let end = params.get("date_end")
        .and_then(|v| v.as_str())
        .map(|d| format!("{}0000", d.replace("-", "")))
        .unwrap_or_else(|| {
            // Default end: start + 1 day
            let end_date = now + Duration::days(1);
            end_date.format("%Y%m%d0000").to_string()
        });

    match entsoe::fetch_prices(&country, &start, &end).await {
        Ok(prices) => {
            if prices.is_empty() {
                return ToolCallResult::error(format!(
                    "No price data available for {} in the requested period. Available countries: DE, FR, NL, BE, AT, ES, PT, IT, CH, PL, CZ, DK1, DK2, NO1, SE3, FI, EE, LV, LT, HU, RO, BG, GR, IE, GB",
                    country
                ));
            }

            let stats = compute_stats(&prices);

            let result = EnergyPriceResult {
                country,
                period_start: start,
                period_end: end,
                currency: "EUR".into(),
                prices,
                stats,
            };

            ToolCallResult::text(serde_json::to_string_pretty(&result).unwrap_or_default())
        }
        Err(e) => ToolCallResult::error(e),
    }
}

fn compute_stats(prices: &[entsoe::PricePoint]) -> PriceStats {
    let sum: f64 = prices.iter().map(|p| p.price_eur_mwh).sum();
    let avg = sum / prices.len() as f64;
    let min = prices.iter().min_by(|a, b| a.price_eur_mwh.partial_cmp(&b.price_eur_mwh).unwrap()).unwrap();
    let max = prices.iter().max_by(|a, b| a.price_eur_mwh.partial_cmp(&b.price_eur_mwh).unwrap()).unwrap();

    PriceStats {
        count: prices.len(),
        avg_eur_mwh: (avg * 100.0).round() / 100.0,
        avg_ct_kwh: (avg / 10.0 * 100.0).round() / 100.0,
        min_eur_mwh: min.price_eur_mwh,
        max_eur_mwh: max.price_eur_mwh,
        min_hour: min.timestamp.clone(),
        max_hour: max.timestamp.clone(),
    }
}
