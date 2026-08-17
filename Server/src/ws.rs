//! WebSocket 实时事件流 —— 对外开放接入。
//!
//! 任何客户端(HA 自动化、自定义脚本、手机 App、其他网关)可连接
//! `ws://<host>:8080/api/ws`,实时接收 PILHOME 全量事件(JSON 文本帧),
//! 实现"允许接入它"的开放能力。

use crate::state::AppState;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::Response;
use serde_json::json;
use std::sync::Arc;

/// WebSocket 升级端点:`/api/ws`。
pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// 连接生命周期:持续推送事件,直到客户端断开。
async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    // 订阅事件广播。
    let mut rx = state.events.subscribe();

    // 握手:推送系统信息。
    let hello = json!({
        "type": "auth_ok",
        "system": { "name": "PILHOME 边缘网关", "version": env!("CARGO_PKG_VERSION") },
        "note": "实时事件流已连接",
    })
    .to_string();
    if socket.send(Message::Text(hello.into())).await.is_err() {
        return;
    }

    loop {
        tokio::select! {
            // 事件推送。
            event = rx.recv() => {
                let Ok(event) = event else { continue };
                let text = json!({ "type": "event", "data": event }).to_string();
                if socket.send(Message::Text(text.into())).await.is_err() {
                    break;
                }
            }
            // 客户端消息(目前仅支持 ping/pong 保活)。
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(text))) if text == "ping" => {
                        if socket.send(Message::Text("pong".into())).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Err(_)) => break,
                    _ => {}
                }
            }
        }
    }
}