//! 存储层:SQLite 本地库 + MySQL 远程同步 + Excel 导出。
//!
//! - [`SqliteStore`] 事件/审计落 SQLite(本地持久化,重启不丢);
//! - [`MysqlSync`] 将事件增量同步到远程 MySQL(配置 `mysql_url` 启用);
//! - [`export_xlsx` / `export_csv`] 导出事件为 Excel / CSV 文件。

use crate::events::AppEvent;
use mysql::prelude::*;
use std::path::PathBuf;
use std::sync::Mutex;

// ============================ SQLite ============================

/// SQLite 事件库(线程安全,失败优雅降级)。
pub struct SqliteStore {
    conn: Mutex<Option<rusqlite::Connection>>,
}

impl SqliteStore {
    /// 打开(或创建)数据库并建表。
    pub fn open(db_path: impl Into<PathBuf>) -> Self {
        let path = db_path.into();
        let conn = rusqlite::Connection::open(&path).ok();
        let store = Self { conn: Mutex::new(conn) };
        store.init();
        store
    }

    fn init(&self) {
        let Ok(guard) = self.conn.lock() else { return };
        let Some(conn) = guard.as_ref() else { return };
        let _ = conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                ts INTEGER NOT NULL,
                level TEXT NOT NULL,
                source TEXT NOT NULL,
                message TEXT NOT NULL,
                device_id TEXT,
                data TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_events_ts ON events(ts);",
        );
    }

    /// 是否可用。
    pub fn available(&self) -> bool {
        self.conn.lock().map(|g| g.is_some()).unwrap_or(false)
    }

    /// 写入一条事件。
    pub fn insert(&self, event: &AppEvent) {
        let Ok(guard) = self.conn.lock() else { return };
        let Some(conn) = guard.as_ref() else { return };
        let data = event.data.as_ref().map(|d| d.to_string());
        let _ = conn.execute(
            "INSERT INTO events (ts, level, source, message, device_id, data) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                event.ts as i64,
                format!("{:?}", event.level).to_lowercase(),
                event.source,
                event.message,
                event.device_id,
                data,
            ],
        );
    }

    /// 批量写入。
    pub fn insert_batch(&self, events: &[AppEvent]) -> usize {
        let mut n = 0;
        for e in events {
            self.insert(e);
            n += 1;
        }
        n
    }

    /// 事件总数。
    pub fn count(&self) -> i64 {
        let Ok(guard) = self.conn.lock() else { return 0 };
        let Some(conn) = guard.as_ref() else { return 0 };
        conn.query_row("SELECT COUNT(*) FROM events", [], |r| r.get(0)).unwrap_or(0)
    }

    /// 最近 N 条(供导出)。
    pub fn recent(&self, limit: usize) -> Vec<AppEvent> {
        let Ok(guard) = self.conn.lock() else { return Vec::new() };
        let Some(conn) = guard.as_ref() else { return Vec::new() };
        let mut stmt = match conn.prepare("SELECT ts, level, source, message, device_id, data FROM events ORDER BY ts DESC LIMIT ?1") {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        let rows = match stmt.query_map([limit as i64], |r| {
                let ts: i64 = r.get(0)?;
                let level: String = r.get(1)?;
                let source: String = r.get(2)?;
                let message: String = r.get(3)?;
                let device_id: Option<String> = r.get(4)?;
                let data: Option<String> = r.get(5)?;
                let level = match level.as_str() {
                    "warn" => crate::events::Level::Warn,
                    "alert" => crate::events::Level::Alert,
                    _ => crate::events::Level::Info,
                };
                Ok(AppEvent {
                    ts: ts as u64,
                    level,
                    source,
                    message,
                    device_id,
                    data: data.and_then(|s| serde_json::from_str(&s).ok()),
                })
            }) {
            Ok(rows) => rows,
            Err(_) => return Vec::new(),
        };
        rows.filter_map(|r| r.ok()).collect()
    }
}

// ============================ MySQL ============================

/// MySQL 增量同步器(配置 `mysql_url` 后启用)。
#[derive(Clone)]
pub struct MysqlSync {
    url: String,
}

impl MysqlSync {
    /// 创建同步器(空 URL = 关闭)。
    pub fn new(url: String) -> Self {
        Self { url }
    }

    /// 是否启用。
    pub fn enabled(&self) -> bool {
        !self.url.is_empty()
    }

    /// 同步一批事件到 MySQL(失败时返回错误信息,由调用方降级)。
    pub fn sync_events(&self, events: &[AppEvent]) -> Result<usize, String> {
        if events.is_empty() {
            return Ok(0);
        }
        let opts = mysql::Opts::from_url(&self.url).map_err(|e| format!("MySQL 地址无效:{e}"))?;
        let mut conn = mysql::Conn::new(opts).map_err(|e| format!("MySQL 连接失败:{e}"))?;
        conn.exec_drop(
            "CREATE TABLE IF NOT EXISTS pilhome_events (
                id BIGINT AUTO_INCREMENT PRIMARY KEY,
                ts BIGINT NOT NULL,
                level VARCHAR(16) NOT NULL,
                source VARCHAR(64) NOT NULL,
                message TEXT NOT NULL,
                device_id VARCHAR(128),
                data TEXT,
                KEY idx_ts (ts)
            ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
            (),
        )
        .map_err(|e| format!("MySQL 建表失败:{e}"))?;

        let rows: Vec<mysql::Params> = events
            .iter()
            .map(|e| {
                mysql::Params::from(vec![
                    mysql::Value::from(e.ts as i64),
                    mysql::Value::from(format!("{:?}", e.level).to_lowercase()),
                    mysql::Value::from(e.source.clone()),
                    mysql::Value::from(e.message.clone()),
                    e.device_id.clone().map(mysql::Value::from).unwrap_or(mysql::Value::NULL),
                    e.data.as_ref().map(|d| mysql::Value::from(d.to_string())).unwrap_or(mysql::Value::NULL),
                ])
            })
            .collect();
        conn.exec_batch(
            "INSERT INTO pilhome_events (ts, level, source, message, device_id, data) VALUES (?, ?, ?, ?, ?, ?)",
            rows,
        )
        .map_err(|e| format!("MySQL 写入失败:{e}"))?;
        Ok(events.len())
    }
}