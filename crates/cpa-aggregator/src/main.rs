use std::error::Error;
use std::time::{Duration, Instant};

use cpa_config::AppConfig;
use cpa_store::dashboard::{GroupedUsage, RecentUsageRequest, TokenUsageDashboard, trend_values};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Sparkline, Table, Wrap};
use ratatui::{DefaultTerminal, Frame};

const REFRESH_INTERVAL: Duration = Duration::from_secs(3);

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = AppConfig::load()?;
    let db_config = config.db_config();
    let db = cpa_store::connect(&db_config).await?;
    cpa_store::health_check(&db).await?;
    cpa_store::ensure_schema(&db).await?;

    let mut terminal = ratatui::try_init()?;
    let result = run_dashboard(&mut terminal, &db).await;
    ratatui::restore();

    result
}

struct App {
    dashboard: TokenUsageDashboard,
    last_updated: chrono::DateTime<chrono::Utc>,
    last_error: Option<String>,
}

impl App {
    async fn load(db: &sea_orm::DatabaseConnection) -> Self {
        match cpa_store::dashboard::load_token_usage_dashboard(db).await {
            Ok(dashboard) => Self {
                dashboard,
                last_updated: chrono::Utc::now(),
                last_error: None,
            },
            Err(err) => Self {
                dashboard: TokenUsageDashboard::default(),
                last_updated: chrono::Utc::now(),
                last_error: Some(err.to_string()),
            },
        }
    }

    async fn refresh(&mut self, db: &sea_orm::DatabaseConnection) {
        match cpa_store::dashboard::load_token_usage_dashboard(db).await {
            Ok(dashboard) => {
                self.dashboard = dashboard;
                self.last_updated = chrono::Utc::now();
                self.last_error = None;
            }
            Err(err) => {
                self.last_updated = chrono::Utc::now();
                self.last_error = Some(err.to_string());
            }
        }
    }
}

async fn run_dashboard(
    terminal: &mut DefaultTerminal,
    db: &sea_orm::DatabaseConnection,
) -> Result<(), Box<dyn Error>> {
    let mut app = App::load(db).await;
    let mut next_refresh = Instant::now() + REFRESH_INTERVAL;

    loop {
        terminal.draw(|frame| render_dashboard(frame, &app))?;

        let timeout = next_refresh
            .checked_duration_since(Instant::now())
            .unwrap_or(Duration::ZERO)
            .min(Duration::from_millis(100));

        if event::poll(timeout)?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                KeyCode::Char('r') => {
                    app.refresh(db).await;
                    next_refresh = Instant::now() + REFRESH_INTERVAL;
                }
                _ => {}
            }
        }

        if Instant::now() >= next_refresh {
            app.refresh(db).await;
            next_refresh = Instant::now() + REFRESH_INTERVAL;
        }
    }
}

fn render_dashboard(frame: &mut Frame<'_>, app: &App) {
    let area = frame.area();

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Length(10),
            Constraint::Length(9),
            Constraint::Min(8),
        ])
        .split(area);

    render_summary(frame, outer[0], app);

    let middle = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(outer[1]);
    render_grouped_table(frame, middle[0], "Models", &app.dashboard.models);
    render_grouped_table(frame, middle[1], "Providers", &app.dashboard.providers);

    render_trends(frame, outer[2], app);
    render_recent(frame, outer[3], &app.dashboard.recent_requests);
}

