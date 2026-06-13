## Context

`cpa-ingestor` writes normalized usage messages into the Postgres `usage_events` table. `cpa-aggregator` currently only connects to Postgres and ensures the schema exists. The new dashboard should turn this existing table into an operator-facing terminal view without adding a web server or changing ingestion.

The dashboard runs as a long-lived terminal process. It must stay responsive while refreshing data every 3 seconds and must present a polished Vercel-like monochrome interface using `ratatui`.

## Goals / Non-Goals

**Goals:**

- Show token usage statistics for the rolling last 24 hours.
- Refresh automatically every 3 seconds and allow manual refresh.
- Display a 24-hour hourly trend plus a 5-minute live mini chart.
- Keep database aggregation logic reusable in `cpa-store`.
- Keep the UI readable in a standard terminal using rounded panels and restrained colors.

**Non-Goals:**

- No configurable time ranges in the first version.
- No web UI, HTTP API, persistence of aggregated snapshots, or alerts.
- No schema migration beyond using the existing `usage_events` table.
- No live push from Postgres; periodic polling is sufficient.

## Decisions

- **Use polling every 3 seconds.** Polling is simple, predictable, and avoids adding Postgres `LISTEN/NOTIFY` or background notification plumbing. The dashboard will query aggregate snapshots on each tick.
- **Use a fixed 24-hour SQL window.** All top-level metrics, breakdowns, recent requests, and hourly trend queries will use `timestamp >= NOW() - INTERVAL '24 hours'` so panels agree with each other.
- **Use a 5-minute mini chart with 10-second buckets.** The mini chart will query `timestamp >= NOW() - INTERVAL '5 minutes'` and bucket records by 10-second intervals, giving a live feel while preserving the 3-second refresh cadence.
- **Put read-side aggregation APIs in `cpa-store`.** `cpa-aggregator` should not contain raw SQL details. Store APIs will return a dashboard snapshot containing summary metrics, grouped rows, trend points, mini-chart points, and recent requests.
- **Render with `ratatui` rounded panels.** Panels will use rounded borders, terminal-default background, white primary text, gray secondary text, and subtle red/green status accents to match a Vercel-style terminal look.
- **Keep interaction minimal.** First version supports `q`/`Esc` to quit and `r` for immediate refresh. Time-range switching is intentionally excluded.

## Risks / Trade-offs

- **Repeated aggregate queries may load Postgres** -> Limit results for model/provider/source/recent tables, index use is expected on `timestamp`, and future work can add indexes or materialized rollups if needed.
- **Terminal dimensions vary** -> Use responsive `ratatui` layouts that degrade by truncating tables and prioritizing summary/trend panels.
- **TUI polish is subjective** -> Use explicit styling rules: rounded borders, monochrome palette, compact panels, and status colors only for success/failure.
- **Postgres time bucketing SQL is database-specific** -> This project already uses Postgres, so SQL can use Postgres functions such as `date_trunc` and `extract(epoch from timestamp)`.

## Migration Plan

No database migration is required. Deploy by building the updated `cpa-aggregator` binary and running it with the existing configuration. Rollback is replacing the binary with the previous placeholder version.

## Open Questions

- None for the first version.
