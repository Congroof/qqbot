use onebot::api::payload::SendGroupMsg;
use onebot::event::message::GroupMessageEvent;
use onebot::Message;

use super::HandlerContext;

use std::collections::HashSet;

pub struct RepeatState {
    pub last_raw: String,
    pub senders: HashSet<i64>,
    pub already_repeated: bool,
}

pub async fn handle_group_message(ctx: &mut HandlerContext, evt: &GroupMessageEvent) {
    let group_id = evt.group_id;
    let raw = &evt.raw_message;

    if raw.is_empty() {
        return;
    }

    let state = ctx.repeat_states.entry(group_id).or_insert_with(|| RepeatState {
        last_raw: String::new(),
        senders: HashSet::new(),
        already_repeated: false,
    });

    if &state.last_raw == raw {
        state.senders.insert(evt.user_id);
    } else {
        state.last_raw = raw.clone();
        state.senders.clear();
        state.senders.insert(evt.user_id);
        state.already_repeated = false;
    }

    if state.senders.len() >= 2 && !state.already_repeated {
        state.already_repeated = true;

        if let Err(e) = ctx.api.call(SendGroupMsg {
            group_id,
            message: Message::String(raw.clone()),
            auto_escape: None,
        }).await {
            tracing::error!(error = %e, group_id, "failed to send repeat message");
        }
    }
}
