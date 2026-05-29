use base64::Engine;
use base64::engine::general_purpose::STANDARD as B64;
use onebot::api::payload::SendGroupMsg;
use onebot::event::message::GroupMessageEvent;
use onebot::message::MessageSegment;
use onebot::Message;
use rsa::pkcs8::DecodePrivateKey;
use rsa::pkcs1v15::SigningKey;
use rsa::signature::SignatureEncoding;
use rsa::signature::Signer;
use sha2::Sha256;

use super::{extract_plain_text, HandlerContext};

const PRIVATE_KEY_PEM: &str = "-----BEGIN PRIVATE KEY-----
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQDBCibJbFsYsO5y
2P0I2eoFy3TsFg0KrqERhEYPP66dERVa4AitJBZWrGggTtGLIlVHkW9m2dqxSW/W
Ly9ud9CkZozdnwgDQbe+v4N5kewBR0UQQQatsGo2geb7K0Dr3+PM6ScdobC7LZA5
YefnNX5cp6CmiPSEfnnj21HF4jRwlPqG+/9sPtfDeRd2YBU6vFBe25NevcUDvtjH
CYX8+ylvXWDxtkBlB11m+cQKNUt9knNo2GUZ0rfbgv2EI1ff6oT5tZQfqwsbs8Qy
i67i/hQSeQG+omGEVkz3kKQslMjQZ8PKE4ZpDaRTybo448c85uCsraTSXbx/yoqD
CPC5zclJAgMBAAECggEAEosGeIsKcaPeVeUq4kingqS9f6erG0t00K1RewnBnNgd
Imh6oOIibLM9Qdw8Y/Z5dWlep4U2DRlqLyuDbYPAJKiur6PFBeYQT9gO+bS3FFTb
qSDr8q+Ldl6yWM55+yZ7uA1t+tpTMxnAfMifOY4hjCY0iAbIITmSfR0uXNk6sxUr
so8pnctrJmZTsNoIzr3lAdlJRv/4sSnEkcjZYRPx1y5fdI93qRlQRf8b51352EKy
f0N6spnEZr7nhIhU/4v8c6U8md0g2l+4QGMyGygngF9rHBRzqIRGsp07I7Nddjc5
vQpM2MaK98iSsZtqjiGEGXDTNVShZfJXN86X7YcpIQKBgQDtQQ1s9ZYhsEvEkPaf
Ip3x+wF8YJy8crh6IYxQKkI+WQlEsjJ3T6AyCXrLCOmOGF8Nt9u8r8sfOXu8mY01
ks4mJhi4V4VS3FIDDSdhfYGfGqY7yRdmNDkQYld0UhZIM18IdCBOVEF4zLeOasRZ
IEC63J4HmaCB8APVhZWrzivbaQKBgQDQSsWKEBneSNXMo6M/6EdJpW1bdJiCSTPN
mm0vzsZcgFFUVSbOgSu5AiMevjGWvnacV2DEFVONh+oJD46NyHu5atPC3ZAyWohl
as8ISZ14Y0xq5o99qkflO/4V6s+KalkXokvvEdQTEyeFYy4bzKX5xkjAlBLf7Ue8
Fcpss24i4QKBgQDBtfyFOxsiVHP4gTerhLMa8IstByDRyIUQyrVqeqZti3rCyQ/l
VHECibTlc1hmOUXayIQz0gBxdRivS1v9IukIQtCqKmNj3Rlk/mdp9PRReIvDgpOF
UhxJYVHwWllxB+iO2WnLKoXuYI96S3gXIPtY1mp84BUqIlKvEou6o/IxCQKBgCRy
sRZzstMe06q3h72LG85bUEOMp5NE0/fKagjPmg5dtd2X+O5x1ADPyu16QpsqQP8i
myA0yyYc/msPedZ9mojblKqosq7dALkec5PzrcZ/OcQLDFjlDyeh09hp+l8yNNzZ
3Ye8Cuw7kdLZhBwBN5n5hImOX68nikHzXjSfQUqBAoGBAMgkEbNl3lRSNWdCpLQS
jB5Aj8IPJ/ehpOEh1j0FHuRi44ZhB8VuJxxwCMyCqlmfTaHRWSYRc5CE2pT6Qvei
7iYHYWcP+iCAV1OpveGl9pBLVHn6mqAoWymZch2r9K+IqxKFiBgoaOtc53sWZAlh
HjVyXb4f/gj01+geWs5fEm62
-----END PRIVATE KEY-----";

