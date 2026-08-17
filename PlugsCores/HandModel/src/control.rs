//! 模块控制器:启停、参数下发、状态收集。

use super::params::ParamSet;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

/// 单个被控模块的句柄。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleHandle {
    /// 模块 ID。
    pub id: String,
    /// 是否在运行。
    pub running: bool,
    /// 当前参数。
    pub params: ParamSet,
    /// 最近状态消息。
    pub last_note: String,
}

/// 模块控制器:并发安全地控制一组模块。
#[derive(Debug, Default)]
pub struct Controller {
    inner: RwLock<HashMap<String, ModuleHandle>>,
}

impl Controller {
    /// 创建控制器。
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册被控模块。
    pub fn register(&self, id: impl Into<String>) {
        self.inner.write().expect("controller poisoned").insert(
            id.into(),
            ModuleHandle { id: String::new(), running: false, params: ParamSet::new(), last_note: String::new() },
        );
    }

    /// 启动模块。
    pub fn start(&self, id: &str) -> bool {
        self.with(id, |h| h.running = true)
    }

    /// 停止模块。
    pub fn stop(&self, id: &str) -> bool {
        self.with(id, |h| h.running = false)
    }

    /// 下发参数(整组覆盖)。
    pub fn apply_params(&self, id: &str, params: ParamSet) -> bool {
        self.with(id, |h| h.params = params)
    }

    /// 记录状态消息。
    pub fn note(&self, id: &str, note: impl Into<String>) -> bool {
        self.with(id, |h| h.last_note = note.into())
    }

    /// 查询句柄。
    pub fn handle(&self, id: &str) -> Option<ModuleHandle> {
        self.inner.read().expect("controller poisoned").get(id).cloned()
    }

    /// 全部句柄(按 ID 排序)。
    pub fn all(&self) -> Vec<ModuleHandle> {
        let mut list: Vec<ModuleHandle> = self.inner.read().expect("controller poisoned").values().cloned().collect();
        list.sort_by(|a, b| a.id.cmp(&b.id));
        list
    }

    /// 运行中的模块数。
    pub fn running_count(&self) -> usize {
        self.all().into_iter().filter(|h| h.running).count()
    }

    fn with(&self, id: &str, f: impl FnOnce(&mut ModuleHandle)) -> bool {
        match self.inner.write().expect("controller poisoned").get_mut(id) {
            Some(h) => {
                f(h);
                true
            }
            None => false,
        }
    }
}