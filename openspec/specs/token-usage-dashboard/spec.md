## Requirements

### Requirement: Dashboard uses rolling 24-hour data
The system SHALL aggregate dashboard metrics from `usage_events` whose `timestamp` is within the rolling last 24 hours.

#### Scenario: Excludes older records
- **WHEN** usage records exist both inside and outside the last 24 hours
- **THEN** the dashboard metrics include only records from the last 24 hours

### Requirement: Dashboard refreshes automatically
The system SHALL refresh dashboard data automatically every 3 seconds while `cpa-aggregator` is running.

#### Scenario: Periodic refresh updates metrics
- **WHEN** new usage records are inserted after the dashboard starts
- **THEN** the dashboard reflects the new records after the next 3-second refresh interval

### Requirement: Dashboard supports manual refresh and exit
The system SHALL allow operators to refresh immediately with `r` and exit with `q` or `Esc`.

#### Scenario: Manual refresh
- **WHEN** the operator presses `r`
- **THEN** the dashboard reloads data without waiting for the next automatic refresh tick

#### Scenario: Exit dashboard
- **WHEN** the operator presses `q` or `Esc`
- **THEN** `cpa-aggregator` exits cleanly and restores the terminal

### Requirement: Dashboard displays summary metrics
The system SHALL display total tokens, input tokens, output tokens, reasoning tokens, cached tokens, request count, failed request count, failure rate, average latency, and p95 latency for the last 24 hours.

#### Scenario: Summary metrics are visible
- **WHEN** the dashboard renders with usage data available
- **THEN** the summary panel shows token totals, request counts, failure metrics, and latency metrics

### Requirement: Dashboard displays grouped usage
The system SHALL display grouped token usage by model and by provider for the last 24 hours.

#### Scenario: Grouped rows are visible
- **WHEN** usage records contain multiple models or providers
- **THEN** the dashboard shows separate grouped rows with token totals and request counts

### Requirement: Dashboard displays token trends
The system SHALL display a 24-hour hourly token trend and a 5-minute live mini chart using 10-second buckets.

#### Scenario: Trend panels render
- **WHEN** usage data exists in the last 24 hours and last 5 minutes
- **THEN** the dashboard renders both the hourly trend and the 5-minute mini chart

### Requirement: Dashboard displays recent requests
The system SHALL display recent usage requests from the last 24 hours with timestamp, model, provider, token total, latency, and status.

#### Scenario: Recent request rows are visible
- **WHEN** recent usage records exist
- **THEN** the dashboard shows the newest requests first with their key request fields

### Requirement: Dashboard uses polished terminal styling
The system SHALL render a Vercel-inspired terminal interface with rounded panels, no explicit background color, white primary text, gray secondary text, and subtle status colors.

#### Scenario: Styled panels render
- **WHEN** the dashboard draws its UI
- **THEN** major sections are displayed as rounded panels using the configured monochrome style
