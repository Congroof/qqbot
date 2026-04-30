pub mod ai_chat;
pub mod cmd;
pub mod fortune;
pub mod idiom;
pub mod like;
pub mod poke;
pub mod quote;
pub mod recall;
pub mod repeater;
pub mod request;
pub mod stats;
pub mod summary;
pub mod verify;
pub mod vocab;

use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::time::Instant;

use ai_chat_sdk::AiClient;
use onebot::event::notice::{NoticeEvent, NotifyEvent};
use onebot::{ApiCaller, Event};
use serde::{Deserialize, Serialize};

use crate::config::BotConfig;
use self::ai_chat::{ChatSession, GroupRoleMap};
use self::quote::QuoteStore;
use self::recall::RecallToggle;
use self::repeater::RepeatState;
use self::stats::MsgStats;
use self::verify::Verification;
use self::vocab::Dictionary;

/// 会话上下文 key：私聊按 user_id，群聊按 (group_id, user_id)。
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum ContextKey {
    Private(i64),
    Group(i64, i64),
}

// ---- 消息缓存（撤回监控 + 消息摘要共用）----

#[derive(Debug, Clone)]
pub struct CachedMessage {
    #[allow(dead_code)]
    pub user_id: i64,
    pub nickname: String,
    pub text: String,
    pub message: onebot::Message,
    pub message_id: i64,
}

const MSG_CACHE_PER_GROUP: usize = 100;

// ---- Token 用量持久化 ----

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct PersistedUsage {
    total_requests: u64,
    prompt_tokens: u64,
    completion_tokens: u64,
}

#[derive(Debug)]
pub struct TokenUsage {
    pub started_at: Instant,
    pub total_requests: u64,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    file_path: PathBuf,
}

impl TokenUsage {
    pub fn load(data_dir: &str) -> Self {
        let dir = Path::new(data_dir);
        std::fs::create_dir_all(dir).ok();
        let file_path = dir.join("token_usage.json");

        let persisted = std::fs::read_to_string(&file_path)
            .ok()
            .and_then(|s| serde_json::from_str::<PersistedUsage>(&s).ok())
            .unwrap_or_default();

        tracing::info!(
            requests = persisted.total_requests,
            tokens = persisted.prompt_tokens + persisted.completion_tokens,
            "loaded token usage from disk"
        );

        Self {
            started_at: Instant::now(),
            total_requests: persisted.total_requests,
            prompt_tokens: persisted.prompt_tokens,
            completion_tokens: persisted.completion_tokens,
            file_path,
        }
    }

    pub fn total_tokens(&self) -> u64 {
        self.prompt_tokens + self.completion_tokens
    }

