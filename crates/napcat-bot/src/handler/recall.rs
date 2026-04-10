use std::collections::HashSet;
use std::path::{Path, PathBuf};

use onebot::api::payload::SendGroupMsg;
use onebot::event::message::GroupMessageEvent;
use onebot::event::notice::GroupRecallEvent;
use onebot::message::MessageSegment;
use onebot::Message;
use serde::{Deserialize, Serialize};

use super::{extract_plain_text, HandlerContext};

/// 持久化的撤回监控开关（按群隔离）。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct RecallConfig {
    enabled_groups: HashSet<i64>,
}

pub struct RecallToggle {
    config: RecallConfig,
    file_path: PathBuf,
}

impl RecallToggle {
    pub fn load(data_dir: &str) -> Self {
        let file_path = Path::new(data_dir).join("recall_toggle.json");
        let config = std::fs::read_to_string(&file_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        Self { config, file_path }
    }

    fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.config) {
            std::fs::write(&self.file_path, json).ok();
        }
    }

    pub fn is_enabled(&self, group_id: i64) -> bool {
        self.config.enabled_groups.contains(&group_id)
    }

    pub fn enable(&mut self, group_id: i64) {
        self.config.enabled_groups.insert(group_id);
        self.save();
    }

    pub fn disable(&mut self, group_id: i64) {
        self.config.enabled_groups.remove(&group_id);
        self.save();
    }
}

fn is_admin(evt: &GroupMessageEvent) -> bool {
    matches!(evt.sender.role.as_deref(), Some("admin" | "owner"))
}

/// 群内管理员指令：`#撤回监控 开启`  `#撤回监控 关闭`  `#撤回监控 状态`
pub async fn handle_recall_cmd(ctx: &mut HandlerContext, evt: &GroupMessageEvent) -> bool {
    let text = extract_plain_text(&evt.message);
    let Some(sub) = text.strip_prefix("#撤回监控").map(|s| s.trim()) else {
        return false;
    };

    if sub == "状态" {
        let status = if ctx.recall_toggle.is_enabled(evt.group_id) { "已开启" } else { "已关闭" };
        let _ = ctx.api.call(SendGroupMsg {
            group_id: evt.group_id,
            message: Message::from(vec![
                MessageSegment::reply(evt.message_id.to_string()),
                MessageSegment::text(format!("本群撤回监控：{status}")),
            ]),
            auto_escape: None,
        }).await;
        return true;
    }

    if !is_admin(evt) {
        let _ = ctx.api.call(SendGroupMsg {
            group_id: evt.group_id,
            message: Message::from(vec![
                MessageSegment::reply(evt.message_id.to_string()),
                MessageSegment::text("仅管理员可以操作撤回监控开关~"),
            ]),
            auto_escape: None,
        }).await;
        return true;
    }

    let reply = match sub {
        "开启" | "开" | "on" => {
            ctx.recall_toggle.enable(evt.group_id);
            "撤回监控已开启~"
        }
        "关闭" | "关" | "off" => {
            ctx.recall_toggle.disable(evt.group_id);
            "撤回监控已关闭~"
        }
        _ => "用法：#撤回监控 开启/关闭/状态",
    };

    let _ = ctx.api.call(SendGroupMsg {
        group_id: evt.group_id,
        message: Message::from(vec![
            MessageSegment::reply(evt.message_id.to_string()),
            MessageSegment::text(reply),
        ]),
        auto_escape: None,
    }).await;

    true
}

pub async fn handle_group_recall(ctx: &HandlerContext, evt: &GroupRecallEvent) {
    if evt.user_id == evt.self_id {
        return;
    }

    if !ctx.recall_toggle.is_enabled(evt.group_id) {
        return;
    }

    let Some(cache) = ctx.message_cache.get(&evt.group_id) else { return };
    let Some(msg) = cache.iter().find(|m| m.message_id == evt.message_id) else { return };

    let content = if msg.raw_message.is_empty() { &msg.text } else { &msg.raw_message };
    if content.is_empty() {
        return;
    }

    let text = format!("{} 撤回了一条消息：{}", msg.nickname, content);

    let _ = ctx.api.call(SendGroupMsg {
        group_id: evt.group_id,
        message: Message::from(vec![MessageSegment::text(text)]),
        auto_escape: None,
    }).await;
}
