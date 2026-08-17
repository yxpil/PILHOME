//! 模块运行时状态:Core 注册表 + 各模块附加指标。

use crate::state::{now_secs, AppState};
use serde::Serialize;

/// 模块状态响应。
#[derive(Debug, Clone, Serialize)]
pub struct ModuleStatus {
    /// 模块 ID。
    pub id: String,
    /// 展示名。
    pub label: String,
    /// 版本。
    pub version: String,
    /// 状态。
    pub status: String,
    /// 附加指标。
    pub metric: String,
}

/// 汇总模块状态。
pub fn collect(state: &AppState, offline_after_secs: u64) -> Vec<ModuleStatus> {
    let online = state.devices.online_count(now_secs(), offline_after_secs);
    let total = state.devices.len();
    let scan = state.last_scan.lock().map(|s| s.as_ref().map(|r| r.unknown_total).unwrap_or(0)).unwrap_or(0);
    let rules = state.schedules.read().map(|s| s.len()).unwrap_or(0);
    let profile_len = state.profile.lock().map(|p| p.len()).unwrap_or(0);
    let gen = state.evolver.lock().map(|e| e.generation()).unwrap_or(0);
    let models_ready = state.models.ready_count();
    let running = state.controller.running_count();

    vec![
        ModuleStatus { id: "netlinker".into(), label: "NetLinker 网络设备连接程序".into(), version: env!("CARGO_PKG_VERSION").into(), status: "运行中".into(), metric: format!("{online}/{total} 设备在线") },
        ModuleStatus { id: "netscanear".into(), label: "NetScanear 网络设备发现程序".into(), version: env!("CARGO_PKG_VERSION").into(), status: "运行中".into(), metric: format!("最近发现陌生设备 {scan} 台") },
        ModuleStatus { id: "sideagent".into(), label: "SideAgent 边缘AI模型".into(), version: env!("CARGO_PKG_VERSION").into(), status: "运行中".into(), metric: format!("{models_ready} 个模型就绪") },
        ModuleStatus { id: "handmodel".into(), label: "HandModel 模块控制程序".into(), version: env!("CARGO_PKG_VERSION").into(), status: "运行中".into(), metric: format!("{running} 个模块受控运行") },
        ModuleStatus { id: "vescaner".into(), label: "VEScaner 视频设备发现程序".into(), version: env!("CARGO_PKG_VERSION").into(), status: "就绪".into(), metric: "RTSP/ONVIF 探测可用".into() },
        ModuleStatus { id: "yololookup".into(), label: "YololookUP 视频粗略代审计模块".into(), version: env!("CARGO_PKG_VERSION").into(), status: "就绪".into(), metric: "YOLO 检测器可插拔".into() },
        ModuleStatus { id: "selflookup".into(), label: "SelflookUP 自我审计程序".into(), version: env!("CARGO_PKG_VERSION").into(), status: "运行中".into(), metric: format!("审计日志 {} 条", state.audit.len()) },
        ModuleStatus { id: "atogrowup".into(), label: "ATOGrowUP 自发成长和变异程序".into(), version: env!("CARGO_PKG_VERSION").into(), status: "运行中".into(), metric: format!("画像 {profile_len} 条 / 已进化 {gen} 代") },
        ModuleStatus { id: "autotime".into(), label: "AUTOTIME 定时和节律控制".into(), version: env!("CARGO_PKG_VERSION").into(), status: "运行中".into(), metric: format!("{rules} 条定时规则") },
    ]
}

