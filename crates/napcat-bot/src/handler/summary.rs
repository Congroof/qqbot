use ai_chat_sdk::{ChatRequest, Message as AiMessage, ResponseFormat, RetryStrategy};
use onebot::api::payload::SendGroupMsg;
use onebot::event::message::GroupMessageEvent;
use onebot::message::MessageSegment;
use onebot::Message;

use super::{extract_plain_text, HandlerContext};

const SUMMARY_PROMPT: &str = "\
你是一个群聊消息摘要助手。用户会给你最近的群聊记录，请你：\
1. 概括聊了哪些主要话题\
2. 提到关键参与人\
3. 给出简短结论或有趣的总结\
总字数不超过 200 字，不要使用 Markdown 格式。";

pub async fn handle_summary(ctx: &mut HandlerContext, evt: &GroupMessageEvent) -> bool {
    let text = extract_plain_text(&evt.message);
    if !matches!(text.as_str(), "总结" | "消息摘要") {
        return false;
    }

    let Some(cache) = ctx.message_cache.get(&evt.group_id) else {
        let _ = ctx.api.call(SendGroupMsg {
            group_id: evt.group_id,
            message: Message::from(vec![
                MessageSegment::reply(evt.message_id.to_string()),
                MessageSegment::text("还没有足够的聊天记录来总结呢~"),
            ]),
            auto_escape: None,
        }).await;
        return true;
    };

    if cache.len() < 5 {
        let _ = ctx.api.call(SendGroupMsg {
            group_id: evt.group_id,
            message: Message::from(vec![
                MessageSegment::reply(evt.message_id.to_string()),
                MessageSegment::text("消息太少了，再多聊一会儿吧~"),
            ]),
            auto_escape: None,
        }).await;
        return true;
    }

    let chat_log: String = cache.iter()
        .filter(|m| !m.text.is_empty())
        .map(|m| format!("{}: {}", m.nickname, m.text))
        .collect::<Vec<_>>()
        .join("\n");

    let request = ChatRequest::builder()
        .model(&ctx.config.ai_model)
        .messages(vec![
            AiMessage::system(SUMMARY_PROMPT),
            AiMessage::user(&format!("以下是最近的群聊记录：\n{chat_log}")),
        ])
        .temperature(0.7)
        .max_completion_tokens(300)
        .response_format(ResponseFormat::text())
        .retry_strategy(RetryStrategy { retry_count: 2, timeout: 20 })
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
                .unwrap_or_else(|| "摘要生成失败~".into())
        }
        Err(e) => {
            tracing::error!(error = %e, "summary ai call failed");
            "摘要生成失败，稍后再试~".into()
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
