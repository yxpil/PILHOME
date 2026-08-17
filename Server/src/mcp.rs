//! MCP (Model Context Protocol) 接入端点 —— 智能体接入网关。
//!
//! 实现 JSON-RPC 2.0 子集:`initialize` / `tools/list` / `tools/call`,
//! 供 Claude Desktop、Cursor 等 MCP 客户端(Streamable HTTP 传输)接入,
//! 以工具调用方式操作 PILHOME(查设备/触发发现/执行场景/唤醒主机…)。
//!
//! 配置示例(Claude Desktop `claude_desktop_config.json`):
//! ```json
//! { "mcpServers": { "pilhome": { "url": "http://127.0.0.1:8080/api/mcp" } } }
//! ```

use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

/// JSON-RPC 请求。
#[derive(Debug, Deserialize)]
pub struct RpcReq {
    pub jsonrpc: Option<String>,
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

/// 工具定义。
fn tool(name: &str, description: &str, schema: Value) -> Value {
    json!({ "name": name, "description": description, "inputSchema": schema })
}

/// 工具注册表。
fn tools() -> Vec<Value> {
    vec![
        tool(
            "list_devices",
            "列出全部设备台账(类型/状态/MAC/厂商)",
            json!({ "type": "object", "properties": {} }),
        ),
        tool(
            "get_status",
            "获取系统状态(设备数/在线数/事件数/模块数/存储)",
            json!({ "type": "object", "properties": {} }),
        ),
        tool(
            "get_events",
            "获取最近事件(可指定条数)",
            json!({ "type": "object", "properties": { "limit": { "type": "integer" } } }),
        ),
        tool(
            "run_scene",
            "执行全屋场景(arm/disarm/away/home)",
            json!({ "type": "object", "properties": { "scene": { "type": "string", "enum": ["arm", "disarm", "away", "home"] } }, "required": ["scene"] }),
        ),
        tool(
            "discover",
            "触发一轮全自动设备发现(扫描+ARP+mDNS+指纹)",
            json!({ "type": "object", "properties": {} }),
        ),
        tool(
            "wol_wake",
            "WOL 网络唤醒(按 MAC 或设备 ID)",
            json!({ "type": "object", "properties": { "mac": { "type": "string" }, "device_id": { "type": "string" } } }),
        ),
        tool(
            "ha_control",
            "控制 HA 实体(如开灯/锁门)",
            json!({ "type": "object", "properties": { "entity_id": { "type": "string" }, "action": { "type": "string" } }, "required": ["entity_id", "action"] }),
        ),
        tool(
            "add_automation",
            "新增自动化规则(事件触发→动作)",
            json!({ "type": "object", "properties": { "name": { "type": "string" }, "device_id": { "type": "string" }, "key": { "type": "string" }, "value": { "type": "string" }, "action_type": { "type": "string" }, "message": { "type": "string" } }, "required": ["name", "action_type"] }),
        ),
    ]
}

/// 执行工具调用。
async fn call_tool(state: &Arc<AppState>, name: &str, params: &Value) -> Result<Value, String> {
    match name {
        "list_devices" => {
            let devices: Vec<Value> = state.devices.all().iter().map(|d| {
                json!({ "id": d.id, "name": d.name, "kind": d.kind.as_str(), "state": format!("{:?}", d.state).to_lowercase(), "address": d.address, "attrs": d.attrs })
            }).collect();
            Ok(json!({ "total": devices.len(), "devices": devices }))
        }
        "get_status" => {
            let now = crate::state::now_secs();
            Ok(json!({
                "devices": state.devices.len(),
                "online": state.devices.online_count(now, 120),
                "events": state.events.len(),
                "modules": state.coord.registry.all().len(),
                "sqlite_available": state.sqlite.available(),
                "sqlite_events": state.sqlite.count(),
                "ha_connected": state.ha.connected.load(std::sync::atomic::Ordering::SeqCst),
                "uptime": now - state.started_at,
            }))
        }
        "get_events" => {
            let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(50).min(500) as usize;
            let events: Vec<Value> = state.events.recent(limit).iter().map(|e| {
                json!({ "ts": e.ts, "level": format!("{:?}", e.level).to_lowercase(), "source": e.source, "message": e.message, "device_id": e.device_id })
            }).collect();
            Ok(json!({ "total": events.len(), "events": events }))
        }
        "run_scene" => {
            let scene = params.get("scene").and_then(|v| v.as_str()).ok_or("缺少 scene 参数")?;
            if !["arm", "disarm", "away", "home"].contains(&scene) {
                return Err(format!("未知场景:{scene}"));
            }
            // 复用 HA 场景执行逻辑(未连接 HA 时返回提示)。
            let connected = state.ha.connected.load(std::sync::atomic::Ordering::SeqCst);
            if !connected {
                return Ok(json!({ "ok": false, "note": "HA 桥未连接,场景未执行" }));
            }
            let entities = state.ha.list_entities();
            let mut executed = 0usize;
            for entity in &entities {
                let action = match (scene, entity.domain.as_str()) {
                    ("arm" | "away", "lock") => Some("lock"),
                    ("arm" | "away", "cover") => Some("close"),
                    ("arm" | "away", "light" | "switch" | "fan") => Some("off"),
                    ("disarm" | "home", "lock") => Some("unlock"),
                    ("disarm" | "home", "cover") => Some("open"),
                    _ => None,
                };
                if let Some(a) = action {
                    if state.ha.control_entity(&entity.entity_id, a) {
                        executed += 1;
                    }
                }
            }
            Ok(json!({ "ok": true, "scene": scene, "executed": executed }))
        }
        "discover" => {
            let state2 = state.clone();
            let report = tokio::task::spawn_blocking(move || crate::discover::auto_discover(&state2))
                .await
                .map_err(|e| format!("发现任务失败:{e}"))?;
            Ok(json!({ "ok": true, "report": report }))
        }
        "wol_wake" => {
            let mac = match params.get("mac").and_then(|v| v.as_str()) {
                Some(m) => m.to_string(),
                None => {
                    let id = params.get("device_id").and_then(|v| v.as_str()).ok_or("缺少 mac 或 device_id")?;
                    state.devices.get(id).and_then(|d| d.attrs.get("mac").cloned()).ok_or("设备不存在或未记录 MAC")?
                }
            };
            let mac_bytes = pilhome_netlinker::parse_mac(&mac).ok_or("MAC 格式无效")?;
            match pilhome_netlinker::send_wol(mac_bytes, std::net::Ipv4Addr::BROADCAST, 9) {
                Ok(()) => Ok(json!({ "ok": true, "mac": mac })),
                Err(e) => Err(format!("发送失败:{e}")),
            }
        }
        "ha_control" => {
            let entity_id = params.get("entity_id").and_then(|v| v.as_str()).ok_or("缺少 entity_id")?;
            let action = params.get("action").and_then(|v| v.as_str()).ok_or("缺少 action")?;
            let ok = state.ha.control_entity(entity_id, action);
            Ok(json!({ "ok": ok, "entity_id": entity_id, "action": action }))
        }
        "add_automation" => {
            let name = params.get("name").and_then(|v| v.as_str()).ok_or("缺少 name")?.to_string();
            let action_type = params.get("action_type").and_then(|v| v.as_str()).ok_or("缺少 action_type")?.to_string();
            let action = match action_type.as_str() {
                "log" => crate::automation::Action::Log { message: params.get("message").and_then(|v| v.as_str()).unwrap_or("").to_string() },
                other => return Err(format!("MCP 暂只支持 log 动作,收到:{other}")),
            };
            let rule = crate::automation::AutomationRule::new(name, crate::automation::Trigger {
                device_id: params.get("device_id").and_then(|v| v.as_str()).map(|s| s.to_string()).filter(|s| !s.is_empty()),
                key: params.get("key").and_then(|v| v.as_str()).map(|s| s.to_string()).filter(|s| !s.is_empty()),
                value: params.get("value").and_then(|v| v.as_str()).map(|s| s.to_string()).filter(|s| !s.is_empty()),
                min_level: None,
            }, None, vec![action]);
            if let Ok(mut rules) = state.automations.write() {
                rules.push(rule.clone());
            }
            Ok(json!({ "ok": true, "rule_id": rule.id }))
        }
        _ => Err(format!("未知工具:{name}")),
    }
}

/// POST /api/mcp —— MCP JSON-RPC 端点。
pub async fn handler(State(state): State<Arc<AppState>>, Json(req): Json<RpcReq>) -> Json<Value> {
    // 校验 JSON-RPC 版本(宽松:缺失/2.0 均可,其余报错)。
    if let Some(ver) = &req.jsonrpc {
        if ver != "2.0" {
            let id = req.id.clone().unwrap_or(Value::Null);
            return Json(json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32600, "message": "Invalid Request: jsonrpc must be 2.0" } }));
        }
    }
    let id = req.id.clone().unwrap_or(Value::Null);
    let result = match req.method.as_str() {
        "initialize" => Ok(json!({
            "protocolVersion": "2025-03-26",
            "capabilities": { "tools": { "listChanged": false } },
            "serverInfo": { "name": "pilhome-gateway", "version": env!("CARGO_PKG_VERSION") },
        })),
        "tools/list" => Ok(json!({ "tools": tools() })),
        "tools/call" => {
            let name = req.params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let args = req.params.get("arguments").cloned().unwrap_or(Value::Null);
            match call_tool(&state, name, &args).await {
                Ok(content) => Ok(json!({ "content": [ { "type": "text", "text": content.to_string() } ], "isError": false })),
                Err(msg) => Ok(json!({ "content": [ { "type": "text", "text": msg } ], "isError": true })),
            }
        }
        "ping" => Ok(Value::Null),
        _ => Err(json!({ "code": -32601, "message": format!("Method not found: {}", req.method) })),
    };

    match result {
        Ok(r) => Json(json!({ "jsonrpc": "2.0", "id": id, "result": r })),
        Err(e) => Json(json!({ "jsonrpc": "2.0", "id": id, "error": e })),
    }
}