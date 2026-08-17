//! 健康检查接口(SelflookUP)。

use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use pilhome_selflookup::{CheckItem, HealthStatus, run_checks};
use serde_json::{Value, json};
use std::sync::Arc;

/// GET /api/health —— 系统自检报告。
pub async fn report(State(state): State<Arc<AppState>>) -> Json<Value> {
    let now = crate::state::now_secs();
    let total = state.devices.len();
    let online = state.devices.online_count(now, 120);
    let events_len = state.events.len();
    let models_ready = state.models.ready_count();
    let audit_len = state.audit.len();

    let mut items = Vec::new();
    // 1) 设备在线率
    let device_status = if total == 0 {
        HealthStatus::Ok
    } else if online >= total {
        HealthStatus::Ok
    } else if online * 2 >= total {
        HealthStatus::Warn
    } else {
        HealthStatus::Critical
    };
    items.push(CheckItem {
        name: "device.online_rate".into(),
        status: device_status,
        detail: format!("{online}/{total} 台在线"),
        checked_at: now,
    });
    // 2) 事件缓冲水位
    // 1.5) 存储健康(SQLite / MySQL)
    items.push(CheckItem {
        name: "storage.sqlite".into(),
        status: if state.sqlite.available() { HealthStatus::Ok } else { HealthStatus::Warn },
        detail: format!(
            "SQLite {} · {} 条事件 · MySQL {}",
            if state.sqlite.available() { "可用" } else { "不可用" },
            state.sqlite.count(),
            if state.mysql.enabled() { "已启用" } else { "未配置" },
        ),
        checked_at: now,
    });
    items.push(CheckItem {
        name: "events.buffer".into(),
        status: if events_len < 3500 { HealthStatus::Ok } else { HealthStatus::Warn },
        detail: format!("{events_len} 条(上限 4096)"),
        checked_at: now,
    });
    // 3) 边缘模型就绪
    items.push(CheckItem {
        name: "models.ready".into(),
        status: if models_ready > 0 { HealthStatus::Ok } else { HealthStatus::Warn },
        detail: format!("{models_ready} 个模型就绪"),
        checked_at: now,
    });
    // 4) 审计日志可用
    items.push(CheckItem {
        name: "audit.log".into(),
        status: HealthStatus::Ok,
        detail: format!("{audit_len} 条记录"),
        checked_at: now,
    });
    // 5) 核心模块状态
    let failed = state.coord.registry.count_by_status(pilhome_core::ModuleStatus::Failed);
    items.push(CheckItem {
        name: "modules.core".into(),
        status: if failed == 0 { HealthStatus::Ok } else { HealthStatus::Critical },
        detail: format!("{failed} 个模块异常"),
        checked_at: now,
    });

    let report = run_checks(items);
    Json(json!({ "ok": true, "report": report }))
}