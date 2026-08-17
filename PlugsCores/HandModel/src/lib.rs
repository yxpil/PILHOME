//! HandModel —— 模块控制程序
//!
//! 作为"控制手",统一管理各 PlugsCores 模块的启停、参数下发与状态收集:
//! - [`params`] 参数定义与取值
//! - [`control`] 模块控制器(启停 / 参数 / 状态)

pub mod control;
pub mod params;

pub use control::{Controller, ModuleHandle};
pub use params::{ParamDef, ParamValue};