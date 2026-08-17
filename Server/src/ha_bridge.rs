//! Home Assistant 双向桥 —— 全屋设备接管。
//!
//! 方向一(下行同步):通过 HA WebSocket API 拉取**全部实体**状态,
//! 映射为 PILHOME 统一设备台账(灯/开关/温控/窗帘/门锁/传感器/摄像头…);
//! 方向二(上行控制):PILHOME 直接调用 HA 服务(turn_on/turn_off/
//! lock/unlock/close_cover/set_temperature…),实现对全屋设备的统一接管。
//!
//! 断线自动重连;实体缓存与设备台账实时同步。

use crate::state::AppState;
use futures_util::{SinkExt, StreamExt};
use pilhome_netlinker::{Device, DeviceKind, DeviceState};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;

/// HA 实体(与 HA `get_states` 返回项对齐)。
#[derive(Debug, Clone, serde::Serialize)]
pub struct HaEntity {
    /// 实体 ID,如 `light.living_room`。
    pub entity_id: String,
    /// 友好名称。
    pub name: String,
    /// 领域(light / switch / climate / lock …)。
    pub domain: String,
    /// 当前状态(on / off / open / locked …)。
    pub state: String,
    /// 关键属性(brightness / temperature / device_class …)。
    pub attributes: BTreeMap<String, Value>,
}

/// 桥命令:上行请求。
#[derive(Debug, Clone)]
pub enum HaCommand {
    /// 调用 HA 服务。
    CallService { domain: String, service: String, data: Value },
    /// 拉取全量状态。
    GetStates,
}

/// HA 双向桥。
pub struct HaBridge {
    ws_url: String,
    token: String,
    /// 实体缓存。
    pub entities: RwLock<Vec<HaEntity>>,
    cmd_tx: Mutex<Option<mpsc::UnboundedSender<HaCommand>>>,
    /// 连接状态。
    pub connected: AtomicBool,
    /// 最近同步时间(Unix 秒)。
    pub last_sync: Mutex<u64>,
}

impl HaBridge {
    /// 创建桥(不立即连接)。
    pub fn new(ws_url: impl Into<String>, token: impl Into<String>) -> Self {
        Self {
            ws_url: ws_url.into(),
            token: token.into(),
            entities: RwLock::new(Vec::new()),
            cmd_tx: Mutex::new(None),
            connected: AtomicBool::new(false),
            last_sync: Mutex::new(0),
        }
    }

    /// 桥是否启用(配置了 ws 地址)。
    pub fn enabled(&self) -> bool {
        !self.ws_url.is_empty()
    }

    /// 全量实体列表(按 entity_id 排序)。
    pub fn list_entities(&self) -> Vec<HaEntity> {
        let mut list = self.entities.read().expect("ha entities poisoned").clone();
        list.sort_by(|a, b| a.entity_id.cmp(&b.entity_id));
        list
    }

    /// 实体数量。
    pub fn entity_count(&self) -> usize {
        self.entities.read().expect("ha entities poisoned").len()
    }

    fn send(&self, cmd: HaCommand) -> bool {
        let guard = self.cmd_tx.lock().expect("ha cmd poisoned");
        match guard.as_ref() {
            Some(tx) => tx.send(cmd).is_ok(),
            None => false,
        }
    }

    /// 上行控制:调用 HA 服务(未连接时返回 false)。
    pub fn call_service(&self, domain: &str, service: &str, data: Value) -> bool {
        self.send(HaCommand::CallService {
            domain: domain.to_string(),
            service: service.to_string(),
            data,
        })
    }

    /// 便捷控制:按实体领域生成合适的服务调用。
    ///
    /// `action`: `on` / `off` / `toggle` / `lock` / `unlock` / `open` / `close`。
    pub fn control_entity(&self, entity_id: &str, action: &str) -> bool {
        let Some(entity) = self.list_entities().into_iter().find(|e| e.entity_id == entity_id) else {
            return false;
        };
        let (service, data) = match (entity.domain.as_str(), action) {
            ("lock", "lock") => ("lock", json!({ "entity_id": entity_id })),
            ("lock", "unlock") => ("unlock", json!({ "entity_id": entity_id })),
            ("cover", "open") => ("open_cover", json!({ "entity_id": entity_id })),
            ("cover", "close") => ("close_cover", json!({ "entity_id": entity_id })),
            (_, "on") => ("turn_on", json!({ "entity_id": entity_id })),
            (_, "off") => ("turn_off", json!({ "entity_id": entity_id })),
            (_, "toggle") => ("toggle", json!({ "entity_id": entity_id })),
            _ => return false,
        };
        self.call_service(&entity.domain, service, data)
    }

    /// 主动触发一次全量同步。
    pub fn request_sync(&self) -> bool {
        self.send(HaCommand::GetStates)
    }
}

