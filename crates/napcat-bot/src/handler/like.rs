use rand::seq::IndexedRandom;

use onebot::api::payload::{SendGroupMsg, SendLike};
use onebot::event::message::GroupMessageEvent;
use onebot::message::MessageSegment;
use onebot::Message;

use super::HandlerContext;

const BATCH_SIZE: i32 = 10;
const MAX_BATCHES: i32 = 5;

const SUCCESS_REPLIES: &[&str] = &[
    "已给你点了 {n} 个赞，今天也要开心鸭~",
    "咔咔咔！{n} 个赞已送达，请查收！",
    "{n} 个赞打包送上，不用谢~",
    "点赞成功！{n} 连赞，你就是最靓的仔！",
    "已疯狂为你点赞 {n} 次，求回赞！",
    "叮～{n} 个赞已到账，余额充足！",
    "{n} 连击！你的人气又涨了~",
    "赞赞赞！{n} 个全给你，今天也是被夸的一天！",
];

const FAIL_REPLIES: &[&str] = &[
    "点赞失败了呜呜... 原因：{e}",
    "没赞成功，可能今天已经赞过了：{e}",
    "赞不动了：{e}",
];

/// 处理群内"赞我"指令，返回 true 表示已处理。
/// 分批调用 send_like（每次 10 个，最多 5 轮共 50 个），遇到失败立即停止。
pub async fn handle_group_like(ctx: &HandlerContext, evt: &GroupMessageEvent) -> bool {
    let text = extract_plain_text(&evt.message);
    if text != "赞我" {
        return false;
    }

    let mut total = 0i32;
    let mut last_error: Option<String> = None;

    for _ in 0..MAX_BATCHES {
        match ctx.api.call_raw(SendLike {
            user_id: evt.user_id,
            times: Some(BATCH_SIZE),
        }).await {
            Ok(resp) if resp.retcode == 0 => {
                total += BATCH_SIZE;
            }
            Ok(resp) => {
                last_error = Some(resp.message.unwrap_or_else(|| format!("错误码 {}", resp.retcode)));
                break;
            }
            Err(e) => {
                last_error = Some(e.to_string());
                break;
            }
        }
    }

    let mut rng = rand::rng();

    let reply_text = if total > 0 {
        let template = SUCCESS_REPLIES.choose(&mut rng).unwrap_or(&"{n} 个赞已送达！");
        let mut text = template.replace("{n}", &total.to_string());
        if let Some(err) = &last_error {
            text.push_str(&format!("（后续点赞受限：{err}）"));
        }
        text
    } else {
        let err = last_error.unwrap_or_else(|| "未知错误".into());
        let template = FAIL_REPLIES.choose(&mut rng).unwrap_or(&"点赞失败：{e}");
        template.replace("{e}", &err)
    };

    let _ = ctx.api.call(SendGroupMsg {
        group_id: evt.group_id,
        message: Message::from(vec![
            MessageSegment::reply(evt.message_id.to_string()),
            MessageSegment::text(reply_text),
        ]),
        auto_escape: None,
    }).await;

    true
}

fn extract_plain_text(msg: &Message) -> String {
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
                .join("")
                .trim()
                .to_string()
        }
    }
}
