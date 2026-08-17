//! MQTT 发布接口 —— 外部系统可经 PILHOME 下发设备命令。

use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

/// 发布请求体。
#[derive(Debug, Deserialize)]
pub struct PublishReq {
    /// 主题(如 `pilhome/gateway/command` 或设备命令主题)。
    pub topic: String,
    /// 负载(任意 JSON 序列化值)。
    pub payload: Value,
}

/// POST /api/mqtt/publish —— 通过网关发布 MQTT 消息。
pub async fn publish(State(state): State<Arc<AppState>>, Json(req): Json<PublishReq>) -> (axum::http::StatusCode, Json<Value>) {
    // 锁内取出 Sender 克隆,立即释放 guard(避免跨 await 持有非 Send 锁)。
    let sender = state.mqtt_tx.lock().ok().and_then(|g| g.clone());
    let ok = match sender {
        Some(tx) => tx
            .send(crate::mqtt::MqttCommand {
                topic: req.topic.clone(),
                payload: req.payload.to_string(),
            })
            .await
            .is_ok(),
        None => false,
    };
    if ok {
        state.audit.record("api", "mqtt.publish", req.topic.as_str(), "外部发布", crate::state::now_secs());
        (axum::http::StatusCode::OK, Json(json!({ "ok": true })))
    } else {
        (axum::http::StatusCode::SERVICE_UNAVAILABLE, Json(json!({ "ok": false, "error": "MQTT 未连接" })))
    }
}