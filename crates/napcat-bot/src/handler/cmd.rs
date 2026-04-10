use onebot::api::payload::SendPrivateMsg;
use onebot::event::message::PrivateMessageEvent;
use onebot::message::MessageSegment;
use onebot::Message;

use super::{extract_plain_text, HandlerContext};

pub async fn handle_private_cmd(ctx: &mut HandlerContext, evt: &PrivateMessageEvent) -> bool {
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
        "stats" | "统计" => format_stats(ctx),
        "help" | "帮助" | "" => format_help(),
        _ => format!("未知指令「{cmd}」，发送 #cmd help 查看可用指令"),
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
--- 私聊管理指令 ---\n\
#cmd stats - 运行状态 & Token 用量\n\
#cmd help  - 显示本帮助\n\
\n\
--- 群聊功能 ---\n\
运势 / 求签     - 今日运势\n\
语录 / 名言     - 随机名人名言\n\
成语接龙 / 结束接龙 - 成语接龙游戏\n\
水群排行        - 今日发言排行\n\
总结            - AI 总结最近聊天\n\
赞我            - 点赞（最多 50 个）\n\
@bot            - AI 聊天\n\
\n\
--- 群管理员指令 ---\n\
#角色 <角色名/默认> - 切换全群 AI 人设\n\
#撤回监控 开启/关闭/状态 - 撤回消息曝光"
        .to_string()
}
