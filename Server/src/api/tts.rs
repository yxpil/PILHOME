//! TTS 语音合成接口。

use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

/// POST /api/tts —— 合成语音并落盘 data/tts/。
#[derive(Debug, Deserialize)]
pub struct TtsReq {
    pub text: String,
}

pub async fn speak(State(state): State<Arc<AppState>>, Json(req): Json<TtsReq>) -> Json<Value> {
    match crate::tts::speak(&state, &req.text) {
        Ok(path) => Json(json!({ "ok": true, "path": path })),
        Err(e) => Json(json!({ "ok": false, "error": e })),
    }
}