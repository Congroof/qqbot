# napcat-bot

基于 Rust 实现的 QQ 机器人，通过 [NapCat](https://github.com/NapNeko/NapCatQQ) 的 OneBot 11 正向 WebSocket 接口与 QQ 交互，集成 AI 大模型实现智能聊天。

## 项目结构

```
crates/
├── napcat-bot/       # 机器人主程序（功能实现）
├── onebot/           # OneBot 11 协议库（事件、消息、API、WebSocket 客户端）
└── ai-chat-sdk/      # AI 聊天 SDK（对接 WPS AI Gateway）
```

## 功能列表

### AI 多轮聊天

- **私聊**：直接发消息即可与 AI 对话
- **群聊**：@机器人 触发 AI 回复，以引用消息的形式回复
- 每个用户独立维护对话上下文，最多保留最近 20 条历史
- 会话 5 分钟无活动自动过期

### 入群欢迎

新成员加入群聊时，自动发送欢迎消息并 @新成员。

### 复读机

群内连续出现 2 条相同消息时，机器人跟着复读一次。内置冷却机制，同一轮复读只触发一次，避免无限循环。

### 戳一戳回应

在群内戳机器人，会随机回复一条趣味文案，如"别戳了，再戳我要报警了！"、"痒！别闹～"等。

### 自动审批

- 自动同意加好友请求
- 自动同意群邀请（仅限 `invite` 类型）

## 快速开始

### 1. 启动 NapCat

```bash
make start
```

通过 Docker 启动 NapCat 容器（端口 3000 提供 OneBot 11 WebSocket 服务），首次启动需扫码登录 QQ。

### 2. 配置环境变量

所有配置通过环境变量读取（已在 `makefile` 中预设）：

| 变量名 | 必填 | 默认值 | 说明 |
|--------|:----:|--------|------|
| `ONEBOT_WS_URL` | 是 | - | OneBot WebSocket 地址 |
| `ONEBOT_ACCESS_TOKEN` | 否 | 空 | OneBot access token |
| `AI_BASE_URL` | 是 | - | AI Gateway 地址 |
| `AI_API_KEY` | 是 | - | AI API 密钥 |
| `AI_GATEWAY_UID` | 是 | - | AI Gateway UID |
| `AI_PRODUCT_NAME` | 是 | - | AI 产品名称 |
| `AI_INTENTION_CODE` | 是 | - | AI 意图码 |
| `AI_MODEL` | 否 | `ali/qwen-plus` | AI 模型名称 |

### 3. 运行机器人

```bash
make dev
```

## 开发

### 前置依赖

- Rust (edition 2024)
- Docker & Docker Compose（用于运行 NapCat）

### 常用命令

```bash
make dev        # 运行机器人
make start      # 启动 NapCat Docker 容器
make stop       # 停止容器
make restart    # 重启容器
make logs       # 查看容器日志
make clean      # 停止并删除容器数据
```

### 日志级别

通过 `RUST_LOG` 环境变量控制，默认 `info`：

```bash
RUST_LOG=debug make dev
```

## 技术栈

| 组件 | 技术 |
|------|------|
| 语言 | Rust (edition 2024) |
| 异步运行时 | tokio |
| QQ 协议 | OneBot 11（正向 WebSocket） |
| QQ 客户端 | NapCat (Docker) |
| AI | WPS AI Gateway (qwen-plus) |
| WebSocket | tokio-tungstenite |
| 日志 | tracing + tracing-subscriber |

## OneBot 协议库

`crates/onebot` 是一个独立的 OneBot 11 协议实现库，可单独使用：

```rust
use onebot::{WsClient, WsConfig, Event, MessageSegment};
use onebot::api::payload::SendGroupMsg;
use onebot::event::MessageEvent;

let mut client = WsClient::connect(
    WsConfig::new("ws://127.0.0.1:3000").with_token("token")
).await?;

let api = client.api().clone();

while let Some(event) = client.next_event().await {
    if let Event::Message(MessageEvent::Group(msg)) = event {
        api.call(SendGroupMsg {
            group_id: msg.group_id,
            message: vec![MessageSegment::text("收到")].into(),
            auto_escape: None,
        }).await?;
    }
}
```

主要特性：

- 完整的 OneBot 11 事件类型（消息、通知、请求、元事件）
- 22 种消息段类型
- 35 个 API 的类型安全调用（`ApiAction` trait）
- 基于 echo 匹配的异步请求-响应机制
- 可克隆的 `ApiCaller`，支持多任务并发调用
