use serde::Serialize;
use serde_json::Value;
use crate::awattar;
use crate::mcp::ToolCallResult;
use chrono::{Utc, Duration};

#[derive(Debug, Serialize)]
struct EnergyCostResult {
    consumption_kwh: f64,
    current_period: PeriodCost,
    comparison: Option<ComparisonResult>,
}

#[derive(Debug, Serialize)]
struct PeriodCost {
    label: String,
    avg_price_ct_kwh: f64,
    total_cost_eur: f64,
    cheapest_hour_ct: f64,
    most_expensive_hour_ct: f64,
}

#[derive(Debug, Serialize)]
struct ComparisonResult {
    label: String,
    avg_price_ct_kwh: f64,
    total_cost_eur: f64,
    delta_eur: f64,
    delta_percent: f64,
    note: String,
}

pub async fn execute(params: &Value) -> ToolCallResult {
    let consumption_kwh = params.get("consumption_kwh")
        .and_then(|v| v.as_f64())
        .unwrap_or(1.0);

    let include_comparison = params.get("compare_last_week")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    // Fetch current prices
    let current = match awattar::fetch_spot_prices(None, None).await {
        Ok(data) => data,
        Err(e) => return ToolCallResult::error(format!("Failed to fetch current prices: {}", e)),
    };

    if current.prices.is_empty() {
        return ToolCallResult::error("No current price data available".into());
    }

    let current_avg = current.stats.avg_ct_kwh;
    let current_cost = consumption_kwh * current_avg / 100.0;

    let current_period = PeriodCost {
        label: "Current period (today/tomorrow)".into(),
        avg_price_ct_kwh: current_avg,
        total_cost_eur: (current_cost * 100.0).round() / 100.0,
        cheapest_hour_ct: current.stats.min_ct_kwh,
        most_expensive_hour_ct: current.stats.max_ct_kwh,
    };

    // Fetch last week's prices for comparison
    let comparison = if include_comparison {
        let now = Utc::now();
        let week_ago_start = (now - Duration::days(7)).timestamp_millis();
        let week_ago_end = (now - Duration::days(6)).timestamp_millis();

        match awattar::fetch_spot_prices(Some(week_ago_start), Some(week_ago_end)).await {
            Ok(past_data) if !past_data.prices.is_empty() => {
                let past_avg = past_data.stats.avg_ct_kwh;
                let past_cost = consumption_kwh * past_avg / 100.0;
                let delta = current_cost - past_cost;
                let delta_pct = if past_cost > 0.0 { delta / past_cost * 100.0 } else { 0.0 };

                Some(ComparisonResult {
                    label: "Same day last week".into(),
                    avg_price_ct_kwh: past_avg,
                    total_cost_eur: (past_cost * 100.0).round() / 100.0,
                    delta_eur: (delta * 100.0).round() / 100.0,
                    delta_percent: (delta_pct * 10.0).round() / 10.0,
                    note: if delta > 0.0 {
                        format!("Prices are {:.1}% higher than last week", delta_pct)
                    } else {
                        format!("Prices are {:.1}% lower than last week", -delta_pct)
                    },
                })
            }
            _ => None,
        }
    } else {
        None
    };

    let result = EnergyCostResult {
        consumption_kwh,
        current_period,
        comparison,
    };

    ToolCallResult::text(serde_json::to_string_pretty(&result).unwrap_or_default())
}
