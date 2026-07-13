use chrono::{NaiveDateTime, Utc};
use reqwest::Client;
use serde::Serialize;
use std::collections::HashMap;

const ENTSOE_BASE_URL: &str = "https://web-api.tp.entsoe.eu/api";
const ENTSOE_TOKEN: &str = "f2c23b1f-81f4-4132-883a-b566ef32d136";

/// Bidding zone EIC codes for European countries
pub fn get_bidding_zone(country: &str) -> Option<&'static str> {
    let map: HashMap<&str, &str> = HashMap::from([
        ("DE", "10Y1001A1001A82H"),  // Germany-Luxembourg
        ("FR", "10YFR-RTE------C"),  // France
        ("NL", "10YNL----------L"),  // Netherlands
        ("BE", "10YBE----------2"),  // Belgium
        ("AT", "10YAT-APG------L"),  // Austria
        ("ES", "10YES-REE------0"),  // Spain
        ("PT", "10YPT-REN------W"),  // Portugal
        ("IT", "10Y1001A1001A73I"),  // Italy North
        ("CH", "10YCH-SWISSGRIDZ"),  // Switzerland
        ("PL", "10YPL-AREA-----S"),  // Poland
        ("CZ", "10YCZ-CEPS-----N"),  // Czech Republic
        ("DK1", "10YDK-1--------W"), // Denmark West
        ("DK2", "10YDK-2--------M"), // Denmark East
        ("NO1", "10YNO-1--------2"), // Norway Oslo
        ("NO2", "10YNO-2--------T"), // Norway South
        ("SE1", "10Y1001A1001A44P"), // Sweden North
        ("SE3", "10Y1001A1001A46L"), // Sweden Stockholm
        ("SE4", "10Y1001A1001A47J"), // Sweden South
        ("FI", "10YFI-1--------U"),  // Finland
        ("EE", "10Y1001A1001A39I"),  // Estonia
        ("LV", "10YLV-1001A00074"),  // Latvia
        ("LT", "10YLT-1001A0008Q"),  // Lithuania
        ("HU", "10YHU-MAVIR----U"),  // Hungary
        ("RO", "10YRO-TEL------P"),  // Romania
        ("BG", "10YCA-BULGARIA-R"),  // Bulgaria
        ("GR", "10YGR-HTSO-----Y"),  // Greece
        ("IE", "10Y1001A1001A59C"),  // Ireland
        ("GB", "10Y1001A1001A92E"),  // Great Britain
    ]);

    let key = country.to_uppercase();
    map.get(key.as_str()).copied()
}

/// Get the generation domain code (different from price domain for DE)
pub fn get_generation_domain(country: &str) -> Option<&'static str> {
    let key = country.to_uppercase();
    if key == "DE" {
        // For generation data, Germany uses a different code
        Some("10Y1001A1001A83F")
    } else {
        get_bidding_zone(country)
    }
}

/// List all available countries
pub fn list_countries() -> Vec<(&'static str, &'static str)> {
    vec![
        ("DE", "Germany"), ("FR", "France"), ("NL", "Netherlands"),
        ("BE", "Belgium"), ("AT", "Austria"), ("ES", "Spain"),
        ("PT", "Portugal"), ("IT", "Italy"), ("CH", "Switzerland"),
        ("PL", "Poland"), ("CZ", "Czech Republic"), ("DK1", "Denmark West"),
        ("DK2", "Denmark East"), ("NO1", "Norway"), ("SE3", "Sweden"),
        ("FI", "Finland"), ("EE", "Estonia"), ("LV", "Latvia"),
        ("LT", "Lithuania"), ("HU", "Hungary"), ("RO", "Romania"),
        ("BG", "Bulgaria"), ("GR", "Greece"), ("IE", "Ireland"), ("GB", "Great Britain"),
    ]
}

/// PSR type codes for generation sources
pub fn psr_type_name(code: &str) -> &'static str {
    match code {
        "B01" => "Biomass",
        "B02" => "Fossil Brown Coal/Lignite",
        "B03" => "Fossil Coal-derived Gas",
        "B04" => "Fossil Gas",
        "B05" => "Fossil Hard Coal",
        "B06" => "Fossil Oil",
        "B07" => "Fossil Oil Shale",
        "B08" => "Fossil Peat",
        "B09" => "Geothermal",
        "B10" => "Hydro Pumped Storage",
        "B11" => "Hydro Run-of-river",
        "B12" => "Hydro Water Reservoir",
        "B13" => "Marine",
        "B14" => "Nuclear",
        "B15" => "Other Renewable",
        "B16" => "Solar",
        "B17" => "Waste",
        "B18" => "Wind Offshore",
        "B19" => "Wind Onshore",
        "B20" => "Other",
        _ => "Unknown",
    }
}

