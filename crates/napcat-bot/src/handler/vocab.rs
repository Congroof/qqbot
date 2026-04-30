use std::collections::HashMap;
use std::path::Path;

use base64::Engine;
use edge_tts_rust::{EdgeTtsClient, SpeakOptions};
use onebot::api::payload::{SendGroupMsg, SendPrivateMsg};
use onebot::event::message::GroupMessageEvent;
use onebot::message::MessageSegment;
use onebot::Message;
use rand::seq::IndexedRandom;

use super::{extract_plain_text, HandlerContext};

const DEEPLX_BASE_URL: &str = "https://api.deeplx.org";

#[derive(Debug, serde::Deserialize)]
struct DeeplxResponse {
    code: i32,
    data: String,
    #[serde(default)]
    alternatives: Vec<String>,
}

/// 词库：单词 -> 中文翻译
pub struct Dictionary {
    entries: HashMap<String, String>,
    words: Vec<String>,
}

impl Dictionary {
    pub fn load(data_dir: &str) -> Self {
        let path = Path::new(data_dir).join("word_translation.csv");
        match Self::load_from_file(&path) {
            Ok(dict) => {
                tracing::info!(count = dict.words.len(), "dictionary loaded");
                dict
            }
            Err(e) => {
                tracing::warn!(error = %e, "failed to load dictionary, using empty");
                Self { entries: HashMap::new(), words: Vec::new() }
            }
        }
    }

    fn load_from_file(path: &Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("read file: {e}"))?;

        let mut entries = HashMap::new();
        let mut words = Vec::new();

        for (i, line) in content.lines().enumerate() {
            if i == 0 { continue; } // skip header
            let line = line.trim();
            if line.is_empty() { continue; }

            if let Some((word, translation)) = parse_csv_line(line) {
                let word_lower = word.to_lowercase();
                if !word_lower.is_empty() && word_lower.chars().all(|c| c.is_ascii_alphabetic() || c == '-' || c == '\'') {
                    words.push(word_lower.clone());
                    entries.insert(word_lower, translation);
                }
            }
        }

