//! 目标检测:结果模型与检测器抽象。

use serde::{Deserialize, Serialize};

/// 单个检测目标。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Detection {
    /// 类别(如 `person` / `car` / `dog`)。
    pub label: String,
    /// 置信度(0.0 ~ 1.0)。
    pub confidence: f32,
    /// 归一化检测框(0.0 ~ 1.0)。
    pub bbox: (f32, f32, f32, f32),
}

/// 检测器抽象:接入 ONNX YOLO / TFLite 模型实现。
///
/// 输入为灰度帧与宽高;输出检测列表。
pub trait Detector: Send + Sync {
    /// 对一帧执行检测。
    fn detect(&self, frame: &[u8], width: usize, height: usize) -> Vec<Detection>;
}

/// 空检测器:未配置权重时返回空列表(优雅降级)。
#[derive(Debug, Clone, Copy, Default)]
pub struct NoopDetector;

impl Detector for NoopDetector {
    fn detect(&self, _frame: &[u8], _width: usize, _height: usize) -> Vec<Detection> {
        Vec::new()
    }
}