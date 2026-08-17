//! 视频粗略代审计引擎。

use crate::detect::{Detection, Detector, NoopDetector};
use serde::{Deserialize, Serialize};

/// 审计事件级别。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditLevel {
    /// 常规记录
    Info,
    /// 可疑(检测到目标但置信度中等)
    Suspicious,
    /// 严重(高置信目标 + 运动)
    Critical,
}

/// 一条视频审计事件。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// 时间戳(Unix 秒)。
    pub ts: u64,
    /// 摄像头 ID。
    pub camera_id: String,
    /// 级别。
    pub level: AuditLevel,
    /// 摘要。
    pub summary: String,
    /// 检出目标。
    pub detections: Vec<Detection>,
}

/// 审计引擎:按帧喂入检测结果,规则化产出审计事件。
pub struct AuditEngine {
    detector: Box<dyn Detector>,
    /// 运动判定阈值(变化像素占比)。
    pub motion_threshold: f32,
    /// 关键目标最小置信度。
    pub min_confidence: f32,
    /// 关键目标类别(命中即触发可疑/严重事件)。
    pub watch_labels: Vec<String>,
}

impl Default for AuditEngine {
    fn default() -> Self {
        Self {
            detector: Box::new(NoopDetector),
            motion_threshold: 0.02,
            min_confidence: 0.55,
            watch_labels: vec!["person".to_string(), "car".to_string()],
        }
    }
}

impl AuditEngine {
    /// 创建审计引擎。
    pub fn new() -> Self {
        Self::default()
    }

    /// 注入检测器(ONNX YOLO 等)。
    pub fn with_detector(&mut self, detector: Box<dyn Detector>) {
        self.detector = detector;
    }

    /// 审计一帧:
    /// - `motion_ratio` 来自 VEScaner 帧差分析;
    /// - 命中关键目标且置信度达标 -> 可疑/严重事件;
    /// - 无目标但运动显著 -> 低级别记录。
    pub fn audit_frame(&self, camera_id: &str, frame: &[u8], width: usize, height: usize, motion_ratio: f32, ts: u64) -> Option<AuditEvent> {
        let detections: Vec<Detection> = self
            .detector
            .detect(frame, width, height)
            .into_iter()
            .filter(|d| d.confidence >= self.min_confidence)
            .collect();

        let watch_hits: Vec<&Detection> = detections.iter().filter(|d| self.watch_labels.contains(&d.label)).collect();

        let level = if !watch_hits.is_empty() && motion_ratio >= self.motion_threshold {
            AuditLevel::Critical
        } else if !watch_hits.is_empty() {
            AuditLevel::Suspicious
        } else if motion_ratio >= self.motion_threshold {
            AuditLevel::Info
        } else {
            return None;
        };

        let summary = if watch_hits.is_empty() {
            format!("检测到画面运动(比例 {:.2})", motion_ratio)
        } else {
            let labels = watch_hits.iter().map(|d| d.label.as_str()).collect::<Vec<_>>().join("、");
            format!("检出关键目标:{labels}(共 {} 个)", watch_hits.len())
        };

        Some(AuditEvent { ts, camera_id: camera_id.to_string(), level, summary, detections })
    }
}