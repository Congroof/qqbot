use serde::{Deserialize, Serialize};

/// 消息内容，支持字符串格式和数组格式两种表示。
/// 反序列化时自动兼容两种格式。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Message {
    Array(Vec<MessageSegment>),
    String(String),
}

impl Message {
    pub fn segments(&self) -> &[MessageSegment] {
        match self {
            Message::Array(segs) => segs,
            Message::String(_) => &[],
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Message::String(s) => Some(s),
            Message::Array(_) => None,
        }
    }
}

impl From<Vec<MessageSegment>> for Message {
    fn from(segs: Vec<MessageSegment>) -> Self {
        Message::Array(segs)
    }
}

impl From<String> for Message {
    fn from(s: String) -> Self {
        Message::String(s)
    }
}

/// OneBot 11 消息段，覆盖协议定义的所有类型。
/// 使用 `#[serde(tag = "type", content = "data")]` 实现 adjacently tagged 序列化。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum MessageSegment {
    #[serde(rename = "text")]
    Text {
        text: String,
    },

    #[serde(rename = "face")]
    Face {
        id: String,
    },

    #[serde(rename = "image")]
    Image {
        file: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        r#type: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        proxy: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        timeout: Option<String>,
    },

    #[serde(rename = "record")]
    Record {
        file: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        magic: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        proxy: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        timeout: Option<String>,
    },

    #[serde(rename = "video")]
    Video {
        file: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        proxy: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        timeout: Option<String>,
    },

    #[serde(rename = "at")]
    At {
        qq: String,
    },

    #[serde(rename = "rps")]
    Rps {},

    #[serde(rename = "dice")]
    Dice {},

    #[serde(rename = "shake")]
    Shake {},

    #[serde(rename = "poke")]
    Poke {
        r#type: String,
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },

    #[serde(rename = "anonymous")]
    Anonymous {
        #[serde(skip_serializing_if = "Option::is_none")]
        ignore: Option<String>,
    },

    #[serde(rename = "share")]
    Share {
        url: String,
        title: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        image: Option<String>,
    },

    #[serde(rename = "contact")]
    Contact {
        r#type: String,
        id: String,
    },

    #[serde(rename = "location")]
    Location {
        lat: String,
        lon: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<String>,
    },

    #[serde(rename = "music")]
    Music {
        r#type: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        audio: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        image: Option<String>,
    },

    #[serde(rename = "reply")]
    Reply {
        id: String,
    },

    #[serde(rename = "forward")]
    Forward {
        id: String,
    },

    #[serde(rename = "node")]
    Node {
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        user_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        nickname: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<Message>,
    },

    #[serde(rename = "xml")]
    Xml {
        data: String,
    },

    #[serde(rename = "json")]
    Json {
        data: String,
    },
}

impl MessageSegment {
    pub fn text(text: impl Into<String>) -> Self {
        MessageSegment::Text { text: text.into() }
    }

    pub fn image(file: impl Into<String>) -> Self {
        MessageSegment::Image {
            file: file.into(),
            r#type: None,
            url: None,
            cache: None,
            proxy: None,
            timeout: None,
        }
    }

    pub fn at(qq: impl Into<String>) -> Self {
        MessageSegment::At { qq: qq.into() }
    }

    pub fn at_all() -> Self {
        MessageSegment::At {
            qq: "all".to_string(),
        }
    }

    pub fn reply(id: impl Into<String>) -> Self {
        MessageSegment::Reply { id: id.into() }
    }

    pub fn face(id: impl Into<String>) -> Self {
        MessageSegment::Face { id: id.into() }
    }
}
