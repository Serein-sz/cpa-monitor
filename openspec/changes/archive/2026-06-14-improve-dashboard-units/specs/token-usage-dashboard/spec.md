## ADDED Requirements

### Requirement: Dashboard uses human-readable units
The system SHALL render dashboard numeric values with human-readable units appropriate to their metric type and magnitude.

#### Scenario: Token values use compact units
- **WHEN** token values are displayed in summary, grouped, trend label, or recent request panels
- **THEN** values at or above one thousand use decimal compact units such as `K`, `M`, or `B`

#### Scenario: Latency values use observable duration units
- **WHEN** latency values are displayed in summary or recent request panels
- **THEN** the dashboard uses milliseconds for sub-second values, seconds for values under one minute, and minutes plus seconds for longer values

#### Scenario: Small non-zero rates remain visible
- **WHEN** a non-zero failure rate rounds below `0.01%`
- **THEN** the dashboard displays the rate as `<0.01%`
