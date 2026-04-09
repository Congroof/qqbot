use serde::Deserialize;

/// 请求事件，按 `request_type` 分发。
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "request_type")]
pub enum RequestEvent {
    #[serde(rename = "friend")]
    Friend(FriendRequestEvent),
    #[serde(rename = "group")]
    Group(GroupRequestEvent),
}

/// 加好友请求
#[derive(Debug, Clone, Deserialize)]
pub struct FriendRequestEvent {
    pub time: i64,
    pub self_id: i64,
    pub user_id: i64,
    pub comment: String,
    pub flag: String,
}

/// 加群请求/邀请
#[derive(Debug, Clone, Deserialize)]
pub struct GroupRequestEvent {
    pub time: i64,
    pub self_id: i64,
    pub sub_type: String,
    pub group_id: i64,
    pub user_id: i64,
    pub comment: String,
    pub flag: String,
}
