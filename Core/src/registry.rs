//! 插件模块注册表。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

/// 模块运行状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModuleStatus {
    /// 已注册,未启动
    Stopped,
    /// 启动中
    Starting,
    /// 正常运行
    Running,
    /// 降级运行(部分功能不可用)
    Degraded,
    /// 异常退出
    Failed,
}

/// 模块信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleInfo {
    /// 模块 ID(如 `netscanear`)。
    pub id: String,
    /// 展示名(如 `NetScanear 网络设备发现程序`)。
    pub label: String,
    /// 版本。
    pub version: String,
    /// 当前状态。
    pub status: ModuleStatus,
    /// 附加说明。
    pub note: String,
}

impl ModuleInfo {
    /// 构造模块信息。
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            status: ModuleStatus::Stopped,
            note: String::new(),
        }
    }
}

/// 模块注册表:并发安全的模块清单。
#[derive(Debug, Default)]
pub struct ModuleRegistry {
    inner: RwLock<HashMap<String, ModuleInfo>>,
}

impl ModuleRegistry {
    /// 创建空注册表。
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册或更新模块信息。
    pub fn register(&self, info: ModuleInfo) {
        self.inner.write().expect("registry poisoned").insert(info.id.clone(), info);
    }

    /// 更新模块状态。
    pub fn set_status(&self, id: &str, status: ModuleStatus, note: impl Into<String>) {
        if let Some(info) = self.inner.write().expect("registry poisoned").get_mut(id) {
            info.status = status;
            info.note = note.into();
        }
    }

    /// 查询单个模块。
    pub fn get(&self, id: &str) -> Option<ModuleInfo> {
        self.inner.read().expect("registry poisoned").get(id).cloned()
    }

    /// 全部模块(按 ID 排序)。
    pub fn all(&self) -> Vec<ModuleInfo> {
        let mut list: Vec<ModuleInfo> = self.inner.read().expect("registry poisoned").values().cloned().collect();
        list.sort_by(|a, b| a.id.cmp(&b.id));
        list
    }

    /// 模块数量。
    pub fn len(&self) -> usize {
        self.inner.read().expect("registry poisoned").len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// 按状态统计模块数。
    pub fn count_by_status(&self, status: ModuleStatus) -> usize {
        self.all().into_iter().filter(|m| m.status == status).count()
    }
}