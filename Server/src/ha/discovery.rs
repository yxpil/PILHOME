//! MQTT Discovery:设备 -> HA 实体注册。

use crate::Config;
use pilhome_netlinker::{Device, DeviceKind};
use rumqttc::{AsyncClient, QoS};

/// 为单台设备生成 HA Discovery 主题。
pub fn topic_for(prefix: &str, device: &Device) -> String {
    let platform = match device.kind {
        DeviceKind::Contact => "binary_sensor",
        DeviceKind::Presence => "binary_sensor",
        DeviceKind::Lock => "lock",
        DeviceKind::Sensor => "sensor",
        _ => "binary_sensor",
    };
    format!("{prefix}/{platform}/pilhome_{}/config", device.id)
}

/// 为单台设备生成 Discovery 负载(保留消息)。
pub fn payload_for(device: &Device, topic_prefix: &str) -> serde_json::Value {
    let state_topic = format!("{topic_prefix}/{}/state", device.id);
    let (device_class, payload_on, payload_off) = match device.kind {
        DeviceKind::Contact => ("door", "open", "closed"),
        DeviceKind::Presence => ("motion", "motion", "clear"),
        DeviceKind::Lock => ("lock", "LOCK", "UNLOCK"),
        _ => ("", "ON", "OFF"),
    };

    let mut payload = serde_json::json!({
        "name": device.name,
        "uniq_id": format!("pilhome_{}", device.id),
        "state_topic": state_topic,
        "availability_topic": format!("{}/availability", topic_prefix),
        "payload_available": "online",
        "payload_not_available": "offline",
        "device": {
            "identifiers": ["pilhome_gateway"],
            "name": "PILHOME 边缘网关",
            "manufacturer": "YxPil",
            "model": "PILHOME 0.1",
        },
    });
    if !device_class.is_empty() {
        payload["device_class"] = serde_json::json!(device_class);
    }
    if !payload_on.is_empty() {
        payload["payload_on"] = serde_json::json!(payload_on);
        payload["payload_off"] = serde_json::json!(payload_off);
    }
    payload
}

/// 将设备列表发布为 HA Discovery 实体。
pub async fn publish_devices(client: &AsyncClient, cfg: &Config, devices: &[Device]) {
    if !cfg.ha.discovery {
        return;
    }
    for device in devices {
        let topic = topic_for(&cfg.ha.discovery_prefix, device);
        let payload = payload_for(device, &cfg.mqtt.topic_prefix).to_string();
        if let Err(err) = client.publish(topic, QoS::AtLeastOnce, true, payload.as_bytes()).await {
            tracing::warn!("Discovery 发布失败({}):{err}", device.id);
        }
    }
}