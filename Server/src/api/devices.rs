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