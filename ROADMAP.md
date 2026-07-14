# Energy MCP — Roadmap & Monetization Plan

## Current State (v0.1.0)

**Live at:** https://energy-mcp.getbrechtai.com/mcp  
**Tools:** 7 (all free, Germany-only, aWATTar data)  
**Infra cost:** ~€0.50/month (Lambda + API Gateway, near-zero traffic)

| Tool | Status |
|------|--------|
| get_spot_prices | ✅ Live |
| get_price_forecast | ✅ Live |
| get_price_stats | ✅ Live |
| find_cheapest_window | ✅ Live |
| compare_tariffs | ✅ Live |
| calculate_energy_cost | ✅ Live |
| analyze_price_trends | ✅ Live |

---

## Phase 2: Multi-Country + Generation Data (after ENTSO-E approval)

### New Tools

| Tool | Description | Data Source | Tier |
|------|-------------|-------------|------|
| `search_energy_prices` | Spot prices for any EU country/bidding zone by date range | ENTSO-E | Free (today only), Pro (historical) |
| `get_generation_mix` | Real-time power generation breakdown by source (solar, wind, gas, nuclear, coal, hydro) | ENTSO-E | Free |
| `get_carbon_intensity` | gCO2/kWh for a given country and hour, derived from generation mix + emission factors | ENTSO-E (calculated) | Free (current), Pro (historical) |
| `get_cross_border_flows` | Electricity import/export between countries | ENTSO-E | Pro |

### Implementation Notes

**ENTSO-E API details:**
- Base URL: `https://web-api.tp.entsoe.eu/api`
- Auth: Security token passed as query param `securityToken=YOUR_TOKEN`
- Format: XML responses (need parsing) — consider caching as JSON
- Rate limits: Undocumented, but reasonable use expected
- Key document types:
  - `A44` — Day-ahead prices
  - `A75` — Actual generation per type
  - `A73` — Actual generation output
  - `A11` — Cross-border physical flows

**Bidding zone codes (important ones):**
- Germany: `10Y1001A1001A83F`
- Netherlands: `10YNL----------L`
- France: `10YFR-RTE------C`
- Austria: `10YAT-APG------L`
- Belgium: `10YBE----------2`
- Spain: `10YES-REE------0`
- Italy North: `10Y1001A1001A73I`

**Carbon intensity calculation:**
```
gCO2/kWh = Σ (generation_by_source × emission_factor) / total_generation

Emission factors (gCO2/kWh):
  Coal: 820
  Gas: 490
  Oil: 650
  Nuclear: 12
  Wind: 11
  Solar: 45
  Hydro: 24
  Biomass: 230
```

---

## Phase 3: Gas Prices (after OilPriceAPI key)

| Tool | Description | Data Source | Tier |
|------|-------------|-------------|------|
| `get_gas_prices` | TTF (Dutch benchmark), THE (German hub) day-ahead and forward curves | OilPriceAPI | Free (day-ahead), Pro (forwards) |
| `calculate_heating_cost` | Cost to heat X m² based on gas price, efficiency, and degree days | OilPriceAPI + weather data | Pro |

**OilPriceAPI details:**
- Endpoint: `https://api.oilpriceapi.com/v1/prices/latest`
- Auth: Bearer token in header
- Free tier: 100 requests/day
- Returns: TTF, NBP, Henry Hub in EUR/MWh and USD/MMBtu

---

## Phase 4: Premium Features & Alerts

| Feature | Description | Tier |
|---------|-------------|------|
| `get_price_alerts` | Set threshold alerts — notify when price drops below X ct/kWh | Pro |
| `optimize_schedule` | Multi-device scheduling (EV + heat pump + dishwasher) with priority constraints | Pro |
| `get_historical_prices` | Full price history (1+ years) for backtesting and analysis | Pro |
| `simulate_savings` | "If you had a dynamic tariff for the last 12 months, you'd have saved €X" | Pro |
| `get_grid_fees` | Network fees by postal code / distribution area (Germany) | Business |

---

## Monetization Implementation

### Tier Structure

```
FREE (no auth required)
├── get_spot_prices (Germany, current day)
├── get_price_forecast (tomorrow)
├── get_price_stats
├── find_cheapest_window
├── compare_tariffs (basic, default assumptions)
├── get_generation_mix (Germany, current)
└── get_carbon_intensity (Germany, current)
    Rate limit: 50 requests/day per IP

PRO (API key required, €9/month)
├── All free tools (unlimited)
├── search_energy_prices (all EU countries, historical)
├── get_carbon_intensity (all countries, historical)
├── get_gas_prices
├── analyze_price_trends (extended history)
├── calculate_energy_cost (YoY comparison, multiple periods)
├── get_historical_prices
└── simulate_savings
    Rate limit: 10,000 requests/day

BUSINESS (API key + SLA, €49/month)
├── All Pro tools
├── optimize_schedule
├── get_grid_fees
├── get_cross_border_flows
├── Webhook alerts
├── Dedicated endpoint (custom subdomain)
└── 99.9% uptime SLA
    Rate limit: 100,000 requests/day
```

