//! 统一事件日志:环形缓冲。

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::VecDeque;
use std::sync::Mutex;
use tokio::sync::broadcast;

/// 事件级别。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Info,
    Warn,
    Alert,
}

/// 统一应用事件。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppEvent {
    /// 时间戳(Unix 秒)。
    pub ts: u64,
    /// 级别。
    pub level: Level,
    /// 来源模块。
    pub source: String,
    /// 摘要。
    pub message: String,
    /// 关联设备(可选)。
    pub device_id: Option<String>,
    /// 附加数据(可选)。
    pub data: Option<Value>,
}

/// 事件日志:线程安全的环形缓冲 + 实时广播。
///
/// 每写入一条事件,同时向广播通道推送,供 WebSocket 实时流
/// 与自动化引擎订阅(发布-订阅,慢订阅者自动丢最旧)。
#[derive(Debug)]
pub struct EventLog {
    inner: Mutex<VecDeque<AppEvent>>,
    capacity: usize,
    tx: broadcast::Sender<AppEvent>,
}

impl EventLog {
    /// 创建事件日志。
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(512);
        Self { inner: Mutex::new(VecDeque::with_capacity(capacity)), capacity, tx }
    }

    /// 记录事件。
    pub fn push(&self, event: AppEvent) {
        let mut guard = self.inner.lock().expect("event log poisoned");
        if guard.len() >= self.capacity {
            guard.pop_front();
        }
        // 广播给订阅方(WebSocket / 自动化引擎),再入缓冲。
        let _ = self.tx.send(event.clone());
        guard.push_back(event);
    }

    /// 订阅实时事件流。
    pub fn subscribe(&self) -> broadcast::Receiver<AppEvent> {
        self.tx.subscribe()
    }

    /// 便捷记录。
    pub fn record(&self, level: Level, source: impl Into<String>, message: impl Into<String>, device_id: Option<&str>, data: Option<Value>) {
        self.push(AppEvent {
            ts: crate::state::now_secs(),
            level,
            source: source.into(),
            message: message.into(),
            device_id: device_id.map(|s| s.to_string()),
            data,
        });
    }

    /// 取最近 N 条(新在前)。
    pub fn recent(&self, n: usize) -> Vec<AppEvent> {
        let guard = self.inner.lock().expect("event log poisoned");
        guard.iter().rev().take(n).cloned().collect()
    }

    /// 当前条数。
    pub fn len(&self) -> usize {
        self.inner.lock().expect("event log poisoned").len()
    }

    /// 是否为空(预留 API)。
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}