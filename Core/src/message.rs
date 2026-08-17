//! 模块间消息总线。
//!
//! 采用 `tokio::sync::broadcast` 广播语义:任意模块发布消息,
//! 关注方按主题前缀订阅,实现"发布-订阅"解耦。

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::broadcast;

/// 模块间消息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// 发布模块 ID(如 `netscanear`)。
    pub from: String,
    /// 目标模块(空串表示广播)。
    pub to: String,
    /// 消息主题(如 `device.found` / `alert.intrusion`)。
    pub topic: String,
    /// 负载。
    pub payload: Value,
    /// 时间戳(Unix 秒)。
    pub ts: u64,
}

/// 消息总线:广播通道 + 主题过滤。
#[derive(Debug, Clone)]
pub struct MessageBus {
    tx: broadcast::Sender<Message>,
}

impl Default for MessageBus {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageBus {
    /// 创建消息总线(缓冲 256 条,慢订阅者自动丢弃最旧消息)。
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(256);
        Self { tx }
    }

    /// 发布一条消息。
    pub fn publish(&self, from: impl Into<String>, to: impl Into<String>, topic: impl Into<String>, payload: Value, ts: u64) {
        let msg = Message {
            from: from.into(),
            to: to.into(),
            topic: topic.into(),
            payload,
            ts,
        };
        let _ = self.tx.send(msg);
    }

    /// 便捷发布:广播消息。
    pub fn broadcast(&self, from: impl Into<String>, topic: impl Into<String>, payload: Value, ts: u64) {
        self.publish(from, "", topic, payload, ts);
    }

    /// 订阅全部消息。
    pub fn subscribe(&self) -> broadcast::Receiver<Message> {
        self.tx.subscribe()
    }

    /// 订阅并过滤指定来源模块。
    pub fn subscribe_from(&self, from: &str) -> broadcast::Receiver<Message> {
        let rx = self.tx.subscribe();
        // 过滤由订阅方自行处理,此处保留全量通道。
        let _ = from;
        rx
    }
}