    pub fn save(&self) {
        let data = PersistedUsage {
            total_requests: self.total_requests,
            prompt_tokens: self.prompt_tokens,
            completion_tokens: self.completion_tokens,
        };
        if let Ok(json) = serde_json::to_string_pretty(&data) {
            if let Err(e) = std::fs::write(&self.file_path, json) {
                tracing::warn!(error = %e, "failed to persist token usage");
            }
        }
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

// ---- 成语接龙游戏状态 ----

pub struct IdiomGame {
    pub last_idiom: String,
    pub scores: HashMap<i64, (String, u32)>,
    pub last_active: Instant,
}

impl IdiomGame {
    pub fn is_expired(&self) -> bool {
        self.last_active.elapsed().as_secs() > 300
    }
}

// ---- HandlerContext ----

pub struct HandlerContext {
    pub api: ApiCaller,
    pub ai: AiClient,
    pub config: BotConfig,
    pub self_id: i64,
    pub chat_sessions: HashMap<ContextKey, ChatSession>,
    pub repeat_states: HashMap<i64, RepeatState>,
    pub token_usage: TokenUsage,
    pub message_cache: HashMap<i64, VecDeque<CachedMessage>>,
    pub fortune_cache: HashMap<(i64, String), String>,
    pub idiom_games: HashMap<i64, IdiomGame>,
    pub pending_verifications: HashMap<(i64, i64), Verification>,
    pub msg_stats: MsgStats,
    #[allow(dead_code)]
    pub quotes: QuoteStore,
    pub group_roles: GroupRoleMap,
    pub recall_toggle: RecallToggle,
    pub dictionary: Dictionary,
}

impl HandlerContext {
    pub fn new(api: ApiCaller, ai: AiClient, config: BotConfig) -> Self {
        let token_usage = TokenUsage::load(&config.data_dir);
        let msg_stats = MsgStats::load(&config.data_dir);
        let quotes = QuoteStore::load(&config.data_dir);
        let recall_toggle = RecallToggle::load(&config.data_dir);
        let dictionary = Dictionary::load(&config.data_dir);
        Self {
            api,
            ai,
            config,
            self_id: 0,
            chat_sessions: HashMap::new(),
            repeat_states: HashMap::new(),
            token_usage,
            message_cache: HashMap::new(),
            fortune_cache: HashMap::new(),
            idiom_games: HashMap::new(),
            pending_verifications: HashMap::new(),
            msg_stats,
            quotes,
            group_roles: HashMap::new(),
            recall_toggle,
            dictionary,
        }
    }

    pub fn cache_group_message(&mut self, evt: &onebot::event::message::GroupMessageEvent) {
        let nickname = evt.sender.card.clone()
            .filter(|c| !c.is_empty())
            .or_else(|| evt.sender.nickname.clone())
            .unwrap_or_else(|| evt.user_id.to_string());

        let text = extract_plain_text(&evt.message);
        let cache = self.message_cache.entry(evt.group_id).or_default();
        cache.push_back(CachedMessage {
            user_id: evt.user_id,
            nickname,
            text,
            message: evt.message.clone(),
            message_id: evt.message_id,
        });
        if cache.len() > MSG_CACHE_PER_GROUP {
            cache.pop_front();
        }
    }
}

use onebot::api::payload::SendGroupMsg;

async fn handle_group_help(ctx: &HandlerContext, evt: &onebot::event::message::GroupMessageEvent) -> bool {
    let text = extract_plain_text(&evt.message);
    if text.trim() != "#cmd help" {
        return false;
    }

    let _ = ctx.api.call(SendGroupMsg {
        group_id: evt.group_id,
        message: onebot::Message::from(vec![
            onebot::message::MessageSegment::reply(evt.message_id.to_string()),
            onebot::message::MessageSegment::text(cmd::format_help()),
        ]),
        auto_escape: None,
    }).await;

    true
}

pub fn extract_plain_text(msg: &onebot::Message) -> String {
    use onebot::message::MessageSegment;
    match msg {
        onebot::Message::String(s) => s.trim().to_string(),
        onebot::Message::Array(segs) => {
            segs.iter()
                .filter_map(|seg| {
                    if let MessageSegment::Text { text } = seg { Some(text.trim()) } else { None }
                })
                .collect::<Vec<_>>()
                .join("")
                .trim()
                .to_string()
        }
    }
}

/// 事件分发入口，将事件路由到各 handler。
pub async fn dispatch(ctx: &mut HandlerContext, event: &Event) {
    verify::check_expired(ctx).await;
    idiom::check_expired_games(ctx).await;

    match event {
        Event::Message(msg_event) => {
            match msg_event {
                onebot::event::MessageEvent::Private(evt) => {
                    ctx.self_id = evt.self_id;
                    if cmd::handle_private_cmd(ctx, evt).await {
                        return;
                    }
                    // 私聊 AI 聊天已禁用，避免误触
                    // ai_chat::handle_private(ctx, evt).await;
                }
                onebot::event::MessageEvent::Group(evt) => {
                    ctx.self_id = evt.self_id;
                    if evt.user_id == evt.self_id {
                        return;
                    }

                    ctx.cache_group_message(evt);
                    stats::record_message(ctx, evt);
                    quote::maybe_collect(ctx, evt);

                    if handle_group_help(ctx, evt).await {
                        return;
                    }
                    if verify::handle_answer(ctx, evt).await {
                        return;
                    }
                    if recall::handle_recall_cmd(ctx, evt).await {
                        return;
                    }
                    if like::handle_group_like(ctx, evt).await {
                        return;
                    }
                    if fortune::handle_fortune(ctx, evt).await {
                        return;
                    }
                    if quote::handle_quote(ctx, evt).await {
                        return;
                    }
                    if idiom::handle_idiom(ctx, evt).await {
                        return;
                    }
                    if vocab::handle_vocab(ctx, evt).await {
                        return;
                    }
                    if stats::handle_stats(ctx, evt).await {
                        return;
                    }
                    if summary::handle_summary(ctx, evt).await {
                        return;
                    }
                    if ai_chat::handle_group_role_switch(ctx, evt).await {
                        return;
                    }

                    repeater::handle_group_message(ctx, evt).await;
                    ai_chat::handle_group(ctx, evt).await;
                }
            }
        }
        Event::Notice(notice_event) => {
            match notice_event {
                NoticeEvent::GroupIncrease(evt) => {
                    ctx.self_id = evt.self_id;
                    verify::handle_group_increase(ctx, evt).await;
                }
                NoticeEvent::GroupRecall(evt) => {
                    ctx.self_id = evt.self_id;
                    recall::handle_group_recall(ctx, evt).await;
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
