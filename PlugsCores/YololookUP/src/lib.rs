//! YololookUP —— 视频粗略代审计模块
//!
//! 以 YOLO 目标检测为核心,对视频流做"粗略代审计":
//! 周期性抓帧 -> 目标检测(人员 / 车辆 / 动物)-> 生成审计事件。
//! - [`detect`] 检测结果模型与检测器抽象
//! - [`audit`] 审计引擎(规则触发 + 周期摘要)

pub mod audit;
pub mod detect;

pub use audit::{AuditEngine, AuditEvent, AuditLevel};
pub use detect::{Detection, Detector};