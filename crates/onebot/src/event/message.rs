use serde::Deserialize;

use crate::message::Message;

/// 消息事件，按 `message_type` 分发为私聊/群聊。
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "message_type")]
pub enum MessageEvent {
    #[serde(rename = "private")]
    Private(PrivateMessageEvent),
    #[serde(rename = "group")]
    Group(GroupMessageEvent),
}

/// 私聊消息事件
#[derive(Debug, Clone, Deserialize)]
pub struct PrivateMessageEvent {
    pub time: i64,
    pub self_id: i64,
    pub sub_type: String,
    pub message_id: i64,
    pub user_id: i64,
    pub message: Message,
    pub raw_message: String,
    pub font: i32,
    pub sender: PrivateSender,
}

/// 私聊消息发送者信息
#[derive(Debug, Clone, Deserialize)]
pub struct PrivateSender {
    #[serde(default)]
    pub user_id: Option<i64>,
    #[serde(default)]
    pub nickname: Option<String>,
    #[serde(default)]
    pub sex: Option<String>,
    #[serde(default)]
    pub age: Option<i32>,
}

/// 群消息事件
#[derive(Debug, Clone, Deserialize)]
pub struct GroupMessageEvent {
    pub time: i64,
    pub self_id: i64,
    pub sub_type: String,
    pub message_id: i64,
    pub group_id: i64,
    pub user_id: i64,
    #[serde(default)]
    pub anonymous: Option<Anonymous>,
    pub message: Message,
    pub raw_message: String,
    pub font: i32,
    pub sender: GroupSender,
}

/// 匿名用户信息
#[derive(Debug, Clone, Deserialize)]
pub struct Anonymous {
    pub id: i64,
    pub name: String,
    pub flag: String,
}

/// 群消息发送者信息
#[derive(Debug, Clone, Deserialize)]
pub struct GroupSender {
    #[serde(default)]
    pub user_id: Option<i64>,
    #[serde(default)]
    pub nickname: Option<String>,
    #[serde(default)]
    pub card: Option<String>,
    #[serde(default)]
    pub sex: Option<String>,
    #[serde(default)]
    pub age: Option<i32>,
    #[serde(default)]
    pub area: Option<String>,
    #[serde(default)]
    pub level: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
}
