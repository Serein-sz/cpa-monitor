use chrono::{DateTime, Utc};
use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr, QueryResult, Statement};

const GROUP_LIMIT: i64 = 8;
const RECENT_LIMIT: i64 = 12;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TokenUsageDashboard {
    pub summary: UsageSummary,
    pub models: Vec<GroupedUsage>,
    pub providers: Vec<GroupedUsage>,
    pub hourly_trend: Vec<TrendPoint>,
    pub mini_trend: Vec<TrendPoint>,
    pub recent_requests: Vec<RecentUsageRequest>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct UsageSummary {
    pub total_tokens: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub reasoning_tokens: i64,
    pub cached_tokens: i64,
    pub request_count: i64,
    pub failed_count: i64,
    pub failure_rate: f64,
    pub avg_latency_ms: f64,
    pub p95_latency_ms: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GroupedUsage {
    pub name: String,
    pub total_tokens: i64,
    pub request_count: i64,
    pub failed_count: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrendPoint {
    pub bucket: DateTime<Utc>,
    pub total_tokens: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecentUsageRequest {
    pub timestamp: DateTime<Utc>,
    pub model: String,
    pub provider: String,
    pub total_tokens: i64,
    pub latency_ms: i64,
    pub failed: bool,
}

pub async fn load_token_usage_dashboard(
    db: &DatabaseConnection,
) -> Result<TokenUsageDashboard, DbErr> {
    let summary = load_summary(db).await?;
    let models = load_grouped(db, "model", GROUP_LIMIT).await?;
    let providers = load_grouped(db, "provider", GROUP_LIMIT).await?;
    let hourly_trend = load_trend(
        db,
        "date_trunc('hour', timestamp)",
        "NOW() - INTERVAL '24 hours'",
        24,
    )
    .await?;
    let mini_trend = load_trend(
        db,
        "to_timestamp(floor(extract(epoch from timestamp) / 10) * 10)",
        "NOW() - INTERVAL '5 minutes'",
        30,
    )
    .await?;
    let recent_requests = load_recent_requests(db, RECENT_LIMIT).await?;

    Ok(TokenUsageDashboard {
        summary,
        models,
        providers,
        hourly_trend,
        mini_trend,
        recent_requests,
    })
}

fn raw_statement(db: &DatabaseConnection, sql: impl Into<String>) -> Statement {
    Statement::from_string(db.get_database_backend(), sql.into())
}

async fn load_summary(db: &DatabaseConnection) -> Result<UsageSummary, DbErr> {
    let row = db
        .query_one_raw(raw_statement(
            db,
            r#"
            SELECT
                COALESCE(SUM(total_tokens), 0)::BIGINT AS total_tokens,
                COALESCE(SUM(input_tokens), 0)::BIGINT AS input_tokens,
                COALESCE(SUM(output_tokens), 0)::BIGINT AS output_tokens,
                COALESCE(SUM(reasoning_tokens), 0)::BIGINT AS reasoning_tokens,
                COALESCE(SUM(cached_tokens), 0)::BIGINT AS cached_tokens,
                COUNT(*)::BIGINT AS request_count,
                COUNT(*) FILTER (WHERE failed)::BIGINT AS failed_count,
                COALESCE(AVG(latency_ms)::DOUBLE PRECISION, 0) AS avg_latency_ms,
                COALESCE(
                    percentile_cont(0.95) WITHIN GROUP (ORDER BY latency_ms)::DOUBLE PRECISION,
                    0
                ) AS p95_latency_ms
            FROM usage_events
            WHERE timestamp >= NOW() - INTERVAL '24 hours'
            "#,
        ))
        .await?;

    let Some(row) = row else {
        return Ok(UsageSummary::default());
    };

    let request_count = get_i64(&row, "request_count")?;
    let failed_count = get_i64(&row, "failed_count")?;

    Ok(UsageSummary {
        total_tokens: get_i64(&row, "total_tokens")?,
        input_tokens: get_i64(&row, "input_tokens")?,
        output_tokens: get_i64(&row, "output_tokens")?,
        reasoning_tokens: get_i64(&row, "reasoning_tokens")?,
        cached_tokens: get_i64(&row, "cached_tokens")?,
        request_count,
        failed_count,
        failure_rate: failure_rate(request_count, failed_count),
        avg_latency_ms: get_f64(&row, "avg_latency_ms")?,
        p95_latency_ms: get_f64(&row, "p95_latency_ms")?,
    })
}

async fn load_grouped(
    db: &DatabaseConnection,
    column: &str,
    limit: i64,
) -> Result<Vec<GroupedUsage>, DbErr> {
    let rows = db
        .query_all_raw(raw_statement(
            db,
            format!(
                r#"
                SELECT
                    {column} AS name,
                    COALESCE(SUM(total_tokens), 0)::BIGINT AS total_tokens,
                    COUNT(*)::BIGINT AS request_count,
                    COUNT(*) FILTER (WHERE failed)::BIGINT AS failed_count
                FROM usage_events
                WHERE timestamp >= NOW() - INTERVAL '24 hours'
                GROUP BY {column}
                ORDER BY total_tokens DESC, request_count DESC
                LIMIT {limit}
                "#
            ),
        ))
        .await?;

    rows.into_iter().map(grouped_from_row).collect()
}

async fn load_trend(
    db: &DatabaseConnection,
    bucket_expr: &str,
    since_expr: &str,
    limit: i64,
) -> Result<Vec<TrendPoint>, DbErr> {
    let rows = db
        .query_all_raw(raw_statement(
            db,
            format!(
                r#"
                SELECT
                    bucket,
                    total_tokens
                FROM (
                    SELECT
                        {bucket_expr} AS bucket,
                        COALESCE(SUM(total_tokens), 0)::BIGINT AS total_tokens
                    FROM usage_events
                    WHERE timestamp >= {since_expr}
                    GROUP BY bucket
                    ORDER BY bucket DESC
                    LIMIT {limit}
                ) buckets
                ORDER BY bucket ASC
                "#
            ),
        ))
        .await?;

    rows.into_iter().map(trend_from_row).collect()
}

async fn load_recent_requests(
    db: &DatabaseConnection,
    limit: i64,
) -> Result<Vec<RecentUsageRequest>, DbErr> {
    let rows = db
        .query_all_raw(raw_statement(
            db,
            format!(
                r#"
                SELECT
                    timestamp,
                    model,
                    provider,
                    total_tokens,
                    latency_ms,
                    failed
                FROM usage_events
                WHERE timestamp >= NOW() - INTERVAL '24 hours'
                ORDER BY timestamp DESC
                LIMIT {limit}
                "#
            ),
        ))
        .await?;

    rows.into_iter().map(recent_from_row).collect()
}

fn grouped_from_row(row: QueryResult) -> Result<GroupedUsage, DbErr> {
    Ok(GroupedUsage {
        name: row.try_get("", "name")?,
        total_tokens: get_i64(&row, "total_tokens")?,
        request_count: get_i64(&row, "request_count")?,
        failed_count: get_i64(&row, "failed_count")?,
    })
}

fn trend_from_row(row: QueryResult) -> Result<TrendPoint, DbErr> {
    Ok(TrendPoint {
        bucket: row.try_get("", "bucket")?,
        total_tokens: get_i64(&row, "total_tokens")?,
    })
}

fn recent_from_row(row: QueryResult) -> Result<RecentUsageRequest, DbErr> {
    Ok(RecentUsageRequest {
        timestamp: row.try_get("", "timestamp")?,
        model: row.try_get("", "model")?,
        provider: row.try_get("", "provider")?,
        total_tokens: get_i64(&row, "total_tokens")?,
        latency_ms: get_i64(&row, "latency_ms")?,
        failed: row.try_get("", "failed")?,
    })
}

fn get_i64(row: &QueryResult, column: &str) -> Result<i64, DbErr> {
    row.try_get("", column)
}

fn get_f64(row: &QueryResult, column: &str) -> Result<f64, DbErr> {
    row.try_get("", column)
}

pub fn failure_rate(request_count: i64, failed_count: i64) -> f64 {
    if request_count == 0 {
        0.0
    } else {
        failed_count as f64 / request_count as f64
    }
}

pub fn trend_values(points: &[TrendPoint]) -> Vec<u64> {
    points
        .iter()
        .map(|point| point.total_tokens.max(0) as u64)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{TrendPoint, failure_rate, trend_values};
    use chrono::{TimeZone, Utc};

    #[test]
    fn calculates_failure_rate() {
        assert_eq!(failure_rate(0, 1), 0.0);
        assert_eq!(failure_rate(4, 1), 0.25);
    }

    #[test]
    fn converts_trend_points_to_non_negative_values() {
        let points = vec![
            TrendPoint {
                bucket: Utc.timestamp_opt(0, 0).single().unwrap(),
                total_tokens: 12,
            },
            TrendPoint {
                bucket: Utc.timestamp_opt(10, 0).single().unwrap(),
                total_tokens: -3,
            },
        ];

        assert_eq!(trend_values(&points), vec![12, 0]);
    }
}
