# Energy MCP — Positioning & Go-to-Market

## One-liner

**The data layer for AI-powered energy management in Europe.**

---

## Problem

Every company building smart energy products (HEMS, EV chargers, heat pump controllers, dynamic tariff apps) solves the same data problem from scratch:

- Fetch ENTSO-E prices (XML parsing, 25+ country codes)
- Normalize aWATTar/EPEX Spot data
- Calculate carbon intensity from generation mix
- Map grid operators to postcodes
- Handle timezone edge cases, resolution differences, API quirks

They all build this internally, maintain it, and it's never their core product. It's plumbing.

---

## Solution

One API that gives any AI agent or application instant access to structured European energy market data. No XML parsing, no API key juggling, no data pipeline maintenance.

```json
{
  "mcpServers": {
    "energy": { "url": "https://energy-mcp.getbrechtai.com/mcp" }
  }
}
```

Or for non-AI apps: a simple REST/chat endpoint that returns actionable answers.

---

## Who Pays

| Segment | Example Companies | What they need | WTP |
|---------|-------------------|----------------|-----|
| **HEMS manufacturers** | Sigenergy, EcoFlow, SolaX, Solis | Real-time prices + forecast for scheduling algorithms | €200-500/mo |
| **Dynamic tariff providers** | Tibber competitors, white-label apps | Multi-country price data without building own pipeline | €100-300/mo |
| **Smart home platforms** | Home Assistant add-ons, Homey | Reliable price sensors, cheapest window logic | €9-29/mo |
| **EV charging operators** | Small fleet managers, workplace charging | Optimal charging schedules across countries | €49-199/mo |
| **Energy consultants / fintechs** | Comparison platforms, switching services | Historical data, tariff simulation, regional breakdowns | €49-99/mo |
| **Individual developers / prosumers** | Tinkerers, HA users, solar owners | Personal automation triggers, dashboards | €0-9/mo |

---

## Positioning vs Alternatives

| | Energy MCP | Tibber API | ENTSO-E Direct | Electricity Maps |
|---|---|---|---|---|
| Access model | Open (MCP + REST) | Tibber customers only | Free but raw XML | Paid API |
| Coverage | Prices + gen mix + carbon + scheduling | Prices only (own customers) | Prices + generation | Carbon only |
| Format | Structured JSON, AI-ready | GraphQL | XML, no tooling | JSON |
| Intelligence | Built-in (cheapest window, tariff comparison, trends) | Basic hourly price | Raw data only | Raw intensity |
| Setup time | 10 seconds (paste URL) | Need Tibber subscription | Days (register, parse XML) | Sign up + integrate |
| Target | Developers + AI agents | End consumers | Data engineers | Carbon-focused apps |

**Key differentiator:** Not a data dump. **Pre-computed intelligence** — cheapest windows, tariff comparisons, trend analysis, carbon ratings. The AI chat layer proves this works for end users too.

---

## Market Context

### Demand Signals

- **10M+ electricity contract switches** in Germany in 2024 (up from 6M in 2023)
- **Germany Smart Home market:** €9.5B (2025) → €15B (2030)
- **Europe Energy Management Systems:** €17B → €67B by 2034 (14.8% CAGR)
- **Dynamic tariff law (§14a EnWG):** Since 2025, every German provider must offer dynamic tariffs
- **Smart meter enforcement:** Bundesnetzagentur opened 77 proceedings (March 2026) to force rollout
- Only **2% of consumers** currently use dynamic tariffs — massive greenfield

### Competitive Landscape

| Player | What they do | Relationship to us |
|--------|-------------|-------------------|
| Sigenergy | AI trading engine + hardware (Intersolar 2026) | Potential customer (needs price data) |
| EcoFlow | OASIS 3.0 AI home energy management | Potential customer |
| Cloover (Berlin) | $1.2B raised, AI energy platform | Potential customer or competitor |
| Tibber/Ostrom | Dynamic tariff providers with apps | Competitor on consumer, but locked to own customers |
| Electricity Maps | Carbon intensity API | Competitor on carbon, but we're broader |
| ENTSO-E/SMARD | Raw public data sources | Our upstream data, not competitors |

