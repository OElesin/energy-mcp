# Launch Preparation — Product Hunt & HackerNews

## Timeline

| Day | Action |
|-----|--------|
| Thursday 15:00 CET | Post Show HN |
| Thursday 15:00–20:00 CET | Monitor and reply to HN comments |
| Friday | Collect feedback, note any issues |
| Following Tuesday 09:01 CET | Product Hunt launch |
| Tuesday all day | Engage with PH comments, share on social |

---

## HackerNews — Show HN

### Title
```
Show HN: Energy MCP – Real-time European electricity prices for AI agents (Rust, 11 tools)
```

### URL
```
https://energy-mcp.getbrechtai.com
```

### Text
```
I built an MCP server that gives AI agents access to European energy market data — electricity spot prices, carbon intensity, generation mix, gas prices, and smart scheduling tools.

The problem: every smart home app, EV charger, and energy startup builds the same data pipeline from scratch (ENTSO-E XML parsing, timezone handling, price normalization). This gives any AI agent or app that data in one endpoint.

11 tools, zero setup:

- Hourly spot prices for 25+ EU countries (ENTSO-E)
- Day-ahead forecast (published 14:00 CET daily)
- "Find cheapest 3-hour window" for EV charging / heat pumps
- Dynamic vs fixed tariff comparison
- Real-time generation mix (solar, wind, gas, nuclear, coal)
- Carbon intensity (gCO2/kWh from live generation data)
- TTF gas benchmark prices
- Weekly price trend analysis with WoW direction

Try it: The landing page has a chat interface — ask "When should I charge my EV?" or "How clean is Germany's grid right now?" in plain English or German. No signup needed.

For developers:

  {"mcpServers": {"energy": {"url": "https://energy-mcp.getbrechtai.com/mcp"}}}

Built in Rust on AWS Lambda (~10ms cold start). The MCP endpoint is stateless HTTP — works with Claude Desktop, Cursor, or any MCP client.

Source: https://github.com/OElesin/energy-mcp
```

### Prepared Replies to Common Questions

**Q: "Why not just use ENTSO-E directly?"**
> You absolutely can. But you'll spend 2-3 days on XML parsing, bidding zone EIC codes, 15-min to hourly aggregation, timezone normalization, and handling API quirks. With this, you paste a URL and get structured JSON back. The entire ENTSO-E integration is ~400 lines of Rust if you want to see how it's done: [link to src/entsoe.rs]

**Q: "What about rate limits / pricing?"**
> Currently free with no hard limits. Planning to add tiers: free (50 req/day, Germany), Pro (unlimited, all countries, historical data). Infrastructure cost is near-zero on Lambda so there's no rush to gate it.

**Q: "Why Rust for this?"**
> Cold starts. Python on Lambda with the MCP SDK takes 1.5-2 seconds to start. This Rust binary cold-starts in ~10ms. For energy data that people want to check 20x/day, that latency matters. Also the binary is 5MB vs 50MB+ for Python with dependencies.

**Q: "How is this different from Electricity Maps?"**
> Electricity Maps is great for carbon intensity specifically. This is broader — prices, forecasts, scheduling optimization, tariff comparison, gas prices, generation mix, AND carbon intensity. Also MCP-native, so any AI agent can use it without custom integration code.

**Q: "How do you make money?"**
> Not yet, it's 2 weeks old. Plan is developer API tiers (Pro/Business) and potentially affiliate revenue from tariff switching (the compare_tariffs tool naturally surfaces which dynamic tariff saves money). Open to B2B partnerships with HEMS companies who need this data.

**Q: "Is this EU-only?"**
> Yes, by design. European energy markets are fragmented (25+ bidding zones, different TSOs, mixed regulations). That fragmentation is the moat — it's genuinely hard to normalize this data across countries.

---

## Product Hunt

### Tagline (60 chars)
```
Real-time EU energy prices for AI agents and smart homes
```

### Description (260 chars)
```
Ask your AI "When should I charge my EV?" and get a real answer. 11 tools covering electricity prices, carbon intensity, generation mix, and gas prices across 25+ EU countries. Rust-powered, ~10ms latency. Free to use.
```

### Topics
- Artificial Intelligence
- Developer Tools
- Sustainability
- Smart Home

