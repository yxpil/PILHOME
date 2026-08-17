//! 数据持久化:事件 / 审计 / 行为画像的 JSON 快照。
//!
//! 设计原则:
//! - 零依赖,纯 `std::fs` + `serde_json`;
//! - 周期快照(默认 300s)+ 启动恢复;
//! - 任何加载失败都优雅降级(空数据),绝不因恢复问题阻塞启动。

use crate::events::AppEvent;
use crate::state::AppState;
use pilhome_atogrowup::EventRecord;
use pilhome_selflookup::AuditEntry;
use std::path::PathBuf;

/// 持久化目录管理。
#[derive(Debug, Clone)]
pub struct Persistence {
    dir: PathBuf,
}

impl Persistence {
    /// 创建持久化层(自动建目录)。
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        let dir = dir.into();
        let _ = std::fs::create_dir_all(&dir);
        Self { dir }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.dir.join(name)
    }

    fn write_json<T: serde::Serialize>(&self, name: &str, value: &T) {
        let path = self.path(name);
        match serde_json::to_vec_pretty(value) {
            Ok(bytes) => {
                if let Err(err) = std::fs::write(&path, bytes) {
                    tracing::warn!("持久化写入失败({name}):{err}");
                }
            }
            Err(err) => tracing::warn!("持久化序列化失败({name}):{err}"),
        }
    }

    fn read_json<T: serde::de::DeserializeOwned>(&self, name: &str) -> Option<T> {
        let path = self.path(name);
        let bytes = std::fs::read(&path).ok()?;
        match serde_json::from_slice(&bytes) {
            Ok(v) => Some(v),
            Err(err) => {
                tracing::warn!("持久化加载失败({name}):{err},忽略该文件");
                None
            }
        }
    }

    /// 保存全部快照。
    pub fn snapshot(&self, state: &AppState) {
        self.write_json("events.json", &state.events.recent(4096));
        self.write_json("audit.json", &state.audit.recent(4096));
        let profile_records = state.profile.lock().map(|p| p.recent(8192)).unwrap_or_default();
        self.write_json("profile.json", &profile_records);
    }

    /// 启动恢复:事件 / 审计 / 画像。
    pub fn restore(&self, state: &AppState) {
        if let Some(events) = self.read_json::<Vec<AppEvent>>("events.json") {
            for e in events {
                state.events.push(e);
            }
            tracing::info!("已恢复 {} 条事件", state.events.len());
        }
        if let Some(audit) = self.read_json::<Vec<AuditEntry>>("audit.json") {
            for e in audit {
                state.audit.append(e);
            }
            tracing::info!("已恢复 {} 条审计记录", state.audit.len());
        }
        if let Some(records) = self.read_json::<Vec<EventRecord>>("profile.json") {
            if let Ok(mut profile) = state.profile.lock() {
                for r in records {
                    profile.record(r);
                }
                tracing::info!("已恢复行为画像 {} 条", profile.len());
            }
        }
    }
}
