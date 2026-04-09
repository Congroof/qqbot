use serde::Deserialize;

/// 通知事件，按 `notice_type` 分发。
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "notice_type")]
pub enum NoticeEvent {
    #[serde(rename = "group_upload")]
    GroupUpload(GroupUploadEvent),
    #[serde(rename = "group_admin")]
    GroupAdmin(GroupAdminEvent),
    #[serde(rename = "group_decrease")]
    GroupDecrease(GroupDecreaseEvent),
    #[serde(rename = "group_increase")]
    GroupIncrease(GroupIncreaseEvent),
    #[serde(rename = "group_ban")]
    GroupBan(GroupBanEvent),
    #[serde(rename = "friend_add")]
    FriendAdd(FriendAddEvent),
    #[serde(rename = "group_recall")]
    GroupRecall(GroupRecallEvent),
    #[serde(rename = "friend_recall")]
    FriendRecall(FriendRecallEvent),
    #[serde(rename = "notify")]
    Notify(NotifyEvent),
}

// ---- 群文件上传 ----

#[derive(Debug, Clone, Deserialize)]
pub struct GroupUploadEvent {
    pub time: i64,
    pub self_id: i64,
    pub group_id: i64,
    pub user_id: i64,
    pub file: UploadFile,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UploadFile {
    pub id: String,
    pub name: String,
    pub size: i64,
    pub busid: i64,
}

// ---- 群管理员变动 ----

#[derive(Debug, Clone, Deserialize)]
pub struct GroupAdminEvent {
    pub time: i64,
    pub self_id: i64,
    pub sub_type: String,
    pub group_id: i64,
    pub user_id: i64,
}

// ---- 群成员减少 ----

#[derive(Debug, Clone, Deserialize)]
pub struct GroupDecreaseEvent {
    pub time: i64,
    pub self_id: i64,
    pub sub_type: String,
    pub group_id: i64,
    pub operator_id: i64,
    pub user_id: i64,
}

// ---- 群成员增加 ----

#[derive(Debug, Clone, Deserialize)]
pub struct GroupIncreaseEvent {
    pub time: i64,
    pub self_id: i64,
    pub sub_type: String,
    pub group_id: i64,
    pub operator_id: i64,
    pub user_id: i64,
}

// ---- 群禁言 ----

#[derive(Debug, Clone, Deserialize)]
pub struct GroupBanEvent {
    pub time: i64,
    pub self_id: i64,
    pub sub_type: String,
    pub group_id: i64,
    pub operator_id: i64,
    pub user_id: i64,
    pub duration: i64,
}

// ---- 好友添加 ----

#[derive(Debug, Clone, Deserialize)]
pub struct FriendAddEvent {
    pub time: i64,
    pub self_id: i64,
    pub user_id: i64,
}

// ---- 群消息撤回 ----

#[derive(Debug, Clone, Deserialize)]
pub struct GroupRecallEvent {
    pub time: i64,
    pub self_id: i64,
    pub group_id: i64,
    pub user_id: i64,
    pub operator_id: i64,
    pub message_id: i64,
}

// ---- 好友消息撤回 ----

#[derive(Debug, Clone, Deserialize)]
pub struct FriendRecallEvent {
    pub time: i64,
    pub self_id: i64,
    pub user_id: i64,
    pub message_id: i64,
}

// ---- notify 类 (poke / lucky_king / honor) ----

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "sub_type")]
pub enum NotifyEvent {
    #[serde(rename = "poke")]
    Poke(PokeEvent),
    #[serde(rename = "lucky_king")]
    LuckyKing(LuckyKingEvent),
    #[serde(rename = "honor")]
    Honor(HonorEvent),
    #[serde(rename = "profile_like")]
    ProfileLike(ProfileLikeEvent),
}

/// 群内戳一戳
#[derive(Debug, Clone, Deserialize)]
pub struct PokeEvent {
    pub time: i64,
    pub self_id: i64,
    pub group_id: i64,
    pub user_id: i64,
    pub target_id: i64,
}

/// 群红包运气王
#[derive(Debug, Clone, Deserialize)]
pub struct LuckyKingEvent {
    pub time: i64,
    pub self_id: i64,
    pub group_id: i64,
    pub user_id: i64,
    pub target_id: i64,
}

/// 群成员荣誉变更
#[derive(Debug, Clone, Deserialize)]
pub struct HonorEvent {
    pub time: i64,
    pub self_id: i64,
    pub group_id: i64,
    pub honor_type: String,
    pub user_id: i64,
}

/// 资料卡点赞通知（NapCat 扩展）
#[derive(Debug, Clone, Deserialize)]
pub struct ProfileLikeEvent {
    pub time: i64,
    pub self_id: i64,
    pub operator_id: i64,
    #[serde(default)]
    pub operator_nick: Option<String>,
    #[serde(default)]
    pub times: Option<i32>,
}
