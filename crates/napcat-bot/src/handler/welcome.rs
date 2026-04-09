use onebot::api::payload::SendGroupMsg;
use onebot::event::notice::GroupIncreaseEvent;
use onebot::message::MessageSegment;
use onebot::Message;

use super::HandlerContext;

pub async fn handle_group_increase(ctx: &HandlerContext, evt: &GroupIncreaseEvent) {
    if evt.user_id == evt.self_id {
        return;
    }

    let message = Message::from(vec![
        MessageSegment::at(evt.user_id.to_string()),
        MessageSegment::text(" 欢迎加入！有什么问题随时 @我 哦~"),
    ]);

    if let Err(e) = ctx.api.call(SendGroupMsg {
        group_id: evt.group_id,
        message,
        auto_escape: None,
    }).await {
        tracing::error!(error = %e, group_id = evt.group_id, "failed to send welcome message");
    }
}
