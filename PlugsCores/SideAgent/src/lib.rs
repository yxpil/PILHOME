//! SideAgent —— 边缘AI模型
//!
//! 管理部署在边缘端的 AI 模型(入侵检测、手势识别、YOLO 目标检测等):
//! - [`model`] 模型信息与种类
//! - [`registry`] 模型注册表(加载 / 卸载 / 状态)
//! - [`runtime`] 推理运行时(规则引擎与可插拔 ONNX 后端)

pub mod model;
pub mod registry;
pub mod runtime;

pub use model::{ModelInfo, ModelKind, ModelStatus};
pub use registry::ModelRegistry;
pub use runtime::{Prediction, RuleRuntime, predict_rule};