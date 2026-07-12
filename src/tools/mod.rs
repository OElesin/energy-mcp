pub mod spot_prices;
pub mod cheapest_window;
pub mod price_stats;

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
    ]
}
