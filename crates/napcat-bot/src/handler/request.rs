use onebot::api::payload::{SetFriendAddRequest, SetGroupAddRequest};
use onebot::event::request::RequestEvent;

use super::HandlerContext;

pub async fn handle_request(ctx: &HandlerContext, event: &RequestEvent) {
    match event {
        RequestEvent::Friend(evt) => {
            tracing::info!(user_id = evt.user_id, comment = %evt.comment, "auto-approving friend request");
            if let Err(e) = ctx.api.call(SetFriendAddRequest {
                flag: evt.flag.clone(),
                approve: Some(true),
                remark: None,
            }).await {
                tracing::error!(error = %e, "failed to approve friend request");
            }
        }
        RequestEvent::Group(evt) => {
            tracing::info!(
                group_id = evt.group_id,
                user_id = evt.user_id,
                sub_type = %evt.sub_type,
                comment = %evt.comment,
                "auto-approving group request"
            );
            if let Err(e) = ctx.api.call(SetGroupAddRequest {
                flag: evt.flag.clone(),
                sub_type: evt.sub_type.clone(),
                approve: Some(true),
                reason: None,
            }).await {
                tracing::error!(error = %e, "failed to approve group request");
            }
        }
    }
}
