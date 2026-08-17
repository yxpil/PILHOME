//! Core —— 负责协调总体
//!
//! 作为 PILHOME 的"中枢神经系统",Core 提供:
//! - [`message::MessageBus`] 模块间消息总线(广播 + 主题路由)
//! - [`registry::ModuleRegistry`] 插件模块注册表
//! - [`coordinator::Coordinator`] 总协调器(生命周期、心跳监控)
//!
//! PlugsCores 下的全部模块作为插件挂载到 Core 之上。

pub mod coordinator;
pub mod message;
pub mod registry;

pub use coordinator::Coordinator;
pub use message::{Message, MessageBus};
pub use registry::{ModuleInfo, ModuleRegistry, ModuleStatus};