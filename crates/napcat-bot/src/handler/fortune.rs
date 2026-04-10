use ai_chat_sdk::{ChatRequest, Message as AiMessage, ResponseFormat, RetryStrategy};
use onebot::api::payload::SendGroupMsg;
use onebot::event::message::GroupMessageEvent;
use onebot::message::MessageSegment;
use onebot::Message;

use super::{extract_plain_text, HandlerContext};

const FORTUNE_PROMPT: &str = "\
你是一个有趣的运势生成器。请为用户生成今日运势，包含以下内容：\
1. 运势等级（大吉/中吉/小吉/末吉/凶/大凶）\
2. 幸运数字（1-99）\
3. 幸运颜色\
4. 一句话运势描述（诙谐有趣）\
5. 一条今日建议\
格式紧凑，不要用 Markdown，不要用列表符号，用 emoji 分隔各项。总共不超过 100 字。";

pub async fn handle_fortune(ctx: &mut HandlerContext, evt: &GroupMessageEvent) -> bool {
    let text = extract_plain_text(&evt.message);
    if !matches!(text.as_str(), "运势" | "求签" | "今日运势") {
        return false;
    }

    let today = today_str();
    let cache_key = (evt.user_id, today.clone());

    if let Some(cached) = ctx.fortune_cache.get(&cache_key) {
        let _ = ctx.api.call(SendGroupMsg {
            group_id: evt.group_id,
            message: Message::from(vec![
                MessageSegment::reply(evt.message_id.to_string()),
                MessageSegment::text(cached),
            ]),
            auto_escape: None,
        }).await;
        return true;
    }

    ctx.fortune_cache.retain(|(_, date), _| *date == today);

    let request = ChatRequest::builder()
        .model(&ctx.config.ai_model)
        .messages(vec![
            AiMessage::system(FORTUNE_PROMPT),
            AiMessage::user(&format!("请为 QQ 用户 {} 生成今日运势", evt.user_id)),
        ])
        .temperature(1.0)
        .max_completion_tokens(200)
        .response_format(ResponseFormat::text())
        .retry_strategy(RetryStrategy { retry_count: 2, timeout: 15 })
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
                .unwrap_or_else(|| "运势生成失败，改天再试吧~".into())
        }
        Err(e) => {
            tracing::error!(error = %e, "fortune ai call failed");
            "运势生成失败，改天再试吧~".into()
        }
    };

    ctx.fortune_cache.insert(cache_key, reply.clone());

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

fn today_str() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let days = (secs + 8 * 3600) / 86400;
    format!("{days}")
}
