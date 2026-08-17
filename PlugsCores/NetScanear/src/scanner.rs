//! 扫描器实现:并发 TCP 探测。

use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::time::{Duration, Instant};

/// 单个扫描到的主机。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Host {
    /// IPv4 地址。
    pub ip: Ipv4Addr,
    /// 探测到的开放端口。
    pub open_ports: Vec<u16>,
    /// 主机名(未解析到时为空)。
    pub hostname: String,
}

impl Host {
    fn new(ip: Ipv4Addr) -> Self {
        Self { ip, open_ports: Vec::new(), hostname: String::new() }
    }

    /// 是否发现任何开放端口。
    pub fn is_alive(&self) -> bool {
        !self.open_ports.is_empty()
    }
}

/// 扫描配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    /// 网段前缀,如 `192.168.1`(对 `1..=254` 主机探测)。
    pub prefix: String,
    /// 探测端口列表。
    pub ports: Vec<u16>,
    /// 单端口连接超时(毫秒)。
    pub timeout_ms: u64,
    /// 并发探测线程数。
    pub concurrency: usize,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            prefix: "192.168.1".to_string(),
            ports: vec![22, 80, 443, 445, 3389, 8123, 1883, 554],
            timeout_ms: 300,
            concurrency: 32,
        }
    }
}

/// 一次扫描的完整结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    /// 扫描耗时(毫秒)。
    pub elapsed_ms: u64,
    /// 活跃主机数。
    pub alive_count: usize,
    /// 全部主机明细。
    pub hosts: Vec<Host>,
}

/// 对单个主机执行端口探测。
fn probe_host(ip: Ipv4Addr, ports: &[u16], timeout: Duration) -> Host {
    let mut host = Host::new(ip);
    for port in ports {
        let addr = SocketAddr::new(IpAddr::V4(ip), *port);
        if TcpStream::connect_timeout(&addr, timeout).is_ok() {
            host.open_ports.push(*port);
        }
    }
    // 尽力反查主机名(反向 DNS),失败则保持为空(优雅降级)。
    if host.is_alive() {
        if let Ok(output) = std::process::Command::new("nslookup").arg(ip.to_string()).output() {
            let text = String::from_utf8_lossy(&output.stdout);
            for line in text.lines() {
                let line = line.trim();
                if let Some(pos) = line.find("name =") {
                    host.hostname = line[pos + 6..].trim().to_string();
                    break;
                }
            }
        }
    }
    host
}

/// 执行全网段扫描(阻塞调用;建议放入独立线程/任务)。
pub fn scan_network(cfg: &ScanConfig) -> ScanResult {
    let started = Instant::now();
    let hosts: Vec<Ipv4Addr> = (1..=254)
        .map(|n| {
            format!("{}.{}", cfg.prefix, n)
                .parse::<Ipv4Addr>()
                .unwrap_or(Ipv4Addr::new(0, 0, 0, 0))
        })
        .collect();

    let timeout = Duration::from_millis(cfg.timeout_ms);
    let workers = cfg.concurrency.max(1);
    let results: Vec<Host> = std::thread::scope(|scope| {
        let mut handles = Vec::with_capacity(workers);
        let chunk = hosts.len().div_ceil(workers);
        for chunk_hosts in hosts.chunks(chunk) {
            let chunk_hosts = chunk_hosts.to_vec();
            let ports = cfg.ports.clone();
            handles.push(scope.spawn(move || {
                chunk_hosts.into_iter().map(|ip| probe_host(ip, &ports, timeout)).collect::<Vec<_>>()
            }));
        }
        let mut all = Vec::new();
        for h in handles {
            all.extend(h.join().unwrap_or_default());
        }
        all
    });

    let alive: Vec<Host> = results.into_iter().filter(|h| h.is_alive()).collect();
    let alive_count = alive.len();
    ScanResult {
        elapsed_ms: started.elapsed().as_millis() as u64,
        alive_count,
        hosts: alive,
    }
}