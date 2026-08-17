//! 审计日志:只追加的关键操作留痕。

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Mutex;

/// 一条审计记录。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// 时间戳(Unix 秒)。
    pub ts: u64,
    /// 操作者(模块名 / 用户)。
    pub actor: String,
    /// 动作(如 `scan.run` / `device.trust` / `rule.mutate`)。
    pub action: String,
    /// 目标(设备 ID / 模块 ID)。
    pub target: String,
    /// 详情。
    pub detail: String,
}

/// 审计日志:并发安全的环形缓冲(只追加,不修改)。
#[derive(Debug)]
pub struct AuditLog {
    inner: Mutex<VecDeque<AuditEntry>>,
    capacity: usize,
}

impl AuditLog {
    /// 创建审计日志。
    pub fn new(capacity: usize) -> Self {
        Self { inner: Mutex::new(VecDeque::with_capacity(capacity)), capacity }
    }

    /// 追加一条记录。
    pub fn append(&self, entry: AuditEntry) {
        let mut guard = self.inner.lock().expect("audit log poisoned");
        if guard.len() >= self.capacity {
            guard.pop_front();
        }
        guard.push_back(entry);
    }

    /// 便捷追加。
    pub fn record(&self, actor: impl Into<String>, action: impl Into<String>, target: impl Into<String>, detail: impl Into<String>, ts: u64) {
        self.append(AuditEntry { ts, actor: actor.into(), action: action.into(), target: target.into(), detail: detail.into() });
    }

    /// 最近 N 条(新在前)。
    pub fn recent(&self, n: usize) -> Vec<AuditEntry> {
        let guard = self.inner.lock().expect("audit log poisoned");
        guard.iter().rev().take(n).cloned().collect()
    }

    /// 按操作者过滤。
    pub fn by_actor(&self, actor: &str) -> Vec<AuditEntry> {
        let guard = self.inner.lock().expect("audit log poisoned");
        guard.iter().rev().filter(|e| e.actor == actor).cloned().collect()
    }

    /// 当前条数。
    pub fn len(&self) -> usize {
        self.inner.lock().expect("audit log poisoned").len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}