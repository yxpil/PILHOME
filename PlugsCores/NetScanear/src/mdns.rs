//! mDNS 零配置服务发现。
//!
//! 向组播地址 `224.0.0.251:5353` 发送服务类型查询,解析响应中的
//! PTR / SRV / A / TXT 记录,自动发现局域网内声明了 mDNS 服务的设备
//! (苹果 HomeKit、小米、华为、美的、海尔、打印机、音箱等)。
//!
//! 纯 `std::net::UdpSocket` 实现,不依赖任何第三方库。

use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use std::time::Duration;

/// mDNS 组播地址。
const MDNS_ADDR: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 251);
const MDNS_PORT: u16 = 5353;

/// 常见智能家居服务类型(自动发现的目标)。
const QUERY_SERVICES: &[&str] = &[
    "_services._dns-sd._udp.local",
    "_hap._tcp.local",
    "_miio._udp.local",
    "_miiot._udp.local",
    "_easylink._tcp.local",
    "_midea._tcp.local",
    "_hiaircon._udp.local",
    "_airplay._tcp.local",
    "_raop._tcp.local",
    "_googlecast._tcp.local",
    "_xiaomi._tcp.local",            // 小爱音响
    "_dlna._tcp.local",              // DLNA(电视/盒子/投屏)
    "_dialsrv._tcp.local",           // DIAL(投屏/机顶盒)
    "_spotify-connect._tcp.local",   // 音箱(Spotify Connect)
    "_sonos._tcp.local",             // Sonos 音响
    "_airsonos._tcp.local",          // Sonos(旧)
    "_cambridgeaudio._tcp.local",    // 剑桥音响
];

/// 一个 mDNS 发现结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MdnsDevice {
    /// 实例名(如 `客厅灯` / `iPhone`)。
    pub name: String,
    /// 服务类型(如 `_hap._tcp.local`)。
    pub service: String,
    /// 设备 IP。
    pub ip: Option<Ipv4Addr>,
    /// 服务端口。
    pub port: Option<u16>,
    /// TXT 属性(如型号/厂商)。
    pub txt: std::collections::BTreeMap<String, String>,
}

impl MdnsDevice {
    /// 设备类别:音响(audio)/ 投屏(cast)/ 机顶盒(stb)/ 智能家居(home)/ 其他。
    pub fn class_hint(&self) -> &'static str {
        match self.service.as_str() {
            s if s.starts_with("_sonos") || s.starts_with("_airsonos") || s.starts_with("_spotify-connect") => "audio",
            s if s.starts_with("_xiaomi") => "audio",
            s if s.starts_with("_cambridgeaudio") => "audio",
            s if s.starts_with("_airplay") || s.starts_with("_raop") => "cast",
            s if s.starts_with("_googlecast") => "cast",
            s if s.starts_with("_dlna") || s.starts_with("_dialsrv") => "stb",
            s if s.starts_with("_hap") => "home",
            _ => "other",
        }
    }

    /// 由服务类型推断厂商。
    pub fn vendor_hint(&self) -> &'static str {
        match self.service.as_str() {
            s if s.starts_with("_hap") => "苹果",
            s if s.starts_with("_airplay") || s.starts_with("_raop") => "苹果",
            s if s.starts_with("_miio") || s.starts_with("_miiot") => "小米",
            s if s.starts_with("_easylink") => "华为",
            s if s.starts_with("_midea") => "美的",
            s if s.starts_with("_hiaircon") => "海尔",
            s if s.starts_with("_xiaomi") => "小米",
            s if s.starts_with("_cambridgeaudio") => "剑桥",
            _ => "通用",
        }
    }
}

