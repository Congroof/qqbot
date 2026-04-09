use rand::seq::IndexedRandom;

use onebot::api::payload::SendGroupMsg;
use onebot::event::notice::PokeEvent;
use onebot::message::MessageSegment;
use onebot::Message;

use super::HandlerContext;

const POKE_REPLIES: &[&str] = &[
    "别戳了，再戳我要报警了！",
    "你戳我干嘛，我又不是按钮 🔘",
    "痒！别闹～",
    "戳你一下，不许还手 👉",
    "请不要骚扰AI谢谢 😤",
    "嗯？有什么事吗？",
    "再戳就把你吃掉！",
    "你好呀，需要帮忙吗？试试 @我 提问~",
    "咚咚咚，谁在敲门？",
    "我正在偷偷学习中，别打扰我！",
];

pub async fn handle_poke(ctx: &HandlerContext, evt: &PokeEvent) {
    if evt.target_id != ctx.self_id {
        return;
    }

    let mut rng = rand::rng();
    let reply = POKE_REPLIES.choose(&mut rng).unwrap_or(&"你好~");

    let message = Message::from(vec![
        MessageSegment::at(evt.user_id.to_string()),
        MessageSegment::text(format!(" {reply}")),
    ]);

    if let Err(e) = ctx.api.call(SendGroupMsg {
        group_id: evt.group_id,
        message,
        auto_escape: None,
    }).await {
        tracing::error!(error = %e, group_id = evt.group_id, "failed to send poke reply");
    }
}
