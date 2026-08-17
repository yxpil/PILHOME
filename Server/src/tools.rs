//! 工具系统:内置工具 + 动态 JS 工具。
//!
//! - 内置工具:设备控制 / MQTT / WOL / 贝叶斯预测 / 事件上报 / 媒体发现 / TTS / 图像生成;
//! - 动态工具:AI 编写 JS 脚本注册为工具(经 rquickjs 沙箱执行);
//! - 搜索:按名称/描述关键词匹配;错误:记录最近一次报错,可调 AI 分析修复建议。

use crate::state::AppState;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::sync::RwLock;

/// 工具类型。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ToolKind {
    /// 内置工具。
    Builtin,
    /// 用户/AI 创建的 JS 工具。
    Js,
}

/// 工具定义。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub id: String,
    pub name: String,
    pub description: String,
    pub kind: ToolKind,
    /// JS 源码(仅 Js 工具)。
    #[serde(default)]
    pub js_code: Option<String>,
    /// 最近一次执行错误(供调试)。
    #[serde(default)]
    pub last_error: Option<String>,
    /// 最近一次执行时间(Unix 秒)。
    #[serde(default)]
    pub last_run: u64,
    /// 执行次数。
    #[serde(default)]
    pub run_count: u64,
}

/// 工具注册表。
pub struct ToolRegistry {
    tools: RwLock<Vec<Tool>>,
}

impl ToolRegistry {
    /// 创建并注册内置工具。
    pub fn new() -> Self {
        let registry = Self { tools: RwLock::new(Vec::new()) };
        for (id, name, desc) in [
            ("list_devices", "设备列表", "列出全部设备台账(含状态/种类/属性)"),
            ("device_command", "设备控制", "控制设备:on/off/toggle/lock/unlock/open/close"),
            ("publish_mqtt", "MQTT 发布", "向 MQTT 主题发布消息"),
            ("wol", "网络唤醒", "发送 WOL 魔法包唤醒主机(需 MAC)"),
            ("bayes_predict", "习惯置信度", "朴素贝叶斯预测用户接受某动作的概率(习惯放行判断)"),
            ("send_event", "事件上报", "写入一条系统事件"),
            ("media_discover", "媒体发现", "发现音响/机顶盒/投屏设备(mDNS)"),
            ("tts", "语音合成", "TTS 播报一段文字"),
            ("generate_image", "图像生成", "调用 ComfyUI(Stable Diffusion)生成图片"),
            ("ai_analyze", "AI 研判", "把事件交给 AI 分析(传感器阈值等)"),
            ("http_get", "HTTP 请求", "GET 一个 URL 返回文本"),
        ] {
            registry.tools.write().expect("tools poisoned").push(Tool {
                id: id.into(),
                name: name.into(),
                description: desc.into(),
                kind: ToolKind::Builtin,
                js_code: None,
                last_error: None,
                last_run: 0,
                run_count: 0,
            });
        }
        registry
    }

    /// 全部工具。
    pub fn list(&self) -> Vec<Tool> {
        self.tools.read().expect("tools poisoned").clone()
    }

    /// 关键词搜索(名称/描述/id)。
    pub fn search(&self, query: &str) -> Vec<Tool> {
        let q = query.to_lowercase();
        self.list().into_iter().filter(|t| {
            q.is_empty()
                || t.name.to_lowercase().contains(&q)
                || t.description.to_lowercase().contains(&q)
                || t.id.to_lowercase().contains(&q)
        }).collect()
    }

    /// 新增 JS 工具。
    pub fn add_js(&self, name: &str, description: &str, js_code: &str) -> Result<Tool, String> {
        if js_code.trim().is_empty() {
            return Err("JS 代码不能为空".into());
        }
        let id = format!("js_{}", crate::state::now_secs() % 100_000_000);
        let tool = Tool {
            id: id.clone(),
            name: name.into(),
            description: description.into(),
            kind: ToolKind::Js,
            js_code: Some(js_code.into()),
            last_error: None,
            last_run: 0,
            run_count: 0,
        };
        self.tools.write().expect("tools poisoned").push(tool.clone());
        Ok(tool)
    }

    /// 删除工具(内置不可删)。
    pub fn remove(&self, id: &str) -> bool {
        let mut guard = self.tools.write().expect("tools poisoned");
        let Some(tool) = guard.iter().find(|t| t.id == id).cloned() else { return false };
        if tool.kind == ToolKind::Builtin {
            return false;
        }
        guard.retain(|t| t.id != id);
        true
    }

    /// 执行工具。返回 (结果, 错误)。
    pub fn run(&self, state: &std::sync::Arc<AppState>, id: &str, args: Value) -> Result<Value, String> {
        let tool = self.tools.read().expect("tools poisoned").iter().find(|t| t.id == id).cloned().ok_or("工具不存在")?;
        let result = match tool.kind {
            ToolKind::Builtin => run_builtin(state, &tool.id, &args),
            ToolKind::Js => {
                let script = tool.js_code.clone().ok_or("JS 工具缺少代码")?;
                let args_json = args.to_string();
                crate::js::run_script(state, &script, &args_json)
            }
        };
        // 记录执行状态。
        if let Ok(mut guard) = self.tools.write() {
            if let Some(t) = guard.iter_mut().find(|t| t.id == id) {
                t.run_count += 1;
                t.last_run = crate::state::now_secs();
                t.last_error = result.as_ref().err().cloned();
            }
        }
        result
    }
}


