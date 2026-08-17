//! 自动化引擎 —— 内置的"自动化"能力(等价于 HA 的 Automation)。
//!
//! 事件触发 + 时间窗口条件 + 多动作执行:
//! - 触发器:设备事件匹配(device_id / key / value / 最低级别);
//! - 条件:可选的时间窗口(小时区间);
//! - 动作:记事件 / 发 MQTT / 调 HA 服务 / 打 Webhook。
//!
//! 周期任务扫描事件流增量匹配,支持启用/停用。

use crate::events::Level;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// 规则 ID 自增。
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// 自动化规则。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationRule {
    /// 规则 ID。
    pub id: String,
    /// 规则名称。
    pub name: String,
    /// 是否启用。
    pub enabled: bool,
    /// 事件触发器。
    pub trigger: Trigger,
    /// 时间窗口条件(小时,`(start, end)`;空 = 全天)。
    pub window: Option<(u8, u8)>,
    /// 动作列表(顺序执行)。
    pub actions: Vec<Action>,
}

/// 事件触发器:全部字段为 None 时匹配任意事件。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Trigger {
    /// 设备 ID(如 `contact_front_door`;空 = 任意设备)。
    pub device_id: Option<String>,
    /// 事件键(如 `contact` / `motion`;空 = 任意)。
    pub key: Option<String>,
    /// 事件值(如 `open`;空 = 任意)。
    pub value: Option<String>,
    /// 最低事件级别(info / warn / alert)。
    pub min_level: Option<Level>,
}

/// 执行动作。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "args", rename_all = "snake_case")]
pub enum Action {
    /// 记录一条系统事件。
    Log { message: String },
    /// 发布 MQTT 消息。
    Mqtt { topic: String, payload: String },
    /// 调用 HA 服务(全屋接管)。
    Ha { entity_id: String, action: String },
    /// 发送 Webhook。
    Webhook { url: String },
}

impl AutomationRule {
    /// 新建规则(自动分配 ID)。
    pub fn new(name: impl Into<String>, trigger: Trigger, window: Option<(u8, u8)>, actions: Vec<Action>) -> Self {
        let id = format!("auto_{:04}", NEXT_ID.fetch_add(1, Ordering::SeqCst));
        Self { id, name: name.into(), enabled: true, trigger, window, actions }
    }

    /// 判断事件是否触发本规则。
    pub fn matches(&self, event: &crate::events::AppEvent, now: u64) -> bool {
        if !self.enabled {
            return false;
        }
        // 时间窗口。
        if let Some((start, end)) = self.window {
            let hour = (now / 3600 % 24) as u8;
            if hour < start || hour > end {
                return false;
            }
        }
        // 触发器匹配。
        let t = &self.trigger;
        if let Some(device) = &t.device_id {
            if event.device_id.as_deref() != Some(device.as_str()) {
                return false;
            }
        }
        if let Some(key) = &t.key {
            let event_key = event.data.as_ref().and_then(|d| d.get("key")).and_then(|v| v.as_str());
            if event_key != Some(key.as_str()) {
                return false;
            }
        }
        if let Some(value) = &t.value {
            let event_value = event.data.as_ref().and_then(|d| d.get("value")).and_then(|v| v.as_str());
            if event_value != Some(value.as_str()) {
                return false;
            }
        }
        if let Some(min) = t.min_level {
            if event.level < min {
                return false;
            }
        }
        true
    }
}

/// 执行一条规则的全体动作。
pub fn execute_rule(state: &AppState, rule: &AutomationRule, event: &crate::events::AppEvent) {
    for action in &rule.actions {
        match action {
            Action::Log { message } => {
                state.events.record(
                    Level::Info,
                    "automation",
                    format!("[{}] {}", rule.name, message),
                    event.device_id.as_deref(),
                    Some(json!({ "rule": rule.id })),
                );
            }
            Action::Mqtt { topic, payload } => {
                if let Ok(guard) = state.mqtt_tx.lock() {
                    if let Some(tx) = guard.as_ref() {
                        let _ = tx.send(crate::mqtt::MqttCommand { topic: topic.clone(), payload: payload.clone() });
                    }
                }
            }
            Action::Ha { entity_id, action } => {
                state.ha.control_entity(entity_id, action);
            }
            Action::Webhook { url } => {
                crate::webhook::Webhook::new(url.clone()).call_now(event);
            }
        }
    }
}

/// 周期扫描任务:对事件流增量匹配自动化规则。
pub fn spawn_engine(state: Arc<AppState>, tick_secs: u64) {
    tokio::spawn(async move {
        let mut last_ts: u64 = 0;
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(tick_secs.max(2)));
        loop {
            interval.tick().await;
            let now = crate::state::now_secs();
            // 取最近事件,过滤已处理过的。
            let candidates: Vec<crate::events::AppEvent> = state
                .events
                .recent(200)
                .into_iter()
                .rev()
                .filter(|e| e.ts > last_ts)
                .collect();
            if let Some(max_ts) = candidates.iter().map(|e| e.ts).max() {
                last_ts = max_ts;
            }
            let rules = state.automations.read().map(|r| r.clone()).unwrap_or_default();
            for event in candidates {
                for rule in &rules {
                    if rule.matches(&event, now) {
                        tracing::info!("自动化触发:[{}] 命中事件 {}", rule.name, event.message);
                        execute_rule(&state, rule, &event);
                    }
                }
            }
        }
    });
}