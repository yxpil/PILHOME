//! 陌生设备发现:扫描结果 vs 白名单。

use crate::scanner::{Host, ScanResult};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// 陌生主机(不在白名单内的活跃主机)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnknownHost {
    /// IP 地址。
    pub ip: String,
    /// 开放端口。
    pub open_ports: Vec<u16>,
    /// 主机名(可空)。
    pub hostname: String,
}

/// 一次扫描的发现报告。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryReport {
    /// 扫描耗时(毫秒)。
    pub elapsed_ms: u64,
    /// 活跃主机总数。
    pub alive_total: usize,
    /// 白名单主机数。
    pub known_total: usize,
    /// 陌生主机数。
    pub unknown_total: usize,
    /// 陌生主机明细。
    pub unknown: Vec<UnknownHost>,
}

/// 对比扫描结果与白名单,产出陌生设备报告。
pub fn analyze(result: &ScanResult, whitelist: &BTreeSet<String>) -> DiscoveryReport {
    let unknown: Vec<UnknownHost> = result
        .hosts
        .iter()
        .filter(|h| !whitelist.contains(&h.ip.to_string()))
        .map(|h: &Host| UnknownHost {
            ip: h.ip.to_string(),
            open_ports: h.open_ports.clone(),
            hostname: h.hostname.clone(),
        })
        .collect();

    let known_total = result.alive_count.saturating_sub(unknown.len());
    DiscoveryReport {
        elapsed_ms: result.elapsed_ms,
        alive_total: result.alive_count,
        known_total,
        unknown_total: unknown.len(),
        unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::{Host, ScanResult};
    use std::net::Ipv4Addr;
    use std::collections::BTreeSet;

    fn host(ip: &str, ports: &[u16]) -> Host {
        Host {
            ip: ip.parse::<Ipv4Addr>().unwrap(),
            open_ports: ports.to_vec(),
            hostname: String::new(),
        }
    }

    #[test]
    fn whitelist_filters_unknown() {
        let result = ScanResult {
            elapsed_ms: 100,
            alive_count: 2,
            hosts: vec![host("192.168.1.10", &[80]), host("192.168.1.99", &[22, 445])],
        };
        let mut whitelist = BTreeSet::new();
        whitelist.insert("192.168.1.10".to_string());
        let report = analyze(&result, &whitelist);
        assert_eq!(report.unknown_total, 1);
        assert_eq!(report.unknown[0].ip, "192.168.1.99");
        assert_eq!(report.unknown[0].open_ports, vec![22, 445]);
    }
}
