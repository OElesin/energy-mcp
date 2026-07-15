"""
Energy MCP Chat Backend
Receives natural language questions, uses Bedrock Claude to interpret them,
calls the MCP tools, and returns conversational responses.
"""

import json
import boto3
import urllib.request

MCP_ENDPOINT = "https://energy-mcp.getbrechtai.com/mcp"
MODEL_ID = "us.anthropic.claude-haiku-4-5-20251001-v1:0"

# Tool definitions matching the MCP server's tools
TOOLS = [
    {
        "name": "get_spot_prices",
        "description": "Get current and upcoming hourly electricity spot prices (EPEX Spot) for Germany. Returns prices in EUR/MWh and ct/kWh with timestamps.",
        "input_schema": {
            "type": "object",
            "properties": {
                "hours_ahead": {"type": "integer", "description": "Number of hours ahead to fetch (default: 24, max: 72)."}
            }
        }
    },
    {
        "name": "get_price_forecast",
        "description": "Get day-ahead hourly electricity prices for tomorrow (EPEX Spot). Prices published daily at 14:00 CET.",
        "input_schema": {"type": "object", "properties": {}}
    },
    {
        "name": "get_price_stats",
        "description": "Get summary statistics for today's and tomorrow's electricity prices — average, min, max, cheapest/most expensive hours.",
        "input_schema": {"type": "object", "properties": {}}
    },
    {
        "name": "find_cheapest_window",
        "description": "Find the cheapest consecutive time window to run an appliance. Useful for EV charging, dishwashers, heat pumps.",
        "input_schema": {
            "type": "object",
            "properties": {
                "duration_hours": {"type": "integer", "description": "How many consecutive hours the appliance needs (e.g., 3 for EV charging)."},
                "consumption_kwh": {"type": "number", "description": "Total energy consumption in kWh (e.g., 11 for EV charge).", "default": 1.0}
            },
            "required": ["duration_hours"]
        }
    },
    {
        "name": "compare_tariffs",
        "description": "Compare dynamic (spot-based) vs fixed electricity tariff for a German household.",
        "input_schema": {
            "type": "object",
            "properties": {
                "consumption_kwh": {"type": "number", "description": "Monthly consumption in kWh (default: 300)."},
                "fixed_rate_ct_kwh": {"type": "number", "description": "Current fixed tariff in ct/kWh (default: 32)."}
            }
        }
    },
    {
        "name": "calculate_energy_cost",
        "description": "Calculate what X kWh costs at current spot prices, with comparison to last week.",
        "input_schema": {
            "type": "object",
            "properties": {
                "consumption_kwh": {"type": "number", "description": "Energy consumption to price in kWh (default: 1.0)."},
                "compare_last_week": {"type": "boolean", "description": "Include comparison with last week (default: true)."}
            }
        }
    },
    {
        "name": "analyze_price_trends",
        "description": "Analyze weekly electricity price trends — current vs prior week, volatility, daily breakdowns, direction.",
        "input_schema": {"type": "object", "properties": {}}
    },
    {
        "name": "search_energy_prices",
        "description": "Get electricity spot prices for any European country (25+ countries). Returns hourly day-ahead prices.",
        "input_schema": {
            "type": "object",
            "properties": {
                "country": {"type": "string", "description": "Country code: DE, FR, NL, BE, AT, ES, PT, IT, CH, PL, CZ, DK1, DK2, NO1, SE3, FI, EE, LV, LT, HU, RO, BG, GR, IE, GB"}
            }
        }
    },
    {
        "name": "get_generation_mix",
        "description": "Get current power generation breakdown by source (solar, wind, gas, nuclear, coal, hydro) for any European country.",
        "input_schema": {
            "type": "object",
            "properties": {
                "country": {"type": "string", "description": "Country code: DE, FR, NL, BE, AT, ES, etc."}
            }
        }
    },
    {
        "name": "get_carbon_intensity",
        "description": "Calculate current carbon intensity (gCO2/kWh) for a European country from real-time generation mix.",
        "input_schema": {
            "type": "object",
            "properties": {
                "country": {"type": "string", "description": "Country code: DE, FR, NL, BE, AT, ES, etc."}
            }
        }
    },
    {
        "name": "get_gas_prices",
        "description": "Get European natural gas prices (TTF Dutch benchmark). Shows current price, historical data, and trend. TTF is the reference price for most European gas contracts and influences electricity prices.",
        "input_schema": {
            "type": "object",
            "properties": {
                "range": {"type": "string", "description": "Time range: 7d, 1mo, 3mo, 6mo, 1y (default: 1mo)."}
            }
        }
    },
]

