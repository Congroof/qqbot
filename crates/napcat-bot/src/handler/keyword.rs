use std::path::{Path, PathBuf};

use onebot::api::payload::SendGroupMsg;
use onebot::event::message::GroupMessageEvent;
use onebot::message::MessageSegment;
use onebot::Message;
use rand::seq::IndexedRandom;
use serde::{Deserialize, Serialize};

use super::{extract_plain_text, HandlerContext};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeywordRule {
    pub keyword: String,
    pub replies: Vec<String>,
    #[serde(default = "default_match_type")]
    pub match_type: String,
}

fn default_match_type() -> String { "exact".into() }

pub struct KeywordStore {
    rules: Vec<KeywordRule>,
    file_path: PathBuf,
}

impl KeywordStore {
    pub fn load(data_dir: &str) -> Self {
        let file_path = Path::new(data_dir).join("keywords.json");
        let mut rules: Vec<KeywordRule> = std::fs::read_to_string(&file_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();

        if rules.is_empty() {
            rules = default_rules();
        }

        Self { rules, file_path }
    }

    fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.rules) {
            std::fs::write(&self.file_path, json).ok();
        }
    }

    fn find_match(&self, text: &str) -> Option<&KeywordRule> {
        self.rules.iter().find(|r| {
            match r.match_type.as_str() {
                "contains" => text.contains(&r.keyword),
                _ => text == r.keyword,
            }
        })
    }

    pub fn add_rule(&mut self, keyword: String, reply: String) {
        if let Some(existing) = self.rules.iter_mut().find(|r| r.keyword == keyword) {
            existing.replies.push(reply);
        } else {
            self.rules.push(KeywordRule {
                keyword,
                replies: vec![reply],
                match_type: "exact".into(),
            });
        }
        self.save();
    }

    pub fn remove_rule(&mut self, keyword: &str) -> bool {
        let before = self.rules.len();
        self.rules.retain(|r| r.keyword != keyword);
        if self.rules.len() < before {
            self.save();
            true
        } else {
            false
        }
    }

    pub fn list_keywords(&self) -> Vec<String> {
        self.rules.iter().map(|r| r.keyword.clone()).collect()
    }
}

fn default_rules() -> Vec<KeywordRule> {
    vec![
        KeywordRule {
            keyword: "菜单".into(),
            replies: vec![
                "功能列表：运势 | 语录 | 成语接龙 | 赞我 | 水群排行 | 总结 | @我 聊天".into(),
            ],
            match_type: "exact".into(),
        },
        KeywordRule {
            keyword: "ping".into(),
            replies: vec!["pong!".into()],
            match_type: "exact".into(),
        },
    ]
}

pub async fn handle_keyword(ctx: &HandlerContext, evt: &GroupMessageEvent) -> bool {
    let text = extract_plain_text(&evt.message);

    let Some(rule) = ctx.keywords.find_match(&text) else {
        return false;
    };

    let reply = rule.replies.choose(&mut rand::rng())
        .map(|s| s.as_str())
        .unwrap_or("...");

    let _ = ctx.api.call(SendGroupMsg {
        group_id: evt.group_id,
        message: Message::from(vec![MessageSegment::text(reply)]),
        auto_escape: None,
    }).await;

    true
}
