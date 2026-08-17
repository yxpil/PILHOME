//! 网络扫描接口。

use crate::state::AppState;
use crate::tasks::run_scan;
use axum::extract::State;
use axum::Json;
use serde_json::{Value, json};
use std::sync::Arc;

/// POST /api/scan/run(立即扫描一次)。
pub async fn run(State(state): State<Arc<AppState>>) -> Json<Value> {
    run_scan(&state).await;
    let last = state.last_scan.lock().map(|s| s.clone()).unwrap_or(None);
    match last {
        Some(report) => Json(json!({ "ok": true, "report": report })),
        None => Json(json!({ "ok": false, "error": "扫描未完成" })),
    }
}

/// GET /api/scan/last(最近一次扫描报告)。
pub async fn last(State(state): State<Arc<AppState>>) -> Json<Value> {
    let last = state.last_scan.lock().map(|s| s.clone()).unwrap_or(None);
    match last {
        Some(report) => Json(json!({ "ok": true, "report": report })),
        None => Json(json!({ "ok": false, "error": "尚未执行扫描" })),
    }
}