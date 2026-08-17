//! 全自动发现 / ARP 表 / 厂商设备接口。

use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use serde_json::{Value, json};
use std::sync::Arc;

/// POST /api/discover —— 触发一轮全自动发现(扫描+ARP+mDNS+指纹入库)。
pub async fn run(State(state): State<Arc<AppState>>) -> Json<Value> {
    let state2 = state.clone();
    let report = tokio::task::spawn_blocking(move || crate::discover::auto_discover(&state2))
        .await
        .unwrap_or_else(|_| crate::discover::auto_discover(&state));
    state.events.record(
        crate::events::Level::Info,
        "netscanear",
        format!("全自动发现完成:新增 {} 台,共 {} 台", report.new_devices, report.total_devices),
        None,
        None,
    );
    Json(json!({ "ok": true, "report": report }))
}

/// GET /api/arp —— 系统 ARP 表。
pub async fn arp() -> Json<Value> {
    let entries = pilhome_netscanear::read_arp_table();
    Json(json!({ "entries": entries, "total": entries.len() }))
}

/// GET /api/vendors —— 厂商生态:支持清单 + mDNS 即时发现。
pub async fn vendors() -> Json<Value> {
    let devices = crate::vendor::discover_mdns_devices();
    Json(json!({
        "supported": crate::vendor::SUPPORTED_VENDORS,
        "discovered": devices,
        "total": devices.len(),
    }))
}