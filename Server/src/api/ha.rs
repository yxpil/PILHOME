//! 全屋设备接管接口(HA 双向桥)。

use crate::state::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use pilhome_netlinker::DeviceKind;
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

/// GET /api/ha/entities —— HA 实体缓存与连接状态。
pub async fn entities(State(state): State<Arc<AppState>>) -> Json<Value> {
    let bridge = &state.ha;
    let last_sync = *bridge.last_sync.lock().expect("ha sync poisoned");
    Json(json!({
        "enabled": bridge.enabled(),
        "connected": bridge.connected.load(std::sync::atomic::Ordering::SeqCst),
        "entity_count": bridge.entity_count(),
        "last_sync": last_sync,
        "entities": bridge.list_entities(),
    }))
}

/// 控制请求体。
#[derive(Debug, Deserialize)]
pub struct ControlReq {
    /// 实体 ID(如 `light.living_room`)。
    pub entity_id: String,
    /// 动作:`on` / `off` / `toggle` / `lock` / `unlock` / `open` / `close`。
    pub action: String,
}

/// POST /api/ha/control —— 单实体控制。
pub async fn control(State(state): State<Arc<AppState>>, Json(req): Json<ControlReq>) -> (StatusCode, Json<Value>) {
    let ok = state.ha.control_entity(&req.entity_id, &req.action);
    if ok {
        state.audit.record("api", "ha.control", &req.entity_id, format!("action={}", req.action), crate::state::now_secs());
        (StatusCode::OK, Json(json!({ "ok": true, "entity_id": req.entity_id, "action": req.action })))
    } else {
        (StatusCode::BAD_REQUEST, Json(json!({ "ok": false, "error": "实体不存在或动作不支持(检查 HA 桥连接)" })))
    }
}

/// POST /api/ha/sync —— 触发一次全量同步。
pub async fn sync(State(state): State<Arc<AppState>>) -> Json<Value> {
    let ok = state.ha.request_sync();
    Json(json!({ "ok": ok, "note": if ok { "已请求同步" } else { "HA 桥未连接" } }))
}

/// 场景请求体。
#[derive(Debug, Deserialize)]
pub struct SceneReq {
    /// 场景名:`arm`(布防)/ `disarm`(撤防)/ `away`(离家)/ `home`(回家)。
    pub scene: String,
}

/// POST /api/ha/scenes —— 全屋场景一键执行。
pub async fn scenes(State(state): State<Arc<AppState>>, Json(req): Json<SceneReq>) -> Json<Value> {
    let bridge = &state.ha;
    if !bridge.connected.load(std::sync::atomic::Ordering::SeqCst) {
        return Json(json!({ "ok": false, "error": "HA 桥未连接" }));
    }

    let entities = bridge.list_entities();
    let mut executed = 0usize;

    // 按场景批量下发(以实体领域 + 设备种类匹配)。
    for entity in &entities {
        let kind = kind_of(entity);
        let action = match (req.scene.as_str(), kind) {
            // 布防:锁门、关窗帘、关灯
            ("arm" | "away", DeviceKind::Lock) => Some("lock"),
            ("arm" | "away", DeviceKind::Cover) => Some("close"),
            ("arm" | "away", DeviceKind::Light | DeviceKind::Switch | DeviceKind::Fan) => Some("off"),
            // 撤防:开锁、开窗帘
            ("disarm" | "home", DeviceKind::Lock) => Some("unlock"),
            ("disarm" | "home", DeviceKind::Cover) => Some("open"),
            _ => None,
        };
        if let Some(action) = action {
            if bridge.control_entity(&entity.entity_id, action) {
                executed += 1;
            }
        }
    }

    state
        .audit
        .record("api", "ha.scene", &req.scene, format!("执行 {executed} 个实体操作"), crate::state::now_secs());
    Json(json!({ "ok": true, "scene": req.scene, "executed": executed }))
}

/// 实体 -> 设备种类(与 ha_bridge 内部映射保持一致)。
fn kind_of(entity: &crate::ha_bridge::HaEntity) -> DeviceKind {
    match entity.domain.as_str() {
        "light" => DeviceKind::Light,
        "switch" => DeviceKind::Switch,
        "climate" => DeviceKind::Climate,
        "cover" => DeviceKind::Cover,
        "fan" => DeviceKind::Fan,
        "siren" => DeviceKind::Siren,
        "lock" => DeviceKind::Lock,
        "camera" => DeviceKind::Camera,
        _ => DeviceKind::Unknown,
    }
}