fn render_summary(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let summary = &app.dashboard.summary;
    let error = app
        .last_error
        .as_deref()
        .map(|err| format!(" | error: {err}"))
        .unwrap_or_default();

    let lines = vec![
        Line::from(vec![
            label("Tokens "),
            value(format_tokens(summary.total_tokens)),
            label("   Requests "),
            value(format_count(summary.request_count)),
            label("   Failed "),
            danger(format!(
                "{} ({})",
                format_count(summary.failed_count),
                format_percent(summary.failure_rate)
            )),
            label("   Avg "),
            value(format_duration_ms(summary.avg_latency_ms.round() as i64)),
            label("   P95 "),
            value(format_duration_ms(summary.p95_latency_ms.round() as i64)),
        ]),
        Line::from(vec![
            label("Input "),
            value(format_tokens(summary.input_tokens)),
            label("   Output "),
            value(format_tokens(summary.output_tokens)),
            label("   Reasoning "),
            value(format_tokens(summary.reasoning_tokens)),
            label("   Cached "),
            value(format_tokens(summary.cached_tokens)),
        ]),
        Line::from(vec![
            label("Last refresh "),
            value(format_utc_plus_8(app.last_updated)),
            label("   Window "),
            value("24h"),
            label("   Auto "),
            value("3s"),
            danger(error),
        ]),
    ];

    frame.render_widget(
        Paragraph::new(lines)
            .block(panel("Last 24h Usage"))
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn render_grouped_table(
    frame: &mut Frame<'_>,
    area: Rect,
    title: &'static str,
    rows: &[GroupedUsage],
) {
    let table_rows = rows.iter().map(|row| {
        Row::new(vec![
            Cell::from(row.name.clone()).style(primary()),
            Cell::from(format_tokens(row.total_tokens)),
            Cell::from(format_count(row.request_count)),
            Cell::from(format_count(row.failed_count)).style(if row.failed_count > 0 {
                status_bad()
            } else {
                muted()
            }),
        ])
    });

    let table = Table::new(
        table_rows,
        [
            Constraint::Percentage(42),
            Constraint::Percentage(24),
            Constraint::Percentage(18),
            Constraint::Percentage(16),
        ],
    )
    .header(
        Row::new(vec!["name", "tokens", "reqs", "fail"])
            .style(muted().add_modifier(Modifier::BOLD))
            .bottom_margin(1),
    )
    .block(panel(title))
    .style(secondary())
    .column_spacing(1);

    frame.render_widget(table, area);
}

fn render_trends(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Length(5)])
        .split(area);

    let hourly = trend_values(&app.dashboard.hourly_trend);
    let mini = trend_values(&app.dashboard.mini_trend);
    let now = app
        .dashboard
        .mini_trend
        .last()
        .map(|point| point.total_tokens)
        .unwrap_or(0);
    let peak = app
        .dashboard
        .mini_trend
        .iter()
        .map(|point| point.total_tokens)
        .max()
        .unwrap_or(0);

    frame.render_widget(
        Sparkline::default()
            .block(panel("24h hourly tokens"))
            .data(hourly)
            .style(Style::default().fg(Color::White)),
        chunks[0],
    );

    let mini_area = chunks[1];
    let mini_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(2)])
        .split(mini_area);
    frame.render_widget(
        Sparkline::default()
            .block(panel("5m live mini chart"))
            .data(mini)
            .style(Style::default().fg(Color::Gray)),
        mini_chunks[0],
    );
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            label("Current bucket "),
            value(format_tokens(now)),
            label(" tokens   Peak "),
            value(format_tokens(peak)),
            label(" tokens"),
        ]))
        .style(secondary()),
        mini_chunks[1],
    );
}

fn render_recent(frame: &mut Frame<'_>, area: Rect, rows: &[RecentUsageRequest]) {
    let table_rows = rows.iter().map(|row| {
        Row::new(vec![
            Cell::from(format_utc_plus_8(row.timestamp)),
            Cell::from(row.model.clone()),
            Cell::from(row.provider.clone()),
            Cell::from(format_tokens(row.total_tokens)),
            Cell::from(format_duration_ms(row.latency_ms)),
            Cell::from(if row.failed { "failed" } else { "ok" }).style(if row.failed {
                status_bad()
            } else {
                status_good()
            }),
        ])
    });

    let table = Table::new(
        table_rows,
        [
            Constraint::Length(8),
            Constraint::Percentage(24),
            Constraint::Percentage(20),
            Constraint::Percentage(16),
            Constraint::Length(9),
            Constraint::Length(8),
        ],
    )
    .header(
        Row::new(vec![
            "time", "model", "provider", "tokens", "latency", "status",
        ])
        .style(muted().add_modifier(Modifier::BOLD))
        .bottom_margin(1),
    )
    .block(panel("Recent Requests"))
    .style(secondary())
    .column_spacing(1);

    frame.render_widget(table, area);
}

fn panel(title: &'static str) -> Block<'static> {
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray))
        .style(Style::default().fg(Color::Gray))
}

fn label(text: impl Into<String>) -> Span<'static> {
    Span::styled(text.into(), muted())
}

fn value(text: impl Into<String>) -> Span<'static> {
    Span::styled(text.into(), primary().add_modifier(Modifier::BOLD))
}

fn danger(text: impl Into<String>) -> Span<'static> {
    Span::styled(text.into(), status_bad())
}

fn primary() -> Style {
    Style::default().fg(Color::White)
}

fn secondary() -> Style {
    Style::default().fg(Color::Gray)
}

fn muted() -> Style {
    Style::default().fg(Color::DarkGray)
}

fn status_good() -> Style {
    Style::default().fg(Color::Green)
}

