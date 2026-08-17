//! 模块控制接口(HandModel)。

use crate::state::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde_json::{Value, json};
use std::sync::Arc;

/// GET /api/control
pub async fn list(State(state): State<Arc<AppState>>) -> Json<Value> {
    Json(json!({ "modules": state.controller.all() }))
}

/// POST /api/control/{id}/start
pub async fn start(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> (StatusCode, Json<Value>) {
    if !state.controller.start(&id) {
        state.controller.register(&id);
        state.controller.start(&id);
    }
    state.coord.start_module(&id);
    state
        .audit
        .record("api", "module.start", id.as_str(), "启动模块", crate::state::now_secs());
    (StatusCode::OK, Json(json!({ "ok": true, "id": id, "running": true })))
}

/// POST /api/control/{id}/stop
pub async fn stop(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> (StatusCode, Json<Value>) {
    if state.controller.stop(&id) {
        state.coord.stop_module(&id);
        state
            .audit
            .record("api", "module.stop", id.as_str(), "停止模块", crate::state::now_secs());
        (StatusCode::OK, Json(json!({ "ok": true, "id": id, "running": false })))
    } else {
        (StatusCode::NOT_FOUND, Json(json!({ "error": "module not registered" })))
    }
}