/// CO2 emission factors in gCO2/kWh per generation source
pub fn emission_factor(psr_type: &str) -> f64 {
    match psr_type {
        "B01" => 230.0,   // Biomass
        "B02" => 1000.0,  // Lignite
        "B03" => 490.0,   // Coal-derived gas
        "B04" => 490.0,   // Natural Gas
        "B05" => 820.0,   // Hard Coal
        "B06" => 650.0,   // Oil
        "B07" => 900.0,   // Oil Shale
        "B08" => 380.0,   // Peat
        "B09" => 38.0,    // Geothermal
        "B10" => 24.0,    // Hydro Pumped Storage
        "B11" => 24.0,    // Hydro Run-of-river
        "B12" => 24.0,    // Hydro Reservoir
        "B13" => 20.0,    // Marine
        "B14" => 12.0,    // Nuclear
        "B15" => 30.0,    // Other Renewable
        "B16" => 45.0,    // Solar
        "B17" => 150.0,   // Waste
        "B18" => 11.0,    // Wind Offshore
        "B19" => 11.0,    // Wind Onshore
        "B20" => 300.0,   // Other
        _ => 300.0,
    }
}

/// Fetched price data point
#[derive(Debug, Serialize, Clone)]
pub struct PricePoint {
    pub timestamp: String,
    pub price_eur_mwh: f64,
    pub price_ct_kwh: f64,
}

/// Fetched generation data point
#[derive(Debug, Serialize, Clone)]
pub struct GenerationSource {
    pub source: String,
    pub psr_type: String,
    pub generation_mw: f64,
    pub share_percent: f64,
    pub is_renewable: bool,
}

/// Fetch day-ahead prices from ENTSO-E for any country
pub async fn fetch_prices(country: &str, start: &str, end: &str) -> Result<Vec<PricePoint>, String> {
    let domain = get_bidding_zone(country)
        .ok_or_else(|| format!("Unknown country code: {}. Available: DE, FR, NL, BE, AT, ES, PT, IT, CH, PL, CZ, DK1, DK2, NO1, SE3, FI, EE, LV, LT, HU, RO, BG, GR, IE, GB", country))?;

    let url = format!(
        "{}?documentType=A44&in_Domain={}&out_Domain={}&periodStart={}&periodEnd={}&securityToken={}",
        ENTSOE_BASE_URL, domain, domain, start, end, ENTSOE_TOKEN
    );

    let client = Client::new();
    let resp = client.get(&url).send().await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("ENTSO-E API error {}: {}", status, &body[..body.len().min(200)]));
    }

    let xml = resp.text().await.map_err(|e| format!("Failed to read response: {}", e))?;
    parse_price_xml(&xml)
}

/// Fetch actual generation per type from ENTSO-E
pub async fn fetch_generation(country: &str, start: &str, end: &str) -> Result<Vec<GenerationSource>, String> {
    let domain = get_generation_domain(country)
        .ok_or_else(|| format!("Unknown country code: {}", country))?;

    let url = format!(
        "{}?documentType=A75&processType=A16&in_Domain={}&periodStart={}&periodEnd={}&securityToken={}",
        ENTSOE_BASE_URL, domain, start, end, ENTSOE_TOKEN
    );

    let client = Client::new();
    let resp = client.get(&url).send().await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("ENTSO-E API error {}: {}", status, &body[..body.len().min(200)]));
    }

    let xml = resp.text().await.map_err(|e| format!("Failed to read response: {}", e))?;
    parse_generation_xml(&xml)
}

