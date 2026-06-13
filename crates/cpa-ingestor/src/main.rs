use std::error::Error;

use cpa_config::AppConfig;
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = AppConfig::load()?;
    let db_config = config.db_config();
    let redis_config = config
        .redis
        .ok_or("missing required redis configuration for cpa-ingestor")?;

    println!("connecting to database: {}", db_config.database_url);
    let db = cpa_store::connect(&db_config).await?;
    cpa_store::health_check(&db).await?;
    cpa_store::ensure_schema(&db).await?;
    println!("database connected and schema verified");

    println!("connecting to redis: {}", redis_config.url);
    let client = redis::Client::open(redis_config.url)?;
    let mut pubsub = client.get_async_pubsub().await?;
    pubsub.subscribe(&redis_config.channel).await?;
    println!("subscribed to redis channel: {}", redis_config.channel);
    println!("waiting for usage events...");

    let mut messages = pubsub.on_message();
    let mut skip_first_message = true;

    while let Some(message) = messages.next().await {
        let payload: String = message.get_payload()?;

        if skip_first_message {
            skip_first_message = false;
            println!(
                "skipped initial message from Redis channel {}",
                redis_config.channel
            );
            continue;
        }

        match cpa_store::UsageEventInput::from_json(&payload) {
            Ok(event) => {
                let request_id = event.request_id.clone();
                cpa_store::insert_usage_event(&db, event).await?;
                println!("inserted usage event {request_id}");
            }
            Err(err) => {
                eprintln!("failed to parse usage event from Redis: {err}");
            }
        }
    }

    Ok(())
}
