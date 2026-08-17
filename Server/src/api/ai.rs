//! AI 接口:聊天 / 事件分析 / 上下文压缩 / 图片分析 / 自我审查。

use crate::ai::ChatMsg;
use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

/// POST /api/ai/chat —— 日常聊天(可带历史)。
#[derive(Debug, Deserialize)]
pub struct ChatReq {
    pub message: String,
    #[serde(default)]
    pub history: Vec<ChatMsg>,
    /// 上下文压缩阈值:历史超过该条数先压缩。
    #[serde(default = "default_compress_at")]
    pub compress_at: usize,
}
fn default_compress_at() -> usize { 20 }

pub async fn chat(State(state): State<Arc<AppState>>, Json(req): Json<ChatReq>) -> Json<Value> {
    if !state.ai.configured() {
        return Json(json!({ "ok": false, "error": "AI 未配置:config.toml 填 ai_base_url,密钥经环境变量 AI_API_KEY" }));
    }
    let mut history = req.history;
    let mut compressed = false;
    if history.len() > req.compress_at {
        if let Ok(summary) = state.ai.compress(&history) {
            history = vec![ChatMsg { role: "system".into(), content: format!("以下为历史对话压缩摘要:{summary}") }];
            compressed = true;
        }
    }
    let mut messages = vec![
        ChatMsg { role: "system".into(), content: "你是家庭智能网关 PILHOME 的 AI 助手。可协助用户管理设备、分析安防事件、编排自动化流程。回答用中文,简洁准确。".into() },
    ];
    messages.extend(history);
    messages.push(ChatMsg { role: "user".into(), content: req.message.clone() });
    match state.ai.chat(&messages) {
        Ok(reply) => Json(json!({ "ok": true, "reply": reply, "compressed": compressed })),
        Err(e) => Json(json!({ "ok": false, "error": e })),
    }
}

/// POST /api/ai/analyze —— 事件研判(传感器阈值等)。
#[derive(Debug, Deserialize)]
pub struct AnalyzeReq {
    pub event: Value,
    #[serde(default)]
    pub context: String,
}

pub async fn analyze(State(state): State<Arc<AppState>>, Json(req): Json<AnalyzeReq>) -> Json<Value> {
    match state.ai.analyze(&req.event, &req.context) {
        Ok(out) => Json(json!({ "ok": true, "analysis": out })),
        Err(e) => Json(json!({ "ok": false, "error": e })),
    }
}

/// POST /api/ai/compress —— 上下文压缩。
pub async fn compress(State(state): State<Arc<AppState>>, Json(req): Json<Vec<ChatMsg>>) -> Json<Value> {
    match state.ai.compress(&req) {
        Ok(summary) => Json(json!({ "ok": true, "summary": summary })),
        Err(e) => Json(json!({ "ok": false, "error": e })),
    }
}

/// POST /api/ai/vision —— 图片分析(传 URL 或 base64)。
#[derive(Debug, Deserialize)]
pub struct VisionReq {
    pub prompt: String,
    #[serde(default)]
    pub image_url: String,
    #[serde(default)]
    pub image_b64: Option<String>,
}

pub async fn vision(State(state): State<Arc<AppState>>, Json(req): Json<VisionReq>) -> Json<Value> {
    if req.image_url.is_empty() && req.image_b64.is_none() {
        return Json(json!({ "ok": false, "error": "需要 image_url 或 image_b64" }));
    }
    match state.ai.vision(&req.prompt, &req.image_url, req.image_b64.as_deref()) {
        Ok(out) => Json(json!({ "ok": true, "analysis": out })),
        Err(e) => Json(json!({ "ok": false, "error": e })),
    }
}

/// POST /api/ai/review —— 立即触发一次自我审查。
pub async fn review(State(state): State<Arc<AppState>>) -> Json<Value> {
    let snapshot = crate::tasks::build_review_snapshot(&state);
    match state.ai.review(&snapshot) {
        Ok(out) => {
            state.events.record(crate::events::Level::Info, "ai.review", format!("AI 自我审查完成:{}", &out[..out.len().min(200)]), None, None);
            Json(json!({ "ok": true, "snapshot": snapshot, "review": out }))
        }
        Err(e) => Json(json!({ "ok": false, "error": e })),
    }
}

/// GET /api/ai/status —— AI 配置状态。
pub async fn status(State(state): State<Arc<AppState>>) -> Json<Value> {
    Json(json!({
        "configured": state.ai.configured(),
        "base_url": state.ai_base_url,
        "model": state.ai_model,
        "vision_model": state.ai_vision_model,
    }))
}