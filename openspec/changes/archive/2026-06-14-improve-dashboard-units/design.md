## Context

The dashboard already aggregates and renders token usage in `cpa-aggregator`, but numeric formatting is currently raw: tokens and counts use comma-separated integers, and latency values use milliseconds only. This makes high-volume panels harder to scan in a terminal UI.

The change is presentation-only. Aggregation queries, stored values, refresh timing, and table schemas remain unchanged.

## Goals / Non-Goals

**Goals:**

- Make token totals readable with compact `K`, `M`, and `B` suffixes.
- Make request and failure counts compact when large.
- Make latency values observable by choosing milliseconds, seconds, or minutes based on magnitude.
- Make failure rates readable while preserving tiny non-zero values.
- Apply formatting consistently across summary, grouped tables, trend labels, and recent requests.

**Non-Goals:**

- No changes to SQL, aggregation windows, refresh intervals, or database schema.
- No configurable unit preferences in the first version.
- No localization or binary-unit formatting.

## Decisions

- **Use decimal units.** `K`, `M`, and `B` use base 1000 because dashboard users expect token and request counts to follow decimal notation.
- **Use at most one decimal place for compact numbers.** Examples: `999`, `1.2K`, `18.4K`, `1.2M`, `125M`. This keeps table columns short and stable.
- **Trim unnecessary `.0`.** Values like `1.0K` render as `1K`, reducing visual noise.
- **Use duration units by magnitude.** Latencies under one second render as milliseconds; latencies under one minute render as seconds with one decimal place when useful; longer latencies render as minutes and seconds.
- **Use explicit tiny-rate display.** Non-zero failure rates below `0.01%` render as `<0.01%` so rare failures are not hidden as `0.00%`.
- **Keep formatting helpers local to aggregator.** The formatting is UI presentation behavior, so helper functions should live near the `ratatui` rendering code unless another crate needs them later.

## Risks / Trade-offs

- **Compact units hide exact values** -> Recent/request panels are for scanning, not auditing. Raw values remain in the database for exact inspection.
- **Column widths can still be tight** -> Compact formatting reduces the risk, and existing `ratatui` table constraints handle truncation.
- **Unit rules may evolve** -> Keep helpers small and covered by unit tests so thresholds can be adjusted safely.

## Migration Plan

No migration is required. Deploy the updated `cpa-aggregator` binary. Rollback is reverting to the previous binary.

## Open Questions

- None for this version.
