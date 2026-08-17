//! 总协调器:串联模块注册、状态监控与消息路由。

use crate::message::MessageBus;
use crate::registry::{ModuleInfo, ModuleRegistry, ModuleStatus};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// 当前 Unix 秒。
pub fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// 总协调器。
///
/// 持有消息总线与模块注册表,提供模块生命周期控制入口。
pub struct Coordinator {
    /// 模块间消息总线。
    pub bus: MessageBus,
    /// 模块注册表。
    pub registry: ModuleRegistry,
    /// 协调器启动时间(Unix 秒)。
    pub started_at: u64,
}

impl Default for Coordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl Coordinator {
    /// 创建总协调器。
    pub fn new() -> Self {
        Self {
            bus: MessageBus::new(),
            registry: ModuleRegistry::new(),
            started_at: now_secs(),
        }
    }

    /// 注册模块并广播启动事件。
    pub fn mount(&self, info: ModuleInfo) {
        self.registry.register(info.clone());
        self.bus.broadcast("core", "module.mounted", serde_json::json!({ "id": info.id }), now_secs());
    }

    /// 启动模块(置为运行态)。
    pub fn start_module(&self, id: &str) {
        self.registry.set_status(id, ModuleStatus::Running, "已启动");
        self.bus.broadcast("core", "module.started", serde_json::json!({ "id": id }), now_secs());
    }

    /// 停止模块。
    pub fn stop_module(&self, id: &str) {
        self.registry.set_status(id, ModuleStatus::Stopped, "已停止");
        self.bus.broadcast("core", "module.stopped", serde_json::json!({ "id": id }), now_secs());
    }

    /// 标记模块降级。
    pub fn degrade_module(&self, id: &str, note: impl Into<String>) {
        self.registry.set_status(id, ModuleStatus::Degraded, note);
        self.bus.broadcast("core", "module.degraded", serde_json::json!({ "id": id }), now_secs());
    }

    /// 运行时长(秒)。
    pub fn uptime_secs(&self) -> u64 {
        now_secs().saturating_sub(self.started_at)
    }
}

/// 便捷:构造共享协调器。
pub fn shared() -> Arc<Coordinator> {
    Arc::new(Coordinator::new())
}