### Technical Implementation (Auth)

**MCP OAuth 2.1 flow:**
1. User signs up at energy-mcp.getbrechtai.com (Stripe checkout)
2. Gets an API key (stored in DynamoDB)
3. Passes key in MCP client config:
   ```json
   {
     "mcpServers": {
       "energy": {
         "url": "https://energy-mcp.getbrechtai.com/mcp",
         "headers": {
           "Authorization": "Bearer em_sk_live_..."
         }
       }
     }
   }
   ```
4. Lambda validates key on each request, checks tier, enforces rate limits
5. Free tier: no key needed, rate limited by IP

**Stack for auth/billing:**
- DynamoDB: API keys, usage tracking, tier info
- Stripe: Subscription management, webhook on payment events
- Lambda@Edge or middleware in Rust handler: key validation, rate limiting

### Affiliate Integration

**compare_tariffs response enhancement:**
```json
{
  "savings": {
    "dynamic_saves_eur": 1.34,
    "recommendation": "Dynamic tariff saves €1.34/month..."
  },
  "switch_options": [
    {
      "provider": "Tibber",
      "monthly_fee": 4.49,
      "surcharge_ct_kwh": 19.5,
      "referral_url": "https://invite.tibber.com/YOUR_CODE",
      "estimated_monthly_cost": 92.00
    },
    {
      "provider": "Ostrom",
      "monthly_fee": 0,
      "surcharge_ct_kwh": 21.0,
      "referral_url": "https://ostrom.de/r/YOUR_CODE",
      "estimated_monthly_cost": 94.50
    }
  ]
}
```

**Affiliate programs to join:**
- Tibber: https://tibber.com/de/partnerships (€50/referral)
- Ostrom: https://ostrom.de/partner (€30/referral)
- 1Komma5°: Contact sales (custom deal for volume)
- aWATTar: https://www.awattar.de/partner (ask)

---

## Phase 5: Data Pipeline & Caching

For historical data and reliability, add a background data pipeline:

```
EventBridge (cron) → Lambda (Rust) → fetch ENTSO-E/aWATTar → store in S3/DynamoDB
                                                                     ↓
                                                          MCP Lambda reads from cache
```

**Benefits:**
- Faster responses (no upstream API call on each request)
- Historical data available for Pro tier
- Resilient to upstream API outages
- Enables analyze_price_trends with months/years of data

**Storage estimate:**
- 24 prices/day × 36 countries × 365 days = ~315K records/year
- At ~200 bytes each = ~63 MB/year in DynamoDB
- Cost: <€1/month

---

## Revenue Projections

### Year 1 (months 1-12)

| Month | Free Users | Pro | Business | Affiliate | Revenue |
|-------|-----------|-----|----------|-----------|---------|
| 1-3 | 50 | 5 | 0 | 10 switches | €545 |
| 4-6 | 200 | 20 | 2 | 30 switches | €2,378 |
| 7-9 | 500 | 50 | 5 | 60 switches | €5,695 |
| 10-12 | 1000 | 100 | 10 | 100 switches | €11,390 |

**Year 1 total: ~€60K** (conservative, assumes organic growth from MCP ecosystem adoption)

### Assumptions
- Pro conversion: 10% of free users
- Business conversion: 1% of free users
- Affiliate: 5% of compare_tariffs callers switch
- Affiliate payout: €40 avg per switch
- Churn: 5% monthly on Pro, 2% on Business

---

## Phase 6: Regional Data — Marktstammdatenregister (MaStR)

### What it unlocks

Hyperlocal energy data for any German postcode — what's installed, who operates it, how much capacity.

### New Tools

| Tool | Description | Tier |
|------|-------------|------|
| `get_local_energy_profile` | What's installed in a given PLZ — solar, wind, battery, biomass count + total capacity | Free |
| `get_local_renewables` | Detailed breakdown of renewable installations by type, age, and operator | Pro |
| `compare_regions` | Compare two PLZ/Landkreis areas by installed capacity, renewable share, grid operator | Pro |
| `get_grid_operator` | Which Netzbetreiber serves a given PLZ, their grid fees | Pro |