/// 内置工具分发。
fn run_builtin(state: &AppState, id: &str, args: &Value) -> Result<Value, String> {
    match id {
        "list_devices" => Ok(json!({ "devices": state.devices.all() })),
        "device_command" => {
            let id = args.get("id").and_then(|v| v.as_str()).ok_or("缺少 id")?;
            let action = args.get("action").and_then(|v| v.as_str()).ok_or("缺少 action")?;
            builtin_device_command(state, id, action).map(|r| json!({ "ok": true, "result": r }))
        }
        "publish_mqtt" => {
            let topic = args.get("topic").and_then(|v| v.as_str()).ok_or("缺少 topic")?;
            let payload = args.get("payload").map(|v| v.to_string()).unwrap_or_default();
            let ok = state.mqtt_tx.lock().ok().and_then(|g| g.as_ref().map(|tx| tx.try_send(crate::mqtt::MqttCommand { topic: topic.into(), payload }).is_ok())).unwrap_or(false);
            if ok { Ok(json!({ "ok": true })) } else { Err("MQTT 未连接".into()) }
        }
        "wol" => {
            let mac = args.get("mac").and_then(|v| v.as_str()).ok_or("缺少 mac")?;
            let m = pilhome_netlinker::parse_mac(mac).ok_or("MAC 格式无效")?;
            pilhome_netlinker::send_wol(m, "255.255.255.255".parse().unwrap(), 9).map_err(|e| e.to_string())?;
            Ok(json!({ "ok": true }))
        }
        "bayes_predict" => {
            let feats: Vec<(String, String)> = args.get("features").and_then(|v| serde_json::from_value(v.clone()).ok()).unwrap_or_default();
            let feats_ref: Vec<(&str, &str)> = feats.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
            let pred = state.bayes.lock().expect("bayes poisoned").predict_with_meta(&feats_ref);
            Ok(json!({ "probability": pred.probability, "samples": pred.samples }))
        }
        "send_event" => {
            let level = args.get("level").and_then(|v| v.as_str()).unwrap_or("info");
            let source = args.get("source").and_then(|v| v.as_str()).unwrap_or("tool");
            let message = args.get("message").and_then(|v| v.as_str()).unwrap_or("");
            let lvl = match level { "alert" => crate::events::Level::Alert, "warn" => crate::events::Level::Warn, _ => crate::events::Level::Info };
            state.events.record(lvl, source, message, None, None);
            Ok(json!({ "ok": true }))
        }
        "media_discover" => Ok(json!({ "devices": crate::media::discover_media_devices() })),
        "tts" => {
            let text = args.get("text").and_then(|v| v.as_str()).ok_or("缺少 text")?;
            crate::tts::speak(state, text).map(|_| json!({ "ok": true }))
        }
        "generate_image" => {
            let prompt = args.get("prompt").and_then(|v| v.as_str()).ok_or("缺少 prompt")?;
            let negative = args.get("negative").and_then(|v| v.as_str());
            crate::sd::generate(state, prompt, negative).map(|r| json!({ "ok": true, "image": r }))
        }
        "ai_analyze" => {
            let event = args.get("event").cloned().unwrap_or(json!({}));
            let ctx = args.get("context").and_then(|v| v.as_str()).unwrap_or("");
            let out = state.ai.analyze(&event, ctx)?;
            Ok(json!({ "analysis": out }))
        }
        "http_get" => {
            let url = args.get("url").and_then(|v| v.as_str()).ok_or("缺少 url")?;
            ureq::get(url).timeout(std::time::Duration::from_secs(15)).call().map_err(|e| e.to_string())?.into_string().map(|s| json!({ "body": s })).map_err(|e| e.to_string())
        }
        other => Err(format!("未知工具:{other}")),
    }
}

/// 设备控制(内置):HA 桥优先,否则经 MQTT 下发命令主题。
pub fn builtin_device_command(state: &AppState, id: &str, action: &str) -> Result<(), String> {
    // 尝试 HA 桥。
    if state.ha.control_entity(id, action) {
        return Ok(());
    }
    // 尝试 MQTT 命令主题。
    let topic = format!("pilhome/{id}/command");
    let ok = state.mqtt_tx.lock().ok().and_then(|g| g.as_ref().map(|tx| tx.try_send(crate::mqtt::MqttCommand { topic, payload: action.into() }).is_ok())).unwrap_or(false);
    if ok { Ok(()) } else { Err("设备不可达(HA 未连接且 MQTT 未连接)".into()) }
}

/// 工具调试:分析最近一次报错(有 AI 则让 AI 给修复建议)。
pub fn debug_tool(state: &AppState, id: &str) -> Result<String, String> {
    let tools = state.tools.list();
    let tool = tools.iter().find(|t| t.id == id).ok_or("工具不存在")?;
    let error = tool.last_error.clone().ok_or("该工具暂无报错")?;
    if !state.ai.configured() {
        return Ok(format!("错误信息:{error}\n(AI 未配置,无法给出修复建议)"));
    }
    let code = tool.js_code.clone().unwrap_or_default();
    let prompt = format!(
        "工具 [{name}] 执行报错:\n{error}\n\nJS 源码:\n{code}\n\n请分析原因并给出修复后的完整 JS 代码(200 字内说明 + 代码)。",
        name = tool.name
    );
    state.ai.chat(&[crate::ai::ChatMsg { role: "user".into(), content: prompt }])
}

/// 工具状态 KV 的类型别名。
pub type ToolState = BTreeMap<String, String>;