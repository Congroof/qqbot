use std::time::Instant;

use onebot::api::payload::{SendGroupMsg, SetGroupKick};
use onebot::event::message::GroupMessageEvent;
use onebot::event::notice::GroupIncreaseEvent;
use onebot::message::MessageSegment;
use onebot::Message;
use rand::Rng;

use super::{extract_plain_text, HandlerContext};

const VERIFY_TIMEOUT_SECS: u64 = 60;

pub struct Verification {
    pub answer: i32,
    pub created_at: Instant,
}

pub async fn handle_group_increase(ctx: &mut HandlerContext, evt: &GroupIncreaseEvent) {
    if evt.user_id == evt.self_id {
        return;
    }

    // 管理员/群主邀请入群时跳过验证
    if evt.sub_type == "invite" {
        return;
    }

    let mut rng = rand::rng();
    let a: i32 = rng.random_range(1..=20);
    let b: i32 = rng.random_range(1..=20);
    let answer = a + b;

    ctx.pending_verifications.insert((evt.group_id, evt.user_id), Verification {
        answer,
        created_at: Instant::now(),
    });

    let _ = ctx.api.call(SendGroupMsg {
        group_id: evt.group_id,
        message: Message::from(vec![
            MessageSegment::at(evt.user_id.to_string()),
            MessageSegment::text(format!(
                " 欢迎！请在 {VERIFY_TIMEOUT_SECS} 秒内回答验证问题：\n{a} + {b} = ?"
            )),
        ]),
        auto_escape: None,
    }).await;
}

pub async fn handle_answer(ctx: &mut HandlerContext, evt: &GroupMessageEvent) -> bool {
    let key = (evt.group_id, evt.user_id);
    let Some(verification) = ctx.pending_verifications.get(&key) else {
        return false;
    };

    let text = extract_plain_text(&evt.message);
    let user_answer: i32 = match text.trim().parse() {
        Ok(n) => n,
        Err(_) => return false,
    };

    if user_answer == verification.answer {
        ctx.pending_verifications.remove(&key);
        let _ = ctx.api.call(SendGroupMsg {
            group_id: evt.group_id,
            message: Message::from(vec![
                MessageSegment::at(evt.user_id.to_string()),
                MessageSegment::text(" 验证通过！欢迎加入，入群请先看群公告与精华消息，包含有说明与指导，有什么问题随时 @我 哦~ -by bot"),
            ]),
            auto_escape: None,
        }).await;
    } else {
        let _ = ctx.api.call(SendGroupMsg {
            group_id: evt.group_id,
            message: Message::from(vec![
                MessageSegment::reply(evt.message_id.to_string()),
                MessageSegment::text("答案不对哦，再想想？"),
            ]),
            auto_escape: None,
        }).await;
    }

    true
}

pub async fn check_expired(ctx: &mut HandlerContext) {
    let expired: Vec<(i64, i64)> = ctx.pending_verifications.iter()
        .filter(|(_, v)| v.created_at.elapsed().as_secs() > VERIFY_TIMEOUT_SECS)
        .map(|(k, _)| *k)
        .collect();

    for (group_id, user_id) in expired {
        ctx.pending_verifications.remove(&(group_id, user_id));

        let _ = ctx.api.call(SendGroupMsg {
            group_id,
            message: Message::from(vec![
                MessageSegment::at(user_id.to_string()),
                MessageSegment::text(" 验证超时，已被移出群聊。欢迎重新申请加入~"),
            ]),
            auto_escape: None,
        }).await;

        let _ = ctx.api.call(SetGroupKick {
            group_id,
            user_id,
            reject_add_request: Some(false),
        }).await;
    }
}
