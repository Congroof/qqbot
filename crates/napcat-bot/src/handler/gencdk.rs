use base64::Engine;
use base64::engine::general_purpose::STANDARD as B64;
use onebot::api::payload::SendGroupMsg;
use onebot::event::message::GroupMessageEvent;
use onebot::message::MessageSegment;
use onebot::Message;
use rsa::pkcs1::DecodeRsaPrivateKey;
use rsa::pkcs1v15::SigningKey;
use rsa::signature::SignatureEncoding;
use rsa::signature::Signer;
use sha2::Sha256;

use super::{extract_plain_text, HandlerContext};

const PRIVATE_KEY_PEM: &str = "-----BEGIN RSA PRIVATE KEY-----
MIIEpAIBAAKCAQEAvgQ0tway0AJLCQiHcJRf9n325WNS1o1o62yBz8gRprB1LBFd
L84hhlmzIipc3abLsQra9BObFLGeIsiz3nwweCmgi1sNErPKfBq9+YsUci4SbIsM
Iu8l01RAaOqTFVPLUGEYE6fXtngNMIXphICjlSW/2mMp/TVl0qqWnQrIldysyxgt
WKvbAv0k3naLxkKDyH6PbXZ/yYWaxePKts2xtfAolAJaimOIQkK9TKVkhNIxuPdx
OP65nV2vC3CKg3xP5wxWurTTNkL4rWK80LUP01wKrfzdp1GztYwCMLONpbnMR36m
vuTYA717trR9gyDZPWGGXHSxmedLXLtgkfZTqwIDAQABAoIBAQCYqexoeFtFv/Hl
ShL3Il6PPdkVp10wv+Bh9YW+GLIFyJP7WeASvnw04vCHLJ37/zx7+4q6ut3IHIQ+
0h2hTQnsIRW5oOe59PVkDGBBk7pTmix3RKf1kUpEpdYx9PVDF1WsOLYNZLZtBbsj
FxPsvyWueOvRXAaqRzKNtTzY44cQzx7fZx8AmHcipcFyaym0uIcpIdZwBC4o85Js
KMM4Ez8GfaS404MKgxBgqXVeAtqSYD7lQbrUzAx2qosmP1Z/bYYB9CBx2OB2joie
x0MozRqboBOoc5QGC7XSYgX8KwfFk8i9K5eL/kfXD7aXzdqd5Xrs5PTm2koUoHBb
ef7XCSIxAoGBAOVrlunEuCNpQ0+rmFb3ITaF9JVkBsmNLuvasKe0CRGH3luBjEUn
XvN/f3kxTR31Asr4cMKS8ln58D6yNsLC1Oc+9kwLj1DAkyKjwT53cHyAzpXfvkTE
GQBImszU/KZrOcFiYKOkk+Ua6VMHa+XBW1Un7RbfXVqzxLVQ6jRCTPrdAoGBANQH
7pFhQSFQF6/gsE8fV08FhtGMJDcITD4ugKYOH47QflyudcDqX5yNCgu9fTQ5708T
rnZL2vNoi22aQamnS0SuTIrB2UrhgobjhgscZ6v4CiEcHew5DiZha5JYuWwItNnu
RAnkXnw7piW2J/oYcxUzaiOQ0KlzN02JJ+iKZMwnAoGBAL7O8+4rkebJxpT9p680
zSfW06xoTAjX5p18/o4Me7pb4YEDxxFBBITKls/KRFRVEeSUKtx5cR2Kddj/SfJE
LuTBhgGLX8AO2pDl13RHzIOQccFPHKV+3zhQKoeP4S3cYmXHl46i8+qJrmNC+edW
IMs7cMIkNjWY7FLNIG0kc3f1AoGAYjUGT/n+48IoJoNoxk0a8HP73QUPJRpHzilV
1xQFk+2ICb+YtPEZtfYxp/xtiIopCLRyA0LhOAq9QdfIAB+HolklBMQCtEc9YOLz
jCPs9N8fOfS++1H19tr6qz9DKwHhWmucwgQvq1UpgKAdZh5691/oEm8Z5tKB0/zq
KjAnvdMCgYBchwzW2bs0P0hcX8hUX1pwZn0DYNdvXR20vJYhSV+0oE2EMFgH4gMX
553QG885HVx+4sIoPt6XgDLGSLInNJmxm8pUYx7Yb5L85fYJfdQESGYqvBbBPcgN
gHHJ30TRnGGCBZbkRAuEc2mrzKQgHsCLnf66Nyheehjg3SV2s1hTqg==
-----END RSA PRIVATE KEY-----";

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

    let private_key = rsa::RsaPrivateKey::from_pkcs1_pem(PRIVATE_KEY_PEM)?;
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
