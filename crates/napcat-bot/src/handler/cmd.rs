use onebot::api::payload::SendPrivateMsg;
use onebot::event::message::PrivateMessageEvent;
use onebot::message::MessageSegment;
use onebot::Message;

use super::HandlerContext;

/// 处理私聊中的 `#cmd` 指令，返回 true 表示已处理。
pub async fn handle_private_cmd(ctx: &HandlerContext, evt: &PrivateMessageEvent) -> bool {
    let text = extract_plain_text(&evt.message);

    let Some(cmd) = text.strip_prefix("#cmd ").or_else(|| text.strip_prefix("#cmd")) else {
        return false;
    };

    let cmd = cmd.trim();
    let reply = dispatch_cmd(ctx, cmd);

    let _ = ctx.api.call(SendPrivateMsg {
        user_id: evt.user_id,
        message: Message::from(vec![MessageSegment::text(reply)]),
        auto_escape: None,
    }).await;

    true
}

fn dispatch_cmd(ctx: &HandlerContext, cmd: &str) -> String {
    match cmd {
        "查看统计" | "统计" | "stats" => format_stats(ctx),
        "帮助" | "help" => format_help(),
        _ => format!("未知指令「{cmd}」，发送 #cmd 帮助 查看可用指令"),
    }
}

fn format_stats(ctx: &HandlerContext) -> String {
    let u = &ctx.token_usage;
    let sessions = ctx.chat_sessions.len();
    format!(
        "--- Bot 运行统计 ---\n\
         运行时长：{uptime}\n\
         活跃会话数：{sessions}\n\
         \n\
         --- AI Token 用量 ---\n\
         总调用次数：{reqs}\n\
         Prompt tokens：{prompt}\n\
         Completion tokens：{completion}\n\
         总 tokens：{total}",
        uptime = u.uptime_display(),
        reqs = u.total_requests,
        prompt = u.prompt_tokens,
        completion = u.completion_tokens,
        total = u.total_tokens(),
    )
}

fn format_help() -> String {
    "\
--- Bot 指令帮助 ---\n\
#cmd 查看统计 - 查看运行状态和 Token 用量\n\
#cmd 帮助 - 显示本帮助信息"
        .to_string()
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
