use std::collections::HashMap;
use std::path::{Path, PathBuf};

use onebot::api::payload::SendGroupMsg;
use onebot::event::message::GroupMessageEvent;
use onebot::message::MessageSegment;
use onebot::Message;
use serde::{Deserialize, Serialize};

use super::{extract_plain_text, HandlerContext};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct DayStats {
    // date -> group_id -> user_id -> UserCount
    counts: HashMap<String, HashMap<String, HashMap<String, UserCount>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserCount {
    nickname: String,
    count: u32,
}

pub struct MsgStats {
    data: DayStats,
    file_path: PathBuf,
}

impl MsgStats {
    pub fn load(data_dir: &str) -> Self {
        let file_path = Path::new(data_dir).join("msg_stats.json");
        let data = std::fs::read_to_string(&file_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        Self { data, file_path }
    }

    fn save(&self) {
        if let Ok(json) = serde_json::to_string(&self.data) {
            std::fs::write(&self.file_path, json).ok();
        }
    }

    fn increment(&mut self, group_id: i64, user_id: i64, nickname: &str) {
        let today = today_str();

        self.data.counts.retain(|date, _| *date == today);

        let day = self.data.counts.entry(today).or_default();
        let group = day.entry(group_id.to_string()).or_default();
        let entry = group.entry(user_id.to_string()).or_insert(UserCount {
            nickname: nickname.to_string(),
            count: 0,
        });
        entry.nickname = nickname.to_string();
        entry.count += 1;

        if entry.count % 10 == 0 {
            self.save();
        }
    }

    fn top10(&self, group_id: i64) -> Vec<(String, u32)> {
        let today = today_str();
        let Some(day) = self.data.counts.get(&today) else { return vec![] };
        let Some(group) = day.get(&group_id.to_string()) else { return vec![] };

        let mut list: Vec<_> = group.values().map(|u| (u.nickname.clone(), u.count)).collect();
        list.sort_by(|a, b| b.1.cmp(&a.1));
        list.truncate(10);
        list
    }
}

pub fn record_message(ctx: &mut HandlerContext, evt: &GroupMessageEvent) {
    let nickname = evt.sender.card.clone()
        .filter(|c| !c.is_empty())
        .or_else(|| evt.sender.nickname.clone())
        .unwrap_or_else(|| evt.user_id.to_string());
    ctx.msg_stats.increment(evt.group_id, evt.user_id, &nickname);
}

pub async fn handle_stats(ctx: &HandlerContext, evt: &GroupMessageEvent) -> bool {
    let text = extract_plain_text(&evt.message);
    if !matches!(text.as_str(), "水群排行" | "发言排行") {
        return false;
    }

    let top = ctx.msg_stats.top10(evt.group_id);
    if top.is_empty() {
        let _ = ctx.api.call(SendGroupMsg {
            group_id: evt.group_id,
            message: Message::from(vec![
                MessageSegment::reply(evt.message_id.to_string()),
                MessageSegment::text("今天还没有发言记录呢~"),
            ]),
            auto_escape: None,
        }).await;
        return true;
    }

    let mut text = "今日水群排行榜\n".to_string();
    let medals = ["🥇", "🥈", "🥉"];
    for (i, (name, count)) in top.iter().enumerate() {
        let prefix = if i < 3 { medals[i] } else { &format!("{}.", i + 1) };
        text.push_str(&format!("{prefix} {name} - {count} 条\n"));
    }

    let _ = ctx.api.call(SendGroupMsg {
        group_id: evt.group_id,
        message: Message::from(vec![
            MessageSegment::reply(evt.message_id.to_string()),
            MessageSegment::text(text.trim_end()),
        ]),
        auto_escape: None,
    }).await;

    true
}

fn today_str() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let days = (secs + 8 * 3600) / 86400;
    format!("{days}")
}
