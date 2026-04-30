use serde::Serialize;

use super::resp;
use super::ApiAction;
use crate::message::{Message, MessageSegment};

// ============================================================
// 消息相关
// ============================================================

#[derive(Debug, Serialize)]
pub struct SendPrivateMsg {
    pub user_id: i64,
    pub message: Message,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_escape: Option<bool>,
}
impl ApiAction for SendPrivateMsg {
    const ACTION: &'static str = "send_private_msg";
    type Response = resp::SendMsgResp;
}

#[derive(Debug, Serialize)]
pub struct SendGroupMsg {
    pub group_id: i64,
    pub message: Message,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_escape: Option<bool>,
}
impl ApiAction for SendGroupMsg {
    const ACTION: &'static str = "send_group_msg";
    type Response = resp::SendMsgResp;
}

#[derive(Debug, Serialize)]
pub struct SendMsg {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<i64>,
    pub message: Message,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_escape: Option<bool>,
}
impl ApiAction for SendMsg {
    const ACTION: &'static str = "send_msg";
    type Response = resp::SendMsgResp;
}

#[derive(Debug, Serialize)]
pub struct DeleteMsg {
    pub message_id: i32,
}
impl ApiAction for DeleteMsg {
    const ACTION: &'static str = "delete_msg";
    type Response = resp::EmptyResp;
}

#[derive(Debug, Serialize)]
pub struct GetMsg {
    pub message_id: i32,
}
impl ApiAction for GetMsg {
    const ACTION: &'static str = "get_msg";
    type Response = resp::GetMsgResp;
}

#[derive(Debug, Serialize)]
pub struct GetForwardMsg {
    pub id: String,
}
impl ApiAction for GetForwardMsg {
    const ACTION: &'static str = "get_forward_msg";
    type Response = resp::GetForwardMsgResp;
}

/// 发送群合并转发消息（NapCat / go-cqhttp 扩展）。
///
/// `messages` 里每个元素必须是 `MessageSegment::Node`。
#[derive(Debug, Serialize)]
pub struct SendGroupForwardMsg {
    pub group_id: i64,
    pub messages: Vec<MessageSegment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
}
impl ApiAction for SendGroupForwardMsg {
    const ACTION: &'static str = "send_group_forward_msg";
    type Response = resp::SendMsgResp;
}

/// 发送私聊合并转发消息（NapCat / go-cqhttp 扩展）。
///
/// `messages` 里每个元素必须是 `MessageSegment::Node`。
#[derive(Debug, Serialize)]
pub struct SendPrivateForwardMsg {
    pub user_id: i64,
    pub messages: Vec<MessageSegment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
}
impl ApiAction for SendPrivateForwardMsg {
    const ACTION: &'static str = "send_private_forward_msg";
    type Response = resp::SendMsgResp;
}

// ============================================================
// 好友操作
// ============================================================

#[derive(Debug, Serialize)]
pub struct SendLike {
    pub user_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub times: Option<i32>,
}
impl ApiAction for SendLike {
    const ACTION: &'static str = "send_like";
    type Response = resp::EmptyResp;
}

// ============================================================
// 群组操作
// ============================================================

#[derive(Debug, Serialize)]
pub struct SetGroupKick {
    pub group_id: i64,
    pub user_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reject_add_request: Option<bool>,
}
impl ApiAction for SetGroupKick {
    const ACTION: &'static str = "set_group_kick";
    type Response = resp::EmptyResp;
}

#[derive(Debug, Serialize)]
pub struct SetGroupBan {
    pub group_id: i64,
    pub user_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<i64>,
}
impl ApiAction for SetGroupBan {
    const ACTION: &'static str = "set_group_ban";
    type Response = resp::EmptyResp;
}

#[derive(Debug, Serialize)]
pub struct SetGroupAnonymousBan {
    pub group_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anonymous: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anonymous_flag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<i64>,
}
impl ApiAction for SetGroupAnonymousBan {
    const ACTION: &'static str = "set_group_anonymous_ban";
    type Response = resp::EmptyResp;
}

