pub mod spot_prices;
pub mod cheapest_window;
pub mod price_stats;
pub mod price_forecast;
pub mod compare_tariffs;
pub mod energy_cost;
pub mod price_trends;
pub mod energy_prices;
pub mod generation_mix;
pub mod carbon_intensity;
pub mod gas_prices;

use serde_json::{json, Value};
use crate::mcp::ToolDef;

/// Return all available tool definitions
pub fn list_tools() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "get_spot_prices".into(),
            description: "Get current and upcoming hourly electricity spot prices (EPEX Spot) for Germany. Returns prices in EUR/MWh and ct/kWh with timestamps.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "hours_ahead": {
                        "type": "integer",
                        "description": "Number of hours ahead to fetch (default: 48, max: 72). Includes available day-ahead prices.",
                        "default": 48
                    }
                }
            }),
        },
        ToolDef {
            name: "get_price_forecast".into(),
            description: "Get day-ahead hourly electricity prices for tomorrow (EPEX Spot via aWATTar). Prices are published daily at 14:00 CET.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDef {
            name: "get_price_stats".into(),
            description: "Get summary statistics for today's and tomorrow's electricity prices — average, min, max, and the cheapest/most expensive hours.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDef {
            name: "find_cheapest_window".into(),
            description: "Find the cheapest consecutive time window to run an appliance. Useful for EV charging, dishwashers, heat pumps, etc.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "duration_hours": {
                        "type": "integer",
                        "description": "How many consecutive hours the appliance needs to run (e.g., 3 for EV charging)."
                    },
                    "consumption_kwh": {
                        "type": "number",
                        "description": "Total energy consumption in kWh (e.g., 11 for a typical EV charge session).",
                        "default": 1.0
                    }
                },
                "required": ["duration_hours"]
            }),
        },
        ToolDef {
            name: "compare_tariffs".into(),
            description: "Compare dynamic (spot-based) vs fixed electricity tariff for a German household. Shows which saves more money based on current market prices.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "consumption_kwh": {
                        "type": "number",
                        "description": "Monthly electricity consumption in kWh (default: 300, German average).",
                        "default": 300.0
                    },
                    "fixed_rate_ct_kwh": {
                        "type": "number",
                        "description": "Your current fixed tariff rate in ct/kWh (default: 32).",
                        "default": 32.0
                    },
                    "dynamic_surcharge_ct_kwh": {
                        "type": "number",
                        "description": "Surcharges on top of spot price for dynamic tariff in ct/kWh — grid fees, taxes, levies (default: 20).",
                        "default": 20.0
                    }
                }
            }),
        },
        ToolDef {
            name: "calculate_energy_cost".into(),
            description: "Calculate what X kWh costs at current spot prices, with comparison to last week. Useful for understanding your real-time energy costs.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "consumption_kwh": {
                        "type": "number",
                        "description": "Energy consumption to price in kWh (default: 1.0).",
                        "default": 1.0
                    },
                    "compare_last_week": {
                        "type": "boolean",
                        "description": "Include comparison with same day last week (default: true).",
                        "default": true
                    }
                }
            }),
        },
        ToolDef {
            name: "analyze_price_trends".into(),
            description: "Analyze weekly electricity price trends — current week vs prior week averages, volatility, daily breakdowns, and week-over-week direction.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDef {
            name: "search_energy_prices".into(),
            description: "Get electricity spot prices for any European country. Supports 25+ countries/bidding zones. Returns hourly day-ahead prices in EUR/MWh and ct/kWh.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "country": {
                        "type": "string",
                        "description": "Country/zone code: DE, FR, NL, BE, AT, ES, PT, IT, CH, PL, CZ, DK1, DK2, NO1, SE3, FI, EE, LV, LT, HU, RO, BG, GR, IE, GB",
                        "default": "DE"
                    },
                    "date": {
                        "type": "string",
                        "description": "Start date in YYYY-MM-DD format (default: today)"
                    },
                    "date_end": {
                        "type": "string",
                        "description": "End date in YYYY-MM-DD format (default: start + 1 day)"
                    }
                }
            }),
        },
        ToolDef {
            name: "get_generation_mix".into(),
            description: "Get the current power generation breakdown by source (solar, wind, gas, nuclear, coal, hydro, etc.) for any European country. Shows renewable vs fossil share.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "country": {
                        "type": "string",
                        "description": "Country code: DE, FR, NL, BE, AT, ES, PL, CZ, FI, SE3, etc.",
                        "default": "DE"
                    }
                }
            }),
        },
        ToolDef {
            name: "get_carbon_intensity".into(),
            description: "Calculate the current carbon intensity (gCO2/kWh) of electricity for a European country, derived from the real-time generation mix. Includes per-source breakdown and rating.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "country": {
                        "type": "string",
                        "description": "Country code: DE, FR, NL, BE, AT, ES, PL, CZ, FI, SE3, etc.",
                        "default": "DE"
                    }
                }
            }),
        },
        ToolDef {
            name: "get_gas_prices".into(),
            description: "Get European natural gas prices (TTF Dutch benchmark). Shows current price, historical data, and trend. TTF is the reference price for most European gas contracts and influences electricity prices.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "range": {
                        "type": "string",
                        "description": "Time range for historical prices: 7d, 1mo, 3mo, 6mo, 1y (default: 1mo).",
                        "default": "1mo"
                    }
                }
            }),
        },
    ]
}
