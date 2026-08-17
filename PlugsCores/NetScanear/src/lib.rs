//! NetScanear —— 网络设备发现程序
//!
//! 多维自动发现:TCP 端口探测 + ARP 表查询 + mDNS 服务发现,
//! 配合端口指纹与 MAC OUI 自动分析设备类型:
//! - [`scanner`] 并发 TCP 端口探测
//! - [`discovery`] 陌生设备判定与发现报告
//! - [`arp`] ARP 缓存表查询(IP ↔ MAC)
//! - [`fingerprint`] 设备指纹分析(类型 / 厂商)
//! - [`mdns`] mDNS 零配置服务发现(小米/苹果/华为/美的/海尔…)

pub mod arp;
pub mod discovery;
pub mod fingerprint;
pub mod mdns;
pub mod scanner;

pub use arp::{ArpEntry, read_arp_table};
pub use discovery::{analyze, DiscoveryReport, UnknownHost};
pub use fingerprint::{Fingerprint, analyze as fingerprint, vendor_from_mac};
pub use mdns::{MdnsDevice, discover_mdns};
pub use scanner::{Host, ScanConfig, ScanResult, scan_network};