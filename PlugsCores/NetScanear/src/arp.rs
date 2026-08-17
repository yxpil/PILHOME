//! ARP 表查询:读取系统 ARP 缓存,补全 IP ↔ MAC 映射。
//!
//! Windows 通过 `arp -a` 输出解析;解析失败返回空表(优雅降级)。

use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;
use std::process::Command;

/// 一条 ARP 记录。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArpEntry {
    /// IPv4 地址。
    pub ip: Ipv4Addr,
    /// MAC 地址(统一 `aa:bb:cc:dd:ee:ff` 小写格式)。
    pub mac: String,
    /// 类型(动态 / 静态)。
    pub kind: String,
}

/// 读取系统 ARP 表。
pub fn read_arp_table() -> Vec<ArpEntry> {
    let output = match Command::new("arp").arg("-a").output() {
        Ok(o) if o.status.success() => o.stdout,
        _ => return Vec::new(),
    };
    parse_arp_output(&String::from_utf8_lossy(&output))
}

/// 解析 `arp -a` 输出(供测试与跨平台复用)。
pub fn parse_arp_output(text: &str) -> Vec<ArpEntry> {
    let mut entries = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        // Windows 格式: `192.168.1.1           aa-bb-cc-dd-ee-ff     动态`
        let mut parts = line.split_whitespace();
        let Some(ip_str) = parts.next() else { continue };
        let Ok(ip) = ip_str.parse::<Ipv4Addr>() else { continue };
        let Some(mac_raw) = parts.next() else { continue };
        let Some(mac) = normalize_mac(mac_raw) else { continue };
        let kind = parts.next().unwrap_or("").to_string();
        entries.push(ArpEntry { ip, mac, kind });
    }
    entries
}

/// 规范化 MAC:`aa-bb-cc-dd-ee-ff` / `aabb.ccdd.eeff` / `aa:bb:..` → `aa:bb:cc:dd:ee:ff`。
pub fn normalize_mac(raw: &str) -> Option<String> {
    let raw = raw.trim().to_lowercase();
    // 去掉常见分隔符。
    let compact: String = raw.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    if compact.len() != 12 {
        return None;
    }
    let bytes: Vec<String> = (0..6)
        .map(|i| compact[i * 2..i * 2 + 2].to_string())
        .collect();
    Some(bytes.join(":"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_windows_arp() {
        let text = "\n接口: 192.168.1.100 --- 0x3\n  Internet 地址         物理地址              类型\n  192.168.1.1           aa-bb-cc-dd-ee-ff     动态\n  192.168.1.105         12-34-56-78-9a-bc     动态\n";
        let entries = parse_arp_output(text);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].ip, Ipv4Addr::new(192, 168, 1, 1));
        assert_eq!(entries[0].mac, "aa:bb:cc:dd:ee:ff");
        assert_eq!(entries[0].kind, "动态");
    }

    #[test]
    fn normalizes_mac_formats() {
        assert_eq!(normalize_mac("AA-BB-CC-DD-EE-FF").as_deref(), Some("aa:bb:cc:dd:ee:ff"));
        assert_eq!(normalize_mac("aabb.ccdd.eeff").as_deref(), Some("aa:bb:cc:dd:ee:ff"));
        assert_eq!(normalize_mac("aa:bb:cc:dd:ee:ff").as_deref(), Some("aa:bb:cc:dd:ee:ff"));
        assert!(normalize_mac("not-a-mac").is_none());
    }
}