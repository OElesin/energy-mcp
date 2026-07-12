# Energy MCP Server

An MCP server for German and European electricity market data. Built in Rust for near-zero cold starts on AWS Lambda.

## Use It Now

```json
{
  "mcpServers": {
    "energy": {
      "url": "https://energy-mcp.getbrechtai.com/mcp"
    }
  }
}
```

## Tools

| Tool | Description |
|------|-------------|
| `get_spot_prices` | Current and upcoming hourly EPEX Spot electricity prices for Germany (EUR/MWh and ct/kWh) |
| `get_price_stats` | Summary statistics — average, min, max, cheapest and most expensive hours |
| `find_cheapest_window` | Find the optimal time to run an appliance (EV charging, dishwasher, heat pump) |

## Examples

**"When should I charge my EV tonight?"**
```
find_cheapest_window(duration_hours: 3, consumption_kwh: 11)
→ Best window: 10:00–13:00, avg 4.07 ct/kWh, total cost: €0.45
```

**"What are electricity prices like right now?"**
```
get_price_stats()
→ avg: 11.55 ct/kWh, min: 3.29 ct/kWh (11:00), max: 16.60 ct/kWh (17:00)
```

## Architecture

```
MCP Client → API Gateway (HTTP API) → Lambda (Rust, arm64) → aWATTar API
```

- **Runtime:** Rust compiled to native binary (`provided.al2023`)
- **Cold start:** ~10ms
- **Data source:** aWATTar EPEX Spot market data (free, no auth)
- **Region:** eu-west-1

## Tech Stack

- **Language:** Rust (tokio async runtime)
- **Lambda:** cargo-lambda for build/deploy
- **MCP Protocol:** Native JSON-RPC handler (no framework overhead)
- **HTTP client:** reqwest with rustls-tls

## Build & Deploy

### Prerequisites
- Rust toolchain (`rustup`)
- `cargo-lambda` (`cargo install cargo-lambda`)
- `zig` (for cross-compilation to Linux arm64)
- AWS credentials configured

### Build
```bash
cargo lambda build --release --arm64
```

### Deploy
```bash
cargo lambda deploy energy-mcp --region eu-west-1 --iam-role <role-arn>
```

## Project Structure

```
energy-mcp/
├── src/
│   ├── main.rs          # Lambda handler + MCP JSON-RPC router
│   ├── mcp.rs           # MCP protocol types (request/response/tool definitions)
│   ├── awattar.rs       # aWATTar API client (EPEX Spot prices)
│   └── tools/
│       ├── mod.rs        # Tool definitions and registry
│       ├── spot_prices.rs
│       ├── price_stats.rs
│       └── cheapest_window.rs
├── Cargo.toml
└── README.md
```

## Data Source

[aWATTar](https://www.awattar.de/) provides free access to EPEX Spot market data for Germany and Austria. Prices are updated daily at 14:00 CET with the following day's hourly prices.

## Roadmap

- [ ] ENTSO-E integration (all EU countries, generation mix)
- [ ] SMARD data (German dynamic household tariffs)
- [ ] Gas prices (TTF benchmark)
- [ ] Carbon intensity per hour
- [ ] Historical price trends with YoY comparison
- [ ] Landing page with live price dashboard

## License

MIT
