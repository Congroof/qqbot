use std::collections::HashMap;
use std::time::Instant;

use ai_chat_sdk::{ChatRequest, Message as AiMessage, ResponseFormat, RetryStrategy};
use onebot::api::payload::{SendGroupMsg, SendPrivateMsg};
use onebot::event::message::{GroupMessageEvent, PrivateMessageEvent};
use onebot::message::MessageSegment;
use onebot::Message;
use rand::Rng;

use super::{extract_plain_text, ContextKey, HandlerContext};

/// 每个群的 AI 角色 prompt（群隔离），由管理员通过 `#角色` 设置。
pub type GroupRoleMap = HashMap<i64, String>;

const GROUP_PROMPT: &str = "\
你是一个QQ群里的智能助手，名字叫清。\
你知识渊博，能准确回答用户提出的各类问题，包括但不限于知识问答、技术咨询、生活建议、创意写作等。\
回复要求：准确、清晰、有条理，根据问题复杂度调整回复长度。不要使用 Markdown 格式。";

const PRIVATE_PROMPT: &str = "\
你是一个智能助手，名字叫清，正在和用户私聊。\
你知识渊博，能准确回答用户提出的各类问题，包括但不限于知识问答、技术咨询、生活建议、创意写作等。\
回复要求：准确、清晰、有条理，根据问题复杂度调整回复长度。不要使用 Markdown 格式。";

const MAX_HISTORY: usize = 20;
const SESSION_TIMEOUT_SECS: u64 = 300;
const TOKEN_RANGE: (u32, u32) = (128, 512);

fn builtin_roles() -> &'static [(&'static str, &'static str)] {
    &[
        ("猫娘", "你是一只可爱的猫娘，说话带有猫的语气，句尾经常加「喵~」。性格活泼黏人，偶尔傲娇。不要使用 Markdown 格式。"),
        ("毒舌", "你是一个毒舌吐槽役，说话犀利但有趣，擅长一针见血的吐槽，但不会真的伤人。不要使用 Markdown 格式。"),
        ("哲学家", "你是一个深邃的哲学家，喜欢用哲学的角度思考问题，偶尔引用名人名言，但说话要通俗易懂。不要使用 Markdown 格式。"),
        ("老中医", "你是一个调侃版的老中医，喜欢用中医养生的口吻聊天，什么都能扯到养生上。不要使用 Markdown 格式。"),
        ("诗人", "你是一个浪漫的诗人，喜欢用诗意的语言回复，偶尔即兴作诗。不要使用 Markdown 格式。"),
    ]
}

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

fn is_admin(evt: &GroupMessageEvent) -> bool {
    matches!(evt.sender.role.as_deref(), Some("admin" | "owner"))
}

/// 群聊角色切换（仅管理员，群隔离）：`#角色 猫娘`
pub async fn handle_group_role_switch(ctx: &mut HandlerContext, evt: &GroupMessageEvent) -> bool {
    let text = extract_plain_text(&evt.message);
    let Some(role_name) = text.strip_prefix("#角色").map(|s| s.trim()) else {
        return false;
    };

    if !is_admin(evt) {
        let _ = ctx.api.call(SendGroupMsg {
            group_id: evt.group_id,
            message: Message::from(vec![
                MessageSegment::reply(evt.message_id.to_string()),
                MessageSegment::text("仅管理员可以切换角色哦~"),
            ]),
            auto_escape: None,
        }).await;
        return true;
    }

    let reply = if role_name == "默认" || role_name.is_empty() {
        ctx.group_roles.remove(&evt.group_id);
        cleanup_group_sessions(ctx, evt.group_id);
        "已恢复默认人设~".to_string()
    } else {
        let prompt = if let Some((_, p)) = builtin_roles().iter().find(|(n, _)| *n == role_name) {
            p.to_string()
        } else {
            format!("你正在扮演「{role_name}」这个角色，请完全以该角色的语气和风格说话。不要使用 Markdown 格式。")
        };
        ctx.group_roles.insert(evt.group_id, prompt);
        cleanup_group_sessions(ctx, evt.group_id);
        format!("已切换为「{role_name}」模式~")
    };

    let _ = ctx.api.call(SendGroupMsg {
        group_id: evt.group_id,
        message: Message::from(vec![
            MessageSegment::reply(evt.message_id.to_string()),
            MessageSegment::text(reply),
        ]),
        auto_escape: None,
    }).await;

    true
}

fn cleanup_group_sessions(ctx: &mut HandlerContext, group_id: i64) {
    ctx.chat_sessions.retain(|key, _| {
        !matches!(key, ContextKey::Group(gid, _) if *gid == group_id)
    });
}

/// 私聊消息 -> 直接触发 AI 聊天
pub async fn handle_private(ctx: &mut HandlerContext, evt: &PrivateMessageEvent) {
    let text = extract_plain_text(&evt.message);
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

    let default_prompt = match key {
        ContextKey::Private(_) => PRIVATE_PROMPT.to_string(),
        ContextKey::Group(gid, _) => {
            ctx.group_roles.get(gid).cloned().unwrap_or_else(|| GROUP_PROMPT.to_string())
        }
    };

    let session = ctx.chat_sessions.entry(key.clone()).or_insert_with(ChatSession::new);
    session.push_user(user_text);

    let mut messages = vec![AiMessage::system(&default_prompt)];
    messages.extend(session.messages.iter().cloned());

    let max_tokens = rand::rng().random_range(TOKEN_RANGE.0..=TOKEN_RANGE.1);

    let request = ChatRequest::builder()
        .model(&ctx.config.ai_model)
        .messages(messages)
        .temperature(0.9)
        .max_completion_tokens(max_tokens)
        .response_format(ResponseFormat::text())
        .retry_strategy(RetryStrategy { retry_count: 3, timeout: 30 })
        .build();

    match ctx.ai.chat().create(request).await {
        Ok(response) => {
            if let Some(usage) = &response.usage {
                ctx.token_usage.total_requests += 1;
                ctx.token_usage.prompt_tokens += usage.prompt_tokens;
                ctx.token_usage.completion_tokens += usage.completion_tokens;
                ctx.token_usage.save();
            }

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