/// Parse ENTSO-E price XML response
fn parse_price_xml(xml: &str) -> Result<Vec<PricePoint>, String> {
    let mut prices = Vec::new();
    let mut in_period = false;
    let mut in_point = false;
    let mut period_start = String::new();
    let mut resolution_minutes: i64 = 60;
    let mut current_position: i64 = 0;
    let mut current_price: Option<f64> = None;

    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();
    let mut current_tag = String::new();
    let mut tag_stack: Vec<String> = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                current_tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                tag_stack.push(current_tag.clone());
                if current_tag == "Period" {
                    in_period = true;
                    period_start.clear();
                }
                if current_tag == "Point" && in_period {
                    in_point = true;
                    current_position = 0;
                    current_price = None;
                }
            }
            Ok(Event::End(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                tag_stack.pop();
                if tag == "Point" && in_period {
                    if let Some(price) = current_price {
                        if current_position > 0 {
                            let offset_minutes = (current_position - 1) * resolution_minutes;
                            if let Ok(start_dt) = NaiveDateTime::parse_from_str(&period_start, "%Y-%m-%dT%H:%MZ") {
                                let ts = start_dt + chrono::Duration::minutes(offset_minutes);
                                prices.push(PricePoint {
                                    timestamp: ts.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                                    price_eur_mwh: (price * 100.0).round() / 100.0,
                                    price_ct_kwh: (price / 10.0 * 100.0).round() / 100.0,
                                });
                            }
                        }
                    }
                    in_point = false;
                }
                if tag == "Period" {
                    in_period = false;
                }
            }
            Ok(Event::Text(ref e)) => {
                let text = e.unescape().unwrap_or_default().trim().to_string();
                if text.is_empty() {
                    buf.clear();
                    continue;
                }
                // Check parent context using tag_stack
                let parent = tag_stack.last().map(|s| s.as_str()).unwrap_or("");
                match parent {
                    "start" if in_period && period_start.is_empty() => {
                        // Only capture the start inside Period > timeInterval > start
                        period_start = text;
                    }
                    "resolution" if in_period => {
                        resolution_minutes = match text.as_str() {
                            "PT15M" => 15,
                            "PT30M" => 30,
                            "PT60M" | "P1H" => 60,
                            _ => 60,
                        };
                    }
                    "position" if in_point => {
                        current_position = text.parse().unwrap_or(0);
                    }
                    "price.amount" if in_point => {
                        current_price = text.parse().ok();
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("XML parse error: {}", e)),
            _ => {}
        }
        buf.clear();
    }

    // Deduplicate by timestamp (take hourly averages if 15-min resolution)
    if resolution_minutes < 60 {
        prices = aggregate_to_hourly(prices);
    }

    Ok(prices)
}

/// Parse ENTSO-E generation XML response
fn parse_generation_xml(xml: &str) -> Result<Vec<GenerationSource>, String> {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut sources: HashMap<String, f64> = HashMap::new();
    let mut current_psr_type = String::new();
    let mut in_time_series = false;
    let mut last_quantity: Option<f64> = None;

    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();
    let mut tag_stack: Vec<String> = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                tag_stack.push(tag.clone());
                if tag == "TimeSeries" {
                    in_time_series = true;
                    current_psr_type.clear();
                    last_quantity = None;
                }
            }
            Ok(Event::End(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                tag_stack.pop();
                if tag == "TimeSeries" {
                    if !current_psr_type.is_empty() {
                        if let Some(qty) = last_quantity {
                            let entry = sources.entry(current_psr_type.clone()).or_insert(0.0);
                            *entry = qty;
                        }
                    }
                    in_time_series = false;
                    last_quantity = None;
                }
            }
            Ok(Event::Text(ref e)) => {
                let text = e.unescape().unwrap_or_default().trim().to_string();
                if text.is_empty() {
                    buf.clear();
                    continue;
                }
                let parent = tag_stack.last().map(|s| s.as_str()).unwrap_or("");
                match parent {
                    "psrType" if in_time_series => {
                        current_psr_type = text;
                    }
                    "quantity" if in_time_series => {
                        last_quantity = text.parse().ok();
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("XML parse error: {}", e)),
            _ => {}
        }
        buf.clear();
    }

    let total: f64 = sources.values().sum();
    if total == 0.0 {
        return Err("No generation data found".into());
    }

    let mut result: Vec<GenerationSource> = sources
        .iter()
        .filter(|(_, &mw)| mw > 0.0)
        .map(|(psr, &mw)| {
            let is_renewable = matches!(
                psr.as_str(),
                "B01" | "B09" | "B10" | "B11" | "B12" | "B13" | "B15" | "B16" | "B18" | "B19"
            );
            GenerationSource {
                source: psr_type_name(psr).to_string(),
                psr_type: psr.clone(),
                generation_mw: (mw * 10.0).round() / 10.0,
                share_percent: ((mw / total * 100.0) * 10.0).round() / 10.0,
                is_renewable,
            }
        })
        .collect();

    result.sort_by(|a, b| b.generation_mw.partial_cmp(&a.generation_mw).unwrap());
    Ok(result)
}

/// Aggregate 15-minute prices to hourly averages
fn aggregate_to_hourly(prices: Vec<PricePoint>) -> Vec<PricePoint> {
    let mut hourly: HashMap<String, Vec<f64>> = HashMap::new();

    for p in &prices {
        // Group by hour: "2026-07-13T14" 
        let hour_key = p.timestamp[..13].to_string();
        hourly.entry(hour_key).or_default().push(p.price_eur_mwh);
    }

    let mut result: Vec<PricePoint> = hourly
        .iter()
        .map(|(hour, vals)| {
            let avg = vals.iter().sum::<f64>() / vals.len() as f64;
            PricePoint {
                timestamp: format!("{}:00:00Z", hour),
                price_eur_mwh: (avg * 100.0).round() / 100.0,
                price_ct_kwh: (avg / 10.0 * 100.0).round() / 100.0,
            }
        })
        .collect();

    result.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    result
}
