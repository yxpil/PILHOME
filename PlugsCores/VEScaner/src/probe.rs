//! 视频设备探测:扫描网段内开放 RTSP / ONVIF 端口的主机。

use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::time::Duration;

/// 探测到的视频设备。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoDevice {
    /// 设备 ID(取 IP)。
    pub id: String,
    /// 展示名。
    pub name: String,
    /// 主机 IP。
    pub ip: Ipv4Addr,
    /// RTSP 端口。
    pub rtsp_port: u16,
    /// ONVIF 端口(0 表示未开放)。
    pub onvif_port: u16,
    /// 推流地址模板(实际地址由 `rtsp://user:pass@ip:port/stream1` 拼接)。
    pub rtsp_url: String,
}

/// 探测配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeConfig {
    /// 网段前缀,如 `192.168.1`。
    pub prefix: String,
    /// RTSP 默认端口。
    pub rtsp_port: u16,
    /// ONVIF 默认端口。
    pub onvif_port: u16,
    /// 连接超时(毫秒)。
    pub timeout_ms: u64,
    /// 并发线程数。
    pub concurrency: usize,
}

impl Default for ProbeConfig {
    fn default() -> Self {
        Self {
            prefix: "192.168.1".to_string(),
            rtsp_port: 554,
            onvif_port: 8000,
            timeout_ms: 400,
            concurrency: 32,
        }
    }
}

fn tcp_open(ip: Ipv4Addr, port: u16, timeout: Duration) -> bool {
    TcpStream::connect_timeout(&SocketAddr::new(IpAddr::V4(ip), port), timeout).is_ok()
}

/// 对单个主机探测 RTSP / ONVIF 端口。
fn probe_host(ip: Ipv4Addr, cfg: &ProbeConfig) -> Option<VideoDevice> {
    let timeout = Duration::from_millis(cfg.timeout_ms);
    let rtsp_open = tcp_open(ip, cfg.rtsp_port, timeout);
    let onvif_open = tcp_open(ip, cfg.onvif_port, timeout);
    if !rtsp_open && !onvif_open {
        return None;
    }
    let port = if rtsp_open { cfg.rtsp_port } else { 0 };
    Some(VideoDevice {
        id: ip.to_string(),
        name: format!("视频设备 {}", ip),
        ip,
        rtsp_port: port,
        onvif_port: if onvif_open { cfg.onvif_port } else { 0 },
        rtsp_url: format!("rtsp://{}:{}/stream1", ip, cfg.rtsp_port),
    })
}

/// 探测整个网段(阻塞调用)。
pub fn probe_network(cfg: &ProbeConfig) -> Vec<VideoDevice> {
    let hosts: Vec<Ipv4Addr> = (1..=254)
        .map(|n| format!("{}.{}", cfg.prefix, n).parse::<Ipv4Addr>().unwrap_or(Ipv4Addr::new(0, 0, 0, 0)))
        .collect();
    let workers = cfg.concurrency.max(1);
    std::thread::scope(|scope| {
        let mut handles = Vec::with_capacity(workers);
        let chunk = hosts.len().div_ceil(workers);
        for part in hosts.chunks(chunk) {
            let part = part.to_vec();
            let cfg = cfg.clone();
            handles.push(scope.spawn(move || {
                part.into_iter().filter_map(|ip| probe_host(ip, &cfg)).collect::<Vec<_>>()
            }));
        }
        let mut out = Vec::new();
        for h in handles {
            out.extend(h.join().unwrap_or_default());
        }
        out
    })
}