#[derive(Debug, Serialize)]
pub struct SetGroupWholeBan {
    pub group_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable: Option<bool>,
}
impl ApiAction for SetGroupWholeBan {
    const ACTION: &'static str = "set_group_whole_ban";
    type Response = resp::EmptyResp;
}

#[derive(Debug, Serialize)]
pub struct SetGroupAdmin {
    pub group_id: i64,
    pub user_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable: Option<bool>,
}
impl ApiAction for SetGroupAdmin {
    const ACTION: &'static str = "set_group_admin";
    type Response = resp::EmptyResp;
}

#[derive(Debug, Serialize)]
pub struct SetGroupAnonymous {
    pub group_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable: Option<bool>,
}
impl ApiAction for SetGroupAnonymous {
    const ACTION: &'static str = "set_group_anonymous";
    type Response = resp::EmptyResp;
}

#[derive(Debug, Serialize)]
pub struct SetGroupCard {
    pub group_id: i64,
    pub user_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub card: Option<String>,
}
impl ApiAction for SetGroupCard {
    const ACTION: &'static str = "set_group_card";
    type Response = resp::EmptyResp;
}

#[derive(Debug, Serialize)]
pub struct SetGroupName {
    pub group_id: i64,
    pub group_name: String,
}
impl ApiAction for SetGroupName {
    const ACTION: &'static str = "set_group_name";
    type Response = resp::EmptyResp;
}

#[derive(Debug, Serialize)]
pub struct SetGroupLeave {
    pub group_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_dismiss: Option<bool>,
}
impl ApiAction for SetGroupLeave {
    const ACTION: &'static str = "set_group_leave";
    type Response = resp::EmptyResp;
}

#[derive(Debug, Serialize)]
pub struct SetGroupSpecialTitle {
    pub group_id: i64,
    pub user_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub special_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<i64>,
}
impl ApiAction for SetGroupSpecialTitle {
    const ACTION: &'static str = "set_group_special_title";
    type Response = resp::EmptyResp;
}

// ============================================================
// 请求处理
// ============================================================

#[derive(Debug, Serialize)]
pub struct SetFriendAddRequest {
    pub flag: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approve: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remark: Option<String>,
}
impl ApiAction for SetFriendAddRequest {
    const ACTION: &'static str = "set_friend_add_request";
    type Response = resp::EmptyResp;
}

#[derive(Debug, Serialize)]
pub struct SetGroupAddRequest {
    pub flag: String,
    pub sub_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approve: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}
impl ApiAction for SetGroupAddRequest {
    const ACTION: &'static str = "set_group_add_request";
    type Response = resp::EmptyResp;
}

// ============================================================
// 信息查询
// ============================================================

#[derive(Debug, Serialize)]
pub struct GetLoginInfo;
impl ApiAction for GetLoginInfo {
    const ACTION: &'static str = "get_login_info";
    type Response = resp::LoginInfoResp;
}

#[derive(Debug, Serialize)]
pub struct GetStrangerInfo {
    pub user_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_cache: Option<bool>,
}
impl ApiAction for GetStrangerInfo {
    const ACTION: &'static str = "get_stranger_info";
    type Response = resp::StrangerInfoResp;
}

#[derive(Debug, Serialize)]
pub struct GetFriendList;
impl ApiAction for GetFriendList {
    const ACTION: &'static str = "get_friend_list";
    type Response = Vec<resp::FriendInfo>;
}

#[derive(Debug, Serialize)]
pub struct GetGroupInfo {
    pub group_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_cache: Option<bool>,
}
impl ApiAction for GetGroupInfo {
    const ACTION: &'static str = "get_group_info";
    type Response = resp::GroupInfoResp;
}

#[derive(Debug, Serialize)]
pub struct GetGroupList;
impl ApiAction for GetGroupList {
    const ACTION: &'static str = "get_group_list";
    type Response = Vec<resp::GroupInfoResp>;
}

