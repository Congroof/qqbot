pub mod ai_chat;
pub mod cmd;
pub mod like;
pub mod poke;
pub mod repeater;
pub mod request;
pub mod welcome;

use std::collections::HashMap;
use std::time::Instant;

use ai_chat_sdk::AiClient;
use onebot::event::notice::{NoticeEvent, NotifyEvent};
use onebot::{ApiCaller, Event};

use crate::config::BotConfig;
use self::ai_chat::ChatSession;
use self::repeater::RepeatState;

/// 会话上下文 key：私聊按 user_id，群聊按 (group_id, user_id)。
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum ContextKey {
    Private(i64),
    Group(i64, i64),
}

/// 累计 token 用量统计。
#[derive(Debug)]
pub struct TokenUsage {
    pub started_at: Instant,
    pub total_requests: u64,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
}

impl Default for TokenUsage {
    fn default() -> Self {
        Self {
            started_at: Instant::now(),
            total_requests: 0,
            prompt_tokens: 0,
            completion_tokens: 0,
        }
    }
}

impl TokenUsage {
    pub fn total_tokens(&self) -> u64 {
        self.prompt_tokens + self.completion_tokens
    }

    pub fn uptime_display(&self) -> String {
        let secs = self.started_at.elapsed().as_secs();
        let days = secs / 86400;
        let hours = (secs % 86400) / 3600;
        let mins = (secs % 3600) / 60;
        if days > 0 {
            format!("{days}天{hours}小时{mins}分钟")
        } else if hours > 0 {
            format!("{hours}小时{mins}分钟")
        } else {
            format!("{mins}分钟")
        }
    }
}

/// 全局 handler 上下文，持有所有 handler 共享的状态。
pub struct HandlerContext {
    pub api: ApiCaller,
    pub ai: AiClient,
    pub config: BotConfig,
    pub self_id: i64,
    pub chat_sessions: HashMap<ContextKey, ChatSession>,
    pub repeat_states: HashMap<i64, RepeatState>,
    pub token_usage: TokenUsage,
}

impl HandlerContext {
    pub fn new(api: ApiCaller, ai: AiClient, config: BotConfig) -> Self {
        Self {
            api,
            ai,
            config,
            self_id: 0,
            chat_sessions: HashMap::new(),
            repeat_states: HashMap::new(),
            token_usage: TokenUsage::default(),
        }
    }
}

/// 事件分发入口，将事件路由到各 handler。
pub async fn dispatch(ctx: &mut HandlerContext, event: &Event) {
    match event {
        Event::Message(msg_event) => {
            match msg_event {
                onebot::event::MessageEvent::Private(evt) => {
                    ctx.self_id = evt.self_id;
                    if cmd::handle_private_cmd(ctx, evt).await {
                        return;
                    }
                    ai_chat::handle_private(ctx, evt).await;
                }
                onebot::event::MessageEvent::Group(evt) => {
                    ctx.self_id = evt.self_id;
                    if evt.user_id == evt.self_id {
                        return;
                    }
                    repeater::handle_group_message(ctx, evt).await;
                    if like::handle_group_like(ctx, evt).await {
                        return;
                    }
                    ai_chat::handle_group(ctx, evt).await;
                }
            }
        }
        Event::Notice(notice_event) => {
            match notice_event {
                NoticeEvent::GroupIncrease(evt) => {
                    ctx.self_id = evt.self_id;
                    welcome::handle_group_increase(ctx, evt).await;
                }
                NoticeEvent::Notify(NotifyEvent::Poke(evt)) => {
                    ctx.self_id = evt.self_id;
                    poke::handle_poke(ctx, evt).await;
                }
                _ => {}
            }
        }
        Event::Request(req_event) => {
            request::handle_request(ctx, req_event).await;
        }
        Event::MetaEvent(meta) => {
            match meta {
                onebot::event::MetaEvent::Lifecycle(evt) => {
                    ctx.self_id = evt.self_id;
                    tracing::info!(self_id = evt.self_id, sub_type = %evt.sub_type, "lifecycle event");
                }
                onebot::event::MetaEvent::Heartbeat(_) => {}
            }
        }
    }
}
