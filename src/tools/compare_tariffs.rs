use serde::Serialize;
use serde_json::Value;
use crate::awattar;
use crate::mcp::ToolCallResult;

#[derive(Debug, Serialize)]
struct TariffComparison {
    period_hours: usize,
    consumption_kwh: f64,
    fixed_tariff: FixedTariffResult,
    dynamic_tariff: DynamicTariffResult,
    savings: SavingsResult,
}

#[derive(Debug, Serialize)]
struct FixedTariffResult {
    rate_ct_kwh: f64,
    total_cost_eur: f64,
}

#[derive(Debug, Serialize)]
struct DynamicTariffResult {
    avg_rate_ct_kwh: f64,
    min_rate_ct_kwh: f64,
    max_rate_ct_kwh: f64,
    total_cost_eur: f64,
    note: String,
}

#[derive(Debug, Serialize)]
struct SavingsResult {
    dynamic_saves_eur: f64,
    dynamic_saves_percent: f64,
    recommendation: String,
}

pub async fn execute(params: &Value) -> ToolCallResult {
    let consumption_kwh = params.get("consumption_kwh")
        .and_then(|v| v.as_f64())
        .unwrap_or(300.0); // Default: avg German monthly household consumption

    let fixed_rate_ct = params.get("fixed_rate_ct_kwh")
        .and_then(|v| v.as_f64())
        .unwrap_or(32.0); // Default: avg German fixed tariff ~32 ct/kWh

    // Surcharges added on top of spot price for dynamic tariffs
    // (grid fees, taxes, levies, provider markup)
    let dynamic_surcharge_ct = params.get("dynamic_surcharge_ct_kwh")
        .and_then(|v| v.as_f64())
        .unwrap_or(20.0); // ~20 ct/kWh surcharges typical in Germany

    // Fetch current spot prices
    match awattar::fetch_spot_prices(None, None).await {
        Ok(data) => {
            if data.prices.is_empty() {
                return ToolCallResult::error("No price data available".into());
            }

            let period_hours = data.prices.len();
            let hourly_consumption = consumption_kwh / period_hours as f64;

            // Fixed tariff cost
            let fixed_total = consumption_kwh * fixed_rate_ct / 100.0;

            // Dynamic tariff cost (spot + surcharges)
            let dynamic_rates: Vec<f64> = data.prices.iter()
                .map(|p| p.price_ct_kwh + dynamic_surcharge_ct)
                .collect();

            let dynamic_avg = dynamic_rates.iter().sum::<f64>() / dynamic_rates.len() as f64;
            let dynamic_min = dynamic_rates.iter().cloned().fold(f64::INFINITY, f64::min);
            let dynamic_max = dynamic_rates.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

            // Assume even consumption across hours for simple comparison
            let dynamic_total = consumption_kwh * dynamic_avg / 100.0;

            let savings_eur = fixed_total - dynamic_total;
            let savings_pct = if fixed_total > 0.0 { savings_eur / fixed_total * 100.0 } else { 0.0 };

            let recommendation = if savings_eur > 0.0 {
                format!("Dynamic tariff saves €{:.2}/month. Worth switching if you can shift consumption to cheap hours.", savings_eur)
            } else {
                format!("Fixed tariff is cheaper by €{:.2}/month with current prices. Dynamic tariff only pays off if you shift load to cheap hours.", -savings_eur)
            };

            let comparison = TariffComparison {
                period_hours,
                consumption_kwh,
                fixed_tariff: FixedTariffResult {
                    rate_ct_kwh: (fixed_rate_ct * 100.0).round() / 100.0,
                    total_cost_eur: (fixed_total * 100.0).round() / 100.0,
                },
                dynamic_tariff: DynamicTariffResult {
                    avg_rate_ct_kwh: (dynamic_avg * 100.0).round() / 100.0,
                    min_rate_ct_kwh: (dynamic_min * 100.0).round() / 100.0,
                    max_rate_ct_kwh: (dynamic_max * 100.0).round() / 100.0,
                    total_cost_eur: (dynamic_total * 100.0).round() / 100.0,
                    note: format!("Spot price + {} ct/kWh surcharges (grid, taxes, levies, markup)", dynamic_surcharge_ct),
                },
                savings: SavingsResult {
                    dynamic_saves_eur: (savings_eur * 100.0).round() / 100.0,
                    dynamic_saves_percent: (savings_pct * 100.0).round() / 100.0,
                    recommendation,
                },
            };

            ToolCallResult::text(serde_json::to_string_pretty(&comparison).unwrap_or_default())
        }
        Err(e) => ToolCallResult::error(format!("Failed to fetch prices: {}", e)),
    }
}
