use std::env;

pub struct BotConfig {
    pub ws_url: String,
    pub access_token: Option<String>,
    pub ai_base_url: String,
    pub ai_api_key: String,
    pub ai_gateway_uid: String,
    pub ai_product_name: String,
    pub ai_intention_code: String,
    pub ai_model: String,
    pub data_dir: String,
    /// 每日定时点赞任务是否启用。
    pub schedule_like_enabled: bool,
    /// 本地时区相对 UTC 的小时偏移（默认 +8，北京时间）。
    pub schedule_tz_offset_hours: i64,
}

impl BotConfig {
    pub fn from_env() -> Self {
        Self {
            ws_url: required_env("ONEBOT_WS_URL"),
            access_token: optional_env("ONEBOT_ACCESS_TOKEN"),
            ai_base_url: required_env("AI_BASE_URL"),
            ai_api_key: required_env("AI_API_KEY"),
            ai_gateway_uid: required_env("AI_GATEWAY_UID"),
            ai_product_name: required_env("AI_PRODUCT_NAME"),
            ai_intention_code: required_env("AI_INTENTION_CODE"),
            ai_model: env::var("AI_MODEL").unwrap_or_else(|_| "ali/qwen-plus".into()),
            data_dir: env::var("DATA_DIR").unwrap_or_else(|_| "./data".into()),
            schedule_like_enabled: env::var("SCHEDULE_LIKE_ENABLED")
                .ok()
                .map(|v| !matches!(v.trim().to_ascii_lowercase().as_str(), "0" | "false" | "no" | "off"))
                .unwrap_or(true),
            schedule_tz_offset_hours: env::var("SCHEDULE_TZ_OFFSET_HOURS")
                .ok()
                .and_then(|v| v.trim().parse().ok())
                .unwrap_or(8),
        }
    }
}

fn required_env(key: &str) -> String {
    env::var(key)
        .map(|v| v.trim().to_string())
        .unwrap_or_else(|_| panic!("environment variable {key} is required"))
}

fn optional_env(key: &str) -> Option<String> {
    env::var(key)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}
