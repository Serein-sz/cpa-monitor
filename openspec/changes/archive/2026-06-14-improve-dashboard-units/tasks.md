## 1. Formatting Helpers

- [x] 1.1 Add compact decimal formatting for token and count values using `K`, `M`, and `B` suffixes.
- [x] 1.2 Add duration formatting for latency values using milliseconds, seconds, or minutes plus seconds by magnitude.
- [x] 1.3 Add percent formatting that preserves tiny non-zero rates as `<0.01%`.

## 2. Dashboard Rendering

- [x] 2.1 Replace summary token, request, failure, latency, and failure-rate displays with human-readable units.
- [x] 2.2 Replace grouped model/provider token and count displays with compact units.
- [x] 2.3 Replace trend current/peak labels with compact token units.
- [x] 2.4 Replace recent request token and latency displays with compact token and duration units.

## 3. Verification

- [x] 3.1 Add unit tests covering compact token/count formatting thresholds.
- [x] 3.2 Add unit tests covering duration formatting thresholds.
- [x] 3.3 Add unit tests covering tiny non-zero failure-rate formatting.
- [x] 3.4 Run `cargo fmt --all -- --check`.
- [x] 3.5 Run `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] 3.6 Run `cargo test --workspace`.
