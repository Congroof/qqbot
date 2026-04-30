use std::time::{Duration, SystemTime, UNIX_EPOCH};

use onebot::api::payload::{
    GetGroupList, GetGroupMemberList, GetLoginInfo, SendLike, SendPrivateForwardMsg,
};
use onebot::message::MessageSegment;
use onebot::ApiCaller;
use onebot::Message;
use tokio::time::sleep;

const TARGET_HOUR: i64 = 23;
const TARGET_MINUTE: i64 = 30;
/// 单次 send_like 调用最多 10 个赞（OneBot/NapCat 限制）。
const BATCH_SIZE: i32 = 10;
/// 每个成员最多送 50 个赞（5 批 × 10）。
const MAX_BATCHES: i32 = 5;
const DELAY_BETWEEN_BATCHES_MS: u64 = 1000;
const DELAY_BETWEEN_MEMBERS_MS: u64 = 500;
const DELAY_BETWEEN_GROUPS_MS: u64 = 2000;

/// 启动每日定时点赞任务（每天 23:30 触发一次）。
///
/// 任务会：
/// 1. 拉取机器人加入的所有群（get_group_list）。
/// 2. 对每个群拉取成员列表（get_group_member_list）并筛选出群主/管理员。
/// 3. 对每位目标成员分批调用 send_like（每批 10 个，最多 5 批 = 50 个），
///    遇到非零 retcode 或错误即停止该成员的后续批次。
pub fn spawn_daily_like_task(api: ApiCaller, tz_offset_hours: i64) {
    tokio::spawn(async move {
        loop {
            let wait = duration_until_next_target(tz_offset_hours);
            tracing::info!(
                wait_secs = wait.as_secs(),
                target = format!("{TARGET_HOUR:02}:{TARGET_MINUTE:02}"),
                tz_offset_hours,
                "daily like task scheduled"
            );
            sleep(wait).await;

            if let Err(e) = run_daily_like(&api).await {
                tracing::error!(error = %e, "daily like task failed");
            }

            // 避免同一分钟内重复触发：先睡 70 秒再重算下一次时间。
            sleep(Duration::from_secs(70)).await;
        }
    });
}

async fn run_daily_like(api: &ApiCaller) -> Result<(), String> {
    let login = api
        .call(GetLoginInfo)
        .await
        .map_err(|e| format!("get_login_info: {e}"))?;
    let self_id = login.user_id;
    let self_nickname = if login.nickname.is_empty() {
        "清".to_string()
    } else {
        login.nickname
    };

    let groups = api
        .call(GetGroupList)
        .await
        .map_err(|e| format!("get_group_list: {e}"))?;

    tracing::info!(groups = groups.len(), self_id, "starting daily like round");

    let mut total_members = 0usize;
    let mut total_likes = 0i32;
    let mut members_full = 0usize;
    let mut members_partial = 0usize;
    let mut members_none = 0usize;

    for group in groups {
        let group_id = group.group_id;
        let members = match api.call(GetGroupMemberList { group_id }).await {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!(group_id, error = %e, "get_group_member_list failed");
                continue;
            }
        };

        let owner_id = members
            .iter()
            .find(|m| m.role.as_deref() == Some("owner"))
            .map(|m| m.user_id);

        let targets: Vec<_> = members
            .into_iter()
            .filter(|m| m.user_id != self_id && is_admin_or_owner(m.role.as_deref()))
            .collect();

        tracing::info!(
            group_id,
            group_name = %group.group_name,
            count = targets.len(),
            "liking group admins/owner"
        );

        let mut group_results: Vec<MemberLikeResult> = Vec::with_capacity(targets.len());
        for m in targets {
            total_members += 1;
            let display = m
                .card
                .as_ref()
                .filter(|c| !c.is_empty())
                .cloned()
                .unwrap_or_else(|| m.nickname.clone());
            let liked = like_one_member(api, m.user_id).await;
            total_likes += liked;
            match liked {
                0 => members_none += 1,
                n if n >= BATCH_SIZE * MAX_BATCHES => members_full += 1,
                _ => members_partial += 1,
            }
            group_results.push(MemberLikeResult {
                user_id: m.user_id,
                display,
                role: m.role.clone().unwrap_or_default(),
                liked,
            });
            sleep(Duration::from_millis(DELAY_BETWEEN_MEMBERS_MS)).await;
        }

        if let Some(owner_id) = owner_id {
            // if owner_id != self_id {
            //     notify_owner(
            //         api,
            //         self_id,
            //         &self_nickname,
            //         owner_id,
            //         &group.group_name,
            //         group_id,
            //         &group_results,
            //     )
            //     .await;
            // }
        } else {
            tracing::debug!(group_id, "no owner found, skip owner notification");
        }

        sleep(Duration::from_millis(DELAY_BETWEEN_GROUPS_MS)).await;
    }

    tracing::info!(
        total_members,
        members_full,
        members_partial,
        members_none,
        total_likes,
        "daily like round finished"
    );

    Ok(())
}

