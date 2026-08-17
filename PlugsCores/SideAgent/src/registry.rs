//! 模型注册表:并发安全的模型清单与状态管理。

use super::model::{ModelInfo, ModelStatus};
use std::collections::HashMap;
use std::sync::RwLock;

/// 模型注册表。
#[derive(Debug, Default)]
pub struct ModelRegistry {
    inner: RwLock<HashMap<String, ModelInfo>>,
}

impl ModelRegistry {
    /// 创建空注册表。
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册模型。
    pub fn register(&self, info: ModelInfo) {
        self.inner.write().expect("registry poisoned").insert(info.id.clone(), info);
    }

    /// 注销模型。
    pub fn unregister(&self, id: &str) -> Option<ModelInfo> {
        self.inner.write().expect("registry poisoned").remove(id)
    }

    /// 更新模型状态。
    pub fn set_status(&self, id: &str, status: ModelStatus) {
        if let Some(m) = self.inner.write().expect("registry poisoned").get_mut(id) {
            m.status = status;
        }
    }

    /// 记录一次推理耗时(滑动平均)。
    pub fn observe_latency(&self, id: &str, ms: f32) {
        if let Some(m) = self.inner.write().expect("registry poisoned").get_mut(id) {
            m.avg_latency_ms = if m.avg_latency_ms <= 0.0 {
                ms
            } else {
                m.avg_latency_ms * 0.9 + ms * 0.1
            };
        }
    }

    /// 查询模型。
    pub fn get(&self, id: &str) -> Option<ModelInfo> {
        self.inner.read().expect("registry poisoned").get(id).cloned()
    }

    /// 全部模型(按 ID 排序)。
    pub fn all(&self) -> Vec<ModelInfo> {
        let mut list: Vec<ModelInfo> = self.inner.read().expect("registry poisoned").values().cloned().collect();
        list.sort_by(|a, b| a.id.cmp(&b.id));
        list
    }

    /// 模型数量。
    pub fn len(&self) -> usize {
        self.inner.read().expect("registry poisoned").len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// 就绪模型数量。
    pub fn ready_count(&self) -> usize {
        self.all().into_iter().filter(|m| m.status == ModelStatus::Ready).count()
    }
}