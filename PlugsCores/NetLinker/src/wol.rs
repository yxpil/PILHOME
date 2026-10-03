//! WOL 网络唤醒:构造魔法包并通过 UDP 广播发送。
//!
//! 魔法包 = 6 字节 `0xFF` + 目标 MAC 重复 16 次(共 102 字节)。
//! 广播地址与端口可在配置中指定(默认 `255.255.255.255:9`)。

use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};

/// 构造 WOL 魔法包。
pub fn magic_packet(mac: [u8; 6]) -> Vec<u8> {
    let mut packet = Vec::with_capacity(102);
    packet.extend_from_slice(&[0xFF; 6]);
    for _ in 0..16 {
        packet.extend_from_slice(&mac);
    }
    packet
}

/// 解析 MAC 字符串(支持 `:` `-` 分隔或连续 12 位十六进制)。
pub fn parse_mac(raw: &str) -> Option<[u8; 6]> {
    let compact: String = raw.trim().chars().filter(|c| c.is_ascii_hexdigit()).collect();
    if compact.len() != 12 {
        return None;
    }
    let mut mac = [0u8; 6];
    for (i, byte) in mac.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&compact[i * 2..i * 2 + 2], 16).ok()?;
    }
    Some(mac)
}

/// 发送 WOL 魔法包。
///
/// `broadcast` 为广播地址(如 `255.255.255.255` 或 `192.168.1.255`)。
pub fn send_wol(mac: [u8; 6], broadcast: Ipv4Addr, port: u16) -> std::io::Result<()> {
    let sock = UdpSocket::bind("0.0.0.0:0")?;
    sock.set_broadcast(true)?;
    let packet = magic_packet(mac);
    sock.send_to(&packet, SocketAddrV4::new(broadcast, port))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn magic_packet_structure() {
        let mac = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55];
        let packet = magic_packet(mac);
        assert_eq!(packet.len(), 102);
        assert_eq!(&packet[0..6], &[0xFF; 6]);
        assert_eq!(&packet[6..12], &mac);
        assert_eq!(&packet[96..102], &mac);
    }

    #[test]
    fn parses_common_formats() {
        assert_eq!(parse_mac("00:11:22:33:44:55"), Some([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]));
        assert_eq!(parse_mac("00-11-22-33-44-55"), Some([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]));
        assert_eq!(parse_mac("001122334455"), Some([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]));
        assert!(parse_mac("zz:zz:zz:zz:zz:zz").is_none());
        assert!(parse_mac("00:11:22").is_none());
    }

    #[test]
    fn hostile_mac_inputs_are_rejected_or_sanitized_never_panic() {
        // 注入测试：MAC 来自不可信输入（API/CLI）。解析器只能产出干净的 6 字节，
        // 垃圾字符要么被过滤后仍不足 12 位而拒绝(None)，绝不能 panic / 越界。
        // 命令注入载荷：非十六进制字符被滤掉后仍凑不成合法 MAC
        assert!(parse_mac("00:11:22:33:44:55; rm -rf /").is_none());
        assert!(parse_mac("00:11:22:33:44:55 && calc.exe").is_none());
        // 路径穿越
        assert!(parse_mac("../../etc/passwd").is_none());
        // XSS / HTML
        assert!(parse_mac("<script>alert(1)</script>").is_none());
        // 空 / 空白 / 带换行的尾巴
        assert!(parse_mac("").is_none());
        assert!(parse_mac("   ").is_none());
        assert!(parse_mac("00:11:22:33:44:55\nINJECTED").is_none());
        // 正常格式回归仍可用
        assert_eq!(
            parse_mac("00-11-22-33-44-55"),
            Some([0x00, 0x11, 0x22, 0x33, 0x44, 0x55])
        );
    }
}