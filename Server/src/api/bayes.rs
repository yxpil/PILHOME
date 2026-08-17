//! 贝叶斯习惯分类器接口。

use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

/// GET /api/bayes/status —— 分类器状态。
pub async fn status(State(state): State<Arc<AppState>>) -> Json<Value> {
    let b = state.bayes.lock().expect("bayes poisoned");
    Json(json!({
        "samples": b.total_pos + b.total_neg,
        "positive": b.total_pos,
        "negative": b.total_neg,
        "features": b.feature_count(),
    }))
}

/// POST /api/bayes/predict —— 预测用户接受某动作的置信度。
#[derive(Debug, Deserialize)]
pub struct PredictReq {
    /// 特征数组:如 [["action","light.off"],["hour","22"]]
    pub features: Vec<(String, String)>,
}

pub async fn predict(State(state): State<Arc<AppState>>, Json(req): Json<PredictReq>) -> Json<Value> {
    let feats: Vec<(&str, &str)> = req.features.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
    let b = state.bayes.lock().expect("bayes poisoned");
    let pred = b.predict_with_meta(&feats);
    Json(json!({
        "probability": pred.probability,
        "samples": pred.samples,
        "positive_samples": pred.positive_samples,
        "verdict": if pred.probability >= 0.6 { "放行" } else if pred.probability >= 0.4 { "存疑" } else { "建议人工确认" },
    }))
}

/// POST /api/bayes/train —— 手动训练一条样本。
#[derive(Debug, Deserialize)]
pub struct TrainReq {
    pub features: Vec<(String, String)>,
    pub positive: bool,
}

pub async fn train(State(state): State<Arc<AppState>>, Json(req): Json<TrainReq>) -> Json<Value> {
    let feats: Vec<(&str, &str)> = req.features.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
    let mut b = state.bayes.lock().expect("bayes poisoned");
    b.train(&feats, req.positive);
    Json(json!({ "ok": true, "samples": b.total_pos + b.total_neg }))
}