#[derive(Debug, Serialize)]
pub struct GetGroupMemberInfo {
    pub group_id: i64,
    pub user_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_cache: Option<bool>,
}
impl ApiAction for GetGroupMemberInfo {
    const ACTION: &'static str = "get_group_member_info";
    type Response = resp::GroupMemberInfoResp;
}

#[derive(Debug, Serialize)]
pub struct GetGroupMemberList {
    pub group_id: i64,
}
impl ApiAction for GetGroupMemberList {
    const ACTION: &'static str = "get_group_member_list";
    type Response = Vec<resp::GroupMemberInfoResp>;
}

#[derive(Debug, Serialize)]
pub struct GetGroupHonorInfo {
    pub group_id: i64,
    pub r#type: String,
}
impl ApiAction for GetGroupHonorInfo {
    const ACTION: &'static str = "get_group_honor_info";
    type Response = resp::GroupHonorInfoResp;
}

// ============================================================
// 凭证相关
// ============================================================

#[derive(Debug, Serialize)]
pub struct GetCookies {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
}
impl ApiAction for GetCookies {
    const ACTION: &'static str = "get_cookies";
    type Response = resp::CookiesResp;
}

#[derive(Debug, Serialize)]
pub struct GetCsrfToken;
impl ApiAction for GetCsrfToken {
    const ACTION: &'static str = "get_csrf_token";
    type Response = resp::CsrfTokenResp;
}

#[derive(Debug, Serialize)]
pub struct GetCredentials {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
}
impl ApiAction for GetCredentials {
    const ACTION: &'static str = "get_credentials";
    type Response = resp::CredentialsResp;
}

// ============================================================
// 文件相关
// ============================================================

#[derive(Debug, Serialize)]
pub struct GetRecord {
    pub file: String,
    pub out_format: String,
}
impl ApiAction for GetRecord {
    const ACTION: &'static str = "get_record";
    type Response = resp::FileResp;
}

#[derive(Debug, Serialize)]
pub struct GetImage {
    pub file: String,
}
impl ApiAction for GetImage {
    const ACTION: &'static str = "get_image";
    type Response = resp::FileResp;
}

// ============================================================
// 能力查询
// ============================================================

#[derive(Debug, Serialize)]
pub struct CanSendImage;
impl ApiAction for CanSendImage {
    const ACTION: &'static str = "can_send_image";
    type Response = resp::BoolResp;
}

#[derive(Debug, Serialize)]
pub struct CanSendRecord;
impl ApiAction for CanSendRecord {
    const ACTION: &'static str = "can_send_record";
    type Response = resp::BoolResp;
}

// ============================================================
// 运行时
// ============================================================

#[derive(Debug, Serialize)]
pub struct GetStatus;
impl ApiAction for GetStatus {
    const ACTION: &'static str = "get_status";
    type Response = resp::StatusResp;
}

#[derive(Debug, Serialize)]
pub struct GetVersionInfo;
impl ApiAction for GetVersionInfo {
    const ACTION: &'static str = "get_version_info";
    type Response = resp::VersionInfoResp;
}

#[derive(Debug, Serialize)]
pub struct SetRestart {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delay: Option<i64>,
}
impl ApiAction for SetRestart {
    const ACTION: &'static str = "set_restart";
    type Response = resp::EmptyResp;
}

#[derive(Debug, Serialize)]
pub struct CleanCache;
impl ApiAction for CleanCache {
    const ACTION: &'static str = "clean_cache";
    type Response = resp::EmptyResp;
}

// ============================================================
// 隐藏 API
// ============================================================

#[derive(Debug, Serialize)]
pub struct HandleQuickOperation {
    pub context: serde_json::Value,
    pub operation: serde_json::Value,
}
impl ApiAction for HandleQuickOperation {
    const ACTION: &'static str = ".handle_quick_operation";
    type Response = resp::EmptyResp;
}
