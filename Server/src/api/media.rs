//! 媒体设备接口:音响/机顶盒/投屏发现 + DIAL 投屏。

use axum::Json;
use serde::Deserialize;
use serde_json::{Value, json};
/// GET /api/media/discover —— 发现音响/机顶盒/投屏设备。
pub async fn discover() -> Json<Value> {
    let devices = crate::media::discover_media_devices();
    let audio: Vec<_> = devices.iter().filter(|d| d.class == "audio").collect();
    let cast: Vec<_> = devices.iter().filter(|d| d.class == "cast").collect();
    let stb: Vec<_> = devices.iter().filter(|d| d.class == "stb").collect();
    Json(json!({
        "devices": devices,
        "audio_count": audio.len(),
        "cast_count": cast.len(),
        "stb_count": stb.len(),
    }))
}

/// POST /api/media/dial —— DIAL 投屏(启动应用)。
#[derive(Debug, Deserialize)]
pub struct DialReq {
    pub ip: String,
    pub port: u16,
    pub app: String,
    #[serde(default)]
    pub payload: Option<String>,
}

pub async fn dial(Json(req): Json<DialReq>) -> Json<Value> {
    match crate::media::dial_launch(&req.ip, req.port, &req.app, req.payload.as_deref()) {
        Ok(status) => Json(json!({ "ok": true, "status": status })),
        Err(e) => Json(json!({ "ok": false, "error": e })),
    }
}