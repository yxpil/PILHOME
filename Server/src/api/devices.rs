//! 设备接口。

use crate::state::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use pilhome_netlinker::{Device, DeviceKind};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

/// GET /api/devices
pub async fn list(State(state): State<Arc<AppState>>) -> Json<Value> {
    Json(json!({ "devices": state.devices.all(), "total": state.devices.len() }))
}

/// 注册/更新设备请求体。
#[derive(Debug, Deserialize)]
pub struct UpsertReq {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub address: String,
}

/// POST /api/devices(注册或更新设备)。
pub async fn upsert(State(state): State<Arc<AppState>>, Json(req): Json<UpsertReq>) -> (StatusCode, Json<Value>) {
    let kind = match req.kind.as_deref() {
        Some("contact") => DeviceKind::Contact,
        Some("presence") => DeviceKind::Presence,
        Some("camera") => DeviceKind::Camera,
        Some("lock") => DeviceKind::Lock,
        Some("sensor") => DeviceKind::Sensor,
        Some("actuator") => DeviceKind::Actuator,
        _ => DeviceKind::Unknown,
    };
    let device = Device::new(&req.id, &req.name, kind, req.address);
    state.devices.upsert(device);
    state
        .audit
        .record("api", "device.upsert", req.id.as_str(), "注册/更新设备", crate::state::now_secs());
    (StatusCode::OK, Json(json!({ "ok": true })))
}

/// POST /api/devices/{id}/trust(加入/移出白名单)。
pub async fn set_trust(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> (StatusCode, Json<Value>) {
    let trusted = body.get("trusted").and_then(|v| v.as_bool()).unwrap_or(true);
    let Some(mut device) = state.devices.get(&id) else {
        return (StatusCode::NOT_FOUND, Json(json!({ "error": "device not found" })));
    };
    device.trusted = trusted;
    state.devices.upsert(device);
    state
        .audit
        .record("api", "device.trust", id.as_str(), format!("trusted={trusted}"), crate::state::now_secs());
    (StatusCode::OK, Json(json!({ "ok": true, "trusted": trusted })))
}
/// 设备事件上报请求(MQTT 直连设备 / WSL 模拟器 / 真实设备的通用入口)。
#[derive(Debug, serde::Deserialize)]
pub struct DeviceEventReq {
    /// 事件键(如 contact / motion / temperature)。
    pub key: String,
    /// 事件值(如 open / 25.5)。
    pub value: String,
    /// 级别(info / warn / alert;默认 info)。
    #[serde(default)]
    pub level: Option<String>,
}

/// POST /api/devices/{id}/event —— 设备上报一条事件(阈值触发可自动告知 AI)。
pub async fn event(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(req): Json<DeviceEventReq>,
) -> Json<Value> {
    let now = crate::state::now_secs();
    // 心跳 + 台账状态。
    state.devices.heartbeat(&id, now);
    if let Some(mut d) = state.devices.get(&id) {
        d.attrs.insert("last_value".to_string(), req.value.clone());
        state.devices.upsert(d);
    }
    let lvl = match req.level.as_deref() {
        Some("alert") => crate::events::Level::Alert,
        Some("warn") => crate::events::Level::Warn,
        _ => crate::events::Level::Info,
    };
    state.events.record(
        lvl,
        &id,
        format!("{}={}", req.key, req.value),
        Some(id.as_str()),
        Some(json!({ "key": req.key, "value": req.value })),
    );

    // 阈值/告警级事件 → 后台告知 AI 研判(异步,不阻塞上报)。
    let mut ai_verdict = None;
    if lvl == crate::events::Level::Alert && state.ai.configured() {
        let event_json = json!({ "device_id": id, "key": req.key, "value": req.value, "level": "alert" });
        let ai = state.ai.clone();
        let state2 = state.clone();
        tokio::task::spawn_blocking(move || {
            if let Ok(out) = ai.analyze(&event_json, "设备事件实时研判") {
                state2.events.record(crate::events::Level::Info, "ai.analyze", format!("AI 研判:{out}"), None, None);
            }
        });
        ai_verdict = Some("AI 研判已提交");
    }

    // 贝叶斯习惯置信度(自动执行放行参考)。
    let hour = format!("{}", now / 3600 % 24);
    let feats: Vec<(&str, &str)> = vec![
        ("device", id.as_str()),
        ("key", req.key.as_str()),
        ("hour", hour.as_str()),
    ];
    let prob = state.bayes.lock().map(|b| b.predict(&feats)).unwrap_or(0.5);

    Json(json!({
        "ok": true,
        "device_id": id,
        "level": format!("{:?}", lvl).to_lowercase(),
        "ai_verdict": ai_verdict,
        "habit_probability": prob,
        "habit_verdict": if prob >= 0.6 { "符合习惯" } else if prob >= 0.4 { "存疑" } else { "建议人工确认" },
    }))
}