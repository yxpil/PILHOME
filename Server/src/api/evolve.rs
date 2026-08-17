//! 规则进化接口(ATOGrowUP 变异器)。

use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use pilhome_atogrowup::{RuleGene, RuleKind};
use serde_json::{Value, json};
use std::sync::Arc;

/// GET /api/evolve —— 当前规则种群与代数。
pub async fn view(State(state): State<Arc<AppState>>) -> Json<Value> {
    let (population, generation) = state
        .evolver
        .lock()
        .map(|e| (e.population().to_vec(), e.generation()))
        .unwrap_or_default();
    Json(json!({ "generation": generation, "population": population, "total": population.len() }))
}

/// POST /api/evolve/seed —— 注入初始规则种群。
pub async fn seed(State(state): State<Arc<AppState>>) -> Json<Value> {
    let seeds = vec![
        RuleGene::new("arm_night", RuleKind::Arm, 23, 6),
        RuleGene::new("disarm_morning", RuleKind::Disarm, 7, 8),
        RuleGene::new("patrol_midday", RuleKind::Patrol, 12, 13),
    ];
    if let Ok(mut evolver) = state.evolver.lock() {
        evolver.seed(seeds);
    }
    state
        .audit
        .record("api", "evolve.seed", "atogrowup", "注入初始规则种群", crate::state::now_secs());
    Json(json!({ "ok": true }))
}

/// POST /api/evolve/tick —— 执行一代变异。
pub async fn tick(State(state): State<Arc<AppState>>) -> Json<Value> {
    let (n, generation) = state
        .evolver
        .lock()
        .map(|mut e| (e.evolve(), e.generation()))
        .unwrap_or((0, 0));
    Json(json!({ "ok": true, "population": n, "generation": generation }))
}