/// 启动 HA 桥后台任务(自动重连,永不退出)。
pub fn spawn(state: Arc<AppState>, bridge: Arc<HaBridge>) {
    tokio::spawn(async move {
        if !bridge.enabled() {
            tracing::info!("HA 双向桥未配置(ws_url 为空),跳过全屋接管");
            return;
        }
        loop {
            if let Err(err) = run_once(&bridge, &state).await {
                tracing::warn!("HA 连接断开:{err};5 秒后重连");
            }
            bridge.connected.store(false, Ordering::SeqCst);
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    });
}

/// 单次连接会话:认证 -> 拉取状态 -> 持续处理命令。
async fn run_once(bridge: &Arc<HaBridge>, state: &Arc<AppState>) -> Result<(), String> {
    let (mut ws, _) = tokio_tungstenite::connect_async(&bridge.ws_url)
        .await
        .map_err(|e| format!("连接失败:{e}"))?;

    // 认证。
    let auth = json!({ "type": "auth", "access_token": bridge.token }).to_string();
    ws.send(Message::Text(auth.into())).await.map_err(|e| format!("发送认证失败:{e}"))?;

    let (tx, mut rx) = mpsc::unbounded_channel::<HaCommand>();
    *bridge.cmd_tx.lock().expect("ha cmd poisoned") = Some(tx);
    bridge.connected.store(true, Ordering::SeqCst);

    let mut msg_id: u64 = 1;

    loop {
        tokio::select! {
            cmd = rx.recv() => {
                let Some(cmd) = cmd else { break };
                match cmd {
                    HaCommand::GetStates => {
                        let payload = json!({ "id": 1u64, "type": "get_states" }).to_string();
                        ws.send(Message::Text(payload.into())).await.map_err(|e| format!("发送同步失败:{e}"))?;
                    }
                    HaCommand::CallService { domain, service, data } => {
                        msg_id += 1;
                        let payload = json!({
                            "id": msg_id,
                            "type": "call_service",
                            "domain": domain,
                            "service": service,
                            "service_data": data,
                        })
                        .to_string();
                        ws.send(Message::Text(payload.into())).await.map_err(|e| format!("发送命令失败:{e}"))?;
                    }
                }
            }
            msg = ws.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        handle_message(&bridge, state, &text).await;
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Err(e)) => return Err(format!("WS 错误:{e}")),
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

/// 处理一条 WS 文本消息(auth 结果 / get_states 响应)。
async fn handle_message(bridge: &HaBridge, state: &AppState, text: &str) {
    let Ok(msg) = serde_json::from_str::<Value>(text) else { return };

    // 认证成功 -> 拉取全量状态。
    if msg.get("type").and_then(|v| v.as_str()) == Some("auth_ok") {
        tracing::info!("HA 认证成功,开始全屋设备同步");
        bridge.request_sync();
        return;
    }

    // get_states 响应 -> 更新实体缓存与设备台账。
    if msg.get("id").and_then(|v| v.as_u64()) == Some(1) {
        if let Some(entities) = msg.get("result").and_then(|v| v.as_array()) {
            let parsed: Vec<HaEntity> = entities.iter().filter_map(parse_entity).collect();
            *bridge.entities.write().expect("ha entities poisoned") = parsed.clone();
            *bridge.last_sync.lock().expect("ha sync poisoned") = crate::state::now_secs();
            sync_devices(state, &parsed);
            tracing::info!("已同步 HA 全屋设备:{} 个实体", parsed.len());
        }
    }
}

/// 解析单个 HA 实体。
fn parse_entity(raw: &Value) -> Option<HaEntity> {
    let entity_id = raw.get("entity_id")?.as_str()?.to_string();
    let attributes = raw.get("attributes")?.as_object()?;
    let name = attributes
        .get("friendly_name")
        .and_then(|v| v.as_str())
        .unwrap_or(&entity_id)
        .to_string();
    let domain = entity_id.split('.').next().unwrap_or("unknown").to_string();
    let mut attrs = BTreeMap::new();
    for (k, v) in attributes {
        if matches!(k.as_str(), "friendly_name" | "supported_features") {
            continue;
        }
        attrs.insert(k.clone(), v.clone());
    }
    Some(HaEntity {
        entity_id,
        name,
        domain,
        state: raw.get("state").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
        attributes: attrs,
    })
}

/// 将 HA 实体同步进 PILHOME 设备台账。
fn sync_devices(state: &AppState, entities: &[HaEntity]) {
    let now = crate::state::now_secs();
    for entity in entities {
        let kind = kind_from_domain(entity);
        let device_state = if entity.state == "unavailable" || entity.state == "unknown" {
            DeviceState::Offline
        } else {
            DeviceState::Online
        };
        let mut attrs = BTreeMap::new();
        attrs.insert("domain".to_string(), entity.domain.clone());
        attrs.insert("ha_state".to_string(), entity.state.clone());
        if let Some(cls) = entity.attributes.get("device_class").and_then(|v| v.as_str()) {
            attrs.insert("device_class".to_string(), cls.to_string());
        }
        if let Some(b) = entity.attributes.get("brightness").and_then(|v| v.as_u64()) {
            attrs.insert("brightness".to_string(), b.to_string());
        }
        if let Some(t) = entity.attributes.get("temperature").and_then(|v| v.as_f64()) {
            attrs.insert("temperature".to_string(), t.to_string());
        }

        let mut device = Device::new(&entity.entity_id, &entity.name, kind, format!("ha://{}", entity.entity_id));
        device.state = device_state;
        device.last_seen = now;
        device.attrs = attrs;
        state.devices.upsert(device);
    }
}

/// HA 领域 -> PILHOME 设备种类。
fn kind_from_domain(entity: &HaEntity) -> DeviceKind {
    match entity.domain.as_str() {
        "light" => DeviceKind::Light,
        "switch" => DeviceKind::Switch,
        "climate" => DeviceKind::Climate,
        "cover" => DeviceKind::Cover,
        "fan" => DeviceKind::Fan,
        "siren" => DeviceKind::Siren,
        "lock" => DeviceKind::Lock,
        "camera" => DeviceKind::Camera,
        "binary_sensor" => match entity
            .attributes
            .get("device_class")
            .and_then(|v| v.as_str())
            .unwrap_or("")
        {
            "motion" | "presence" | "occupancy" => DeviceKind::Presence,
            "door" | "window" | "opening" | "garage_door" => DeviceKind::Contact,
            "smoke" | "gas" | "water" | "moisture" => DeviceKind::Sensor,
            _ => DeviceKind::Sensor,
        },
        "sensor" => DeviceKind::Sensor,
        _ => DeviceKind::Unknown,
    }
}