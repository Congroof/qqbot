mod config;
mod handler;

use ai_chat_sdk::AiClient;
use onebot::{WsClient, WsConfig};
use tracing_subscriber::EnvFilter;

use crate::config::BotConfig;
use crate::handler::HandlerContext;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let config = BotConfig::from_env();

    let ai = AiClient::builder()
        .base_url(&config.ai_base_url)
        .api_key(&config.ai_api_key)
        .gateway_uid(&config.ai_gateway_uid)
        .product_name(&config.ai_product_name)
        .intention_code(&config.ai_intention_code)
        .build_client()
        .expect("failed to build AI client");

    let ws_config = match &config.access_token {
        Some(token) => WsConfig::new(&config.ws_url).with_token(token),
        None => WsConfig::new(&config.ws_url),
    };

    let mut ws = WsClient::connect(ws_config)
        .await
        .expect("failed to connect to OneBot WebSocket");

    tracing::info!(url = %config.ws_url, "bot started");

    let mut ctx = HandlerContext::new(ws.api().clone(), ai, config);

    while let Some(event) = ws.next_event().await {
        handler::dispatch(&mut ctx, &event).await;
    }

    tracing::info!("websocket connection closed, bot exiting");
}
