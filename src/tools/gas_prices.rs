use serde::{Deserialize, Serialize};
use serde_json::Value;
use reqwest::Client;
use crate::mcp::ToolCallResult;

const YAHOO_CHART_URL: &str = "https://query1.finance.yahoo.com/v8/finance/chart/TTF=F";

#[derive(Debug, Serialize)]
struct GasPriceResult {
    symbol: String,
    name: String,
    currency: String,
    unit: String,
    current_price_eur_mwh: f64,
    current_price_ct_kwh: f64,
    prices: Vec<GasPricePoint>,
    stats: GasStats,
    context: GasContext,
}

#[derive(Debug, Serialize)]
struct GasPricePoint {
    date: String,
    price_eur_mwh: f64,
    price_ct_kwh: f64,
}

#[derive(Debug, Serialize)]
struct GasStats {
    period_days: usize,
    avg_eur_mwh: f64,
    min_eur_mwh: f64,
    max_eur_mwh: f64,
    change_percent_7d: f64,
}

#[derive(Debug, Serialize)]
struct GasContext {
    description: String,
    relevance: String,
}

#[derive(Debug, Deserialize)]
struct YahooResponse {
    chart: YahooChart,
}

#[derive(Debug, Deserialize)]
struct YahooChart {
    result: Option<Vec<YahooResult>>,
    error: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct YahooResult {
    meta: YahooMeta,
    timestamp: Option<Vec<i64>>,
    indicators: YahooIndicators,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct YahooMeta {
    regular_market_price: Option<f64>,
    currency: Option<String>,
}

#[derive(Debug, Deserialize)]
struct YahooIndicators {
    quote: Vec<YahooQuote>,
}

#[derive(Debug, Deserialize)]
struct YahooQuote {
    close: Option<Vec<Option<f64>>>,
}

pub async fn execute(params: &Value) -> ToolCallResult {
    let range = params.get("range")
        .and_then(|v| v.as_str())
        .unwrap_or("1mo");

    // Validate range
    let range = match range {
        "7d" | "1mo" | "3mo" | "6mo" | "1y" => range,
        _ => "1mo",
    };

    let url = format!("{}?interval=1d&range={}", YAHOO_CHART_URL, range);

    let client = Client::new();
    let resp = client
        .get(&url)
        .header("User-Agent", "Mozilla/5.0")
        .send()
        .await;

    let resp = match resp {
        Ok(r) => r,
        Err(e) => return ToolCallResult::error(format!("Failed to fetch gas prices: {}", e)),
    };

    let body: YahooResponse = match resp.json().await {
        Ok(b) => b,
        Err(e) => return ToolCallResult::error(format!("Failed to parse response: {}", e)),
    };

    let results = match body.chart.result {
        Some(r) if !r.is_empty() => r,
        _ => {
            let err = body.chart.error.map(|e| format!("{}", e)).unwrap_or("No data".into());
            return ToolCallResult::error(format!("Yahoo Finance error: {}", err));
        }
    };
    let result = &results[0];

    // This is a workaround since we can't move out of the borrow
    let meta = &result.meta;
    let timestamps = result.timestamp.as_ref();
    let closes = result.indicators.quote.first()
        .and_then(|q| q.close.as_ref());

    let (timestamps, closes) = match (timestamps, closes) {
        (Some(ts), Some(cl)) => (ts, cl),
        _ => return ToolCallResult::error("No price data available".into()),
    };

    let current_price = meta.regular_market_price.unwrap_or(0.0);

    let mut prices: Vec<GasPricePoint> = Vec::new();
    for (ts, close) in timestamps.iter().zip(closes.iter()) {
        if let Some(price) = close {
            let dt = chrono::DateTime::from_timestamp(*ts, 0)
                .map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_default();
            prices.push(GasPricePoint {
                date: dt,
                price_eur_mwh: (*price * 100.0).round() / 100.0,
                price_ct_kwh: (*price / 10.0 * 100.0).round() / 100.0,
            });
        }
    }

    if prices.is_empty() {
        return ToolCallResult::error("No valid price data".into());
    }

    let all_prices: Vec<f64> = prices.iter().map(|p| p.price_eur_mwh).collect();
    let avg = all_prices.iter().sum::<f64>() / all_prices.len() as f64;
    let min = all_prices.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = all_prices.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    let change_7d = if prices.len() >= 2 {
        let first = prices.first().unwrap().price_eur_mwh;
        let last = prices.last().unwrap().price_eur_mwh;
        if first > 0.0 { ((last - first) / first * 100.0 * 10.0).round() / 10.0 } else { 0.0 }
    } else {
        0.0
    };

    let output = GasPriceResult {
        symbol: "TTF=F".into(),
        name: "Dutch TTF Natural Gas Futures (Front Month)".into(),
        currency: meta.currency.clone().unwrap_or("EUR".into()),
        unit: "EUR/MWh".into(),
        current_price_eur_mwh: (current_price * 100.0).round() / 100.0,
        current_price_ct_kwh: (current_price / 10.0 * 100.0).round() / 100.0,
        prices,
        stats: GasStats {
            period_days: all_prices.len(),
            avg_eur_mwh: (avg * 100.0).round() / 100.0,
            min_eur_mwh: (min * 100.0).round() / 100.0,
            max_eur_mwh: (max * 100.0).round() / 100.0,
            change_percent_7d: change_7d,
        },
        context: GasContext {
            description: "TTF (Title Transfer Facility) is the European benchmark for natural gas pricing. It reflects wholesale gas costs that flow through to consumer heating and electricity bills.".into(),
            relevance: "When TTF rises, gas-fired electricity generation becomes more expensive, pushing up spot electricity prices. About 13% of German electricity comes from gas.".into(),
        },
    };

    ToolCallResult::text(serde_json::to_string_pretty(&output).unwrap_or_default())
}
