use onebot::api::payload::{SendGroupForwardMsg, SendPrivateMsg};
use onebot::event::message::PrivateMessageEvent;
use onebot::message::MessageSegment;
use onebot::Message;

use super::HandlerContext;

pub async fn handle_private_cmd(ctx: &mut HandlerContext, evt: &PrivateMessageEvent) -> bool {
    let text = extract_plain_text_preserve_newlines(&evt.message);

    let Some(cmd) = text.strip_prefix("#cmd ").or_else(|| text.strip_prefix("#cmd")) else {
        return false;
    };

    let cmd = cmd.trim_start_matches(' ').trim_end();

    // `forward` 命令是多行的且需要异步调用 API，单独处理。
    let reply = if let Some(body) = cmd.strip_prefix("forward") {
        handle_forward(ctx, body).await
    } else {
        dispatch_cmd(ctx, cmd)
    };

    let _ = ctx.api.call(SendPrivateMsg {
        user_id: evt.user_id,
        message: Message::from(vec![MessageSegment::text(reply)]),
        auto_escape: None,
    }).await;

    true
}

/// 保留换行符的纯文本抽取（forward 命令需要多行输入）。
fn extract_plain_text_preserve_newlines(msg: &Message) -> String {
    match msg {
        Message::String(s) => s.clone(),
        Message::Array(segs) => segs
            .iter()
            .filter_map(|seg| {
                if let MessageSegment::Text { text } = seg { Some(text.as_str()) } else { None }
            })
            .collect::<Vec<_>>()
            .join(""),
    }
}

fn dispatch_cmd(ctx: &HandlerContext, cmd: &str) -> String {
    match cmd {
        "stats" | "统计" => format_stats(ctx),
        "help" | "帮助" | "" => format_help(),
        _ => format!("未知指令「{cmd}」，发送 #cmd help 查看可用指令"),
    }
}

// ============================================================
// forward: 发送群合并转发
// ============================================================

const FORWARD_USAGE: &str = "\
用法（多行）：\n\
#cmd forward <群号>\n\
<QQ号>[|昵称] <消息内容>\n\
<QQ号>[|昵称] <消息内容>\n\
...\n\
\n\
示例：\n\
#cmd forward 1038115684\n\
2862858494 我是大笨蛋\n\
2469930868|CC 确实";

async fn handle_forward(ctx: &HandlerContext, body: &str) -> String {
    let parsed = match parse_forward_body(body) {
        Ok(p) => p,
        Err(e) => return format!("解析失败：{e}\n\n{FORWARD_USAGE}"),
    };

    let nodes: Vec<MessageSegment> = parsed
        .nodes
        .into_iter()
        .map(|n| MessageSegment::Node {
            id: None,
            user_id: Some(n.user_id.to_string()),
            nickname: Some(n.nickname),
            content: Some(Message::from(vec![MessageSegment::Text { text: n.content }])),
        })
        .collect();

    let count = nodes.len();
    match ctx
        .api
        .call(SendGroupForwardMsg {
            group_id: parsed.group_id,
            messages: nodes,
        })
        .await
    {
        Ok(_) => format!("已向群 {} 发送 {} 条节点的合并转发。", parsed.group_id, count),
        Err(e) => format!("发送失败：{e}"),
    }
}

struct ParsedForward {
    group_id: i64,
    nodes: Vec<ParsedNode>,
}

struct ParsedNode {
    user_id: i64,
    nickname: String,
    content: String,
}

fn parse_forward_body(body: &str) -> Result<ParsedForward, String> {
    let mut lines = body.lines().map(str::trim_end);

    let group_line = loop {
        match lines.next() {
            Some(l) if l.trim().is_empty() => continue,
            Some(l) => break l,
            None => return Err("缺少群号（首行应为 `#cmd forward <群号>`）".into()),
        }
    };
    let group_id: i64 = group_line
        .trim()
        .parse()
        .map_err(|_| format!("群号不合法：`{}`", group_line.trim()))?;

    let mut nodes = Vec::new();
    for (idx, raw) in lines.enumerate() {
        if raw.trim().is_empty() {
            continue;
        }
        let node = parse_node_line(raw)
            .map_err(|e| format!("第 {} 条消息解析失败（`{raw}`）：{e}", idx + 1))?;
        nodes.push(node);
    }

    if nodes.is_empty() {
        return Err("至少需要 1 条消息".into());
    }

    Ok(ParsedForward { group_id, nodes })
}

/// 解析单行：`<qq>[|<nickname>] <content>`
fn parse_node_line(line: &str) -> Result<ParsedNode, String> {
    let (head, content) = line
        .split_once(char::is_whitespace)
        .ok_or("缺少消息内容（QQ号 与 内容之间用空格分隔）")?;
    let content = content.trim();
    if content.is_empty() {
        return Err("消息内容为空".into());
    }

    let (qq_str, nickname_opt) = match head.split_once('|') {
        Some((qq, nick)) => (qq, Some(nick.trim().to_string())),
        None => (head, None),
    };
    let user_id: i64 = qq_str.trim().parse().map_err(|_| format!("QQ号不合法：`{qq_str}`"))?;

    let nickname = nickname_opt.filter(|n| !n.is_empty()).unwrap_or_default();

    Ok(ParsedNode {
        user_id,
        nickname,
        content: content.to_string(),
    })
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
#cmd forward <群号>\\n<qq>[|昵称] <内容>...  - 发送群合并转发\n\
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