/// 执行一次 mDNS 发现(阻塞约 1.5 秒,返回去重后的设备列表)。
pub fn discover_mdns() -> Vec<MdnsDevice> {
    // 绑定随机端口并加入组播组(Windows 系统 mDNS 服务占用 5353,故用随机端口)。
    let Ok(sock) = UdpSocket::bind("0.0.0.0:0") else { return Vec::new() };
    if sock.join_multicast_v4(&MDNS_ADDR, &Ipv4Addr::UNSPECIFIED).is_err() {
        return Vec::new();
    }
    let _ = sock.set_read_timeout(Some(Duration::from_millis(120)));

    // 构造查询包(多问题)。
    let mut query = Vec::new();
    query.extend_from_slice(&[0x00, 0x01]); // ID
    query.extend_from_slice(&[0x00, 0x00]); // flags
    query.extend_from_slice(&[(QUERY_SERVICES.len() >> 8) as u8, (QUERY_SERVICES.len() &0xff) as u8]);
    query.extend_from_slice(&[0, 0, 0, 0, 0, 0]);
    for service in QUERY_SERVICES {
        encode_name(service, &mut query);
        query.extend_from_slice(&[0x00, 0x0c, 0x00, 0x01]); // PTR, IN
    }

    let dst = SocketAddrV4::new(MDNS_ADDR, MDNS_PORT);
    let _ = sock.send_to(&query, dst);
    let _ = sock.send_to(&query, SocketAddrV4::new(Ipv4Addr::BROADCAST, MDNS_PORT));

    // 接收并解析。
    let mut devices: Vec<MdnsDevice> = Vec::new();
    let deadline = std::time::Instant::now() + Duration::from_millis(1500);
    let mut buf = [0u8; 4096];
    while std::time::Instant::now() < deadline {
        match sock.recv_from(&mut buf) {
            Ok((len, _)) => parse_response(&buf[..len], &mut devices),
            Err(_) => continue,
        }
    }

    // 去重:同名同服务同 IP。
    devices.sort_by(|a, b| {
        format!("{}|{}|{:?}", a.name, a.service, a.ip).cmp(&format!("{}|{}|{:?}", b.name, b.service, b.ip))
    });
    devices.dedup_by(|a, b| a.name == b.name && a.service == b.service && a.ip == b.ip);
    devices
}

/// DNS 名称编码(长度前缀 + 结尾 0)。
fn encode_name(name: &str, out: &mut Vec<u8>) {
    for label in name.split('.') {
        if label.is_empty() {
            continue;
        }
        out.push(label.len() as u8);
        out.extend_from_slice(label.as_bytes());
    }
    out.push(0);
}

/// 解析 DNS 响应,提取 PTR/SRV/A/TXT 记录。
fn parse_response(data: &[u8], devices: &mut Vec<MdnsDevice>) {
    if data.len() < 12 {
        return;
    }
    let ancount = u16::from_be_bytes([data[6], data[7]]) as usize;
    let qdcount = u16::from_be_bytes([data[4], data[5]]) as usize;
    let mut offset = 12usize;
    // 跳过问题区。
    for _ in 0..qdcount {
        let (_, next) = read_name(data, offset);
        offset = next + 4;
        if offset >= data.len() {
            return;
        }
    }

    let mut pending: Vec<(String, String, Option<Ipv4Addr>, Option<u16>, Vec<String>)> = Vec::new();

    for _ in 0..ancount {
        if offset + 10 > data.len() {
            return;
        }
        let (name, next) = read_name(data, offset);
        let rtype = u16::from_be_bytes([data[next], data[next + 1]]);
        let rdlength = u16::from_be_bytes([data[next + 8], data[next + 9]]) as usize;
        let rdata_start = next + 10;
        if rdata_start + rdlength > data.len() {
            return;
        }
        let rdata = &data[rdata_start..rdata_start + rdlength];
        offset = rdata_start + rdlength;

        match rtype {
            12 => { // PTR
                let (target, _) = read_name(data, rdata_start);
                pending.push((name, target, None, None, Vec::new()));
            }
            33 => { // SRV
                if rdlength >= 6 {
                    let port = u16::from_be_bytes([rdata[4], rdata[5]]);
                    let (target, _) = read_name(data, rdata_start + 6);
                    pending.push((name, target, None, Some(port), Vec::new()));
                }
            }
            1 => { // A
                if rdlength >= 4 {
                    let ip = Ipv4Addr::new(rdata[0], rdata[1], rdata[2], rdata[3]);
                    pending.push((name, String::new(), Some(ip), None, Vec::new()));
                }
            }
            16 => { // TXT
                let mut txt = Vec::new();
                let mut pos = 0usize;
                while pos < rdlength {
                    let len = rdata[pos] as usize;
                    pos += 1;
                    if pos + len <= rdlength {
                        let kv = String::from_utf8_lossy(&rdata[pos..pos + len]).to_string();
                        txt.push(kv);
                        pos += len;
                    } else {
                        break;
                    }
                }
                pending.push((name, String::new(), None, None, txt));
            }
            _ => {}
        }
    }

    // 归并:实例名 -> 完整设备。
    let mut map: std::collections::BTreeMap<String, MdnsDevice> = std::collections::BTreeMap::new();
    for (name, target, ip, port, txt) in pending {
        let entry = map.entry(name.clone()).or_insert_with(|| MdnsDevice {
            name: name.clone(),
            service: String::new(),
            ip: None,
            port: None,
            txt: std::collections::BTreeMap::new(),
        });
        if !target.is_empty() {
            if target.ends_with(".local") {
                entry.service = target;
            } else {
                entry.name = target;
            }
        }
        if ip.is_some() {
            entry.ip = ip;
        }
        if port.is_some() {
            entry.port = port;
        }
        for kv in txt {
            if let Some((k, v)) = kv.split_once('=') {
                entry.txt.insert(k.to_string(), v.to_string());
            } else {
                entry.txt.insert(kv, String::new());
            }
        }
    }
    devices.extend(map.into_values());
}

