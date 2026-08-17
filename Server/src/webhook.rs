//! 告警 Webhook 推送:alert 级事件 -> HTTP POST 到外部(HA 通知桥 / 手机推送)。
//!
//! 零依赖实现:仅支持 http://(不含 https),使用 `std::net::TcpStream`
//! 发送 HTTP/1.1 POST。失败静默降级,不影响主流程。

use crate::state::AppState;
use std::io::{Read, Write};
use std::sync::Mutex;
use std::time::Duration;

/// Webhook 推送器。
#[derive(Debug)]
pub struct Webhook {
    /// 目标 URL(空 = 关闭)。
    pub url: String,
    /// 已推送的最大事件时间戳(去重)。
    last_ts: Mutex<u64>,
}

impl Webhook {
    /// 创建推送器。
    pub fn new(url: String) -> Self {
        Self { url, last_ts: Mutex::new(0) }
    }

    /// 扫描最近事件,推送新增的 alert 级事件。
    pub fn poll_and_push(&self, state: &AppState) {
        if self.url.is_empty() {
            return;
        }
        let recent = state.events.recent(200);
        let mut last = *self.last_ts.lock().expect("webhook poisoned");
        for event in recent.into_iter().rev() {
            if event.level == crate::events::Level::Alert && event.ts > last {
                last = event.ts;
                self.post(&event.source, &event.message);
            }
        }
        *self.last_ts.lock().expect("webhook poisoned") = last;
    }

    /// 立即发送一条事件(供自动化引擎动作调用)。
    pub fn call_now(&self, event: &crate::events::AppEvent) {
        if self.url.is_empty() {
            return;
        }
        self.post(&event.source, &event.message);
    }

    /// 发送单条告警。
    fn post(&self, source: &str, message: &str) {
        let Some((host, port, path)) = parse_http_url(&self.url) else {
            tracing::warn!("Webhook 地址无效:{}", self.url);
            return;
        };
        let body = serde_json::json!({
            "source": source,
            "message": message,
            "ts": crate::state::now_secs(),
            "level": "alert",
        })
        .to_string();

        let request = format!(
            "POST {path} HTTP/1.1\r\nHost: {host}:{port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );

        if let Ok(mut stream) = std::net::TcpStream::connect((host.as_str(), port)) {
            let _ = stream.set_read_timeout(Some(Duration::from_secs(3)));
            let _ = stream.write_all(request.as_bytes());
            let mut buf = [0u8; 256];
            let _ = stream.read(&mut buf);
            tracing::info!("Webhook 已推送告警:{message}");
        } else {
            tracing::warn!("Webhook 连接失败:{host}:{port}");
        }
    }
}
/// 极简 http:// URL 解析:返回 (host, port, path)。
/// 仅支持 http,不处理用户信息与查询串(够用且零依赖)。
fn parse_http_url(raw: &str) -> Option<(String, u16, String)> {
    let rest = raw.strip_prefix("http://")?;
    let (authority, path) = match rest.split_once('/') {
        Some((a, p)) => (a, format!("/{p}")),
        None => (rest, "/".to_string()),
    };
    if authority.is_empty() {
        return None;
    }
    let (host, port) = match authority.rsplit_once(':') {
        Some((h, p)) => (h.to_string(), p.parse::<u16>().unwrap_or(80)),
        None => (authority.to_string(), 80),
    };
    Some((host, port, path))
}