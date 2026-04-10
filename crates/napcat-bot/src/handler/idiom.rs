use std::collections::HashMap;
use std::time::Instant;

use ai_chat_sdk::{ChatRequest, Message as AiMessage, RetryStrategy};
use onebot::api::payload::SendGroupMsg;
use onebot::event::message::GroupMessageEvent;
use onebot::message::MessageSegment;
use onebot::Message;

use super::{extract_plain_text, HandlerContext, IdiomGame};

const JUDGE_PROMPT: &str = "\
你是成语接龙裁判。用户会给你上一个成语和玩家提交的成语。\
请判断：1) 提交的是否是合法的四字成语；2) 首字是否与上一个成语的尾字相同（同音即可）。\
然后你再接龙一个新成语（尾字尽量刁钻但合法）。\
严格按以下 JSON 格式回复，不要有任何多余文字：\
{\"valid\":true,\"reason\":\"正确\",\"next\":\"你接龙的成语\"}\
如果不合法：{\"valid\":false,\"reason\":\"失败原因\",\"next\":\"\"}\
";

const START_PROMPT: &str = "\
你是成语接龙裁判。请随机给出一个四字成语作为接龙的开头。\
严格按以下 JSON 格式回复：{\"idiom\":\"四字成语\"}";

pub async fn handle_idiom(ctx: &mut HandlerContext, evt: &GroupMessageEvent) -> bool {
    let text = extract_plain_text(&evt.message);

    if text == "成语接龙" {
        start_game(ctx, evt).await;
        return true;
    }

    if text == "结束接龙" {
        if ctx.idiom_games.contains_key(&evt.group_id) {
            end_game(ctx, evt.group_id).await;
        } else {
            let _ = ctx.api.call(SendGroupMsg {
                group_id: evt.group_id,
                message: Message::from(vec![MessageSegment::text("当前没有进行中的接龙哦")]),
                auto_escape: None,
            }).await;
        }
        return true;
    }

    if !ctx.idiom_games.contains_key(&evt.group_id) {
        return false;
    }

    if text.chars().count() != 4 {
        return false;
    }

    handle_player_answer(ctx, evt, &text).await;
    true
}

async fn start_game(ctx: &mut HandlerContext, evt: &GroupMessageEvent) {
    if ctx.idiom_games.contains_key(&evt.group_id) {
        let _ = ctx.api.call(SendGroupMsg {
            group_id: evt.group_id,
            message: Message::from(vec![MessageSegment::text("接龙正在进行中！直接发四字成语参与吧")]),
            auto_escape: None,
        }).await;
        return;
    }

    let request = ChatRequest::builder()
        .model(&ctx.config.ai_model)
        .messages(vec![AiMessage::system(START_PROMPT), AiMessage::user("开始")])
        .temperature(1.0)
        .max_completion_tokens(60)
        .retry_strategy(RetryStrategy { retry_count: 2, timeout: 10 })
        .build();

    let first_idiom = match ctx.ai.chat().create(request).await {
        Ok(resp) => {
            track_usage(ctx, &resp);
            let raw = resp.choices.first()
                .and_then(|c| c.message.content.as_ref()?.as_text().map(|s| s.to_string()))
                .unwrap_or_default();
            serde_json::from_str::<serde_json::Value>(&raw)
                .ok()
                .and_then(|v| v["idiom"].as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "一马当先".into())
        }
        Err(_) => "一马当先".into(),
    };

    ctx.idiom_games.insert(evt.group_id, IdiomGame {
        last_idiom: first_idiom.clone(),
        scores: HashMap::new(),
        last_active: Instant::now(),
    });

    let _ = ctx.api.call(SendGroupMsg {
        group_id: evt.group_id,
        message: Message::from(vec![
            MessageSegment::text(format!("成语接龙开始！\n第一个成语：{first_idiom}\n请接下一个（5 分钟无人作答自动结束）")),
        ]),
        auto_escape: None,
    }).await;
}