/// 读取 DNS 名字(支持压缩指针)。返回 (名字, 消费后的偏移)。
fn read_name(data: &[u8], start: usize) -> (String, usize) {
    let mut labels = Vec::new();
    let mut offset = start;
    let mut jumped = false;
    let mut end = start;
    loop {
        if offset >= data.len() {
            break;
        }
        let len = data[offset] as usize;
        if len == 0 {
            offset += 1;
            if !jumped {
                end = offset;
            }
            break;
        }
        if len &0xC0 == 0xC0 {
            let ptr = (((len &0x3F) as usize) << 8) | data[offset + 1] as usize;
            if !jumped {
                end = offset + 2;
                jumped = true;
            }
            offset = ptr;
            continue;
        }
        if offset + 1 + len > data.len() {
            break;
        }
        labels.push(String::from_utf8_lossy(&data[offset + 1..offset + 1 + len]).to_string());
        offset += 1 + len;
    }
    (labels.join("."), end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_names() {
        let mut out = Vec::new();
        encode_name("_hap._tcp.local", &mut out);
        assert_eq!(out, vec![4, b'_', b'h', b'a', b'p', 4, b'_', b't', b'c', b'p', 5, b'l', b'o', b'c', b'a', b'l', 0]);
    }

    #[test]
    fn reads_plain_name() {
        let data = [4, b'_', b'h', b'a', b'p', 4, b'_', b't', b'c', b'p', 5, b'l', b'o', b'c', b'a', b'l', 0];
        let (name, end) = read_name(&data, 0);
        assert_eq!(name, "_hap._tcp.local");
        assert_eq!(end, data.len());
    }

    #[test]
    fn reads_compressed_name() {
        let data = [4, b'_', b'h', b'a', b'p', 4, b'_', b't', b'c', b'p', 5, b'l', b'o', b'c', b'a', b'l', 0, 0xC0, 0x00];
        let (name, end) = read_name(&data, data.len() - 2);
        assert_eq!(name, "_hap._tcp.local");
        assert_eq!(end, data.len());
    }

    #[test]
    fn parses_response() {
        let mut pkt = Vec::new();
        pkt.extend_from_slice(&[0, 1, 0x84, 0, 0, 1, 0, 2, 0, 0, 0, 0]);
        encode_name("_services._dns-sd._udp.local", &mut pkt);
        pkt.extend_from_slice(&[0, 12, 0, 1]);
        encode_name("_services._dns-sd._udp.local", &mut pkt);
        pkt.extend_from_slice(&[0, 12, 0, 1, 0, 0, 0, 120, 0, 5, 4, b'_', b'h', b'a', b'p']);
        encode_name("_hap._tcp.local", &mut pkt);
        pkt.extend_from_slice(&[0, 1, 0, 1, 0, 0, 0, 120, 0, 4, 192, 168, 1, 50]);

        let mut devices = Vec::new();
        parse_response(&pkt, &mut devices);
        assert!(!devices.is_empty());
        assert_eq!(devices[0].ip, Some(Ipv4Addr::new(192, 168, 1, 50)));
    }
}