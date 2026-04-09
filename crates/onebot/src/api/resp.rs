use serde::Deserialize;

use crate::message::Message;

/// 空响应 (无 data 字段或 data 为 null)
#[derive(Debug, Deserialize)]
pub struct EmptyResp;

/// send_private_msg / send_group_msg / send_msg 响应
#[derive(Debug, Deserialize)]
pub struct SendMsgResp {
    pub message_id: i32,
}

/// get_msg 响应
#[derive(Debug, Deserialize)]
pub struct GetMsgResp {
    pub time: i32,
    pub message_type: String,
    pub message_id: i32,
    pub real_id: i32,
    pub sender: serde_json::Value,
    pub message: Message,
}

/// get_forward_msg 响应
#[derive(Debug, Deserialize)]
pub struct GetForwardMsgResp {
    pub message: Message,
}

/// get_login_info 响应
#[derive(Debug, Deserialize)]
pub struct LoginInfoResp {
    pub user_id: i64,
    pub nickname: String,
}

/// get_stranger_info 响应
#[derive(Debug, Deserialize)]
pub struct StrangerInfoResp {
    pub user_id: i64,
    pub nickname: String,
    pub sex: String,
    pub age: i32,
}

/// get_friend_list 数组元素
#[derive(Debug, Deserialize)]
pub struct FriendInfo {
    pub user_id: i64,
    pub nickname: String,
    pub remark: String,
}

/// get_group_info / get_group_list 响应
#[derive(Debug, Deserialize)]
pub struct GroupInfoResp {
    pub group_id: i64,
    pub group_name: String,
    pub member_count: i32,
    pub max_member_count: i32,
}

/// get_group_member_info / get_group_member_list 响应
#[derive(Debug, Deserialize)]
pub struct GroupMemberInfoResp {
    pub group_id: i64,
    pub user_id: i64,
    pub nickname: String,
    #[serde(default)]
    pub card: Option<String>,
    #[serde(default)]
    pub sex: Option<String>,
    #[serde(default)]
    pub age: Option<i32>,
    #[serde(default)]
    pub area: Option<String>,
    #[serde(default)]
    pub join_time: Option<i32>,
    #[serde(default)]
    pub last_sent_time: Option<i32>,
    #[serde(default)]
    pub level: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub unfriendly: Option<bool>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub title_expire_time: Option<i32>,
    #[serde(default)]
    pub card_changeable: Option<bool>,
}

/// get_group_honor_info 响应
#[derive(Debug, Deserialize)]
pub struct GroupHonorInfoResp {
    pub group_id: i64,
    #[serde(default)]
    pub current_talkative: Option<CurrentTalkative>,
    #[serde(default)]
    pub talkative_list: Option<Vec<HonorListItem>>,
    #[serde(default)]
    pub performer_list: Option<Vec<HonorListItem>>,
    #[serde(default)]
    pub legend_list: Option<Vec<HonorListItem>>,
    #[serde(default)]
    pub strong_newbie_list: Option<Vec<HonorListItem>>,
    #[serde(default)]
    pub emotion_list: Option<Vec<HonorListItem>>,
}

#[derive(Debug, Deserialize)]
pub struct CurrentTalkative {
    pub user_id: i64,
    pub nickname: String,
    pub avatar: String,
    pub day_count: i32,
}

#[derive(Debug, Deserialize)]
pub struct HonorListItem {
    pub user_id: i64,
    pub nickname: String,
    pub avatar: String,
    #[serde(default)]
    pub description: Option<String>,
}

/// get_cookies 响应
#[derive(Debug, Deserialize)]
pub struct CookiesResp {
    pub cookies: String,
}

/// get_csrf_token 响应
#[derive(Debug, Deserialize)]
pub struct CsrfTokenResp {
    pub token: i32,
}

/// get_credentials 响应
#[derive(Debug, Deserialize)]
pub struct CredentialsResp {
    pub cookies: String,
    pub csrf_token: i32,
}

/// get_record / get_image 响应
#[derive(Debug, Deserialize)]
pub struct FileResp {
    pub file: String,
}

/// can_send_image / can_send_record 响应
#[derive(Debug, Deserialize)]
pub struct BoolResp {
    pub yes: bool,
}

/// get_status 响应
#[derive(Debug, Deserialize)]
pub struct StatusResp {
    #[serde(default)]
    pub online: Option<bool>,
    pub good: bool,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// get_version_info 响应
#[derive(Debug, Deserialize)]
pub struct VersionInfoResp {
    pub app_name: String,
    pub app_version: String,
    pub protocol_version: String,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}
