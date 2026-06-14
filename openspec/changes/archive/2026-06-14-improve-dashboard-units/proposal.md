## Why

The token usage dashboard currently renders large numbers and durations as raw values, which makes dense terminal panels harder to scan. Human-readable units will improve observability by making token volume, request counts, latency, and rates comparable at a glance.

## What Changes

- Format token values with compact units such as `K`, `M`, and `B`.
- Format request counts and failure counts with compact count units when values are large.
- Format latency values with operator-friendly duration units, using milliseconds for sub-second values and seconds or minutes for larger values.
- Keep percent displays concise and readable, including very small non-zero rates.
- Apply the formatting consistently across summary, grouped model/provider panels, trend labels, and recent request rows.
- No change to aggregation logic, database schema, refresh frequency, or TUI layout.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `token-usage-dashboard`: Dashboard numeric values use human-readable units across panels.

## Impact

- `cpa-aggregator`: Add formatting helpers and replace raw numeric formatting in dashboard rendering.
- Tests: Add unit tests for token, count, duration, and rate formatting behavior.
- Specs: Update `token-usage-dashboard` requirements to include human-readable unit display.
