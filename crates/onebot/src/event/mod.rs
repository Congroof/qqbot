pub mod message;
pub mod meta;
pub mod notice;
pub mod request;

use serde::Deserialize;

pub use self::message::{GroupMessageEvent, GroupSender, MessageEvent, PrivateMessageEvent, PrivateSender, Anonymous};
pub use self::meta::{MetaEvent, HeartbeatEvent, LifecycleEvent};
pub use self::notice::NoticeEvent;
pub use self::request::RequestEvent;

/// OneBot 11 顶层事件枚举，按 `post_type` 分发。
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "post_type")]
pub enum Event {
    #[serde(rename = "message")]
    Message(MessageEvent),
    #[serde(rename = "notice")]
    Notice(NoticeEvent),
    #[serde(rename = "request")]
    Request(RequestEvent),
    #[serde(rename = "meta_event")]
    MetaEvent(MetaEvent),
}
