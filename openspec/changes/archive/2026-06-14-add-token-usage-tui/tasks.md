## 1. Data Aggregation

- [x] 1.1 Add `cpa-store` dashboard data structs for summary metrics, grouped usage rows, trend points, mini-chart points, and recent requests.
- [x] 1.2 Implement a `load_token_usage_dashboard` query API that aggregates `usage_events` over the rolling last 24 hours.
- [x] 1.3 Add SQL for hourly 24-hour trend buckets and 5-minute mini-chart buckets using 10-second intervals.
- [x] 1.4 Limit grouped and recent-request query results to keep the TUI compact and predictable.

## 2. Aggregator TUI Runtime

- [x] 2.1 Replace the placeholder `cpa-aggregator` output with terminal setup and teardown for a `ratatui` application.
- [x] 2.2 Implement a dashboard event loop that refreshes automatically every 3 seconds.
- [x] 2.3 Implement keyboard handling for `r` manual refresh and `q`/`Esc` clean exit.
- [x] 2.4 Preserve existing config loading, database connection, health check, and schema initialization.

## 3. Dashboard Rendering

- [x] 3.1 Render a top summary panel with token totals, token breakdowns, request counts, failure rate, average latency, and p95 latency.
- [x] 3.2 Render grouped model and provider panels with token totals and request counts.
- [x] 3.3 Render a trend panel containing both the 24-hour hourly chart and 5-minute live mini chart.
- [x] 3.4 Render a recent requests panel with timestamp, model, provider, total tokens, latency, and status.
- [x] 3.5 Apply Vercel-style terminal theming with terminal-default background, white primary text, gray secondary text, rounded borders, and subtle status colors.

## 4. Verification

- [x] 4.1 Add unit tests for JSON-independent aggregation helper behavior where practical.
- [x] 4.2 Add tests or compile-time checks for dashboard data structs and query mapping.
- [x] 4.3 Run `cargo fmt --all -- --check`.
- [x] 4.4 Run `cargo check --workspace`.
- [x] 4.5 Run `cargo test --workspace`.
