//! Stable Diffusion 图像生成接口(ComfyUI)。

use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

/// POST /api/sd/generate —— 生成图片(同步等待,最长 120s)。
#[derive(Debug, Deserialize)]
pub struct SdReq {
    pub prompt: String,
    #[serde(default)]
    pub negative: Option<String>,
}

pub async fn generate(State(state): State<Arc<AppState>>, Json(req): Json<SdReq>) -> Json<Value> {
    match crate::sd::generate(&state, &req.prompt, req.negative.as_deref()) {
        Ok(result) => Json(json!({ "ok": true, "result": result })),
        Err(e) => Json(json!({ "ok": false, "error": e })),
    }
}