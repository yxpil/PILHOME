//! 自动化规则接口(内置自动化引擎)。

use crate::automation::{Action, AutomationRule, Trigger};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

/// GET /api/automations —— 规则列表。
pub async fn list(State(state): State<Arc<AppState>>) -> Json<Value> {
    let rules = state.automations.read().map(|r| r.clone()).unwrap_or_default();
    Json(json!({ "automations": rules, "total": rules.len() }))
}

/// 新建规则请求体。
#[derive(Debug, Deserialize)]
pub struct AddReq {
    pub name: String,
    #[serde(default)]
    pub device_id: Option<String>,
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub value: Option<String>,
    /// 动作:log / mqtt / ha / webhook。
    pub action_type: String,
    /// 动作参数(MQTT:topic+payload;HA:entity_id+action;webhook:url;log:message)。
    #[serde(default)]
    pub topic: Option<String>,
    #[serde(default)]
    pub payload: Option<String>,
    #[serde(default)]
    pub entity_id: Option<String>,
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
}

/// POST /api/automations —— 新增规则。
pub async fn add(State(state): State<Arc<AppState>>, Json(req): Json<AddReq>) -> (StatusCode, Json<Value>) {
    let trigger = Trigger {
        device_id: req.device_id.filter(|s| !s.is_empty()),
        key: req.key.filter(|s| !s.is_empty()),
        value: req.value.filter(|s| !s.is_empty()),
        min_level: None,
    };
    let action = match req.action_type.as_str() {
        "log" => Action::Log { message: req.message.unwrap_or_default() },
        "mqtt" => Action::Mqtt { topic: req.topic.unwrap_or_default(), payload: req.payload.unwrap_or_default() },
        "ha" => Action::Ha { entity_id: req.entity_id.unwrap_or_default(), action: req.action.unwrap_or_default() },
        "webhook" => Action::Webhook { url: req.url.unwrap_or_default() },
        other => return (StatusCode::BAD_REQUEST, Json(json!({ "error": format!("未知动作类型:{other}") }))),
    };
    let rule = AutomationRule::new(&req.name, trigger, None, vec![action]);
    if let Ok(mut rules) = state.automations.write() {
        rules.push(rule.clone());
    }
    state.audit.record("api", "automation.add", &rule.id, &req.name, crate::state::now_secs());
    (StatusCode::OK, Json(json!({ "ok": true, "rule": rule })))
}

/// POST /api/automations/{id}/toggle —— 启用/停用。
pub async fn toggle(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> (StatusCode, Json<Value>) {
    if let Ok(mut rules) = state.automations.write() {
        if let Some(rule) = rules.iter_mut().find(|r| r.id == id) {
            rule.enabled = !rule.enabled;
            return (StatusCode::OK, Json(json!({ "ok": true, "id": id, "enabled": rule.enabled })));
        }
    }
    (StatusCode::NOT_FOUND, Json(json!({ "error": "规则不存在" })))
}

/// DELETE /api/automations/{id} —— 删除规则。
pub async fn remove(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> (StatusCode, Json<Value>) {
    if let Ok(mut rules) = state.automations.write() {
        let before = rules.len();
        rules.retain(|r| r.id != id);
        if rules.len() < before {
            return (StatusCode::OK, Json(json!({ "ok": true })));
        }
    }
    (StatusCode::NOT_FOUND, Json(json!({ "error": "规则不存在" })))
}