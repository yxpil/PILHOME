//! 厂商生态接入:统一适配器框架 + mDNS 自动发现归类。
//!
//! - **自动发现(真实可用)**:mDNS 零配置协议自动发现局域网内的
//!   小米(miio)、华为(easylink)、美的(midea)、海尔(hiaircon)、
//!   苹果(HomeKit/AirPlay)等设备,无需任何密钥;
//! - **云 SDK 适配器(接口预留)**:`VendorAdapter` trait 为各厂商
//!   云平台(米家 / 华为 Hilink / 美的美居 / 海尔智家 / HomeKit)
//!   预留,填入开发者密钥与协议实现即可启用,网关代码无需改动。

use pilhome_netscanear::MdnsDevice;

/// 厂商发现的设备。
#[derive(Debug, Clone, serde::Serialize)]
pub struct VendorDevice {
    /// 实例名。
    pub name: String,
    /// 厂商(小米/华为/美的/海尔/苹果/通用)。
    pub vendor: &'static str,
    /// 服务类型。
    pub service: String,
    /// IP。
    pub ip: Option<String>,
    /// 端口。
    pub port: Option<u16>,
    /// 型号(TXT 属性,尽力解析)。
    pub model: String,
    /// 额外属性。
    pub attrs: std::collections::BTreeMap<String, String>,
}

#[allow(dead_code)] // 预留:云 SDK 接入指南(填入密钥即启用)
/// 厂商云 SDK 适配器接口。
///
/// 实现示例:
/// ```ignore
/// struct XiaomiCloud { key: String }
/// impl VendorAdapter for XiaomiCloud {
///     fn vendor(&self) -> &'static str { "小米" }
///     fn discover(&self) -> Vec<VendorDevice> { /* 米家云 API 拉取设备 */ }
///     fn control(&self, device_id: &str, action: &str) -> bool { /* 云端下发 */ }
/// }
/// ```
pub trait VendorAdapter {
    /// 厂商名。
    fn vendor(&self) -> &'static str;
    /// 拉取设备列表。
    fn discover(&self) -> Vec<VendorDevice>;
    /// 控制设备(on/off/…)。
    fn control(&self, device_id: &str, action: &str) -> bool;
}

/// 执行 mDNS 自动发现并归类厂商。
pub fn discover_mdns_devices() -> Vec<VendorDevice> {
    let found = pilhome_netscanear::discover_mdns();
    found.iter().map(to_vendor_device).collect()
}

/// 将 mDNS 结果转为厂商设备。
fn to_vendor_device(d: &MdnsDevice) -> VendorDevice {
    let mut attrs = std::collections::BTreeMap::new();
    for (k, v) in &d.txt {
        attrs.insert(k.clone(), v.clone());
    }
    let model = attrs
        .get("md")
        .or_else(|| attrs.get("model"))
        .or_else(|| attrs.get("am"))
        .cloned()
        .unwrap_or_default();
    VendorDevice {
        name: d.name.clone(),
        vendor: d.vendor_hint(),
        service: d.service.clone(),
        ip: d.ip.map(|ip| ip.to_string()),
        port: d.port,
        model,
        attrs,
    }
}

/// 支持的厂商清单(用于状态页展示)。
pub const SUPPORTED_VENDORS: &[&str] = &["小米", "华为", "美的", "海尔", "苹果"];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn mdns_device_vendor_classified() {
        let mut txt = BTreeMap::new();
        txt.insert("md".to_string(), "lumi-gateway".to_string());
        let d = MdnsDevice {
            name: "网关".into(),
            service: "_miio._udp.local".into(),
            ip: Some("192.168.1.9".parse().unwrap()),
            port: Some(54321),
            txt,
        };
        let vd = to_vendor_device(&d);
        assert_eq!(vd.vendor, "小米");
        assert_eq!(vd.model, "lumi-gateway");
        assert_eq!(vd.ip.as_deref(), Some("192.168.1.9"));
    }
}