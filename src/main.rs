use axum::{
    Router,
    routing::get,
    extract::State,
    response::{Html, IntoResponse},
};
use std::{env, net::SocketAddr, sync::Arc};
use dotenv::dotenv;
use reqwest::Client;
use serde::Serialize;
use tokio::net::TcpListener;

#[derive(Clone)]
struct AppState {
    telegram_url: String,
    telegram_body: serde_json::Value,
    matrix_url: String,
    matrix_body: MatrixMessage,
    client: Client,
}

#[derive(Serialize, Clone)]
struct MatrixMessage {
    msgtype: String,
    body: String,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let bot_token = env::var("BOT_TOKEN").expect("BOT_TOKEN not set");
    let chat_id = env::var("CHAT_ID").expect("CHAT_ID not set");
    let matrix_token = env::var("MATRIX_TOKEN").expect("MATRIX_TOKEN not set");
    let matrix_room_id = env::var("MATRIX_ROOM_ID").expect("MATRIX_ROOM_ID not set");

    let telegram_url = format!("https://api.telegram.org/bot{}/sendMessage", bot_token);
    let telegram_body = serde_json::json!({
        "chat_id": chat_id,
        "text": "🚪 누군가 문 앞에서 기다리고 있어요!"
    });

    let matrix_url = format!(
        "https://matrix-client.matrix.org/_matrix/client/r0/rooms/{}/send/m.room.message?access_token={}",
        matrix_room_id, matrix_token
    );
    let matrix_body = MatrixMessage {
        msgtype: "m.text".to_string(),
        body: "🚪 누군가 문 앞에서 기다리고 있어요!".to_string(),
    };

    let state = Arc::new(AppState {
        telegram_url,
        telegram_body,
        matrix_url,
        matrix_body,
        client: Client::new(),
    });

    let app = Router::new()
        .route("/", get(home))
        .route("/notify", get(notify))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await.unwrap();

    println!("Server running at http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn home() -> Html<&'static str> {
    Html("QR Doorbell is alive!")
}

async fn notify(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let telegram_resp = state
        .client
        .post(&state.telegram_url)
        .json(&state.telegram_body)
        .send()
        .await;

    if let Err(e) = telegram_resp {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Telegram Error: {}", e),
        );
    }

    let matrix_resp = state
        .client
        .post(&state.matrix_url)
        .json(&state.matrix_body)
        .send()
        .await;

    if let Err(e) = matrix_resp {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Matrix Error: {}", e),
        );
    }

    (
        axum::http::StatusCode::OK,
        "🔔 방문 요청이 접수되었습니다. 잠시만 기다려 주세요.".to_string(),
    )
}
