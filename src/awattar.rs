use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};

const AWATTAR_DE_URL: &str = "https://api.awattar.de/v1/marketdata";

#[derive(Debug, Deserialize)]
struct AwattarResponse {
    data: Vec<AwattarEntry>,
}

#[derive(Debug, Deserialize)]
struct AwattarEntry {
    start_timestamp: i64,
    end_timestamp: i64,
    marketprice: f64, // EUR/MWh
    unit: String,
}

/// A single hourly price point
#[derive(Debug, Serialize, Clone)]
pub struct HourlyPrice {
    pub start: String,
    pub end: String,
    pub price_eur_mwh: f64,
    pub price_ct_kwh: f64,
}

/// Result of fetching spot prices
#[derive(Debug, Serialize)]
pub struct SpotPriceData {
    pub country: String,
    pub currency: String,
    pub timezone: String,
    pub fetched_at: String,
    pub prices: Vec<HourlyPrice>,
    pub stats: PriceStats,
}

/// Summary statistics
#[derive(Debug, Serialize)]
pub struct PriceStats {
    pub count: usize,
    pub avg_ct_kwh: f64,
    pub min_ct_kwh: f64,
    pub max_ct_kwh: f64,
    pub min_hour: String,
    pub max_hour: String,
}

/// Cheapest window result
#[derive(Debug, Serialize)]
pub struct CheapestWindow {
    pub duration_hours: usize,
    pub start: String,
    pub end: String,
    pub avg_price_ct_kwh: f64,
    pub total_cost_eur: f64,
    pub consumption_kwh: f64,
    pub prices: Vec<HourlyPrice>,
}

pub async fn fetch_spot_prices(start_ms: Option<i64>, end_ms: Option<i64>) -> Result<SpotPriceData, String> {
    let client = Client::new();
    let mut url = AWATTAR_DE_URL.to_string();

    let mut params = vec![];
    if let Some(start) = start_ms {
        params.push(format!("start={}", start));
    }
    if let Some(end) = end_ms {
        params.push(format!("end={}", end));
    }
    if !params.is_empty() {
        url = format!("{}?{}", url, params.join("&"));
    }

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("aWATTar API returned status {}", resp.status()));
    }

    let data: AwattarResponse = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let prices: Vec<HourlyPrice> = data
        .data
        .iter()
        .map(|entry| {
            let start_dt = DateTime::from_timestamp_millis(entry.start_timestamp)
                .unwrap_or_default();
            let end_dt = DateTime::from_timestamp_millis(entry.end_timestamp)
                .unwrap_or_default();
            HourlyPrice {
                start: start_dt.format("%Y-%m-%dT%H:%M:%S%z").to_string(),
                end: end_dt.format("%Y-%m-%dT%H:%M:%S%z").to_string(),
                price_eur_mwh: (entry.marketprice * 100.0).round() / 100.0,
                price_ct_kwh: (entry.marketprice / 10.0 * 100.0).round() / 100.0,
            }
        })
        .collect();

    let stats = compute_stats(&prices);

    Ok(SpotPriceData {
        country: "DE".into(),
        currency: "EUR".into(),
        timezone: "Europe/Berlin".into(),
        fetched_at: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        prices,
        stats,
    })
}

pub fn find_cheapest_window(prices: &[HourlyPrice], duration_hours: usize, consumption_kwh: f64) -> Option<CheapestWindow> {
    if prices.len() < duration_hours || duration_hours == 0 {
        return None;
    }

    let mut best_start = 0;
    let mut best_avg = f64::MAX;

    for i in 0..=(prices.len() - duration_hours) {
        let window = &prices[i..i + duration_hours];
        let avg: f64 = window.iter().map(|p| p.price_ct_kwh).sum::<f64>() / duration_hours as f64;
        if avg < best_avg {
            best_avg = avg;
            best_start = i;
        }
    }

    let window = &prices[best_start..best_start + duration_hours];
    let total_cost = consumption_kwh * best_avg / 100.0; // ct to EUR

    Some(CheapestWindow {
        duration_hours,
        start: window.first()?.start.clone(),
        end: window.last()?.end.clone(),
        avg_price_ct_kwh: (best_avg * 100.0).round() / 100.0,
        total_cost_eur: (total_cost * 100.0).round() / 100.0,
        consumption_kwh,
        prices: window.to_vec(),
    })
}

fn compute_stats(prices: &[HourlyPrice]) -> PriceStats {
    if prices.is_empty() {
        return PriceStats {
            count: 0,
            avg_ct_kwh: 0.0,
            min_ct_kwh: 0.0,
            max_ct_kwh: 0.0,
            min_hour: String::new(),
            max_hour: String::new(),
        };
    }

    let sum: f64 = prices.iter().map(|p| p.price_ct_kwh).sum();
    let avg = sum / prices.len() as f64;

    let min_entry = prices.iter().min_by(|a, b| a.price_ct_kwh.partial_cmp(&b.price_ct_kwh).unwrap()).unwrap();
    let max_entry = prices.iter().max_by(|a, b| a.price_ct_kwh.partial_cmp(&b.price_ct_kwh).unwrap()).unwrap();

    PriceStats {
        count: prices.len(),
        avg_ct_kwh: (avg * 100.0).round() / 100.0,
        min_ct_kwh: min_entry.price_ct_kwh,
        max_ct_kwh: max_entry.price_ct_kwh,
        min_hour: min_entry.start.clone(),
        max_hour: max_entry.start.clone(),
    }
}
