//! 定时规则接口(AUTOTIME)。

use crate::state::AppState;
use crate::tasks::rhythm_suggestion;
use axum::extract::State;
use axum::Json;
use pilhome_autotime::parse_rule;
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

/// 新增规则请求体。
#[derive(Debug, Deserialize)]
pub struct AddRuleReq {
    pub name: String,
    pub action: String,
    pub expr: String,
}

/// GET /api/schedules
pub async fn list(State(state): State<Arc<AppState>>) -> Json<Value> {
    let rules = state.schedules.read().map(|s| s.clone()).unwrap_or_default();
    Json(json!({ "schedules": rules, "total": rules.len() }))
}

/// POST /api/schedules
pub async fn add(State(state): State<Arc<AppState>>, Json(req): Json<AddRuleReq>) -> (axum::http::StatusCode, Json<Value>) {
    match parse_rule(&req.name, &req.action, &req.expr) {
        Ok(rule) => {
            if let Ok(mut rules) = state.schedules.write() {
                rules.push(rule);
            }
            state
                .audit
                .record("api", "schedule.add", req.name.as_str(), format!("{} {}", req.action, req.expr), crate::state::now_secs());
            (axum::http::StatusCode::OK, Json(json!({ "ok": true })))
        }
        Err(err) => (axum::http::StatusCode::BAD_REQUEST, Json(json!({ "error": err }))),
    }
}

/// GET /api/rhythm(节律曲线与布防建议)。
pub async fn rhythm(State(state): State<Arc<AppState>>) -> Json<Value> {
    Json(rhythm_suggestion(&state))
}