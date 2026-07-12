use serde::Serialize;
use serde_json::Value;
use crate::awattar;
use crate::mcp::ToolCallResult;
use chrono::{Utc, Duration};

#[derive(Debug, Serialize)]
struct PriceTrends {
    reference_date: String,
    current_week: WeekStats,
    prior_week: WeekStats,
    week_over_week: WoWComparison,
    daily_averages: Vec<DailyAvg>,
}

#[derive(Debug, Serialize)]
struct WeekStats {
    label: String,
    avg_ct_kwh: f64,
    min_ct_kwh: f64,
    max_ct_kwh: f64,
    volatility_ct: f64,
}

#[derive(Debug, Serialize)]
struct WoWComparison {
    delta_ct_kwh: f64,
    delta_percent: f64,
    trend: String,
}

#[derive(Debug, Serialize)]
struct DailyAvg {
    date: String,
    avg_ct_kwh: f64,
    min_ct_kwh: f64,
    max_ct_kwh: f64,
}

pub async fn execute(_params: &Value) -> ToolCallResult {
    let now = Utc::now();

    // Fetch current week (last 7 days)
    let current_start = (now - Duration::days(7)).timestamp_millis();
    let current_end = now.timestamp_millis();

    // Fetch prior week (7-14 days ago)
    let prior_start = (now - Duration::days(14)).timestamp_millis();
    let prior_end = (now - Duration::days(7)).timestamp_millis();

    let current_data = match awattar::fetch_spot_prices(Some(current_start), Some(current_end)).await {
        Ok(d) => d,
        Err(e) => return ToolCallResult::error(format!("Failed to fetch current week: {}", e)),
    };

    let prior_data = match awattar::fetch_spot_prices(Some(prior_start), Some(prior_end)).await {
        Ok(d) => d,
        Err(e) => return ToolCallResult::error(format!("Failed to fetch prior week: {}", e)),
    };

    if current_data.prices.is_empty() {
        return ToolCallResult::error("No current price data available".into());
    }

    // Calculate stats
    let current_prices: Vec<f64> = current_data.prices.iter().map(|p| p.price_ct_kwh).collect();
    let prior_prices: Vec<f64> = prior_data.prices.iter().map(|p| p.price_ct_kwh).collect();

    let current_avg = avg(&current_prices);
    let prior_avg = avg(&prior_prices);

    let current_week = WeekStats {
        label: "Current week (last 7 days)".into(),
        avg_ct_kwh: round2(current_avg),
        min_ct_kwh: current_data.stats.min_ct_kwh,
        max_ct_kwh: current_data.stats.max_ct_kwh,
        volatility_ct: round2(std_dev(&current_prices)),
    };

    let prior_week = WeekStats {
        label: "Prior week (7-14 days ago)".into(),
        avg_ct_kwh: round2(prior_avg),
        min_ct_kwh: if prior_prices.is_empty() { 0.0 } else { prior_prices.iter().cloned().fold(f64::INFINITY, f64::min) },
        max_ct_kwh: if prior_prices.is_empty() { 0.0 } else { prior_prices.iter().cloned().fold(f64::NEG_INFINITY, f64::max) },
        volatility_ct: round2(std_dev(&prior_prices)),
    };

    let delta = current_avg - prior_avg;
    let delta_pct = if prior_avg > 0.0 { delta / prior_avg * 100.0 } else { 0.0 };
    let trend = if delta > 0.5 { "rising" } else if delta < -0.5 { "falling" } else { "stable" };

    let wow = WoWComparison {
        delta_ct_kwh: round2(delta),
        delta_percent: round2(delta_pct),
        trend: trend.into(),
    };

    // Daily averages for the current week
    let daily_averages = compute_daily_averages(&current_data.prices);

    let result = PriceTrends {
        reference_date: now.format("%Y-%m-%d").to_string(),
        current_week,
        prior_week,
        week_over_week: wow,
        daily_averages,
    };

    ToolCallResult::text(serde_json::to_string_pretty(&result).unwrap_or_default())
}

fn compute_daily_averages(prices: &[awattar::HourlyPrice]) -> Vec<DailyAvg> {
    use std::collections::BTreeMap;

    let mut by_day: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    for p in prices {
        let date = p.start[..10].to_string(); // "2026-07-12"
        by_day.entry(date).or_default().push(p.price_ct_kwh);
    }

    by_day.into_iter().map(|(date, vals)| {
        DailyAvg {
            date,
            avg_ct_kwh: round2(avg(&vals)),
            min_ct_kwh: round2(vals.iter().cloned().fold(f64::INFINITY, f64::min)),
            max_ct_kwh: round2(vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max)),
        }
    }).collect()
}

fn avg(vals: &[f64]) -> f64 {
    if vals.is_empty() { return 0.0; }
    vals.iter().sum::<f64>() / vals.len() as f64
}

fn std_dev(vals: &[f64]) -> f64 {
    if vals.len() < 2 { return 0.0; }
    let mean = avg(vals);
    let variance = vals.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / vals.len() as f64;
    variance.sqrt()
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}