### Maker Comment (post immediately after launch)
```
Hi PH! 👋

I built this because I was tired of manually checking electricity prices before charging my EV. Germany made dynamic tariffs mandatory in 2025, but the tools to actually use them haven't caught up.

So I built an MCP server — a standardized API that any AI assistant (Claude, Cursor, ChatGPT) can connect to and answer energy questions with real data:

⚡ "When's the cheapest time to run my dishwasher?" → finds the optimal window
🌱 "How clean is Germany's grid right now?" → real-time carbon intensity from generation mix
📊 "Compare dynamic vs fixed tariff for my 300 kWh/month" → shows actual savings

The chat on the landing page is a live demo — try it in English or German, no signup needed.

Tech: Rust on AWS Lambda (~10ms cold start), ENTSO-E for 25+ EU countries, aWATTar for German spot prices. Open source.

What we're building next:
- Home Assistant integration
- Historical data for price trend analysis
- API tiers for commercial use
- Regional data (installed solar/wind by postcode)

Would love feedback on what tools would be most useful for your energy setup! 🔌
```

### Screenshots to Capture

Take these from your browser (1270x760px, or 2x for retina):

1. **Hero shot** — Full landing page above the fold
   - URL: https://energy-mcp.getbrechtai.com
   - Show: hero text, config snippet, "Live" badge

2. **Chat in action** — The AI answering an EV charging question
   - Click "⚡ EV charging" example button, wait for response
   - Screenshot the chat with the full answer visible

3. **Playground with generation mix** — Technical credibility
   - Select "get_generation_mix", enter "DE", click Run
   - Screenshot showing the JSON response with solar/wind/gas breakdown

4. **Multi-country comparison** — Shows breadth
   - In chat: ask "Compare electricity prices in France vs Germany"
   - Screenshot the response

5. **Carbon intensity result** — Sustainability angle
   - Click "🌱 Grid cleanliness" in chat
   - Screenshot showing gCO2/kWh, rating, renewable share

### Thumbnail/Logo (240x240)
- Use a simple icon: ⚡ on emerald green background
- Or text: "Energy MCP" in Inter Bold on dark background

---

## Social Sharing (LinkedIn, Twitter/X)

### LinkedIn Post (post day of HN launch)
```
I launched an open-source MCP server for European energy data today.

It gives AI agents access to:
→ Real-time electricity spot prices (25+ EU countries)
→ Carbon intensity from live generation mix
→ "Find cheapest 3-hour window" for EV charging
→ Dynamic vs fixed tariff comparison
→ Gas prices (TTF benchmark)

The reason: Germany made dynamic tariffs mandatory in 2025. Millions of new customers need tools to actually use them. Every HEMS company, every EV charger, every smart home app solves the same data problem from scratch.

This is the shared data layer.

Built in Rust (~10ms cold start on Lambda), 11 tools, free to use.

Try it: https://energy-mcp.getbrechtai.com
(The chat interface works — ask it when to charge your EV)

Source: https://github.com/OElesin/energy-mcp

#OpenSource #Energy #AI #MCP #Rust #CleanTech
```

### Twitter/X Thread
```
🧵 I built an MCP server for European energy data. Here's why:

1/ Germany made dynamic electricity tariffs mandatory in 2025. But the tools to use them? Still stuck in 2020.

2/ Every smart home company builds the same pipeline: fetch ENTSO-E XML → parse → normalize → expose. Over and over.

3/ So I built the shared layer. One endpoint, 11 tools, 25+ EU countries. Any AI agent can use it.

4/ Ask it: "When should I charge my EV?" It actually checks the spot market and finds the cheapest window.

5/ Tech: Rust on Lambda (~10ms cold start), ENTSO-E + aWATTar for real data, Bedrock Claude for the chat.

6/ Free, open source, live now 👇
https://energy-mcp.getbrechtai.com

Try the chat — it speaks German too 🇩🇪
```

---

## Pre-launch Checklist

- [ ] Test the landing page loads fast (check on mobile too)
- [ ] Test the chat responds within 5 seconds
- [ ] Test all playground examples still work
- [ ] Create Product Hunt maker account
- [ ] Prepare 5 screenshots (see above)
- [ ] Line up 5-10 people to upvote on PH in the first hour
- [ ] Schedule LinkedIn post for same time as HN
- [ ] Have the GitHub repo README clean and up to date
