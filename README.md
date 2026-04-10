# napcat-bot

基于 Rust 实现的 QQ 机器人，通过 [NapCat](https://github.com/NapNeko/NapCatQQ) 的 OneBot 11 正向 WebSocket 接口与 QQ 交互，集成 AI 大模型实现智能聊天及丰富的群聊功能。

## 项目结构

```
crates/
├── napcat-bot/       # 机器人主程序（功能实现）
├── onebot/           # OneBot 11 协议库（事件、消息、API、WebSocket 客户端）
└── ai-chat-sdk/      # AI 聊天 SDK（对接 WPS AI Gateway）
```

## 功能列表

### AI 智能聊天

- **私聊**：直接发消息即可与 AI 对话
- **群聊**：@机器人触发 AI 回复，以引用消息的形式回复
- 每个用户独立维护对话上下文，最多保留最近 20 条历史
- 会话 5 分钟无活动自动过期
- 回复长度随机化，更贴近真人聊天风格
- 内置重试机制（3 次），应对网络抖动

### AI 角色切换（群管理员）

群内发送 `#角色 猫娘` 切换全群 AI 人设，`#角色 默认` 恢复。内置角色：猫娘、毒舌、哲学家、老中医、诗人，也支持自定义角色名。按群隔离，仅管理员可操作。

### 今日运势

群内发送 `运势` / `求签` / `今日运势`，AI 生成个性化运势，每人每天缓存一次结果。

### 随机名人名言

群内发送 `语录` / `名言` / `名人名言`，AI 随机生成一条名人名言，涵盖哲学、文学、科学、励志等领域。

### 成语接龙

群内发送 `成语接龙` 开始游戏，AI 出第一个成语，群友接龙，AI 当裁判并给出下一个。发送 `结束接龙` 或 5 分钟无人参与自动结束，展示排行榜。

### 群消息统计

自动统计每日群内发言数量，群内发送 `水群排行` / `发言排行` 查看今日 Top 10 水群达人。

### AI 消息摘要

群内发送 `总结` / `消息摘要`，AI 自动总结最近群聊内容要点。

### 撤回监控（群管理员）

群内发送 `#撤回监控 开启` 开启本群撤回消息曝光功能，有人撤回消息时 bot 会将原内容发出来。默认关闭，按群隔离，管理员可通过 `#撤回监控 关闭` / `#撤回监控 状态` 管理。

### 入群验证

新成员入群后自动发出算术验证题，60 秒内回答正确即通过，超时或答错自动踢出。

### 点赞

群内发送 `赞我`，bot 为你点赞，最多 50 个（分 5 次，每次 10 个）。

### 复读机

群内连续出现 2 条相同消息时，机器人跟着复读一次。内置冷却机制避免无限循环。

### 戳一戳回应

在群内戳机器人，随机回复趣味文案。

### 自动审批

- 自动同意加好友请求
- 自动同意群邀请（仅限 `invite` 类型）

### 私聊管理指令

| 指令 | 说明 |
|------|------|
| `#cmd stats` | 查看运行状态和 AI Token 用量 |
| `#cmd help` | 显示帮助菜单 |

### Token 用量统计

自动统计 AI 调用次数、prompt/completion/总 token 数，数据持久化到磁盘，Docker 重启不丢失。通过 `#cmd stats` 私聊查看。

## 部署

### Docker 部署（推荐）

1. 复制 `.env.example` 为 `.env` 并填写配置：

```bash
cp .env.example .env
```

2. 构建并启动：

```bash
make build   # 构建镜像
make up      # 启动服务
```

首次启动 NapCat 需通过 `http://localhost:6099` 扫码登录 QQ。

3. 查看日志：

```bash
make logs      # 查看所有服务日志
make logs-bot  # 仅查看 bot 日志
```

### 本地开发

1. 先通过 Docker 启动 NapCat：

```bash
make up
```

2. 配置环境变量并运行：

```bash
make dev
```

环境变量已在 `makefile` 中预设，也可通过 `.env` 文件覆盖。

## 配置项

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
| `DATA_DIR` | 否 | `./data` | 数据持久化目录 |
| `RUST_LOG` | 否 | `info` | 日志级别 |

## 常用命令

```bash
make dev        # 本地运行机器人
make build      # 构建 Docker 镜像
make up         # 启动所有服务
make down       # 停止所有服务
make restart    # 重启所有服务
make logs       # 查看所有日志
make logs-bot   # 查看 bot 日志
make clean      # 停止并删除容器数据
```

## 技术栈

| 组件 | 技术 |
|------|------|
| 语言 | Rust (edition 2024) |
| 异步运行时 | tokio |
| QQ 协议 | OneBot 11（正向 WebSocket） |
| QQ 客户端 | NapCat (Docker) |
| AI | WPS AI Gateway |
| WebSocket | tokio-tungstenite |
| 日志 | tracing + tracing-subscriber |
| 容器化 | Docker + Docker Compose |

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