### Data Source

- **API:** `https://www.marktstammdatenregister.de/MaStRApi` (SOAP/WSDL)
- **Auth:** Webdienst-Key (free, register at marktstammdatenregister.de)
- **Rate limit:** 100,000 calls/day
- **Data volume:** 6.2M+ installations
- **Alternative:** Bulk download via [open-MaStR](https://github.com/OpenEnergyPlatform/open-MaStR) Python package

### Implementation Plan

**Approach: Bulk download → DynamoDB (indexed by PLZ)**

```
Step 1: Download bulk data via open-MaStR (one-time, ~2GB CSV)
Step 2: Process + aggregate by PLZ → summary records
Step 3: Store in DynamoDB (partition key: PLZ, ~8,000 unique PLZ entries)
Step 4: Rust Lambda queries DynamoDB for instant lookups
Step 5: Weekly EventBridge cron refreshes the data (delta function)
```

**DynamoDB schema:**
```json
{
  "plz": "80331",
  "total_installations": 1847,
  "total_capacity_kw": 45200,
  "solar": { "count": 1650, "capacity_kw": 32000 },
  "wind": { "count": 12, "capacity_kw": 8400 },
  "battery": { "count": 85, "capacity_kw": 2100 },
  "biomass": { "count": 8, "capacity_kw": 1200 },
  "other": { "count": 92, "capacity_kw": 1500 },
  "grid_operator": "Stadtwerke München Netze GmbH",
  "bundesland": "Bayern",
  "landkreis": "München",
  "last_updated": "2026-07-14"
}
```

**Storage cost:** ~8,000 PLZ records × ~500 bytes = ~4 MB in DynamoDB ≈ €0/month (free tier)

### Prerequisites

1. [ ] Register at marktstammdatenregister.de
2. [ ] Create Webdienst-Benutzer in account settings
3. [ ] Get Webdienst-Key
4. [ ] Run initial bulk download and process

### Example Queries

```
User: "What solar capacity is installed in my area?" (PLZ 10115)
→ get_local_energy_profile(plz: "10115")
→ "Berlin Mitte has 2,400 solar installations with 15.2 MW total capacity,
   plus 3 battery storage systems (450 kW). No wind turbines in this urban area."

User: "Compare Munich vs Hamburg for renewable installations"
→ compare_regions(plz_a: "80331", plz_b: "20095")
→ "Munich: 45 MW installed (71% solar), Hamburg: 38 MW (52% wind, 30% solar)"
```

---

1. **[Now]** Sign up for ENTSO-E API (email sent, wait 3 days)
2. **[Now]** Sign up for OilPriceAPI (instant key)
3. **[Day 3]** Implement search_energy_prices + get_generation_mix + get_carbon_intensity
4. **[Day 4]** Add API key validation in Rust handler (DynamoDB lookup)
5. **[Day 5]** Stripe integration + signup page on landing page
6. **[Week 2]** Launch on HackerNews / Reddit r/homeassistant / MCP community
7. **[Week 3]** Apply to Tibber/Ostrom affiliate programs
8. **[Week 4]** Add historical data pipeline (EventBridge + S3)

---

## Competitive Landscape

| Competitor | What they do | Our advantage |
|-----------|-------------|---------------|
| energy-charts.info | Charts for German energy data | We're MCP-native (AI-first, not browser-first) |
| Electricity Maps | Carbon intensity API | We offer prices + carbon + gas in one endpoint |
| Tibber API | Hourly prices for Tibber customers | We're provider-agnostic, no subscription needed |
| SMARD | Raw data download | We're structured, real-time, API-first |

**Our moat:** First MCP server for European energy data. As MCP adoption grows (Claude, Cursor, Copilot, custom agents), we become the default integration. Lock-in comes from agents being configured once and never changed.

## Immediate Next Steps

1. ~~[Done] Sign up for ENTSO-E API~~
2. ~~[Done] Implement ENTSO-E tools (search_energy_prices, get_generation_mix, get_carbon_intensity)~~
3. ~~[Done] Add conversational chat interface (Bedrock Claude)~~
4. **[Now] Register at marktstammdatenregister.de** → Create Webdienst-Benutzer → Get Webdienst-Key
5. **[Now] Sign up for OilPriceAPI** (instant key, adds gas prices)
6. **[This week]** Build API key auth + Stripe for monetization
7. **[This week]** Launch on HackerNews / Reddit
8. **[Next week]** MaStR bulk download + DynamoDB pipeline
9. **[Next week]** Add historical data pipeline (EventBridge + S3)
10. **[Week 3]** Apply to Tibber/Ostrom affiliate programs
