//! 边缘模型接口(SideAgent)。

use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use pilhome_sideagent::{ModelInfo, ModelKind, ModelStatus};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

/// GET /api/models
pub async fn list(State(state): State<Arc<AppState>>) -> Json<Value> {
    Json(json!({ "models": state.models.all(), "total": state.models.len() }))
}

/// 加载模型请求体。
#[derive(Debug, Deserialize)]
pub struct LoadReq {
    pub id: String,
    pub name: String,
    #[serde(default = "default_kind")]
    pub kind: String,
    pub format: String,
    pub path: String,
}

fn default_kind() -> String {
    "detection".to_string()
}

/// POST /api/models(注册并标记为就绪)。
pub async fn load(State(state): State<Arc<AppState>>, Json(req): Json<LoadReq>) -> Json<Value> {
    let kind = match req.kind.as_str() {
        "classification" => ModelKind::Classification,
        "gesture" => ModelKind::Gesture,
        "behavior" => ModelKind::Behavior,
        "rule" => ModelKind::Rule,
        _ => ModelKind::Detection,
    };
    let mut info = ModelInfo::new(&req.id, &req.name, kind, &req.format, &req.path);
    info.status = ModelStatus::Ready;
    state.models.register(info);
    state
        .audit
        .record("api", "model.load", req.id.as_str(), "注册边缘模型", crate::state::now_secs());
    Json(json!({ "ok": true }))
}