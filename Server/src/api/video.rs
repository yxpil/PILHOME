//! 视频设备与审计接口(VEScaner × YololookUP)。

use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use pilhome_vescaner::ProbeConfig;
use pilhome_yololookup::{AuditEngine, Detection};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

/// POST /api/video/probe —— 探测网段内视频设备(RTSP/ONVIF)。
pub async fn probe(State(state): State<Arc<AppState>>, Json(cfg): Json<ProbeConfig>) -> Json<Value> {
    let cfg = if cfg.prefix.is_empty() { ProbeConfig::default() } else { cfg };
    let state2 = state.clone();
    let devices = tokio::task::spawn_blocking(move || pilhome_vescaner::probe_network(&cfg))
        .await
        .unwrap_or_default();
    state2
        .audit
        .record("api", "video.probe", "vescaner", format!("探测到 {} 台视频设备", devices.len()), crate::state::now_secs());
    Json(json!({ "ok": true, "devices": devices, "total": devices.len() }))
}

/// 视频审计请求体。
#[derive(Debug, Deserialize)]
pub struct AuditReq {
    /// 摄像头 ID。
    pub camera_id: String,
    /// 画面运动比例(0.0 ~ 1.0,来自帧差分析)。
    pub motion_ratio: f32,
    /// 模拟检测结果(未接真实 YOLO 时可注入演示)。
    #[serde(default)]
    pub detections: Vec<Detection>,
}

/// POST /api/video/audit —— 视频粗略代审计(规则引擎版)。
pub async fn audit(State(state): State<Arc<AppState>>, Json(req): Json<AuditReq>) -> Json<Value> {
    let mut engine = AuditEngine::new();
    // 注入演示检测结果;接入真实 YOLO 时替换为 Detector 实现。
    let detections = req.detections.clone();
    let event = {
        let engine_mut = &mut engine;
        if !detections.is_empty() {
            engine_mut.with_detector(Box::new(StaticDetector(detections)));
        }
        engine_mut.audit_frame(&req.camera_id, &[], 0, 0, req.motion_ratio, crate::state::now_secs())
    };

    state
        .audit
        .record("api", "video.audit", &req.camera_id, format!("运动比例 {:.2}", req.motion_ratio), crate::state::now_secs());

    match event {
        Some(e) => Json(json!({ "ok": true, "event": e })),
        None => Json(json!({ "ok": true, "event": null, "note": "无触发条件,未产生审计事件" })),
    }
}

/// 静态检测器:直接返回注入的检测列表(演示用)。
struct StaticDetector(Vec<Detection>);

impl pilhome_yololookup::Detector for StaticDetector {
    fn detect(&self, _frame: &[u8], _width: usize, _height: usize) -> Vec<Detection> {
        self.0.clone()
    }
}