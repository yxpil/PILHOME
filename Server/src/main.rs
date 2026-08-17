//! PILHOME 网关服务入口。
//!
//! 职责:负责管理网页(托管 WebUI 静态资源)和内核桥接
//! (聚合 Core 协调器与 PlugsCores 模块,对外暴露 REST API,
//!  对接 MQTT 与 Home Assistant)。

mod ai;
mod api;
mod auth;
mod automation;
mod config;
mod discover;
mod events;
mod excel;
mod storage;
mod sd;
mod tools;
mod tts;
mod vendor;
mod ha;
mod ha_bridge;
mod js;
mod media;
mod modules;
mod mqtt;
mod persistence;
mod state;
mod webhook;
mod ws;
mod tasks;

use crate::config::Config;
use crate::state::AppState;
use axum::Router;
use pilhome_core::ModuleInfo;
use pilhome_sideagent::{ModelInfo, ModelKind};
use std::sync::Arc;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,pilhome_server=info".into()),
        )
        .init();

    // 1) 加载配置并构建全局状态。
    let config_path = std::env::args().nth(1).map(|p| std::path::PathBuf::from(p));
    let cfg = Config::load(config_path.as_deref());

    // 1.5) 创建 HA 双向桥并构建全局状态。
    let ha_bridge = Arc::new(ha_bridge::HaBridge::new(&cfg.ha.ws_url, &cfg.ha.token));
    let state = AppState::new(
        cfg.event_capacity,
        cfg.data_dir.clone(),
        ha_bridge.clone(),
        &cfg.api_tokens,
        cfg.auth_enabled,
        &cfg.mysql_url,
        &cfg.ai_base_url,
        &cfg.ai_model,
        &cfg.ai_vision_model,
        &cfg.ai_api_key,
        &cfg.tts_url,
        &cfg.comfyui_url,
    );

    // 1.6) 恢复上次快照(事件/审计/画像)。
    state.persistence.restore(&state);

    // 2) Core 注册全部模块。
    mount_modules(&state);

    // 3) 内置边缘模型注册(SideAgent)。
    seed_models(&state);

    // 4) 启动 MQTT 桥接、HA 全屋接管、告警推送、自动化引擎与周期任务。
    let webhook = Arc::new(webhook::Webhook::new(cfg.webhook_url.clone()));
    let mqtt_tx = mqtt::spawn(state.clone(), cfg.clone());
    *state.mqtt_tx.lock().expect("mqtt tx poisoned") = Some(mqtt_tx);
    ha_bridge::spawn(state.clone(), ha_bridge.clone());
    automation::spawn_engine(state.clone(), cfg.tasks.cron_tick_secs);
    tasks::spawn_storage(state.clone());
    tasks::spawn_ai_review(state.clone(), cfg.ai_review_secs);
    tasks::spawn_bayes_learn(state.clone());
    tasks::spawn_periodic(state.clone(), cfg.clone(), webhook);

    // 5) 构建路由:API + WebUI 静态托管。
    let webui = cfg.webui_dir.clone();
    let app: Router = api::router()
        .with_state(state.clone())
        .layer(axum::middleware::from_fn_with_state(state.clone(), auth::require_token))
        .fallback_service(ServeDir::new(&webui));

    // 6) 启动 HTTP 服务。
    let listener = tokio::net::TcpListener::bind(&cfg.listen)
        .await
        .unwrap_or_else(|e| panic!("监听 {} 失败:{e}", cfg.listen));
    tracing::info!("PILHOME 网关已启动: http://{} (WebUI: {})", cfg.listen, webui.display());
    axum::serve(listener, app).await.expect("服务器异常退出");
}

/// 向 Core 注册全部 PlugsCores 模块。
fn mount_modules(state: &Arc<AppState>) {
    let mods = [
        ("netlinker", "NetLinker 网络设备连接程序"),
        ("netscanear", "NetScanear 网络设备发现程序"),
        ("sideagent", "SideAgent 边缘AI模型"),
        ("handmodel", "HandModel 模块控制程序"),
        ("vescaner", "VEScaner 视频设备发现程序"),
        ("yololookup", "YololookUP 视频粗略代审计模块"),
        ("selflookup", "SelflookUP 自我审计程序"),
        ("atogrowup", "ATOGrowUP 自发成长和变异程序"),
        ("autotime", "AUTOTIME 定时和节律控制"),
    ];
    for (id, label) in mods {
        state.coord.mount(ModuleInfo::new(id, label));
        state.coord.start_module(id);
        state.controller.register(id);
        state.controller.start(id);
    }
    tracing::info!("已挂载 {} 个核心模块", mods.len());
}

/// 内置边缘模型(占位权重路径,接入真实模型时替换)。
fn seed_models(state: &Arc<AppState>) {
    let models = [
        ("yolo_v8n_person", "人员检测 YOLOv8n", "detection", "onnx", "models/yolov8n.onnx"),
        ("gesture_rule", "手势识别规则引擎", "gesture", "rule", "builtin"),
    ];
    for (id, name, kind, format, path) in models {
        let kind = match kind {
            "detection" => ModelKind::Detection,
            _ => ModelKind::Gesture,
        };
        let mut info = ModelInfo::new(id, name, kind, format, path);
        info.status = pilhome_sideagent::ModelStatus::Ready;
        state.models.register(info);
    }
}