SYSTEM_PROMPT = """You are an energy assistant for European electricity consumers. You help people:
- Find the cheapest times to run appliances (EV charging, dishwashers, heat pumps)
- Understand current electricity prices and trends
- Compare dynamic vs fixed tariffs
- Check how clean/green the grid is right now
- Get prices for any European country

Be concise and practical. Give direct answers with specific numbers and times.
When mentioning prices, use ct/kWh for consumers (not EUR/MWh).
When mentioning times, use local time (CET/CEST for Germany).
If the user asks in German, respond in German."""


def call_mcp_tool(tool_name: str, arguments: dict) -> str:
    """Call an MCP tool and return the text result."""
    payload = json.dumps({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {"name": tool_name, "arguments": arguments}
    }).encode("utf-8")

    req = urllib.request.Request(
        MCP_ENDPOINT,
        data=payload,
        headers={"Content-Type": "application/json", "Accept": "application/json, text/event-stream"},
        method="POST"
    )

    with urllib.request.urlopen(req, timeout=25) as resp:
        body = resp.read().decode("utf-8")

    # Parse SSE response
    for line in body.split("\n"):
        if line.startswith("data: "):
            data = json.loads(line[6:])
            if "result" in data and "content" in data["result"]:
                return data["result"]["content"][0]["text"]
            elif "error" in data:
                return json.dumps(data["error"])

    return "No response from MCP server"


def invoke_bedrock(messages: list) -> dict:
    """Call Bedrock Claude with tool definitions."""
    client = boto3.client("bedrock-runtime", region_name="us-east-1")

    body = {
        "anthropic_version": "bedrock-2023-05-31",
        "max_tokens": 1024,
        "system": SYSTEM_PROMPT,
        "messages": messages,
        "tools": [
            {"name": t["name"], "description": t["description"], "input_schema": t["input_schema"]}
            for t in TOOLS
        ],
    }

    response = client.invoke_model(
        modelId=MODEL_ID,
        body=json.dumps(body),
        contentType="application/json",
    )

    return json.loads(response["body"].read())


def handler(event, context):
    """Lambda handler for the /chat endpoint."""
    # Handle CORS preflight
    http_method = event.get("requestContext", {}).get("http", {}).get("method", "")
    if http_method == "OPTIONS":
        return response_json(200, {})

    # Parse request
    try:
        body = json.loads(event.get("body", "{}"))
    except json.JSONDecodeError:
        return response_json(400, {"error": "Invalid JSON"})

    user_message = body.get("message", "").strip()
    if not user_message:
        return response_json(400, {"error": "No message provided"})

    # Build conversation (single turn for now)
    messages = [{"role": "user", "content": user_message}]

    try:
        # First call to Claude — may return tool_use or text
        result = invoke_bedrock(messages)

        # Handle tool calls (loop for multi-tool)
        max_iterations = 3
        iteration = 0

        while iteration < max_iterations:
            iteration += 1
            stop_reason = result.get("stop_reason", "")

            if stop_reason != "tool_use":
                # Final text response
                text_parts = [
                    block["text"]
                    for block in result.get("content", [])
                    if block.get("type") == "text"
                ]
                return response_json(200, {
                    "response": "\n".join(text_parts) or "I couldn't generate a response.",
                    "tools_called": iteration - 1,
                })

            # Extract tool calls and execute them
            assistant_content = result["content"]
            messages.append({"role": "assistant", "content": assistant_content})

            tool_results = []
            for block in assistant_content:
                if block.get("type") == "tool_use":
                    tool_name = block["name"]
                    tool_input = block.get("input", {})
                    tool_id = block["id"]

                    # Call the MCP tool
                    mcp_result = call_mcp_tool(tool_name, tool_input)

                    tool_results.append({
                        "type": "tool_result",
                        "tool_use_id": tool_id,
                        "content": mcp_result,
                    })

            messages.append({"role": "user", "content": tool_results})

            # Call Claude again with tool results
            result = invoke_bedrock(messages)

        # If we exhausted iterations
        return response_json(200, {
            "response": "I needed too many tool calls to answer that. Try a simpler question.",
            "tools_called": max_iterations,
        })

    except Exception as e:
        return response_json(500, {"error": f"Internal error: {str(e)}"})


def response_json(status_code: int, body: dict) -> dict:
    return {
        "statusCode": status_code,
        "headers": {
            "Content-Type": "application/json",
            "Access-Control-Allow-Origin": "*",
            "Access-Control-Allow-Methods": "POST, OPTIONS",
            "Access-Control-Allow-Headers": "Content-Type",
        },
        "body": json.dumps(body),
    }
