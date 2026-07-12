use lambda_http::{run, service_fn, tracing, Body, Error, Request, Response};
use serde_json::{json, Value};

mod awattar;
mod mcp;
mod tools;

use mcp::{JsonRpcRequest, JsonRpcResponse};

const SERVER_NAME: &str = "Energy-MCP";
const SERVER_VERSION: &str = "0.1.0";
const PROTOCOL_VERSION: &str = "2025-03-26";

async fn handler(event: Request) -> Result<Response<Body>, Error> {
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
                "get_price_stats" => tools::price_stats::execute(&arguments).await,
                "find_cheapest_window" => tools::cheapest_window::execute(&arguments).await,
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
