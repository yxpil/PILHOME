//! 事件接口。

use crate::state::AppState;
use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

/// 查询参数。
#[derive(Debug, Deserialize)]
pub struct QueryOpts {
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    100
}

/// GET /api/events?limit=100
pub async fn list(State(state): State<Arc<AppState>>, Query(q): Query<QueryOpts>) -> Json<Value> {
    let limit = q.limit.clamp(1, 1000);
    Json(json!({ "events": state.events.recent(limit), "total": state.events.len() }))
}