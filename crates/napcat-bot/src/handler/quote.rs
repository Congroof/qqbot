use ai_chat_sdk::{ChatRequest, Message as AiMessage, ResponseFormat, RetryStrategy};
use onebot::api::payload::SendGroupMsg;
use onebot::event::message::GroupMessageEvent;
use onebot::message::MessageSegment;
use onebot::Message;

use super::{extract_plain_text, HandlerContext};

const QUOTE_PROMPT: &str = "\
请随机给出一条名人名言（中文或英文均可，英文需附带中文翻译）。\
格式：「名言内容」——作者\n\
要求：每次给不同的名言，涵盖哲学、文学、科学、励志等各个领域。\
不要用 Markdown 格式，不要加任何多余的解释。";

/// 不再需要持久化存储，改为空结构占位以兼容 HandlerContext
pub struct QuoteStore;

impl QuoteStore {
    pub fn load(_data_dir: &str) -> Self { Self }
}

pub fn maybe_collect(_ctx: &mut HandlerContext, _evt: &GroupMessageEvent) {}

pub async fn handle_quote(ctx: &mut HandlerContext, evt: &GroupMessageEvent) -> bool {
    let text = extract_plain_text(&evt.message);
    if !matches!(text.as_str(), "语录" | "名言" | "名人名言") {
        return false;
    }

    let request = ChatRequest::builder()
        .model(&ctx.config.ai_model)
        .messages(vec![
            AiMessage::system(QUOTE_PROMPT),
            AiMessage::user("来一条"),
        ])
        .temperature(1.0)
        .max_completion_tokens(150)
        .response_format(ResponseFormat::text())
        .retry_strategy(RetryStrategy { retry_count: 2, timeout: 10 })
        .build();

    let reply = match ctx.ai.chat().create(request).await {
        Ok(resp) => {
            if let Some(usage) = &resp.usage {
                ctx.token_usage.total_requests += 1;
                ctx.token_usage.prompt_tokens += usage.prompt_tokens;
                ctx.token_usage.completion_tokens += usage.completion_tokens;
                ctx.token_usage.save();
            }
            resp.choices.first()
                .and_then(|c| c.message.content.as_ref()?.as_text().map(|s| s.to_string()))
                .unwrap_or_else(|| "名言生成失败，下次再试~".into())
        }
        Err(e) => {
            tracing::error!(error = %e, "quote ai call failed");
            "名言生成失败，下次再试~".into()
        }
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
