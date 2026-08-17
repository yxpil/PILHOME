//! 模型信息与种类。

use serde::{Deserialize, Serialize};

/// 模型任务种类。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelKind {
    /// 目标检测(YOLO 系)
    Detection,
    /// 图像分类
    Classification,
    /// 姿态 / 手势识别
    Gesture,
    /// 行为 / 序列模型
    Behavior,
    /// 规则引擎(轻量,无权重)
    Rule,
}

/// 模型运行状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelStatus {
    /// 已注册未加载
    Unloaded,
    /// 加载中
    Loading,
    /// 就绪可推理
    Ready,
    /// 推理失败(显存不足 / 权重损坏等)
    Error,
}

/// 边缘模型信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// 模型 ID(如 `yolo_v8n_person`)。
    pub id: String,
    /// 展示名(如 `人员检测 YOLOv8n`)。
    pub name: String,
    /// 任务种类。
    pub kind: ModelKind,
    /// 权重格式(onnx / tflite / rule)。
    pub format: String,
    /// 权重文件路径。
    pub path: String,
    /// 运行状态。
    pub status: ModelStatus,
    /// 最近一次推理平均耗时(毫秒)。
    pub avg_latency_ms: f32,
}

impl ModelInfo {
    /// 构造模型信息(初始未加载)。
    pub fn new(id: impl Into<String>, name: impl Into<String>, kind: ModelKind, format: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            kind,
            format: format.into(),
            path: path.into(),
            status: ModelStatus::Unloaded,
            avg_latency_ms: 0.0,
        }
    }
}