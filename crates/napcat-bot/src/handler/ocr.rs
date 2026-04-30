use std::time::{Duration, Instant};

use onebot::api::payload::SendPrivateMsg;
use onebot::event::message::PrivateMessageEvent;
use onebot::message::MessageSegment;
use onebot::Message;
use tokio::process::Command;

use super::HandlerContext;

const OCR_TIMEOUT: Duration = Duration::from_secs(60);

/// 检查用户是否处于"等待 OCR 图片"状态，如果是则提取图片并识别。
pub async fn handle_pending_ocr(ctx: &mut HandlerContext, evt: &PrivateMessageEvent) -> bool {
    let user_id = evt.user_id;

    let Some(&start_time) = ctx.pending_ocr.get(&user_id) else {
        return false;
    };

    if start_time.elapsed() > OCR_TIMEOUT {
        ctx.pending_ocr.remove(&user_id);
        let _ = ctx
            .api
            .call(SendPrivateMsg {
                user_id,
                message: Message::from(vec![MessageSegment::text(
                    "识别图片已超时（60秒），请重新发送 #cmd 识别图片",
                )]),
                auto_escape: None,
            })
            .await;
        return true;
    }

    let text = super::extract_plain_text(&evt.message);
    if text == "取消" || text == "cancel" {
        ctx.pending_ocr.remove(&user_id);
        let _ = ctx
            .api
            .call(SendPrivateMsg {
                user_id,
                message: Message::from(vec![MessageSegment::text("已退出图片识别模式")]),
                auto_escape: None,
            })
            .await;
        return true;
    }

    let image_url = extract_image_url(&evt.message);
    let Some(url) = image_url else {
        let _ = ctx
            .api
            .call(SendPrivateMsg {
                user_id,
                message: Message::from(vec![MessageSegment::text(
                    "请发送一张图片，或发送「取消」退出识别模式",
                )]),
                auto_escape: None,
            })
            .await;
        return true;
    };

    ctx.pending_ocr.remove(&user_id);

    let _ = ctx
        .api
        .call(SendPrivateMsg {
            user_id,
            message: Message::from(vec![MessageSegment::text("正在识别中，请稍候...")]),
            auto_escape: None,
        })
        .await;

    let result = perform_ocr(&url).await;

    let reply = match result {
        Ok(text) if text.trim().is_empty() => "未识别到任何文字内容".to_string(),
        Ok(text) => format!("识别结果：\n{}", text.trim()),
        Err(e) => {
            tracing::error!(error = %e, "OCR failed");
            format!("识别失败：{e}")
        }
    };

    let _ = ctx
        .api
        .call(SendPrivateMsg {
            user_id,
            message: Message::from(vec![MessageSegment::text(reply)]),
            auto_escape: None,
        })
        .await;

    true
}

fn extract_image_url(msg: &Message) -> Option<String> {
    let segs = match msg {
        Message::Array(segs) => segs,
        _ => return None,
    };

    for seg in segs {
        if let MessageSegment::Image { file, url, .. } = seg {
            if let Some(u) = url {
                if !u.is_empty() {
                    return Some(u.clone());
                }
            }
            if file.starts_with("http://") || file.starts_with("https://") {
                return Some(file.clone());
            }
        }
    }
    None
}

async fn perform_ocr(image_url: &str) -> Result<String, String> {
    let img_bytes = reqwest::get(image_url)
        .await
        .map_err(|e| format!("下载图片失败：{e}"))?
        .bytes()
        .await
        .map_err(|e| format!("读取图片数据失败：{e}"))?;

    let tmp_dir = std::env::temp_dir();
    let input_path = tmp_dir.join(format!("ocr_input_{}.png", std::process::id()));
    let output_base = tmp_dir.join(format!("ocr_output_{}", std::process::id()));
    let output_path = tmp_dir.join(format!("ocr_output_{}.txt", std::process::id()));

    tokio::fs::write(&input_path, &img_bytes)
        .await
        .map_err(|e| format!("保存临时图片失败：{e}"))?;

    let status = Command::new("tesseract")
        .arg(&input_path)
        .arg(&output_base)
        .arg("-l")
        .arg("chi_sim+eng")
        .output()
        .await
        .map_err(|e| format!("调用 tesseract 失败：{e}"))?;

    let _ = tokio::fs::remove_file(&input_path).await;

    if !status.status.success() {
        let stderr = String::from_utf8_lossy(&status.stderr);
        let _ = tokio::fs::remove_file(&output_path).await;
        return Err(format!("tesseract 执行错误：{stderr}"));
    }

    let text = tokio::fs::read_to_string(&output_path)
        .await
        .map_err(|e| format!("读取识别结果失败：{e}"))?;

    let _ = tokio::fs::remove_file(&output_path).await;

    Ok(text)
}
