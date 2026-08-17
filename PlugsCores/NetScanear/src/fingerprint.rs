//! 设备指纹分析:按开放端口与厂商 OUI 自动推断设备类型。

use serde::{Deserialize, Serialize};

/// 指纹分析结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fingerprint {
    /// 推断的设备类型(contact/presence/camera/gateway/…)。
    pub kind: &'static str,
    /// 可读描述(如 `网络摄像头` / `路由器` / `NAS`)。
    pub label: String,
    /// 可信度(0.0 ~ 1.0)。
    pub confidence: f32,
}

/// 常见厂商 MAC OUI 前缀(前 3 字节)。
const OUIS: &[(&str, &str)] = &[
    ("f0:2f:74", "小米"), ("94:65:2d", "小米"), ("78:11:dc", "小米"),
    ("44:03:2c", "华为"), ("e0:19:1d", "华为"), ("24:09:95", "华为"),
    ("28:6e:d4", "美的"), ("20:36:2b", "美的"),
    ("64:09:80", "海尔"), ("dc:8a:7a", "海尔"),
    ("dc:2b:2a", "苹果"), ("a4:83:e7", "苹果"), ("3c:22:fb", "苹果"), ("f0:18:98", "苹果"),
    ("34:8a:ae", "TP-Link"), ("50:c7:bf", "TP-Link"), ("14:cf:92", "TP-Link"),
    ("30:de:4b", "小米"), ("64:6e:97", "小米"),
];

/// 端口 -> 设备描述。
fn describe_ports(ports: &[u16]) -> (&'static str, &'static str, f32) {
    if ports.contains(&554) {
        ("camera", "网络摄像头", 0.85)
    } else if ports.contains(&5353) && ports.contains(&7000) {
        ("unknown", "苹果设备(HomeKit/mDNS)", 0.7)
    } else if ports.contains(&8123) {
        ("gateway", "Home Assistant 网关", 0.9)
    } else if ports.contains(&1883) {
        ("gateway", "MQTT 网关/设备", 0.7)
    } else if ports.contains(&9100) {
        ("unknown", "打印机", 0.8)
    } else if ports.contains(&22) {
        ("unknown", "NAS/服务器(SSH)", 0.6)
    } else if ports.contains(&80) || ports.contains(&443) {
        ("unknown", "路由器/网络设备(Web)", 0.5)
    } else {
        ("unknown", "未知设备", 0.2)
    }
}

/// 按 MAC OUI 识别厂商。
pub fn vendor_from_mac(mac: &str) -> Option<&'static str> {
    let mac = mac.to_lowercase();
    for (oui, vendor) in OUIS {
        if mac.starts_with(oui) {
            return Some(vendor);
        }
    }
    None
}

/// 综合指纹:端口 + MAC。
pub fn analyze(open_ports: &[u16], mac: Option<&str>) -> Fingerprint {
    let (kind, label, mut confidence) = describe_ports(open_ports);
    let mut label = label.to_string();
    if let Some(vendor) = mac.and_then(vendor_from_mac) {
        label = format!("{label}({vendor})");
        confidence = (confidence + 0.15).min(1.0);
    }
    Fingerprint { kind, label, confidence }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camera_by_rtsp_port() {
        let fp = analyze(&[554], None);
        assert_eq!(fp.kind, "camera");
        assert!(fp.confidence > 0.8);
    }

    #[test]
    fn xiaomi_oui_detected() {
        assert_eq!(vendor_from_mac("f0:2f:74:12:34:56"), Some("小米"));
        assert_eq!(vendor_from_mac("a4:83:e7:aa:bb:cc"), Some("苹果"));
        assert_eq!(vendor_from_mac("00:11:22:33:44:55"), None);
    }

    #[test]
    fn vendor_boosts_confidence() {
        let fp = analyze(&[80], Some("f0:2f:74:12:34:56"));
        assert!(fp.label.contains("小米"));
    }
}