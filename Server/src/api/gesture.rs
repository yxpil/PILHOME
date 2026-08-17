//! 手势推理接口(SideAgent 规则引擎演示)。

use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use pilhome_sideagent::predict_rule;
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

/// 推理请求体:8 维归一化手指开合特征。
#[derive(Debug, Deserialize)]
pub struct GestureReq {
    /// 8 维特征向量(0.0 ~ 1.0)。
    pub features: [f32; 8],
}

/// POST /api/gesture —— 边缘规则引擎手势分类。
pub async fn infer(State(state): State<Arc<AppState>>, Json(req): Json<GestureReq>) -> Json<Value> {
    let started = std::time::Instant::now();
    let results = predict_rule(&req.features);
    let elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;

    state
        .audit
        .record("api", "gesture.infer", "handmodel", "执行手势推理", crate::state::now_secs());

    Json(json!({
        "ok": true,
        "latency_ms": elapsed_ms,
        "top": results.first(),
        "results": results,
    }))
}
