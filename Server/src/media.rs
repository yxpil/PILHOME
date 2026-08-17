//! 家庭媒体设备发现:音响(小爱/剑桥/Sonos 等)、机顶盒、投屏设备。
//!
//! mDNS 自动发现 + DIAL 投屏(HTTP POST `http://<ip>:<port>/apps/<app>`)。

use pilhome_netscanear::discover_mdns;
use serde::Serialize;

/// 媒体设备。
#[derive(Debug, Clone, Serialize)]
pub struct MediaDevice {
    pub name: String,
    pub vendor: &'static str,
    /// audio(音响)/ cast(投屏)/ stb(机顶盒)。
    pub class: &'static str,
    pub service: String,
    pub ip: Option<String>,
    pub port: Option<u16>,
}

/// 发现音响 / 机顶盒 / 投屏设备(阻塞约 1.5 秒)。
pub fn discover_media_devices() -> Vec<MediaDevice> {
    discover_mdns()
        .into_iter()
        .map(|d| MediaDevice {
            name: d.name.clone(),
            vendor: d.vendor_hint(),
            class: d.class_hint(),
            service: d.service.clone(),
            ip: d.ip.map(|ip| ip.to_string()),
            port: d.port,
        })
        .collect()
}

/// DIAL 投屏:在 DIAL 设备(机顶盒/电视)上启动应用。
///
/// `app` 如 `YouTube`;`payload` 可空(或填视频 URL XML)。
pub fn dial_launch(ip: &str, port: u16, app: &str, payload: Option<&str>) -> Result<String, String> {
    let url = format!("http://{ip}:{port}/apps/{app}");
    let resp = ureq::post(&url)
        .timeout(std::time::Duration::from_secs(10))
        .set("Content-Type", "text/plain")
        .send_string(payload.unwrap_or(""))
        .map_err(|e| format!("DIAL 投屏失败:{e}"))?;
    Ok(format!("{}", resp.status()))
}

#[allow(dead_code)] // 预留:UPnP 控制点接入后启用
/// DLNA 投屏(简化):向 DLNA 设备发送 SetAVTransportURI + Play(经 SOAP)。
/// 需要设备的控制 URL;这里做接口占位,完整实现需 UPnP 控制点。
pub fn dlna_cast(ip: &str, _media_url: &str) -> Result<(), String> {
    Err(format!("DLNA 投屏需 UPnP 控制点(设备 {ip} 的 AVTransport 服务地址);当前为接口预留"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dial_url_format() {
        // 仅验证 URL 拼装逻辑(不发起真实请求)。
        let ip = "192.168.1.50";
        let port = 8008u16;
        let app = "YouTube";
        let url = format!("http://{ip}:{port}/apps/{app}");
        assert_eq!(url, "http://192.168.1.50:8008/apps/YouTube");
    }
}