async fn handle_player_answer(ctx: &mut HandlerContext, evt: &GroupMessageEvent, answer: &str) {
    let game = ctx.idiom_games.get(&evt.group_id).unwrap();
    let last = game.last_idiom.clone();

    let request = ChatRequest::builder()
        .model(&ctx.config.ai_model)
        .messages(vec![
            AiMessage::system(JUDGE_PROMPT),
            AiMessage::user(&format!("上一个成语：{last}，玩家提交：{answer}")),
        ])
        .temperature(0.3)
        .max_completion_tokens(120)
        .retry_strategy(RetryStrategy { retry_count: 2, timeout: 10 })
        .build();

    let result = match ctx.ai.chat().create(request).await {
        Ok(resp) => {
            track_usage(ctx, &resp);
            resp.choices.first()
                .and_then(|c| c.message.content.as_ref()?.as_text().map(|s| s.to_string()))
                .unwrap_or_default()
        }
        Err(e) => {
            tracing::error!(error = %e, "idiom judge failed");
            return;
        }
    };

    let parsed: serde_json::Value = match serde_json::from_str(&result) {
        Ok(v) => v,
        Err(_) => {
            let _ = ctx.api.call(SendGroupMsg {
                group_id: evt.group_id,
                message: Message::from(vec![MessageSegment::text("裁判判断出错了，换一个试试？")]),
                auto_escape: None,
            }).await;
            return;
        }
    };

    let valid = parsed["valid"].as_bool().unwrap_or(false);
    let reason = parsed["reason"].as_str().unwrap_or("");
    let next = parsed["next"].as_str().unwrap_or("");

    if !valid {
        let _ = ctx.api.call(SendGroupMsg {
            group_id: evt.group_id,
            message: Message::from(vec![
                MessageSegment::reply(evt.message_id.to_string()),
                MessageSegment::text(format!("不对哦~ {reason}")),
            ]),
            auto_escape: None,
        }).await;
        return;
    }

    let nickname = evt.sender.card.clone()
        .filter(|c| !c.is_empty())
        .or_else(|| evt.sender.nickname.clone())
        .unwrap_or_else(|| evt.user_id.to_string());

    let game = ctx.idiom_games.get_mut(&evt.group_id).unwrap();
    game.last_active = Instant::now();
    let entry = game.scores.entry(evt.user_id).or_insert((nickname, 0));
    entry.1 += 1;

    if next.is_empty() || next.chars().count() != 4 {
        game.last_idiom = answer.to_string();
        let _ = ctx.api.call(SendGroupMsg {
            group_id: evt.group_id,
            message: Message::from(vec![MessageSegment::text(format!(
                "正确！+1 分\n我接不上来了，你赢了！当前成语：{answer}，继续接龙吧~"
            ))]),
            auto_escape: None,
        }).await;
    } else {
        game.last_idiom = next.to_string();
        let _ = ctx.api.call(SendGroupMsg {
            group_id: evt.group_id,
            message: Message::from(vec![MessageSegment::text(format!(
                "正确！+1 分\n我接：{next}"
            ))]),
            auto_escape: None,
        }).await;
    }
}

pub async fn end_game(ctx: &mut HandlerContext, group_id: i64) {
    let Some(game) = ctx.idiom_games.remove(&group_id) else { return };

    if game.scores.is_empty() {
        let _ = ctx.api.call(SendGroupMsg {
            group_id,
            message: Message::from(vec![MessageSegment::text("接龙结束，没有人得分~")]),
            auto_escape: None,
        }).await;
        return;
    }

    let mut ranking: Vec<_> = game.scores.into_values().collect();
    ranking.sort_by(|a, b| b.1.cmp(&a.1));

    let mut text = "成语接龙结束！排名：\n".to_string();
    for (i, (name, score)) in ranking.iter().enumerate() {
        text.push_str(&format!("{}. {} - {} 分\n", i + 1, name, score));
    }

    let _ = ctx.api.call(SendGroupMsg {
        group_id,
        message: Message::from(vec![MessageSegment::text(text.trim_end())]),
        auto_escape: None,
    }).await;
}

pub async fn check_expired_games(ctx: &mut HandlerContext) {
    let expired: Vec<i64> = ctx.idiom_games.iter()
        .filter(|(_, g)| g.is_expired())
        .map(|(gid, _)| *gid)
        .collect();
    for gid in expired {
        end_game(ctx, gid).await;
    }
}

fn track_usage(ctx: &mut HandlerContext, resp: &ai_chat_sdk::ChatResponse) {
    if let Some(usage) = &resp.usage {
        ctx.token_usage.total_requests += 1;
        ctx.token_usage.prompt_tokens += usage.prompt_tokens;
        ctx.token_usage.completion_tokens += usage.completion_tokens;
        ctx.token_usage.save();
    }
}
