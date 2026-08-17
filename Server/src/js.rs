//! JS 解释器:基于 rquickjs(QuickJS)的脚本沙箱。
//!
//! AI 可编写 JS 工具/流程,注入的安全 API:
//! `log` / `listDevices` / `deviceCommand` / `publishMqtt` / `wol` /
//! `bayesPredict` / `httpGet` / `sendEvent` / `tts` / `generateImage` /
//! `getState` / `setState`。所有参数与返回值均以字符串 JSON 传递。

use crate::state::AppState;
use rquickjs::function::Func;
use rquickjs::{Context, Runtime};
use serde_json::{Value, json};
use std::sync::Arc;

/// 执行 JS 脚本,返回 JSON 字符串结果(或错误文本)。
pub fn run_script(state: &Arc<AppState>, script: &str, args_json: &str) -> Result<Value, String> {
    let rt = Runtime::new().map_err(|e| format!("创建 JS 运行时失败:{e}"))?;
    let ctx = Context::full(&rt).map_err(|e| format!("创建 JS 上下文失败:{e}"))?;
    let state2 = state.clone();

    ctx.with(|ctx| {
        let globals = ctx.globals();
        let state3 = state2.clone();

        // 基础日志。
        globals
            .set("log", Func::new(move |msg: String| { tracing::info!("[JS] {msg}"); }))
            .map_err(|e| e.to_string())?;

        // 设备列表。
        let s = state3.clone();
        globals
            .set("listDevices", Func::new(move || -> String {
                json!(s.devices.all()).to_string()
            }))
            .map_err(|e| e.to_string())?;

        // 设备控制。
        let s = state3.clone();
        globals
            .set(
                "deviceCommand",
                Func::new(move |id: String, action: String| -> String {
                    let ok = s.ha.control_entity(&id, &action) || crate::tools::builtin_device_command(&s, &id, &action).is_ok();
                    json!({ "ok": ok, "id": id, "action": action }).to_string()
                }),
            )
            .map_err(|e| e.to_string())?;

        // MQTT 发布。
        let s = state3.clone();
        globals
            .set(
                "publishMqtt",
                Func::new(move |topic: String, payload: String| -> String {
                    let ok = s
                        .mqtt_tx
                        .lock()
                        .ok()
                        .and_then(|g| g.as_ref().map(|tx| tx.try_send(crate::mqtt::MqttCommand { topic: topic.clone(), payload: payload.clone() }).is_ok()))
                        .unwrap_or(false);
                    json!({ "ok": ok }).to_string()
                }),
            )
            .map_err(|e| e.to_string())?;

        // WOL 唤醒。
        globals
            .set(
                "wol",
                Func::new(move |mac: String| -> String {
                    let ok = pilhome_netlinker::parse_mac(&mac)
                        .map(|m| pilhome_netlinker::send_wol(m, "255.255.255.255".parse().unwrap(), 9).is_ok())
                        .unwrap_or(false);
                    json!({ "ok": ok }).to_string()
                }),
            )
            .map_err(|e| e.to_string())?;

        // 贝叶斯预测(用户习惯置信度)。
        let s = state3.clone();
        globals
            .set(
                "bayesPredict",
                Func::new(move |json_str: String| -> String {
                    let feats: Vec<(String, String)> = serde_json::from_str(&json_str).unwrap_or_default();
                    let feats_ref: Vec<(&str, &str)> = feats.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
                    let p = s.bayes.lock().expect("bayes poisoned").predict(&feats_ref);
                    json!({ "probability": p }).to_string()
                }),
            )
            .map_err(|e| e.to_string())?;

        // HTTP GET。
        globals
            .set(
                "httpGet",
                Func::new(move |url: String| -> String {
                    match ureq::get(&url).timeout(std::time::Duration::from_secs(15)).call() {
                        Ok(resp) => resp.into_string().unwrap_or_default(),
                        Err(e) => json!({ "error": e.to_string() }).to_string(),
                    }
                }),
            )
            .map_err(|e| e.to_string())?;

        // 事件上报。
        let s = state3.clone();
        globals
            .set(
                "sendEvent",
                Func::new(move |level: String, source: String, message: String| -> String {
                    let lvl = match level.as_str() {
                        "alert" => crate::events::Level::Alert,
                        "warn" => crate::events::Level::Warn,
                        _ => crate::events::Level::Info,
                    };
                    s.events.record(lvl, &source, &message, None, None);
                    json!({ "ok": true }).to_string()
                }),
            )
            .map_err(|e| e.to_string())?;

        // TTS。
        let s = state3.clone();
        globals
            .set(
                "tts",
                Func::new(move |text: String| -> String {
                    crate::tts::speak(&s, &text).map(|_| json!({ "ok": true }).to_string()).unwrap_or_else(|e| json!({ "ok": false, "error": e }).to_string())
                }),
            )
            .map_err(|e| e.to_string())?;

        // 图像生成(SD / ComfyUI)。
        let s = state3.clone();
        globals
            .set(
                "generateImage",
                Func::new(move |prompt: String| -> String {
                    match crate::sd::generate(&s, &prompt, None) {
                        Ok(resp) => serde_json::to_string(&resp).unwrap_or_default(),
                        Err(e) => json!({ "ok": false, "error": e }).to_string(),
                    }
                }),
            )
            .map_err(|e| e.to_string())?;

        // 工具自身状态(KV)。
        let s = state3.clone();
        globals
            .set(
                "getState",
                Func::new(move |key: String| -> String {
                    s.tool_state.read().map(|m| m.get(&key).cloned().unwrap_or_default()).unwrap_or_default()
                }),
            )
            .map_err(|e| e.to_string())?;
        let s = state3.clone();
        globals
            .set(
                "setState",
                Func::new(move |key: String, value: String| -> String {
                    if let Ok(mut m) = s.tool_state.write() {
                        m.insert(key, value);
                    }
                    "ok".to_string()
                }),
            )
            .map_err(|e| e.to_string())?;

        // 注入调用参数(JSON 字符串)。
        globals.set("__ARGS__", args_json).map_err(|e| e.to_string())?;

        // 执行脚本(允许 async/await 与顶层表达式;脚本可读取 __ARGS__)。
        let code = format!(
            r#"(function() {{
                const ARGS = JSON.parse(__ARGS__ || "{{}}");
                {script}
            }})()"#
        );
        let result: rquickjs::Value = ctx.eval(code).map_err(|e| format!("JS 执行错误:{e}"))?;
        if result.is_undefined() {
            Ok(json!({ "ok": true }))
        } else if let Ok(text) = result.get::<String>() {
            Ok(Value::String(text))
        } else {
            Ok(json!({ "ok": true, "note": "脚本执行完成" }))
        }
    })
}