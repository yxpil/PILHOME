//! 全自动设备发现:ARP 表 + TCP 扫描 + mDNS + 指纹分析 → 设备台账。
//!
//! 流程:并发扫描局域网 → ARP 表补全 MAC → 指纹分析类型/厂商 →
//! mDNS 发现智能家居设备 → 全部合并入库(重复设备按 IP/MAC 去重)。

use crate::state::AppState;
use pilhome_netscanear::{ScanConfig, scan_network};
use serde::Serialize;

/// 一次发现任务的汇总报告。
#[derive(Debug, Clone, Serialize)]
pub struct DiscoverReport {
    pub scanned_hosts: usize,
    pub alive_hosts: usize,
    pub arp_entries: usize,
    pub mdns_devices: usize,
    pub new_devices: usize,
    pub total_devices: usize,
}

/// 执行一轮全自动发现(同步,耗时数秒)。
pub fn auto_discover(state: &AppState) -> DiscoverReport {
    let cfg = ScanConfig { prefix: "192.168.1".to_string(), ports: vec![22, 80, 443, 554, 1883, 5353, 8123, 9100], timeout_ms: 300, concurrency: 64 };
    let scan = scan_network(&cfg);
    let arp = pilhome_netscanear::read_arp_table();
    let now = crate::state::now_secs();

    // MAC 反查表(IP -> mac)。
    let mut mac_by_ip = std::collections::BTreeMap::new();
    for entry in &arp {
        mac_by_ip.insert(entry.ip.to_string(), entry.mac.clone());
    }

    let mut new_devices = 0usize;

    // 1) TCP 扫描结果入库(带指纹分析)。
    for host in &scan.hosts {
        let mac = mac_by_ip.get(&host.ip.to_string()).cloned();
        let fp = pilhome_netscanear::fingerprint(&host.open_ports, mac.as_deref());
        let kind = kind_from_fp(fp.kind);
        let mut attrs = std::collections::BTreeMap::new();
        attrs.insert("ports".to_string(), host.open_ports.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(","));
        attrs.insert("fingerprint".to_string(), fp.label.clone());
        attrs.insert("confidence".to_string(), format!("{:.2}", fp.confidence));
        if let Some(m) = &mac {
            attrs.insert("mac".to_string(), m.clone());
            if let Some(v) = pilhome_netscanear::vendor_from_mac(m) {
                attrs.insert("vendor".to_string(), v.to_string());
            }
        }
        let id = format!("net_{}", host.ip);
        if state.devices.get(&id).is_none() {
            new_devices += 1;
        }
        let mut device = pilhome_netlinker::Device::new(&id, &host.ip.to_string(), kind, format!("tcp://{}", host.ip));
        device.state = pilhome_netlinker::DeviceState::Online;
        device.last_seen = now;
        device.attrs = attrs;
        state.devices.upsert(device);
    }

    // 2) ARP 表中未扫描到的设备也入库(静默设备)。
    for entry in &arp {
        let id = format!("net_{}", entry.ip);
        if state.devices.get(&id).is_some() {
            continue;
        }
        let mut attrs = std::collections::BTreeMap::new();
        attrs.insert("mac".to_string(), entry.mac.clone());
        if let Some(v) = pilhome_netscanear::vendor_from_mac(&entry.mac) {
            attrs.insert("vendor".to_string(), v.to_string());
        }
        let mut device = pilhome_netlinker::Device::new(&id, &entry.ip.to_string(), pilhome_netlinker::DeviceKind::Unknown, format!("arp://{}", entry.ip));
        device.state = pilhome_netlinker::DeviceState::Online;
        device.last_seen = now;
        device.attrs = attrs;
        state.devices.upsert(device);
        new_devices += 1;
    }

    // 3) mDNS 发现的智能家居设备入库。
    let vendor_devices = crate::vendor::discover_mdns_devices();
    for vd in &vendor_devices {
        let Some(ip) = &vd.ip else { continue };
        let id = format!("mdns_{}", ip);
        if state.devices.get(&id).is_none() {
            new_devices += 1;
        }
        let mut attrs = std::collections::BTreeMap::new();
        attrs.insert("vendor".to_string(), vd.vendor.to_string());
        attrs.insert("service".to_string(), vd.service.clone());
        if !vd.model.is_empty() {
            attrs.insert("model".to_string(), vd.model.clone());
        }
        if let Some(port) = vd.port {
            attrs.insert("port".to_string(), port.to_string());
        }
        let mut device = pilhome_netlinker::Device::new(&id, &vd.name, pilhome_netlinker::DeviceKind::Unknown, format!("mdns://{}", ip));
        device.state = pilhome_netlinker::DeviceState::Online;
        device.last_seen = now;
        device.attrs = attrs;
        state.devices.upsert(device);
    }

    // 陌生设备告警(白名单 = 已入库设备 IP)。
    let whitelist: std::collections::BTreeSet<String> = state
        .devices
        .all()
        .iter()
        .filter_map(|d| d.address.split("://").nth(1).map(|s| s.to_string()))
        .collect();
    let report = pilhome_netscanear::analyze(&scan, &whitelist);
    for host in report.unknown {
        state.events.record(
            crate::events::Level::Alert,
            "netscanear",
            format!("发现陌生设备 {}:{}", host.ip, host.open_ports.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(",")),
            Some(host.ip.as_str()),
            None,
        );
    }

    DiscoverReport {
        scanned_hosts: scan.hosts.len(),
        alive_hosts: scan.alive_count,
        arp_entries: arp.len(),
        mdns_devices: vendor_devices.len(),
        new_devices,
        total_devices: state.devices.len(),
    }
}
/// 指纹类型 -> 设备种类。
fn kind_from_fp(kind: &str) -> pilhome_netlinker::DeviceKind {
    match kind {
        "camera" => pilhome_netlinker::DeviceKind::Camera,
        "contact" => pilhome_netlinker::DeviceKind::Contact,
        "presence" => pilhome_netlinker::DeviceKind::Presence,
        "lock" => pilhome_netlinker::DeviceKind::Lock,
        "sensor" => pilhome_netlinker::DeviceKind::Sensor,
        _ => pilhome_netlinker::DeviceKind::Unknown,
    }
}