/// 给单个成员分批点赞，返回实际成功送出的赞数。
///
/// 每批 10 个，最多 5 批；任一批次非零 retcode 或错误即停止，返回已累计的成功数。
async fn like_one_member(api: &ApiCaller, user_id: i64) -> i32 {
    let mut sent = 0i32;
    for batch in 0..MAX_BATCHES {
        match api
            .call_raw(SendLike {
                user_id,
                times: Some(BATCH_SIZE),
            })
            .await
        {
            Ok(resp) if resp.retcode == 0 => {
                sent += BATCH_SIZE;
            }
            Ok(resp) => {
                tracing::debug!(
                    user_id,
                    batch,
                    sent,
                    retcode = resp.retcode,
                    msg = ?resp.message,
                    "like stopped: non-zero retcode"
                );
                break;
            }
            Err(e) => {
                tracing::debug!(user_id, batch, sent, error = %e, "like stopped: error");
                break;
            }
        }
        if batch + 1 < MAX_BATCHES {
            sleep(Duration::from_millis(DELAY_BETWEEN_BATCHES_MS)).await;
        }
    }
    sent
}

/// 判断群成员角色是否为群主或管理员。
fn is_admin_or_owner(role: Option<&str>) -> bool {
    matches!(role, Some("owner") | Some("admin"))
}

struct MemberLikeResult {
    user_id: i64,
    display: String,
    role: String,
    liked: i32,
}

/// 把本群的点赞结果以合并转发私聊的形式发送给群主。
///
/// 消息结构：
/// - 第 1 个 node：汇总（群名、目标人数、累计赞数）
/// - 后续 node：每个成员一条，写明昵称、角色与送出的赞数
async fn notify_owner(
    api: &ApiCaller,
    self_id: i64,
    self_nickname: &str,
    owner_id: i64,
    group_name: &str,
    group_id: i64,
    results: &[MemberLikeResult],
) {
    let max_possible = BATCH_SIZE * MAX_BATCHES;
    let total: i32 = results.iter().map(|r| r.liked).sum();

    let summary = format!(
        "【每日点赞报告】\n群：{group_name}（{group_id}）\n目标（群主/管理员）：{} 人\n累计送出：{total} 赞",
        results.len()
    );

    let mut nodes: Vec<MessageSegment> = Vec::with_capacity(results.len() + 2);
    nodes.push(make_node(self_id, self_nickname, summary));

    if results.is_empty() {
        nodes.push(make_node(
            self_id,
            self_nickname,
            "（本群没有可点赞的管理员）".to_string(),
        ));
    } else {
        for r in results {
            let role_cn = match r.role.as_str() {
                "owner" => "群主",
                "admin" => "管理员",
                other => other,
            };
            let status = if r.liked >= max_possible {
                "满赞".to_string()
            } else if r.liked == 0 {
                "未成功".to_string()
            } else {
                format!("部分 {}/{}", r.liked, max_possible)
            };
            let text = format!(
                "{}（{}）\n角色：{role_cn}\n送出：{} 赞（{status}）",
                r.display, r.user_id, r.liked
            );
            nodes.push(make_node(self_id, self_nickname, text));
        }
    }

    match api
        .call(SendPrivateForwardMsg {
            user_id: owner_id,
            messages: nodes,
            source: None,
            summary: None,
            prompt: None,
        })
        .await
    {
        Ok(_) => {
            tracing::info!(owner_id, group_id, "owner notification sent");
        }
        Err(e) => {
            tracing::warn!(owner_id, group_id, error = %e, "owner notification failed");
        }
    }
}

/// 构造一个合并转发节点。
fn make_node(user_id: i64, nickname: &str, text: String) -> MessageSegment {
    MessageSegment::Node {
        id: None,
        user_id: Some(user_id.to_string()),
        nickname: Some(nickname.to_string()),
        content: Some(Message::from(vec![MessageSegment::Text { text }])),
    }
}

/// 计算到下一次本地 TARGET_HOUR:TARGET_MINUTE 的剩余时长。
///
/// 由于标准库不提供时区支持，通过 `tz_offset_hours` 手动换算本地日界。
fn duration_until_next_target(tz_offset_hours: i64) -> Duration {
    let now_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let tz_offset_secs = tz_offset_hours * 3600;
    let local_now = now_unix + tz_offset_secs;
    let day_start = local_now.div_euclid(86400) * 86400;
    let target_today = day_start + TARGET_HOUR * 3600 + TARGET_MINUTE * 60;
    let target = if target_today > local_now {
        target_today
    } else {
        target_today + 86400
    };
    Duration::from_secs((target - local_now).max(0) as u64)
}