### Why Now

1. **Regulatory push:** Dynamic tariffs mandatory in Germany since 2025
2. **Smart meter rollout:** Government forcing deployment, enforcement in 2026
3. **AI agent adoption:** MCP standard gaining traction (Claude, Cursor, Copilot)
4. **Hardware boom:** Every HEMS company adding "AI optimization" — they all need this data
5. **Post-crisis awareness:** European consumers care about energy prices like never before

---

## Go-to-Market

### Phase 1: Developer Traction (now → 3 months)

- Free tier: 50 req/day, Germany, current data
- Launch on HackerNews, Reddit r/homeassistant, Home Assistant forums
- Publish Home Assistant custom component (uses our API as backend)
- Open source stays — builds trust, attracts contributors
- Chat interface demonstrates consumer value

### Phase 2: B2B Outreach (month 2-4)

- Identify 20 HEMS/energy startups in Germany (Intersolar exhibitor lists)
- Offer free Pro tier for 3 months in exchange for feedback
- Case study: "How [Company X] reduced integration time from 2 weeks to 2 hours"
- Direct outreach to Sigenergy, EcoFlow developer relations teams

### Phase 3: Self-serve Revenue (month 3-6)

- Stripe checkout on landing page
- Tier pricing:
  - Free: 50 req/day, Germany, current data
  - Pro: €9/mo (individuals) — unlimited, all countries, historical
  - Business: €49/mo (startups) — higher limits, priority support
  - Enterprise: €199/mo — SLA, custom endpoints, dedicated support
- Affiliate integration for tariff switches (Tibber, Ostrom)

### Phase 4: Enterprise (month 6+)

- Custom SLAs
- On-premise deployment option (Rust binary runs anywhere)
- White-label: "Powered by Energy MCP" inside partner products
- Dedicated infrastructure for high-volume customers

---

## Metrics & Targets

| Metric | 3 months | 6 months | 12 months |
|--------|----------|----------|-----------|
| API calls/day | 1,000 | 10,000 | 50,000 |
| Registered developers | 50 | 200 | 1,000 |
| Paying customers | 5 | 30 | 100 |
| MRR | €200 | €3,000 | €15,000 |
| Countries covered | 25 | 25 | 30+ |
| Tools available | 10 | 15 | 20+ |
| Home Assistant installs | 100 | 500 | 2,000 |

---

## Strategic Decision

**Developer tools company (Stripe/Twilio model) — recommended**

| | Developer tools | Consumer product |
|---|---|---|
| Revenue model | API subscriptions | Affiliate + premium features |
| Effort | Lower (product exists) | Higher (app, marketing, support) |
| Competition | Low (no one does this) | High (Tibber, Ostrom, 1Komma5°) |
| Ceiling | €500K-5M ARR | €10M+ ARR (needs funding) |
| Your advantage | Technical (Rust, MCP, first mover) | None vs funded competitors |

The chat interface proves consumer viability and serves as marketing. Revenue comes from B2B.

If organic consumer demand explodes, add a consumer app later — on top of your own API.

---

## Current State

- ✅ 10 tools live (7 aWATTar + 3 ENTSO-E)
- ✅ 25+ European countries
- ✅ Landing page with interactive playground
- ✅ AI chat interface (Bedrock Claude)
- ✅ Custom domain (energy-mcp.getbrechtai.com)
- ✅ Near-zero latency (Rust, ~10ms cold start)
- ✅ Near-zero cost (Lambda, pay-per-use)
- ⬜ API key auth + rate limiting
- ⬜ Stripe integration
- ⬜ Home Assistant component
- ⬜ Historical data pipeline
- ⬜ Regional data (MaStR)
- ⬜ Gas prices

---

## Infrastructure Cost at Scale

| Traffic | Lambda cost | Data transfer | Total |
|---------|------------|---------------|-------|
| 1,000 req/day | ~€3/mo | ~€1/mo | ~€4/mo |
| 10,000 req/day | ~€15/mo | ~€5/mo | ~€20/mo |
| 100,000 req/day | ~€80/mo | ~€30/mo | ~€110/mo |

Margins stay above 90% at any scale with current architecture.
