use std::time::{Duration, SystemTime, UNIX_EPOCH};

use onebot::api::payload::{GetGroupList, GetGroupMemberList, GetLoginInfo, SendLike};
use onebot::ApiCaller;
use tokio::time::sleep;

const TARGET_HOUR: i64 = 23;
const TARGET_MINUTE: i64 = 30;
const LIKE_TIMES_PER_MEMBER: i32 = 50;
const DELAY_BETWEEN_LIKES_MS: u64 = 200;
const DELAY_BETWEEN_GROUPS_MS: u64 = 2000;

/// 启动每日定时点赞任务（每天 23:30 触发一次）。
///
/// 任务会：
/// 1. 拉取机器人加入的所有群（get_group_list）。
/// 2. 对每个群拉取成员列表（get_group_member_list）。
/// 3. 对每位成员（不含自己）调用 send_like 送 10 个赞。
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
    let self_id = api
        .call(GetLoginInfo)
        .await
        .map(|info| info.user_id)
        .map_err(|e| format!("get_login_info: {e}"))?;

    let groups = api
        .call(GetGroupList)
        .await
        .map_err(|e| format!("get_group_list: {e}"))?;

    tracing::info!(groups = groups.len(), self_id, "starting daily like round");

    let mut total_members = 0usize;
    let mut total_success = 0usize;
    let mut total_failed = 0usize;

    for group in groups {
        let group_id = group.group_id;
        let members = match api.call(GetGroupMemberList { group_id }).await {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!(group_id, error = %e, "get_group_member_list failed");
                continue;
            }
        };

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

        for m in targets {
            total_members += 1;
            match api
                .call_raw(SendLike {
                    user_id: m.user_id,
                    times: Some(LIKE_TIMES_PER_MEMBER),
                })
                .await
            {
                Ok(resp) if resp.retcode == 0 => {
                    total_success += 1;
                }
                Ok(resp) => {
                    total_failed += 1;
                    tracing::debug!(
                        user_id = m.user_id,
                        retcode = resp.retcode,
                        msg = ?resp.message,
                        "like returned non-zero"
                    );
                }
                Err(e) => {
                    total_failed += 1;
                    tracing::debug!(user_id = m.user_id, error = %e, "like failed");
                }
            }
            sleep(Duration::from_millis(DELAY_BETWEEN_LIKES_MS)).await;
        }

        sleep(Duration::from_millis(DELAY_BETWEEN_GROUPS_MS)).await;
    }

    tracing::info!(
        total = total_members,
        success = total_success,
        failed = total_failed,
        "daily like round finished"
    );

    Ok(())
}

/// 判断群成员角色是否为群主或管理员。
fn is_admin_or_owner(role: Option<&str>) -> bool {
    matches!(role, Some("owner") | Some("admin"))
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
