//! 行为画像:设备 × 时段的活跃期望与异常识别。

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// 单条行为事件。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRecord {
    /// 设备 ID。
    pub device_id: String,
    /// 事件键(如 `contact` / `motion`)。
    pub key: String,
    /// 事件值(如 `open` / `closed`)。
    pub value: String,
    /// 事件时间(Unix 秒)。
    pub ts: u64,
}

/// 一条安全建议。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    /// 严重程度(`info` / `warn` / `alert`)。
    pub severity: String,
    /// 涉及设备。
    pub device_id: String,
    /// 标题。
    pub title: String,
    /// 详情。
    pub detail: String,
}

/// 行为画像器。
#[derive(Debug)]
pub struct Profile {
    records: VecDeque<EventRecord>,
    max_records: usize,
    /// 设备 × 小时 -> 活跃次数。
    counts: HashMap<(String, usize), u32>,
    /// 设备总事件数。
    totals: HashMap<String, u32>,
}

impl Default for Profile {
    fn default() -> Self {
        Self::with_capacity(8192)
    }
}

impl Profile {
    /// 创建画像器。
    pub fn with_capacity(max_records: usize) -> Self {
        Self {
            records: VecDeque::with_capacity(max_records),
            max_records,
            counts: HashMap::new(),
            totals: HashMap::new(),
        }
    }

    /// 记录一条事件。
    pub fn record(&mut self, event: EventRecord) {
        let hour = (event.ts / 3600 % 24) as usize;
        *self.counts.entry((event.device_id.clone(), hour)).or_insert(0) += 1;
        *self.totals.entry(event.device_id.clone()).or_insert(0) += 1;
        if self.records.len() >= self.max_records {
            self.records.pop_front();
        }
        self.records.push_back(event);
    }

    /// 事件总数。
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// 设备被观察天数(按去重日期估算,至少 1)。
    fn days_observed(&self, device: &str) -> u32 {
        let mut days = std::collections::BTreeSet::new();
        for r in &self.records {
            if r.device_id == device {
                days.insert(r.ts / 86400);
            }
        }
        days.len().max(1) as u32
    }

    /// 设备在某小时的历史平均活跃次数。
    pub fn hourly_average(&self, device: &str, hour: usize) -> f32 {
        let days = self.days_observed(device);
        if days == 0 {
            return 0.0;
        }
        *self.counts.get(&(device.to_string(), hour)).unwrap_or(&0) as f32 / days as f32
    }

    /// 设备的 24 小时活跃曲线(供节律模块参考)。
    pub fn activity_curve(&self, device: &str) -> [f32; 24] {
        let mut curve = [0.0f32; 24];
        for hour in 0..24 {
            curve[hour] = self.hourly_average(device, hour);
        }
        curve
    }

    /// 生成建议:当前小时活跃次数 ≥ 历史均值 × 3 且 ≥ 3 次时告警。
    pub fn suggestions(&self, now: u64) -> Vec<Suggestion> {
        let hour = (now / 3600 % 24) as usize;
        let mut out = Vec::new();
        for (device, total) in &self.totals {
            if *total < 8 {
                continue;
            }
            let avg = self.hourly_average(device, hour);
            let current = *self.counts.get(&(device.clone(), hour)).unwrap_or(&0) as f32;
            if avg > 0.0 && current >= avg * 3.0 && current >= 3.0 {
                out.push(Suggestion {
                    severity: "alert".to_string(),
                    device_id: device.clone(),
                    title: format!("设备 {device} 活跃度异常"),
                    detail: format!("该时段历史均值约 {avg:.1} 次,今日已达 {current:.0} 次,请确认是否异常。"),
                });
            }
        }
        out
    }

    /// 有画像记录的设备 ID 列表(按字典序)。
    pub fn devices(&self) -> Vec<String> {
        let mut list: Vec<String> = self.totals.keys().cloned().collect();
        list.sort();
        list
    }

    /// 最近 N 条事件。
    pub fn recent(&self, n: usize) -> Vec<EventRecord> {
        self.records.iter().rev().take(n).cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_anomalous_activity() {
        let mut profile = Profile::with_capacity(64);
        let base = 1_785_600_000u64;
        // 连续 10 天,每天 03:00 各 1 次,形成历史画像。
        for day in 0..10u64 {
            profile.record(EventRecord {
                device_id: "door".into(),
                key: "contact".into(),
                value: "open".into(),
                ts: base + day * 86400 + 3 * 3600,
            });
        }
        // 今天 03:00 突然 5 次。
        for i in 0..5u64 {
            profile.record(EventRecord {
                device_id: "door".into(),
                key: "contact".into(),
                value: "open".into(),
                ts: base + 10 * 86400 + 3 * 3600 + i * 60,
            });
        }
        let suggestions = profile.suggestions(base + 10 * 86400 + 3 * 3600);
        assert!(!suggestions.is_empty());
        assert_eq!(suggestions[0].device_id, "door");
        assert_eq!(suggestions[0].severity, "alert");
    }

    #[test]
    fn insufficient_samples_no_suggestion() {
        let profile = Profile::default();
        assert!(profile.suggestions(1_785_600_000).is_empty());
    }
}
