//! 系统总览接口。

use crate::modules;
use crate::state::{now_secs, AppState};
use axum::extract::State;
use axum::Json;
use serde_json::{Value, json};
use std::sync::Arc;

/// 默认离线判定阈值(秒)。
const OFFLINE_AFTER_SECS: u64 = 120;

/// GET /api/status(系统信息 + 模块状态 + 概览指标)。
pub async fn overview(State(state): State<Arc<AppState>>) -> Json<Value> {
    let now = now_secs();
    let total = state.devices.len();
    let online = state.devices.online_count(now, OFFLINE_AFTER_SECS);
    let mods = modules::collect(&state, OFFLINE_AFTER_SECS);

    Json(json!({
        "system": {
            "name": "PILHOME 边缘网关",
            "version": env!("CARGO_PKG_VERSION"),
            "uptime_secs": state.uptime_secs(),
            "now": now,
        },
        "overview": {
            "devices_total": total,
            "devices_online": online,
            "events": state.events.len(),
            "audit": state.audit.len(),
            "models_ready": state.models.ready_count(),
            "connections": state.connector.count(pilhome_netlinker::ConnState::Connected),
        },
        "modules": mods,
    }))
}