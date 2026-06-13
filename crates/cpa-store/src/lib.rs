use chrono::{DateTime, Utc};
use sea_orm::{ActiveModelTrait, ConnectionTrait, Database, DatabaseConnection, DbErr, Set};
use serde::Deserialize;

pub mod dashboard;
pub mod usage_event;

#[derive(Debug, Clone)]
pub struct DbConfig {
    pub database_url: String,
}

impl DbConfig {
    pub fn from_env() -> Result<Self, std::env::VarError> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")?,
        })
    }
}

pub async fn connect(config: &DbConfig) -> Result<DatabaseConnection, DbErr> {
    Database::connect(&config.database_url).await
}

pub async fn health_check(db: &DatabaseConnection) -> Result<(), DbErr> {
    db.execute_unprepared("SELECT 1").await?;

    Ok(())
}

pub async fn ensure_schema(db: &DatabaseConnection) -> Result<(), DbErr> {
    db.execute_unprepared(
        r#"
        CREATE TABLE IF NOT EXISTS usage_events (
            id BIGSERIAL PRIMARY KEY,
            timestamp TIMESTAMPTZ NOT NULL,
            latency_ms BIGINT NOT NULL,
            source TEXT NOT NULL,
            auth_index TEXT NOT NULL,
            input_tokens BIGINT NOT NULL,
            output_tokens BIGINT NOT NULL,
            reasoning_tokens BIGINT NOT NULL,
            cached_tokens BIGINT NOT NULL,
            total_tokens BIGINT NOT NULL,
            failed BOOLEAN NOT NULL,
            provider TEXT NOT NULL,
            model TEXT NOT NULL,
            alias TEXT NOT NULL,
            endpoint TEXT NOT NULL,
            auth_type TEXT NOT NULL,
            api_key TEXT NOT NULL,
            request_id TEXT NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .await?;

    Ok(())
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct UsageEventInput {
    pub timestamp: DateTime<Utc>,
    pub latency_ms: i64,
    pub source: String,
    pub auth_index: String,
    pub tokens: UsageTokensInput,
    pub failed: bool,
    pub provider: String,
    pub model: String,
    pub alias: String,
    pub endpoint: String,
    pub auth_type: String,
    pub api_key: String,
    pub request_id: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct UsageTokensInput {
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub reasoning_tokens: i64,
    pub cached_tokens: i64,
    pub total_tokens: i64,
}

impl UsageEventInput {
    pub fn from_json(payload: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(payload)
    }
}

pub async fn insert_usage_event(
    db: &DatabaseConnection,
    input: UsageEventInput,
) -> Result<usage_event::Model, DbErr> {
    usage_event::ActiveModel {
        timestamp: Set(input.timestamp),
        latency_ms: Set(input.latency_ms),
        source: Set(input.source),
        auth_index: Set(input.auth_index),
        input_tokens: Set(input.tokens.input_tokens),
        output_tokens: Set(input.tokens.output_tokens),
        reasoning_tokens: Set(input.tokens.reasoning_tokens),
        cached_tokens: Set(input.tokens.cached_tokens),
        total_tokens: Set(input.tokens.total_tokens),
        failed: Set(input.failed),
        provider: Set(input.provider),
        model: Set(input.model),
        alias: Set(input.alias),
        endpoint: Set(input.endpoint),
        auth_type: Set(input.auth_type),
        api_key: Set(input.api_key),
        request_id: Set(input.request_id),
        ..Default::default()
    }
    .insert(db)
    .await
}

#[cfg(test)]
mod tests {
    use super::UsageEventInput;

    #[test]
    fn parses_usage_event_and_ignores_response_headers() {
        let payload = r#"{
          "timestamp": "2026-04-25T00:00:00Z",
          "latency_ms": 1500,
          "source": "user@example.com",
          "auth_index": "0",
          "tokens": {
            "input_tokens": 10,
            "output_tokens": 20,
            "reasoning_tokens": 0,
            "cached_tokens": 0,
            "total_tokens": 30
          },
          "failed": false,
          "provider": "openai",
          "model": "gpt-5.4",
          "alias": "client-gpt",
          "endpoint": "POST /v1/chat/completions",
          "auth_type": "apikey",
          "api_key": "test-key",
          "request_id": "ctx-request-id",
          "response_headers": {
            "X-Upstream-Request-Id": ["upstream-req-1"],
            "Retry-After": ["30"]
          }
        }"#;

        let event = UsageEventInput::from_json(payload).expect("usage event should parse");

        assert_eq!(event.latency_ms, 1500);
        assert_eq!(event.tokens.input_tokens, 10);
        assert_eq!(event.tokens.output_tokens, 20);
        assert_eq!(event.tokens.total_tokens, 30);
        assert_eq!(event.request_id, "ctx-request-id");
    }
}
