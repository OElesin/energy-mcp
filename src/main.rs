use lambda_http::{run, service_fn, tracing, Body, Error, Request, RequestExt, Response};
use serde_json::{json, Value};

mod auth;
mod awattar;
mod entsoe;
mod mcp;
mod tools;

use mcp::{JsonRpcRequest, JsonRpcResponse};

const SERVER_NAME: &str = "Energy-MCP";
const SERVER_VERSION: &str = "0.2.0";
const PROTOCOL_VERSION: &str = "2025-03-26";

async fn handler(event: Request) -> Result<Response<Body>, Error> {
    // Initialize DynamoDB client
    let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let dynamo = aws_sdk_dynamodb::Client::new(&config);

    // Extract source IP from request context
    let source_ip = event
        .request_context_ref()
        .map(|ctx| match ctx {
            lambda_http::request::RequestContext::ApiGatewayV2(http) => {
                http.http.source_ip.clone().unwrap_or_else(|| "unknown".into())
            }
            _ => "unknown".into(),
        })
        .unwrap_or_else(|| "unknown".into());

    let api_key = event
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.trim_start_matches("Bearer ").to_string());

    // Validate auth/rate limit
    match auth::validate_request(&dynamo, api_key.as_deref(), &source_ip).await {
        Ok(_auth_result) => {
            // Proceed with request
        }
        Err(auth::AuthError::RateLimited { limit, tier }) => {
            let resp = JsonRpcResponse::error(
                None,
                -32000,
                format!("Rate limit exceeded. {} tier allows {} requests/day. Upgrade at https://energy-mcp.getbrechtai.com", tier, limit),
            );
            return build_response(&resp);
        }
        Err(auth::AuthError::InvalidKey) => {
            let resp = JsonRpcResponse::error(
                None,
                -32001,
                "Invalid API key. Get one at https://energy-mcp.getbrechtai.com".into(),
            );
            return build_response(&resp);
        }
        Err(auth::AuthError::InternalError(_)) => {
            // Don't block on auth errors — fail open
        }
    }

    // Parse the JSON-RPC request from the body
    let body = event.body();
    let body_str = match body {
        Body::Text(s) => s.clone(),
        Body::Binary(b) => String::from_utf8_lossy(b).to_string(),
        _ => String::new(),
    };

    let request: JsonRpcRequest = match serde_json::from_str(&body_str) {
        Ok(r) => r,
        Err(e) => {
            let resp = JsonRpcResponse::error(None, -32700, format!("Parse error: {}", e));
            return build_response(&resp);
        }
    };

    let response = route_request(request).await;
    build_response(&response)
}

async fn route_request(request: JsonRpcRequest) -> JsonRpcResponse {
    let id = request.id.clone();

    match request.method.as_str() {
        "initialize" => {
            JsonRpcResponse::success(id, json!({
                "protocolVersion": PROTOCOL_VERSION,
                "capabilities": {
                    "tools": { "listChanged": false }
                },
                "serverInfo": {
                    "name": SERVER_NAME,
                    "version": SERVER_VERSION
                }
            }))
        }

        "notifications/initialized" => {
            // Client acknowledgment — no response needed, but we return empty for HTTP
            JsonRpcResponse::success(id, json!({}))
        }

        "tools/list" => {
            let tools = tools::list_tools();
            JsonRpcResponse::success(id, json!({ "tools": tools }))
        }

        "tools/call" => {
            let tool_name = request.params.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let arguments = request.params.get("arguments")
                .cloned()
                .unwrap_or(json!({}));

            let result = match tool_name {
                "get_spot_prices" => tools::spot_prices::execute(&arguments).await,
                "get_price_forecast" => tools::price_forecast::execute(&arguments).await,
                "get_price_stats" => tools::price_stats::execute(&arguments).await,
                "find_cheapest_window" => tools::cheapest_window::execute(&arguments).await,
                "compare_tariffs" => tools::compare_tariffs::execute(&arguments).await,
                "calculate_energy_cost" => tools::energy_cost::execute(&arguments).await,
                "analyze_price_trends" => tools::price_trends::execute(&arguments).await,
                "search_energy_prices" => tools::energy_prices::execute(&arguments).await,
                "get_generation_mix" => tools::generation_mix::execute(&arguments).await,
                "get_carbon_intensity" => tools::carbon_intensity::execute(&arguments).await,
                "get_gas_prices" => tools::gas_prices::execute(&arguments).await,
                _ => mcp::ToolCallResult::error(format!("Unknown tool: {}", tool_name)),
            };

            JsonRpcResponse::success(id, serde_json::to_value(result).unwrap_or(json!({})))
        }

        _ => {
            JsonRpcResponse::error(id, -32601, format!("Method not found: {}", request.method))
        }
    }
}

fn build_response(resp: &JsonRpcResponse) -> Result<Response<Body>, Error> {
    let json_body = serde_json::to_string(resp).unwrap_or_default();

    // Return as SSE format for MCP client compatibility
    let sse_body = format!("event: message\ndata: {}\n\n", json_body);

    let response = Response::builder()
        .status(200)
        .header("Content-Type", "text/event-stream")
        .header("Cache-Control", "no-cache, no-transform")
        .header("X-Accel-Buffering", "no")
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "POST, GET, OPTIONS")
        .header("Access-Control-Allow-Headers", "Content-Type, Authorization, Mcp-Session-Id")
        .body(Body::Text(sse_body))
        .map_err(Box::new)?;

    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();
    run(service_fn(handler)).await
}
