//! 工具接口:列表 / 搜索 / 创建 JS 工具 / 执行 / 调试(报错分析)/ 删除。

use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

/// GET /api/tools —— 全部工具。
pub async fn list(State(state): State<Arc<AppState>>) -> Json<Value> {
    let tools = state.tools.list();
    Json(json!({ "tools": tools, "total": tools.len() }))
}

/// GET /api/tools/search?q= —— 搜索工具。
#[derive(Debug, Deserialize)]
pub struct SearchQuery { pub q: Option<String> }

pub async fn search(State(state): State<Arc<AppState>>, Query(q): Query<SearchQuery>) -> Json<Value> {
    let tools = state.tools.search(q.q.as_deref().unwrap_or(""));
    Json(json!({ "tools": tools, "total": tools.len() }))
}

/// 创建 JS 工具请求。
#[derive(Debug, Deserialize)]
pub struct CreateReq {
    pub name: String,
    pub description: String,
    pub code: String,
}

/// POST /api/tools —— 创建 JS 工具。
pub async fn create(State(state): State<Arc<AppState>>, Json(req): Json<CreateReq>) -> (StatusCode, Json<Value>) {
    match state.tools.add_js(&req.name, &req.description, &req.code) {
        Ok(tool) => {
            state.audit.record("api", "tool.create", &tool.id, &req.name, crate::state::now_secs());
            (StatusCode::OK, Json(json!({ "ok": true, "tool": tool })))
        }
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({ "ok": false, "error": e }))),
    }
}

/// 执行工具请求。
#[derive(Debug, Deserialize)]
pub struct RunReq {
    #[serde(default)]
    pub args: Value,
}

/// POST /api/tools/{id}/run —— 执行工具(JS 工具经沙箱)。
pub async fn run(State(state): State<Arc<AppState>>, Path(id): Path<String>, Json(req): Json<RunReq>) -> Json<Value> {
    match state.tools.run(&state, &id, req.args) {
        Ok(result) => Json(json!({ "ok": true, "result": result })),
        Err(e) => Json(json!({ "ok": false, "error": e })),
    }
}

/// POST /api/tools/{id}/debug —— 报错分析(AI 给修复建议)。
pub async fn debug(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    match crate::tools::debug_tool(&state, &id) {
        Ok(advice) => Json(json!({ "ok": true, "advice": advice })),
        Err(e) => Json(json!({ "ok": false, "error": e })),
    }
}

/// DELETE /api/tools/{id} —— 删除工具(内置不可删)。
pub async fn remove(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> (StatusCode, Json<Value>) {
    if state.tools.remove(&id) {
        (StatusCode::OK, Json(json!({ "ok": true })))
    } else {
        (StatusCode::BAD_REQUEST, Json(json!({ "ok": false, "error": "内置工具不可删除或不存在" })))
    }
}