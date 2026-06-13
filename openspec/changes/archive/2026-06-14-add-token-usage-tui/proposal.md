## Why

Operators need a live view of token consumption from the collected `usage_events` data so they can understand recent traffic, cost drivers, failures, and latency without writing ad hoc SQL. The aggregator crate already connects to Postgres and now has `ratatui`, making it the right place for a focused terminal dashboard.

## What Changes

- Add a real-time terminal dashboard to `cpa-aggregator` for token usage statistics.
- Aggregate `usage_events` over a fixed rolling 24-hour window.
- Refresh dashboard data automatically every 3 seconds, with a manual refresh shortcut.
- Display total token usage, token breakdowns, request counts, failure rate, latency, model/provider breakdowns, recent requests, a 24-hour trend, and a 5-minute live mini chart.
- Render panels with rounded borders and a Vercel-inspired monochrome style with subtle status colors.
- No breaking changes to ingestor, config, or store public behavior.

## Capabilities

### New Capabilities

- `token-usage-dashboard`: Terminal dashboard for real-time token usage aggregation and visualization.

### Modified Capabilities

- None.

## Impact

- `cpa-aggregator`: Replace the placeholder startup message with an interactive `ratatui` dashboard loop.
- `cpa-store`: Add read-side aggregation queries over `usage_events`.
- Dependencies: Use the existing `ratatui` dependency and add terminal event handling support only if needed by implementation.
- Database: No schema change expected; queries use the existing `usage_events` table.
