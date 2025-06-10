use std::env;
use serde::Serialize;
use reqwest::Client;

#[derive(Serialize)]
struct MatrixMessage {
    msgtype: String,
    body: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let bot_token = env::var("BOT_TOKEN")?;
    let chat_id = env::var("CHAT_ID")?;
    let matrix_token = env::var("MATRIX_TOKEN")?;
    let matrix_room_id = env::var("MATRIX_ROOM_ID")?;

    let telegram_url = format!("https://api.telegram.org/bot{}/sendMessage", bot_token);
    let telegram_data = serde_json::json!({
        "chat_id": chat_id,
        "text": "🚪 누군가 문 앞에서 기다리고 있어요!"
    });

    let matrix_url = format!(
        "https://matrix-client.matrix.org/_matrix/client/r0/rooms/{}/send/m.room.message?access_token={}",
        matrix_room_id, matrix_token
    );
    let matrix_message = MatrixMessage {
        msgtype: "m.text".to_string(),
        body: "🚪 누군가 문 앞에서 기다리고 있어요!".to_string(),
    };

    let client = Client::new();

    let telegram_resp = client.post(&telegram_url).json(&telegram_data).send().await?;
    if !telegram_resp.status().is_success() {
        return Err(format!("Telegram failed: {}", telegram_resp.status()).into());
    }

    let matrix_resp = client.post(&matrix_url).json(&matrix_message).send().await?;
    if !matrix_resp.status().is_success() {
        return Err(format!("Matrix failed: {}", matrix_resp.status()).into());
    }

    println!("🔔 방문 요청이 전송되었습니다.");

    Ok(())
}
