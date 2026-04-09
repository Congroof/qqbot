use serde::Deserialize;

/// 元事件，按 `meta_event_type` 分发。
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "meta_event_type")]
pub enum MetaEvent {
    #[serde(rename = "lifecycle")]
    Lifecycle(LifecycleEvent),
    #[serde(rename = "heartbeat")]
    Heartbeat(HeartbeatEvent),
}

/// 生命周期事件
#[derive(Debug, Clone, Deserialize)]
pub struct LifecycleEvent {
    pub time: i64,
    pub self_id: i64,
    pub sub_type: String,
}

/// 心跳事件
#[derive(Debug, Clone, Deserialize)]
pub struct HeartbeatEvent {
    pub time: i64,
    pub self_id: i64,
    pub status: serde_json::Value,
    pub interval: i64,
}
