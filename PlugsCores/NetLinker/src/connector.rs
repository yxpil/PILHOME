//! 连接管理器:设备连接的建立 / 维持 / 释放。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::Duration;

/// 连接状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnState {
    /// 空闲
    Idle,
    /// 连接中
    Connecting,
    /// 已连接
    Connected,
    /// 重连等待
    Reconnecting,
    /// 已断开
    Disconnected,
}

/// 连接管理器:按设备 ID 维护连接状态与重试策略。
#[derive(Debug, Default)]
pub struct Connector {
    inner: RwLock<HashMap<String, ConnState>>,
}

impl Connector {
    /// 创建连接管理器。
    pub fn new() -> Self {
        Self::default()
    }

    /// 发起连接(置为连接中)。
    pub fn connect(&self, device_id: &str) {
        self.set(device_id, ConnState::Connecting);
    }

    /// 连接成功。
    pub fn mark_connected(&self, device_id: &str) {
        self.set(device_id, ConnState::Connected);
    }

    /// 连接失败,进入重连等待。
    pub fn mark_failed(&self, device_id: &str) {
        self.set(device_id, ConnState::Reconnecting);
    }

    /// 主动断开。
    pub fn disconnect(&self, device_id: &str) {
        self.set(device_id, ConnState::Disconnected);
    }

    /// 查询连接状态。
    pub fn state(&self, device_id: &str) -> ConnState {
        self.inner.read().expect("connector poisoned").get(device_id).copied().unwrap_or(ConnState::Idle)
    }

    /// 全部连接状态快照。
    pub fn snapshot(&self) -> Vec<(String, ConnState)> {
        let mut list: Vec<(String, ConnState)> =
            self.inner.read().expect("connector poisoned").iter().map(|(k, v)| (k.clone(), *v)).collect();
        list.sort_by(|a, b| a.0.cmp(&b.0));
        list
    }

    /// 统计指定状态的连接数。
    pub fn count(&self, state: ConnState) -> usize {
        self.snapshot().into_iter().filter(|(_, s)| *s == state).count()
    }

    fn set(&self, device_id: &str, state: ConnState) {
        self.inner.write().expect("connector poisoned").insert(device_id.to_string(), state);
    }
}

/// 重试退避策略:第 n 次重试的等待时长(指数退避 + 上限)。
pub fn backoff(attempt: u32, base_ms: u64, max_ms: u64) -> Duration {
    let ms = base_ms.saturating_mul(1u64 << attempt.min(10)).min(max_ms);
    Duration::from_millis(ms)
}