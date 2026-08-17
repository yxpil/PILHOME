//! NetLinker —— 网络设备连接程序
//!
//! 负责与家庭网络中的各类设备建立、维持与释放连接:
//! - [`device`] 统一设备模型(种类、状态、心跳)
//! - [`registry`] 设备注册中心(并发安全台账)
//! - [`connector`] 连接管理器(连接状态机与自动重试)

pub mod connector;
pub mod wol;
pub mod device;
pub mod registry;

pub use connector::{Connector, ConnState};
pub use wol::{magic_packet, parse_mac, send_wol};
pub use device::{Device, DeviceKind, DeviceState};
pub use registry::DeviceRegistry;