const DEFAULT_DURATION_HOURS: u64 = 2;
const DEFAULT_SUB: &str = "bot";

pub async fn handle_gencdk(ctx: &HandlerContext, evt: &GroupMessageEvent) -> bool {
    let text = extract_plain_text(&evt.message);

    let hwid = match text.strip_prefix("gencdk ") {
        Some(h) => h.trim(),
        None => return false,
    };

    if hwid.is_empty() {
        let _ = ctx
            .api
            .call(SendGroupMsg {
                group_id: evt.group_id,
                message: Message::from(vec![
                    MessageSegment::reply(evt.message_id.to_string()),
                    MessageSegment::text("用法: gencdk <hwid>"),
                ]),
                auto_escape: None,
            })
            .await;
        return true;
    }

    let reply = match generate_cdk(hwid) {
        Ok(cdk) => {
            let exp_ms = now_ms() + DEFAULT_DURATION_HOURS * 3600 * 1000;
            let exp_str = format_timestamp_ms(exp_ms);
            format!("CDK 已生成 (有效期 {DEFAULT_DURATION_HOURS}h, 至 {exp_str}):\n{cdk}")
        }
        Err(e) => {
            tracing::error!(error = %e, "gencdk failed");
            format!("CDK 生成失败: {e}")
        }
    };

    let _ = ctx
        .api
        .call(SendGroupMsg {
            group_id: evt.group_id,
            message: Message::from(vec![
                MessageSegment::reply(evt.message_id.to_string()),
                MessageSegment::text(reply),
            ]),
            auto_escape: None,
        })
        .await;

    true
}

fn generate_cdk(hwid: &str) -> Result<String, Box<dyn std::error::Error>> {
    let now = now_ms();
    let exp = now + DEFAULT_DURATION_HOURS * 3600 * 1000;

    // Keep key order consistent with the JS generator: exp, iat, sub, hwid
    let payload_json = format!(
        r#"{{"exp":{exp},"iat":{now},"sub":"{sub}","hwid":"{hwid}"}}"#,
        exp = exp,
        now = now,
        sub = DEFAULT_SUB,
        hwid = hwid,
    );

    let payload_b64 = B64.encode(payload_json.as_bytes());

    let private_key = rsa::RsaPrivateKey::from_pkcs8_pem(PRIVATE_KEY_PEM)?;
    let signing_key = SigningKey::<Sha256>::new(private_key);
    let signature = signing_key.sign(payload_b64.as_bytes());
    let signature_b64 = B64.encode(signature.to_bytes());

    Ok(format!("{payload_b64}.{signature_b64}"))
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

fn format_timestamp_ms(ms: u64) -> String {
    let secs = (ms / 1000) as i64;
    let offset_secs = secs + 8 * 3600; // UTC+8
    let days_since_epoch = offset_secs / 86400;
    let time_of_day = offset_secs % 86400;

    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;

    // Simple date calculation
    let mut y = 1970i64;
    let mut remaining_days = days_since_epoch;

    loop {
        let days_in_year = if is_leap(y) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        y += 1;
    }

    let months_days: [i64; 12] = if is_leap(y) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut m = 0;
    for (i, &d) in months_days.iter().enumerate() {
        if remaining_days < d {
            m = i + 1;
            break;
        }
        remaining_days -= d;
    }
    let day = remaining_days + 1;

    format!("{y}-{m:02}-{day:02} {hours:02}:{minutes:02}")
}

fn is_leap(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}
