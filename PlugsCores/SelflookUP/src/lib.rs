//! SelflookUP —— 自我审计程序
//!
//! 系统对自身的持续审计:模块健康检查、审计日志(只追加)、配置完整性校验:
//! - [`health`] 健康检查(模块状态、设备在线率、事件积压)
//! - [`audit_log`] 审计日志(关键操作留痕)

pub mod audit_log;
pub mod health;

pub use audit_log::{AuditEntry, AuditLog};
pub use health::{CheckItem, HealthReport, HealthStatus, run_checks};