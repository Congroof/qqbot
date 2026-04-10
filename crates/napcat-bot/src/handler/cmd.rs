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

fn dispatch_cmd(ctx: &mut HandlerContext, cmd: &str) -> String {
    if let Some(rest) = cmd.strip_prefix("添加关键词 ") {
        return handle_add_keyword(ctx, rest);
    }
    if let Some(kw) = cmd.strip_prefix("删除关键词 ") {
        return handle_remove_keyword(ctx, kw.trim());
    }

    match cmd {
        "查看统计" | "统计" | "stats" => format_stats(ctx),
        "关键词列表" | "keywords" => format_keywords(ctx),
        "帮助" | "help" => format_help(),
        _ => format!("未知指令「{cmd}」，发送 #cmd 帮助 查看可用指令"),
    }
}

fn handle_add_keyword(ctx: &mut HandlerContext, rest: &str) -> String {
    let parts: Vec<&str> = rest.splitn(2, ' ').collect();
    if parts.len() < 2 {
        return "格式：#cmd 添加关键词 <关键词> <回复内容>".into();
    }
    let keyword = parts[0].trim().to_string();
    let reply = parts[1].trim().to_string();
    ctx.keywords.add_rule(keyword.clone(), reply);
    format!("已添加关键词「{keyword}」")
}

fn handle_remove_keyword(ctx: &mut HandlerContext, keyword: &str) -> String {
    if ctx.keywords.remove_rule(keyword) {
        format!("已删除关键词「{keyword}」")
    } else {
        format!("关键词「{keyword}」不存在")
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

fn format_keywords(ctx: &HandlerContext) -> String {
    let list = ctx.keywords.list_keywords();
    if list.is_empty() {
        return "暂无自定义关键词".into();
    }
    let mut text = "--- 关键词列表 ---\n".to_string();
    for kw in &list {
        text.push_str(&format!("- {kw}\n"));
    }
    text.trim_end().to_string()
}

fn format_help() -> String {
    "\
--- Bot 私聊指令 ---\n\
#cmd 统计 - 查看运行状态和 Token 用量\n\
#cmd 关键词列表 - 查看所有关键词\n\
#cmd 添加关键词 <词> <回复> - 添加关键词\n\
#cmd 删除关键词 <词> - 删除关键词\n\
#cmd 帮助 - 显示本帮助\n\
\n\
--- 群聊功能 ---\n\
运势 / 求签 - 今日运势\n\
语录 / 名言 - 随机名人名言\n\
成语接龙 / 结束接龙 - 成语接龙游戏\n\
水群排行 - 今日发言排行\n\
总结 - AI 总结最近聊天\n\
赞我 - 点赞（最多 50 个）\n\
@bot - AI 聊天\n\
\n\
--- 群管理员指令 ---\n\
#角色 <角色名/默认> - 切换全群 AI 人设\n\
#撤回监控 开启/关闭/状态 - 撤回消息曝光"
        .to_string()
}
