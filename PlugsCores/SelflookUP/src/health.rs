//! 健康检查:系统自检项与汇总报告。

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// 当前 Unix 秒。
pub fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// 单项检查状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    /// 通过
    Ok,
    /// 警告(可降级运行)
    Warn,
    /// 严重(需要干预)
    Critical,
}

/// 单项检查结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckItem {
    /// 检查项名称(如 `module.netlinker`)。
    pub name: String,
    /// 状态。
    pub status: HealthStatus,
    /// 详情。
    pub detail: String,
    /// 检查时间(Unix 秒)。
    pub checked_at: u64,
}

/// 健康报告。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    /// 生成时间。
    pub generated_at: u64,
    /// 各项检查。
    pub items: Vec<CheckItem>,
    /// 总体状态(取最差单项)。
    pub overall: HealthStatus,
}

impl HealthReport {
    /// 总体状态为最差单项。
    pub fn overall(&self) -> HealthStatus {
        self.items
            .iter()
            .map(|i| i.status)
            .max_by_key(|s| match s {
                HealthStatus::Ok => 0,
                HealthStatus::Warn => 1,
                HealthStatus::Critical => 2,
            })
            .unwrap_or(HealthStatus::Ok)
    }
}

/// 聚合型健康检查(各项结果由调用方注入,保持模块纯净)。
pub fn run_checks(items: Vec<CheckItem>) -> HealthReport {
    let report = HealthReport { generated_at: now_secs(), items, overall: HealthStatus::Ok };
    // overall 字段由 compute_overall 重新计算,这里直接构建。
    HealthReport { generated_at: report.generated_at, items: report.items.clone(), overall: report.overall() }
}