        Ok(Self { entries, words })
    }

    pub fn lookup(&self, word: &str) -> Option<&str> {
        self.entries.get(&word.to_lowercase()).map(|s| s.as_str())
    }

    pub fn random_word(&self) -> Option<&str> {
        let mut rng = rand::rng();
        self.words.choose(&mut rng).map(|s| s.as_str())
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// 解析 CSV 行："word","translation"
fn parse_csv_line(line: &str) -> Option<(String, String)> {
    let mut chars = line.chars().peekable();
    let word = parse_csv_field(&mut chars)?;
    if chars.next() != Some(',') { return None; }
    let translation = parse_csv_field(&mut chars)?;
    Some((word, translation))
}

fn parse_csv_field(chars: &mut std::iter::Peekable<std::str::Chars>) -> Option<String> {
    if chars.peek() == Some(&'"') {
        chars.next(); // consume opening quote
        let mut field = String::new();
        loop {
            match chars.next() {
                Some('"') => {
                    if chars.peek() == Some(&'"') {
                        chars.next();
                        field.push('"');
                    } else {
                        break;
                    }
                }
                Some(c) => field.push(c),
                None => break,
            }
        }
        Some(field)
    } else {
        let mut field = String::new();
        while let Some(&c) = chars.peek() {
            if c == ',' { break; }
            field.push(c);
            chars.next();
        }
        Some(field)
    }
}

pub async fn handle_vocab(ctx: &mut HandlerContext, evt: &GroupMessageEvent) -> bool {
    let text = extract_plain_text(&evt.message);

    if matches!(text.as_str(), "随机单词" | "背单词") {
        return handle_random_word(ctx, evt).await;
    }

    if let Some(content) = text.strip_prefix("朗读 ").or_else(|| text.strip_prefix("read ")) {
        let content = content.trim();
        if !content.is_empty() && content.len() <= 500 {
            return handle_tts(ctx, evt, content).await;
        }
    }

    let word = if let Some(w) = text.strip_prefix("单词 ") {
        w.trim()
    } else if let Some(w) = text.strip_prefix("查单词 ") {
        w.trim()
    } else if let Some(w) = text.strip_prefix("word ") {
        w.trim()
    } else {
        return false;
    };

    if word.is_empty() || word.len() > 100 {
        let _ = ctx.api.call(SendGroupMsg {
            group_id: evt.group_id,
            message: Message::from(vec![
                MessageSegment::reply(evt.message_id.to_string()),
                MessageSegment::text("请输入有效内容"),
            ]),
            auto_escape: None,
        }).await;
        return true;
    }

    handle_word_query(ctx, evt, word).await
}

async fn handle_tts(ctx: &HandlerContext, evt: &GroupMessageEvent, text: &str) -> bool {
    match generate_pronunciation(text).await {
        Ok(audio_base64) => {
            let _ = ctx.api.call(SendGroupMsg {
                group_id: evt.group_id,
                message: Message::from(vec![
                    MessageSegment::Record {
                        file: format!("base64://{audio_base64}"),
                        magic: None,
                        url: None,
                        cache: None,
                        proxy: None,
                        timeout: None,
                    },
                ]),
                auto_escape: None,
            }).await;
        }
        Err(e) => {
            tracing::warn!(error = %e, "tts generation failed");
            let _ = ctx.api.call(SendGroupMsg {
                group_id: evt.group_id,
                message: Message::from(vec![
                    MessageSegment::reply(evt.message_id.to_string()),
                    MessageSegment::text(format!("语音生成失败：{e}")),
                ]),
                auto_escape: None,
            }).await;
        }
    }
    true
}

async fn handle_random_word(ctx: &mut HandlerContext, evt: &GroupMessageEvent) -> bool {
    let word = match ctx.dictionary.random_word() {
        Some(w) => w.to_string(),
        None => {
            let _ = ctx.api.call(SendGroupMsg {
                group_id: evt.group_id,
                message: Message::from(vec![
                    MessageSegment::reply(evt.message_id.to_string()),
                    MessageSegment::text("词库未加载"),
                ]),
                auto_escape: None,
            }).await;
            return true;
        }
    };

    handle_word_query(ctx, evt, &word).await
}

async fn handle_word_query(ctx: &mut HandlerContext, evt: &GroupMessageEvent, word: &str) -> bool {
    let word_lower = word.to_lowercase();

    let translation = if let Some(t) = ctx.dictionary.lookup(&word_lower) {
        t.to_string()
    } else if let Some(api_key) = &ctx.config.deeplx_api_key {
        match translate_deeplx(api_key, word, "auto", "ZH").await {
            Ok(resp) => {
                let mut result = resp.data;
                if !resp.alternatives.is_empty() {
                    let alts = resp.alternatives.iter()
                        .take(3)
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(" / ");
                    result.push_str(&format!("\n💡 {alts}"));
                }
                result
            }
            Err(e) => {
                tracing::error!(error = %e, word = %word_lower, "deeplx translate failed");
                let _ = ctx.api.call(SendGroupMsg {
                    group_id: evt.group_id,
                    message: Message::from(vec![
                        MessageSegment::reply(evt.message_id.to_string()),
                        MessageSegment::text(format!("查询「{word}」失败：{e}")),
                    ]),
                    auto_escape: None,
                }).await;
                return true;
            }
        }
    } else {
        let _ = ctx.api.call(SendGroupMsg {
            group_id: evt.group_id,
            message: Message::from(vec![
                MessageSegment::reply(evt.message_id.to_string()),
                MessageSegment::text(format!("词库中未找到「{word}」，且翻译服务未配置")),
            ]),
            auto_escape: None,
        }).await;
        return true;
    };

    let text_reply = format!("📖 {word}\n🔤 {translation}");

    let _ = ctx.api.call(SendGroupMsg {
        group_id: evt.group_id,
        message: Message::from(vec![
            MessageSegment::reply(evt.message_id.to_string()),
            MessageSegment::text(&text_reply),
        ]),
        auto_escape: None,
    }).await;

    if is_single_word(&word_lower) {
        match generate_pronunciation(&word_lower).await {
            Ok(audio_base64) => {
                let _ = ctx.api.call(SendGroupMsg {
                    group_id: evt.group_id,
                    message: Message::from(vec![
                        MessageSegment::Record {
                            file: format!("base64://{audio_base64}"),
                            magic: None,
                            url: None,
                            cache: None,
                            proxy: None,
                            timeout: None,
                        },
                    ]),
                    auto_escape: None,
                }).await;
            }
            Err(e) => {
                tracing::warn!(error = %e, word = %word_lower, "tts generation failed");
            }
        }
    }

    true
}

/// 私聊 `#cmd word xxx` 触发
pub async fn handle_private_vocab(ctx: &mut HandlerContext, user_id: i64, word: &str) {
    let word_lower = word.to_lowercase();

    let translation = if let Some(t) = ctx.dictionary.lookup(&word_lower) {
        t.to_string()
    } else if let Some(api_key) = &ctx.config.deeplx_api_key {
        match translate_deeplx(api_key, word, "auto", "ZH").await {
            Ok(resp) => {
                let mut result = resp.data;
                if !resp.alternatives.is_empty() {
                    let alts = resp.alternatives.iter()
                        .take(3)
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(" / ");
                    result.push_str(&format!("\n💡 {alts}"));
                }
                result
            }
            Err(e) => {
                tracing::error!(error = %e, word = %word_lower, "deeplx translate failed (private)");
                let _ = ctx.api.call(SendPrivateMsg {
                    user_id,
                    message: Message::from(vec![
                        MessageSegment::text(format!("查询「{word}」失败：{e}")),
                    ]),
                    auto_escape: None,
                }).await;
                return;
            }
        }
    } else {
        let _ = ctx.api.call(SendPrivateMsg {
            user_id,
            message: Message::from(vec![
                MessageSegment::text(format!("词库中未找到「{word}」，且翻译服务未配置")),
            ]),
            auto_escape: None,
        }).await;
        return;
    };

    let text_reply = format!("📖 {word}\n🔤 {translation}");

    let _ = ctx.api.call(SendPrivateMsg {
        user_id,
        message: Message::from(vec![MessageSegment::text(&text_reply)]),
        auto_escape: None,
    }).await;

    if is_single_word(&word_lower) {
        match generate_pronunciation(&word_lower).await {
            Ok(audio_base64) => {
                let _ = ctx.api.call(SendPrivateMsg {
                    user_id,
                    message: Message::from(vec![
                        MessageSegment::Record {
                            file: format!("base64://{audio_base64}"),
                            magic: None,
                            url: None,
                            cache: None,
                            proxy: None,
                            timeout: None,
                        },
                    ]),
                    auto_escape: None,
                }).await;
            }
            Err(e) => {
                tracing::warn!(error = %e, word = %word_lower, "tts generation failed (private)");
            }
        }
    }
}

fn is_single_word(text: &str) -> bool {
    text.split_whitespace().count() <= 3
}

async fn translate_deeplx(
    api_key: &str,
    text: &str,
    source_lang: &str,
    target_lang: &str,
) -> Result<DeeplxResponse, String> {
    let url = format!("{DEEPLX_BASE_URL}/{api_key}/translate");

    let body = serde_json::json!({
        "text": text,
        "source_lang": source_lang,
        "target_lang": target_lang,
    });

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }

    let result: DeeplxResponse = resp.json().await
        .map_err(|e| format!("JSON parse failed: {e}"))?;

    if result.code != 200 {
        return Err(format!("API error code: {}", result.code));
    }

    Ok(result)
}

async fn generate_pronunciation(word: &str) -> Result<String, String> {
    let client = EdgeTtsClient::new()
        .map_err(|e| format!("TTS client init failed: {e}"))?;

    let result = client
        .synthesize(
            word,
            SpeakOptions {
                voice: "en-US-JennyNeural".into(),
                ..SpeakOptions::default()
            },
        )
        .await
        .map_err(|e| format!("TTS synthesis failed: {e}"))?;

    let base64_str = base64::engine::general_purpose::STANDARD.encode(&result.audio);
    Ok(base64_str)
}