fn status_bad() -> Style {
    Style::default().fg(Color::Red)
}

fn format_tokens(value: i64) -> String {
    format_compact_decimal(value)
}

fn format_count(value: i64) -> String {
    format_compact_decimal(value)
}

fn format_compact_decimal(value: i64) -> String {
    let sign = if value < 0 { "-" } else { "" };
    let abs = value.abs() as f64;

    let (scaled, suffix) = if abs >= 1_000_000_000.0 {
        (abs / 1_000_000_000.0, "B")
    } else if abs >= 1_000_000.0 {
        (abs / 1_000_000.0, "M")
    } else if abs >= 1_000.0 {
        (abs / 1_000.0, "K")
    } else {
        return format!("{value}");
    };

    format!("{sign}{}{}", trim_one_decimal(scaled), suffix)
}

fn format_duration_ms(value: i64) -> String {
    let sign = if value < 0 { "-" } else { "" };
    let abs = value.abs();

    if abs < 1_000 {
        format!("{value}ms")
    } else if abs < 60_000 {
        let seconds = abs as f64 / 1_000.0;
        format!("{sign}{}s", trim_one_decimal(seconds))
    } else {
        let minutes = abs / 60_000;
        let seconds = (abs % 60_000) / 1_000;
        if seconds == 0 {
            format!("{sign}{minutes}m")
        } else {
            format!("{sign}{minutes}m {seconds}s")
        }
    }
}

fn format_percent(rate: f64) -> String {
    if rate == 0.0 {
        "0%".to_owned()
    } else {
        let percent = rate * 100.0;
        if percent.abs() < 0.01 {
            if percent.is_sign_negative() {
                ">-0.01%".to_owned()
            } else {
                "<0.01%".to_owned()
            }
        } else {
            format!("{}%", trim_two_decimals(percent))
        }
    }
}

fn format_utc_plus_8(value: chrono::DateTime<chrono::Utc>) -> String {
    let offset = chrono::FixedOffset::east_opt(8 * 60 * 60).expect("UTC+8 offset should be valid");
    value
        .with_timezone(&offset)
        .format("%H:%M:%S UTC+8")
        .to_string()
}

fn trim_one_decimal(value: f64) -> String {
    let formatted = format!("{value:.1}");
    trim_decimal_zeros(formatted)
}

fn trim_two_decimals(value: f64) -> String {
    let formatted = format!("{value:.2}");
    trim_decimal_zeros(formatted)
}

fn trim_decimal_zeros(mut value: String) -> String {
    if value.contains('.') {
        while value.ends_with('0') {
            value.pop();
        }
        if value.ends_with('.') {
            value.pop();
        }
    }
    value
}

#[cfg(test)]
mod tests {
    use super::{
        format_count, format_duration_ms, format_percent, format_tokens, format_utc_plus_8,
    };
    use chrono::TimeZone;

    #[test]
    fn formats_compact_tokens_and_counts() {
        assert_eq!(format_tokens(999), "999");
        assert_eq!(format_tokens(1_000), "1K");
        assert_eq!(format_tokens(1_250), "1.2K");
        assert_eq!(format_tokens(18_400), "18.4K");
        assert_eq!(format_tokens(1_250_000), "1.2M");
        assert_eq!(format_tokens(125_000_000), "125M");
        assert_eq!(format_tokens(1_250_000_000), "1.2B");
        assert_eq!(format_count(-1_250), "-1.2K");
    }

    #[test]
    fn formats_latency_by_magnitude() {
        assert_eq!(format_duration_ms(850), "850ms");
        assert_eq!(format_duration_ms(1_000), "1s");
        assert_eq!(format_duration_ms(1_500), "1.5s");
        assert_eq!(format_duration_ms(59_900), "59.9s");
        assert_eq!(format_duration_ms(60_000), "1m");
        assert_eq!(format_duration_ms(72_000), "1m 12s");
        assert_eq!(format_duration_ms(-1_500), "-1.5s");
    }

    #[test]
    fn formats_rates_without_hiding_tiny_nonzero_values() {
        assert_eq!(format_percent(0.0), "0%");
        assert_eq!(format_percent(0.0012), "0.12%");
        assert_eq!(format_percent(0.42), "42%");
        assert_eq!(format_percent(0.0000001), "<0.01%");
    }

    #[test]
    fn formats_refresh_time_as_utc_plus_8() {
        let time = chrono::Utc
            .with_ymd_and_hms(2026, 4, 24, 16, 30, 5)
            .unwrap();

        assert_eq!(format_utc_plus_8(time), "00:30:05 UTC+8");
    }
}
