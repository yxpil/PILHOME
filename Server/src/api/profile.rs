//! 行为画像接口(ATOGrowUP)。

use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use serde_json::{Value, json};
use std::sync::Arc;

/// GET /api/profile —— 各设备 24 小时活跃曲线与异常建议。
pub async fn view(State(state): State<Arc<AppState>>) -> Json<Value> {
    let (curves, suggestions, total) = {
        let profile = state.profile.lock().ok();
        match profile {
            Some(p) => {
                let mut curves = serde_json::Map::new();
                for device in p.devices() {
                    curves.insert(device.clone(), json!(p.activity_curve(&device)));
                }
                let suggestions = p.suggestions(crate::state::now_secs());
                (curves, suggestions, p.len())
            }
            None => (serde_json::Map::new(), Vec::new(), 0),
        }
    };

    Json(json!({
        "total_events": total,
        "curves": curves,
        "suggestions": suggestions,
    }))
}