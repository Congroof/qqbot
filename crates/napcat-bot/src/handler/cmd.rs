use std::time::Instant;

use onebot::api::payload::{SendGroupForwardMsg, SendPrivateForwardMsg, SendPrivateMsg};
use onebot::event::message::PrivateMessageEvent;
use onebot::message::MessageSegment;
use onebot::Message;

use super::HandlerContext;
use super::vocab;

pub async fn handle_private_cmd(ctx: &mut HandlerContext, evt: &PrivateMessageEvent) -> bool {
    let text = extract_plain_text_preserve_newlines(&evt.message);

    let Some(cmd) = text.strip_prefix("#cmd ").or_else(|| text.strip_prefix("#cmd")) else {
        return false;
    };

    let cmd = cmd.trim_start_matches(' ').trim_end();
    let sender_id = evt.user_id;

    // `识别图片` / `ocr` 命令：进入等待图片状态
    if cmd == "识别图片" || cmd == "ocr" {
        ctx.pending_ocr.insert(sender_id, Instant::now());
        let _ = ctx.api.call(SendPrivateMsg {
            user_id: sender_id,
            message: Message::from(vec![MessageSegment::text(
                "请发送需要识别的图片（支持中英文）",
            )]),
            auto_escape: None,
        }).await;
        return true;
    }

    // `word` 命令需要发送语音，单独处理。
    if let Some(word) = cmd.strip_prefix("word ") {
        let word = word.trim();
        if !word.is_empty() {
            vocab::handle_private_vocab(ctx, evt.user_id, word).await;
            return true;
        }
    }

    // `forward` / `forward_private` 命令是多行的且需要异步调用 API，单独处理。
    let reply = if let Some(body) = cmd.strip_prefix("forward_private") {
        handle_forward_private(ctx, body, sender_id).await
    } else if let Some(body) = cmd.strip_prefix("forward") {
        handle_forward_group(ctx, body, sender_id).await
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

const FORWARD_GROUP_USAGE: &str = "\
用法（多行，支持任意多条消息）：\n\
#cmd forward <群号>\n\
<QQ号>/<昵称> <消息内容>\n\
<QQ号>/<昵称> <消息内容>\n\
...（一行一条，可以写很多行）\n\
\n\
说明：每行用 `/` 分隔 QQ 与昵称，再用空格分隔昵称与正文。\n\
\n\
示例：\n\
#cmd forward 123456789\n\
10001/张三 你好呀\n\
10002/李四 今天天气不错\n\
10003/王五 我也觉得";

async fn handle_forward_group(ctx: &HandlerContext, body: &str, _sender_id: i64) -> String {
    let parsed = match parse_forward_body(body, "群号") {
        Ok(p) => p,
        Err(e) => return format!("解析失败：{e}\n\n{FORWARD_GROUP_USAGE}"),
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
            group_id: parsed.target_id,
            messages: nodes,
            source: None,
            summary: None,
            prompt: None,
        })
        .await
    {
        Ok(_) => format!("已向群 {} 发送 {} 条节点的合并转发。", parsed.target_id, count),
        Err(e) => format!("发送失败：{e}"),
    }
}

// ============================================================
// forward_private: 发送私聊合并转发
// ============================================================

const FORWARD_PRIVATE_USAGE: &str = "\
用法（多行，支持任意多条消息）：\n\
#cmd forward_private [自定义标题]\n\
<QQ号>/<昵称> <消息内容>\n\
<QQ号>/<昵称> <消息内容>\n\
...（一行一条，可以写很多行）\n\
\n\
说明：\n\
- 标题可选，不填则默认显示「合并转发」\n\
- 每行用 `/` 分隔 QQ 与昵称，再用空格分隔昵称与正文\n\
- 消息会发送给当前私聊对象（即你自己）\n\
\n\
示例：\n\
#cmd forward_private 今日聊天精选\n\
10001/张三 你好呀\n\
10002/李四 今天天气不错\n\
10003/王五 我也觉得";

async fn handle_forward_private(ctx: &HandlerContext, body: &str, sender_id: i64) -> String {
    let normalized = body.replace("\r\n", "\n").replace('\r', "\n");
    let trimmed = normalized.trim_start_matches('\n');

    // 第一行为可选标题，如果第一行不是消息节点（不含 `/`），则当作标题
    let (title, msg_body) = match trimmed.split_once('\n') {
        Some((first_line, rest)) => {
            let fl = first_line.trim();
            if fl.is_empty() || fl.contains('/') {
                ("合并转发".to_string(), trimmed.to_string())
            } else {
                (fl.to_string(), rest.to_string())
            }
        }
        None => {
            let fl = trimmed.trim();
            if fl.is_empty() || fl.contains('/') {
                ("合并转发".to_string(), trimmed.to_string())
            } else {
                return format!("解析失败：至少需要 1 条消息\n\n{FORWARD_PRIVATE_USAGE}");
            }
        }
    };

    let parsed = match parse_forward_body_no_target(&msg_body) {
        Ok(p) => p,
        Err(e) => return format!("解析失败：{e}\n\n{FORWARD_PRIVATE_USAGE}"),
    };

    let nodes: Vec<MessageSegment> = parsed
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
        .call(SendPrivateForwardMsg {
            user_id: sender_id,
            messages: nodes,
            source: Some(title.clone()),
            summary: Some(format!("查看 {} 条消息", count)),
            prompt: Some(title),
        })
        .await
    {
        Ok(_) => format!("已向你发送 {} 条节点的私聊合并转发。", count),
        Err(e) => format!("发送失败：{e}"),
    }
}

struct ParsedForward {
    target_id: i64,
    nodes: Vec<ParsedNode>,
}

struct ParsedNode {
    user_id: i64,
    nickname: String,
    content: String,
}

fn parse_forward_body(body: &str, target_name: &str) -> Result<ParsedForward, String> {
    // 统一处理各种换行符：\r\n, \n, \r（手机 QQ 可能使用不同的换行方式）
    let normalized = body.replace("\r\n", "\n").replace('\r', "\n");
    let mut lines = normalized.lines().map(str::trim_end);

    let target_line = loop {
        match lines.next() {
            Some(l) if l.trim().is_empty() => continue,
            Some(l) => break l,
            None => return Err(format!("缺少{target_name}（首行应为目标号码）")),
        }
    };
    let target_id: i64 = target_line
        .trim()
        .parse()
        .map_err(|_| format!("{target_name}不合法：`{}`", target_line.trim()))?;

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

    Ok(ParsedForward { target_id, nodes })
}

/// 解析单行：`<qq>/<nickname> <content>`
fn parse_node_line(line: &str) -> Result<ParsedNode, String> {
    let (head, content) = line
        .split_once(char::is_whitespace)
        .ok_or("缺少消息内容（昵称 与 内容之间用空格分隔）")?;
    let content = content.trim();
    if content.is_empty() {
        return Err("消息内容为空".into());
    }

    let (qq_str, nickname) = head
        .split_once('/')
        .ok_or("缺少昵称（QQ号 与 昵称之间用 `/` 分隔）")?;
    let nickname = nickname.trim().to_string();
    if nickname.is_empty() {
        return Err("昵称为空".into());
    }
    let user_id: i64 = qq_str.trim().parse().map_err(|_| format!("QQ号不合法：`{qq_str}`"))?;

    Ok(ParsedNode {
        user_id,
        nickname,
        content: content.to_string(),
    })
}

/// 解析不需要目标号码的转发内容（用于 forward_private，目标为当前发送者）
fn parse_forward_body_no_target(body: &str) -> Result<Vec<ParsedNode>, String> {
    let normalized = body.replace("\r\n", "\n").replace('\r', "\n");
    let lines = normalized.lines().map(str::trim_end);

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

    Ok(nodes)
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

pub fn format_help() -> String {
    "\
--- 私聊指令 ---\n\
#cmd stats - 运行状态 & Token 用量\n\
#cmd help  - 显示本帮助\n\
#cmd word <单词> - 查单词释义+发音\n\
#cmd forward <群号> - 发送群合并转发\n\
#cmd forward_private [标题] - 私聊合并转发（可选自定义标题）\n\
（合并转发格式：每行「QQ号/昵称 消息内容」）\n\
#cmd 识别图片 - OCR 文字识别（发指令后60秒内发图片）\n\
\n\
--- 群聊功能 ---\n\
运势 / 求签 / 今日运势 - 今日运势（每日一次）\n\
语录 / 名言 / 名人名言 - 随机名人名言\n\
成语接龙 - 开始成语接龙游戏\n\
结束接龙 - 结束当前接龙\n\
水群排行 - 今日发言排行榜\n\
总结 / 消息摘要 - AI 总结最近聊天\n\
赞我 - 点赞（最多 50 个）\n\
单词 <英文> - 查单词翻译+发音\n\
随机单词 / 背单词 - 随机学一个单词\n\
gencdk <hwid> - 生成 2 小时有效期的 CDK\n\
朗读 <英文文本> - 文字转语音朗读\n\
戳一戳机器人 - 随机回复\n\
@机器人 + 消息 - AI 聊天（5分钟会话记忆）\n\
\n\
--- 群管理员指令 ---\n\
#角色 <名称> - 切换 AI 人设（猫娘/毒舌/哲学家/老中医/诗人/默认）\n\
#撤回监控 开启 - 开启撤回消息曝光\n\
#撤回监控 关闭 - 关闭撤回消息曝光\n\
#撤回监控 状态 - 查看当前状态\n\
\n\
--- 自动功能 ---\n\
新成员入群验证（算术题，60秒超时踢出）\n\
复读机（连续相同消息自动复读）\n\
每日定时点赞（群主/管理员）"
        .to_string()
}
