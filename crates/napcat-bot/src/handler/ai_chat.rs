use std::time::Instant;

use ai_chat_sdk::{ChatRequest, Message as AiMessage, ResponseFormat};
use onebot::api::payload::{SendGroupMsg, SendPrivateMsg};
use onebot::event::message::{GroupMessageEvent, PrivateMessageEvent};
use onebot::message::MessageSegment;
use onebot::Message;

use super::{ContextKey, HandlerContext};

const SYSTEM_PROMPT: &str = "你是一个QQ群里的智能助手，名字叫清，回复要简洁有趣，适合聊天场景。不要使用 Markdown 格式。";
const MAX_HISTORY: usize = 20;
const SESSION_TIMEOUT_SECS: u64 = 300;

pub struct ChatSession {
    pub messages: Vec<AiMessage>,
    pub last_active: Instant,
}

impl ChatSession {
    fn new() -> Self {
        Self {
            messages: Vec::new(),
            last_active: Instant::now(),
        }
    }

    fn is_expired(&self) -> bool {
        self.last_active.elapsed().as_secs() > SESSION_TIMEOUT_SECS
    }

    fn push_user(&mut self, content: &str) {
        self.messages.push(AiMessage::user(content));
        self.last_active = Instant::now();
        self.trim();
    }

    fn push_assistant(&mut self, content: &str) {
        self.messages.push(AiMessage::assistant(content));
        self.last_active = Instant::now();
        self.trim();
    }

    fn trim(&mut self) {
        if self.messages.len() > MAX_HISTORY {
            let drain_count = self.messages.len() - MAX_HISTORY;
            self.messages.drain(..drain_count);
        }
    }
}

/// 私聊消息 -> 直接触发 AI 聊天
pub async fn handle_private(ctx: &mut HandlerContext, evt: &PrivateMessageEvent) {
    let text = extract_text(&evt.message);
    if text.is_empty() {
        return;
    }

    let key = ContextKey::Private(evt.user_id);
    let reply = chat_with_ai(ctx, &key, &text).await;

    if let Some(reply) = reply {
        if let Err(e) = ctx.api.call(SendPrivateMsg {
            user_id: evt.user_id,
            message: Message::from(vec![MessageSegment::text(&reply)]),
            auto_escape: None,
        }).await {
            tracing::error!(error = %e, "failed to send private reply");
        }
    }
}

/// 群消息 -> 只在被 @bot 时触发 AI 聊天
pub async fn handle_group(ctx: &mut HandlerContext, evt: &GroupMessageEvent) {
    if !is_at_bot(&evt.message, ctx.self_id) {
        return;
    }

    let text = extract_text_without_at(&evt.message);
    if text.is_empty() {
        return;
    }

    let key = ContextKey::Group(evt.group_id, evt.user_id);
    let reply = chat_with_ai(ctx, &key, &text).await;

    if let Some(reply) = reply {
        if let Err(e) = ctx.api.call(SendGroupMsg {
            group_id: evt.group_id,
            message: Message::from(vec![
                MessageSegment::reply(evt.message_id.to_string()),
                MessageSegment::text(&reply),
            ]),
            auto_escape: None,
        }).await {
            tracing::error!(error = %e, "failed to send group reply");
        }
    }
}

async fn chat_with_ai(ctx: &mut HandlerContext, key: &ContextKey, user_text: &str) -> Option<String> {
    cleanup_expired_sessions(ctx);

    let session = ctx.chat_sessions.entry(key.clone()).or_insert_with(ChatSession::new);
    session.push_user(user_text);

    let mut messages = vec![AiMessage::system(SYSTEM_PROMPT)];
    messages.extend(session.messages.iter().cloned());

    let request = ChatRequest::builder()
        .model(&ctx.config.ai_model)
        .messages(messages)
        .temperature(0.8)
        .max_completion_tokens(512)
        .response_format(ResponseFormat::text())
        .build();

    match ctx.ai.chat().create(request).await {
        Ok(response) => {
            let text = response.choices.first().and_then(|c| {
                c.message.content.as_ref().and_then(|c| c.as_text()).map(|s| s.to_string())
            }).unwrap_or_default();

            if !text.is_empty() {
                session.push_assistant(&text);
            }
            Some(text)
        }
        Err(e) => {
            tracing::error!(error = %e, "ai chat call failed");
            Some("AI 暂时开小差了，请稍后再试~".to_string())
        }
    }
}

fn cleanup_expired_sessions(ctx: &mut HandlerContext) {
    ctx.chat_sessions.retain(|_, session| !session.is_expired());
}

fn is_at_bot(msg: &Message, self_id: i64) -> bool {
    let self_id_str = self_id.to_string();
    for seg in msg.segments() {
        if let MessageSegment::At { qq } = seg {
            if qq == &self_id_str || qq == "all" {
                return true;
            }
        }
    }
    false
}

fn extract_text(msg: &Message) -> String {
    match msg {
        Message::String(s) => s.trim().to_string(),
        Message::Array(segs) => {
            segs.iter()
                .filter_map(|seg| {
                    if let MessageSegment::Text { text } = seg {
                        Some(text.trim())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
                .trim()
                .to_string()
        }
    }
}

fn extract_text_without_at(msg: &Message) -> String {
    match msg {
        Message::String(s) => s.trim().to_string(),
        Message::Array(segs) => {
            segs.iter()
                .filter_map(|seg| {
                    match seg {
                        MessageSegment::Text { text } => Some(text.trim()),
                        MessageSegment::At { .. } => None,
                        _ => None,
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
                .trim()
                .to_string()
        }
    }
}
