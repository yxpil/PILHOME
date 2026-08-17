//! WOL 网络唤醒接口。

use crate::state::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde_json::{Value, json};
use std::net::Ipv4Addr;
use std::sync::Arc;

/// WOL 请求体。
#[derive(Debug, Deserialize)]
pub struct WolReq {
    /// MAC 地址(`aa:bb:cc:dd:ee:ff` / `aa-bb-...`)。
    pub mac: Option<String>,
    /// 设备 ID(从设备台账 attrs.mac 取)。
    pub device_id: Option<String>,
    /// 广播地址(默认 255.255.255.255)。
    pub broadcast: Option<String>,
}

/// POST /api/wol —— 发送 WOL 魔法包唤醒目标主机。
pub async fn wake(State(state): State<Arc<AppState>>, Json(req): Json<WolReq>) -> (StatusCode, Json<Value>) {
    // 解析 MAC:直接给或按设备 ID 查台账。
    let mac_str = match (&req.mac, &req.device_id) {
        (Some(m), _) => m.clone(),
        (None, Some(id)) => match state.devices.get(id) {
            Some(d) => d.attrs.get("mac").cloned().unwrap_or_default(),
            None => return (StatusCode::NOT_FOUND, Json(json!({ "ok": false, "error": "设备不存在或未记录 MAC" }))),
        },
        _ => return (StatusCode::BAD_REQUEST, Json(json!({ "ok": false, "error": "缺少 mac 或 device_id" }))),
    };
    let Some(mac) = pilhome_netlinker::parse_mac(&mac_str) else {
        return (StatusCode::BAD_REQUEST, Json(json!({ "ok": false, "error": "MAC 格式无效" })));
    };
    let broadcast = req
        .broadcast
        .as_deref()
        .unwrap_or("255.255.255.255")
        .parse::<Ipv4Addr>()
        .unwrap_or(Ipv4Addr::BROADCAST);

    match pilhome_netlinker::send_wol(mac, broadcast, 9) {
        Ok(()) => {
            state.audit.record("api", "wol.send", mac_str.as_str(), "网络唤醒", crate::state::now_secs());
            (StatusCode::OK, Json(json!({ "ok": true, "mac": mac_str, "broadcast": broadcast.to_string() })))
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "ok": false, "error": format!("发送失败:{e}") }))),
    }
}